use crate::job::{JobAction, JobConfig, JobData, JobName, LockStatus, Repo};
use crate::Result;
use chrono::Utc;
use log::{error, trace};
use std::fmt::{Debug, Formatter};
use tokio::sync::oneshot::Receiver;
use tokio::time::{sleep, Duration};

pub struct Shared<JR> {
    instance: String,
    name: JobName,
    job_repo: JR,
    cancel_signal_rx: Option<Receiver<()>>,
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
            Executor::Initial { .. } => f.write_str("--------------------------- initial"),
            Executor::Sleeping { .. } => f.write_str("--------------------------- sleeping"),
            Executor::Start { .. } => f.write_str("------------------------------------ start"),
            Executor::TryLock { .. } => f.write_str("------------------------------------ trylock"),
            Executor::CheckDue { .. } => {
                f.write_str("------------------------------------ CheckDue")
            }
            Executor::Run { .. } => f.write_str("------------------------------------ run"),
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
        cancel_signal_rx: Receiver<()>,
        delay: Duration,
    ) -> Self {
        Executor::Initial(
            Shared {
                instance,
                name: data.name.clone(),
                job_repo,
                cancel_signal_rx: Some(cancel_signal_rx),
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
    let mut cancel_signal_rx = shared.cancel_signal_rx.take().unwrap();

    let cancel = tokio::select! {
        _ = sleep(delay) =>  false,
        _ = &mut cancel_signal_rx => true
    };

    if cancel {
        Executor::Done
    } else {
        shared.cancel_signal_rx = Some(cancel_signal_rx);
        Executor::CheckDue(shared, delay)
    }
}

async fn on_start<R: Repo>(mut shared: Shared<R>, jdata: JobData) -> Executor<R> {
    match shared.job_repo.get(jdata.name.clone().into()).await {
        Err(e) => {
            error!("get job data: {:?}", e);
            Executor::Initial(shared, jdata, Duration::from_secs(1)) // TODO Backoff
        }
        Ok(None) => {
            match shared.job_repo.create(jdata.clone()).await {
                Err(e) => {
                    error!("create job data: {:?}", e);
                    Executor::Initial(shared, jdata, Duration::from_secs(1)) // TODO Backoff
                }
                // TODO here should be a due() check
                Ok(()) => Executor::TryLock(shared, jdata.check_interval),
            }
        }
        Ok(Some(jdata)) => {
            trace!("Start({})", jdata);
            if jdata.due(Utc::now()) {
                Executor::TryLock(shared, jdata.check_interval)
            } else {
                Executor::Sleeping(shared, jdata.check_interval)
            }
        }
    }
}
async fn on_check_due<R: Repo>(mut shared: Shared<R>, delay: Duration) -> Executor<R> {
    match shared.job_repo.get(shared.name.clone()).await {
        // TODO split these two cases for clarity
        Err(_) | Ok(None) => Executor::Sleeping(shared, delay), // TODO Retry interval, attempt counter, bbackoff },
        Ok(Some(jdata)) if jdata.due(Utc::now()) => Executor::TryLock(shared, jdata.check_interval),
        Ok(Some(_)) => Executor::Sleeping(shared, delay),
    }
}
async fn on_try_lock<R: Repo>(mut shared: Shared<R>, delay: Duration) -> Executor<R> {
    let instance = shared.instance.clone(); // try to avoid
    match shared
        .job_repo
        .lock(shared.name.clone(), instance, Duration::from_secs(10))
        .await
    {
        // TODO use duration from jobdata
        Err(_) => Executor::Sleeping(shared, delay), // TODO Retry interval, attempt counter, bbackoff },
        Ok(LockStatus::AlreadyLocked) => Executor::Sleeping(shared, delay),
        Ok(LockStatus::Acquired(jdata, lock)) => {
            trace!("TryLock({})", jdata);
            if jdata.due(Utc::now()) {
                Executor::Run(shared, jdata, lock)
            } else {
                shared
                    .job_repo
                    .save(jdata.name, jdata.last_run, jdata.state)
                    .await
                    .unwrap();
                Executor::Sleeping(shared, delay)
            }
        }
    }
}
async fn on_run<R: Repo>(mut shared: Shared<R>, jdata: JobData, lock: R::Lock) -> Executor<R> {
    //let mut cancel_signal_rx = shared.cancel_signal_rx.take().unwrap();
    let name = jdata.name.clone();
    if jdata.due(Utc::now()) {
        let job_fut = shared.action.call(jdata.state);
        tokio::select! {
            sxx = job_fut => {
                match sxx {
                 Ok(xxx) => {
                let _x = shared.job_repo.save(jdata.name.clone(), Utc::now(), xxx).await.unwrap();
                Executor::Sleeping(shared, jdata.check_interval)
                },
                Err(_) => Executor::Done,
                }
            }
            _ = lock => {
                Executor::Done
            }
            // _ = &mut cancel_signal_rx => {
            //     Executor::Done
            // }
        }
    } else {
        Executor::Sleeping(shared, jdata.check_interval)
    }
}
