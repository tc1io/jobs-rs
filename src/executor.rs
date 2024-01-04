use crate::job::{JobAction, JobConfig, JobData, JobName, LockStatus, Repo};
use crate::Result;
use chrono::Utc;
use log::{error, trace};
use std::fmt::{Debug, Formatter};
use tokio::sync::oneshot::Receiver;
use tokio::time::{sleep, Duration};

pub struct Shared<JR> {
    instance: String,
    job_repo: JR,
    cancel_signal_rx: Option<Receiver<()>>,
    action: Box<dyn JobAction + Send + Sync>,
}

pub enum Executor<JR: Repo> {
    InitialDelay {
        shared: Shared<JR>,
        jdata: JobData,
        delay: Duration,
    },
    Sleeping {
        shared: Shared<JR>,
        name: JobName,
        delay: Duration,
    },
    Start {
        shared: Shared<JR>,
        jdata: JobData,
    },
    TryLock {
        shared: Shared<JR>,
        name: JobName,
        delay: Duration,
    },
    Run {
        shared: Shared<JR>,
        jdata: JobData,
        lock: JR::Lock,
    },
    Done,
}

impl<JR: Repo> Debug for Executor<JR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Executor::InitialDelay { .. } => f.write_str("--------------------------- initial"),
            Executor::Sleeping { .. } => f.write_str("--------------------------- sleeping"),
            Executor::Start { .. } => f.write_str("------------------------------------ start"),
            Executor::TryLock { .. } => f.write_str("------------------------------------ trylock"),
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
        Executor::InitialDelay {
            shared: Shared {
                instance,
                job_repo,
                cancel_signal_rx: Some(cancel_signal_rx),
                action,
            },
            jdata: JobData::from(data),
            delay,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            trace!("loop {:?}", self);
            self = match self {
                Self::InitialDelay {
                    shared,
                    jdata,
                    delay,
                } => on_initial_delay(shared, jdata, delay).await,
                Self::Start { shared, jdata } => on_start(shared, jdata).await,
                Self::Sleeping {
                    shared,
                    name,
                    delay,
                } => on_sleeping(shared, name, delay).await,
                Self::TryLock {
                    shared,
                    name,
                    delay,
                } => on_try_lock(shared, name, delay).await,

                Self::Run {
                    shared,
                    jdata,
                    lock,
                } => on_run(shared, jdata, lock).await,
                Self::Done => return Ok(()),
            }
        }
    }
}

async fn on_initial_delay<R: Repo>(
    shared: Shared<R>,
    jdata: JobData,
    delay: Duration,
) -> Executor<R> {
    sleep(delay).await;
    Executor::Start { shared, jdata }
}

async fn on_sleeping<R: Repo>(
    mut shared: Shared<R>,
    name: JobName,
    delay: Duration,
) -> Executor<R> {
    let mut cancel_signal_rx = shared.cancel_signal_rx.take().unwrap();

    let cancel = tokio::select! {
        _ = sleep(delay) =>  false,
        _ = &mut cancel_signal_rx => true
    };

    if cancel {
        Executor::Done
    } else {
        shared.cancel_signal_rx = Some(cancel_signal_rx);
        Executor::TryLock {
            shared,
            name,
            delay,
        }
    }
}

async fn on_start<R: Repo>(mut shared: Shared<R>, jdata: JobData) -> Executor<R> {
    match shared.job_repo.get(jdata.name.clone().into()).await {
        Err(e) => {
            error!("get job data: {:?}", e);
            Executor::InitialDelay {
                shared,
                jdata,
                delay: Duration::from_secs(1),
            } // TODO Backoff
        }
        Ok(None) => {
            match shared.job_repo.create(jdata.clone()).await {
                Err(e) => {
                    error!("create job data: {:?}", e);
                    Executor::InitialDelay {
                        shared,
                        jdata,
                        delay: Duration::from_secs(1),
                    } // TODO Backoff
                }
                // TODO here should be a due() check
                Ok(()) => Executor::TryLock {
                    shared,
                    name: jdata.name,
                    delay: jdata.check_interval,
                },
            }
        }
        Ok(Some(jdata)) => {
            trace!("OK ....jc: {:?}", jdata);
            if jdata.due(Utc::now()) {
                Executor::TryLock {
                    shared,
                    name: jdata.name,
                    delay: jdata.check_interval,
                }
            } else {
                Executor::Sleeping {
                    shared,
                    name: jdata.name,
                    delay: jdata.check_interval,
                }
            }
        }
    }
}
async fn on_try_lock<R: Repo>(
    mut shared: Shared<R>,
    name: JobName,
    delay: Duration,
) -> Executor<R> {
    let instance = shared.instance.clone(); // try to avoid
    match shared.job_repo.lock(name.clone(), instance).await {
        Err(_) => Executor::Sleeping {
            shared,
            name,
            delay, // TODO Retry interval, attempt counter, bbackoff
        },
        Ok(LockStatus::AlreadyLocked) => Executor::Sleeping {
            shared,
            name,
            delay,
        },
        Ok(LockStatus::Acquired(jdata, lock)) => {
            if jdata.due(Utc::now()) {
                Executor::Run {
                    shared,
                    jdata,
                    lock,
                }
            } else {
                Executor::Sleeping {
                    shared,
                    name,
                    delay,
                }
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
                Executor::Sleeping {
                    shared,
                    name,
                    delay: jdata.check_interval,
                }
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
        Executor::Sleeping {
            shared,
            name,
            delay: jdata.check_interval,
        }
    }
}
