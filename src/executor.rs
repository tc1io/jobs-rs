use crate::job::{JobAction, JobConfig, JobData, JobName, LockStatus, Repo};
use crate::{Error, Result};
use chrono::Utc;
use log::{error, trace};
use std::fmt::{Debug, Formatter};
use tokio::sync::oneshot::Receiver;
use tokio::time::{sleep, Duration};

pub struct Shared<JR> {
    instance: String,
    name: JobName,
    repo: JR,
    cancel: Receiver<()>,
    action: Box<dyn JobAction + Send + Sync>,
}

pub enum Executor<JR: Repo> {
    Initial(Shared<JR>, JobData, Duration),
    Sleeping(Shared<JR>, Duration),
    Start(Shared<JR>, JobData),
    CheckDue(Shared<JR>, Duration),
    TryLock(Shared<JR>, Duration),
    Run(Shared<JR>, JobData, JR::Lock),
    Done,
}

impl<JR: Repo> Debug for Executor<JR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Executor::Initial(..) => f.write_str("--------------------------- initial"),
            Executor::Sleeping(..) => f.write_str("--------------------------- sleeping"),
            Executor::Start(..) => f.write_str("------------------------------------ start"),
            Executor::TryLock(..) => f.write_str("------------------------------------ trylock"),
            Executor::CheckDue(..) => f.write_str("------------------------------------ CheckDue"),
            Executor::Run(..) => f.write_str("------------------------------------ run"),
            Executor::Done => f.write_str("------------------------------------ done"),
        }
    }
}

impl<J: Repo + Clone + Send + Sync> Executor<J> {
    pub fn new(
        instance: String,
        data: JobConfig,
        action: Box<dyn JobAction + Send + Sync>,
        job_repo: J,
        cancel: Receiver<()>,
        delay: Duration,
    ) -> Self {
        Executor::Initial(
            Shared {
                instance,
                name: data.name.clone(),
                repo: job_repo,
                cancel,
                action,
            },
            JobData::from(data),
            delay,
        )
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            trace!("loop {:?}", self);
            self = match self {
                Self::Initial(shared, jdata, delay) => on_initial(shared, jdata, delay).await,
                Self::Start(shared, jdata) => on_start(shared, jdata).await,
                Self::Sleeping(shared, delay) => on_sleeping(shared, delay).await,
                Self::CheckDue(shared, delay) => on_check_due(shared, delay).await,
                Self::TryLock(shared, delay) => on_try_lock(shared, delay).await,
                Self::Run(shared, jdata, lock) => on_run(shared, jdata, lock).await,
                Self::Done => return Ok(()),
            }
        }
    }
}

async fn on_initial<R: Repo>(shared: Shared<R>, jdata: JobData, delay: Duration) -> Executor<R> {
    sleep(delay).await;
    Executor::Start(shared, jdata)
}

async fn on_sleeping<R: Repo>(mut shared: Shared<R>, delay: Duration) -> Executor<R> {
    let done = tokio::select! {
        _ = sleep(delay) =>  false,
        _ = &mut shared.cancel => true
    };

    if done {
        Executor::Done
    } else {
        Executor::CheckDue(shared, delay)
    }
}

async fn on_start<R: Repo>(mut shared: Shared<R>, jdata: JobData) -> Executor<R> {
    match shared.repo.get(jdata.name.clone().into()).await {
        Err(e) => {
            error!("get job data: {:?}", e);
            Executor::Initial(shared, jdata, Duration::from_secs(1)) // TODO Backoff
        }
        Ok(None) => {
            match shared.repo.create(jdata.clone()).await {
                Err(e) => {
                    error!("create job data: {:?}", e);
                    Executor::Initial(shared, jdata, Duration::from_secs(1)) // TODO Backoff
                }
                Ok(()) => Executor::TryLock(shared, jdata.check_interval),
            }
        }
        Ok(Some(jdata)) if jdata.due(Utc::now()) => Executor::TryLock(shared, jdata.check_interval),
        Ok(Some(jdata)) => Executor::Sleeping(shared, jdata.check_interval),
    }
}

async fn on_check_due<R: Repo>(mut shared: Shared<R>, delay: Duration) -> Executor<R> {
    match shared.repo.get(shared.name.clone()).await {
        // TODO split these two cases for clarity
        Err(_) | Ok(None) => Executor::Sleeping(shared, delay), // TODO Retry interval, attempt counter, bbackoff },
        Ok(Some(jdata)) if jdata.due(Utc::now()) => Executor::TryLock(shared, jdata.check_interval),
        Ok(Some(_)) => Executor::Sleeping(shared, delay),
    }
}
async fn on_try_lock<R: Repo>(mut shared: Shared<R>, delay: Duration) -> Executor<R> {
    match shared
        .repo
        .lock(
            shared.name.clone(),
            shared.instance.clone(),
            Duration::from_secs(10),
        )
        .await
    {
        // TODO use duration from jobdata
        Err(_) => Executor::Sleeping(shared, delay), // TODO Retry interval, attempt counter, bbackoff },
        Ok(LockStatus::AlreadyLocked) => Executor::Sleeping(shared, delay),
        Ok(LockStatus::Acquired(jdata, lock)) if jdata.due(Utc::now()) => {
            Executor::Run(shared, jdata, lock)
        }
        Ok(LockStatus::Acquired(jdata, _)) => {
            // We hold the lock but job is not due, so we call save with existing data to
            // release the lock. Since we do a get lock and due check before even going to
            // TryLock, this is an edge case only and nt the normal mode of operation.
            // Usually the job shoud be due when we reach TryLock.
            match shared
                .repo
                .save(jdata.name, jdata.last_run, jdata.state)
                .await
            {
                Ok(()) => Executor::Sleeping(shared, delay),
                Err(e) => {
                    error!("unlock failed in try-lock-but-not-due edge case: {:?}", e);
                    Executor::Sleeping(shared, delay)
                }
            }
        }
    }
}
async fn on_run<R: Repo>(mut shared: Shared<R>, jdata: JobData, lock: R::Lock) -> Executor<R> {
    if !jdata.due(Utc::now()) {
        return Executor::Sleeping(shared, jdata.check_interval);
    }

    let job_fut = shared.action.call(jdata.state);
    let select_result = tokio::select! {
        job_result = job_fut => {
            match job_result {
                Ok(state) => {
                    match shared.repo.save(jdata.name.clone(), Utc::now(), state).await {
                        Ok(()) => RunSelectResult::Success,
                        Err(e) => RunSelectResult::SaveFailure(e)
                    }
                },
                Err(e) => RunSelectResult::JobFailure(e)
            }
        }
        Err(e) = lock => {
            RunSelectResult::LockFailure(e)
        }
        _ = &mut shared.cancel => {
            RunSelectResult::Cancaled
         }
    };

    // TODO refine all the Done cases to proper sleeps + backoff
    match select_result {
        RunSelectResult::Success => Executor::Sleeping(shared, jdata.check_interval),
        RunSelectResult::JobFailure(_) => Executor::Done,
        RunSelectResult::LockFailure(_) => Executor::Done,
        RunSelectResult::SaveFailure(_) => Executor::Done,
        RunSelectResult::Cancaled => Executor::Done,
    }
}

enum RunSelectResult {
    Success,
    JobFailure(Error),
    LockFailure(Error),
    SaveFailure(Error),
    Cancaled,
}
