use std::fmt::{Debug, Formatter};
use crate::job::Schedule;
use crate::job::{JobAction, JobConfig, JobName, JobRepo};
use crate::lock::{LockRepo, LockStatus};
use chrono::Utc;
use log::trace;
use rand::Rng;
use tokio::sync::oneshot::Receiver;
use tokio::time::{sleep, Duration};
use crate::{Error,Result};

struct Repos<JR, LR> {
    instance: String,
    job_repo: JR,
    lock_repo: LR,
    cancel_signal_rx: Option<Receiver<()>>,
    action: Box<dyn JobAction + Send + Sync>,
}

pub enum Executor<JR, LR: LockRepo> {
    InitialDelay {
        repos: Repos<JR, LR>,
        job_config: JobConfig,
        delay: Duration,
    },
    Sleeping {
        repos: Repos<JR, LR>,
        name: JobName,
        d: Duration,
    },
    Start {
        repos: Repos<JR, LR>,
        job_config: JobConfig,
    },
    TryLock {
        repos: Repos<JR, LR>,
        name: JobName,
    },
    Run {
        repos: Repos<JR, LR>,
        name: JobName,
        lock: LR::Lock,
    },
    Done,
}

impl<JR,LR: LockRepo> Debug for Executor<JR,LR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Executor::InitialDelay { .. } =>  f.write_str("------------------------------------ initial"),
            Executor::Sleeping { .. } => f.write_str("------------------------------------ sleeping"),
            Executor::Start { .. } => f.write_str("------------------------------------ start"),
            Executor::TryLock { .. } => f.write_str("------------------------------------ trylock"),
            Executor::Run { .. } => f.write_str("------------------------------------ run"),
            Executor::Done => f.write_str("------------------------------------ done"),
        }

    }
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        instance: String,
        job_name: JobName,
        action: Box<dyn JobAction + Send + Sync>,
        schedule: Schedule,
        job_repo: J,
        lock_repo: L,
        cancel_signal_rx: Receiver<()>,
        delay: Duration,
    ) -> Self {
        Executor::InitialDelay {
            repos: Repos {
                instance,
                job_repo,
                lock_repo,
                cancel_signal_rx: Some(cancel_signal_rx),
                action,
            },
            job_config: JobConfig::new(job_name, schedule),
            delay,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            trace!("loop");
            dbg!(&self);
            self = match self {
                Self::InitialDelay { repos, job_config, delay } => {
                    sleep(delay).await;
                    Self::Start { repos, job_config }
                }
                Self::Start {
                    mut repos,
                    job_config,
                } => match repos.job_repo.get_job(job_config.name.clone().into()).await {
                    Err(_) => todo!(), //TODO error sleep state  Self::Start { repos, job_config },
                    Ok(None) => {
                        match repos
                            .job_repo
                            .create_or_update_job(job_config.clone())
                            .await
                        {
                            Err(e) => panic!("{:?}",e), //Self::Start { repos, job_config },
                            Ok(_) => Self::TryLock {
                                repos,
                                name: job_config.name.clone(),
                            },
                        }
                    }
                    Ok(Some(job_config)) => {
                        trace!("OK ....jc: {:?}",job_config);
                        if job_config.run_job_now().unwrap()
                        {
                            Self::TryLock {
                                repos,
                                name: job_config.name,
                            }
                        } else {
                            trace!("Not time yet, to run job again - sleeping again");
                            Self::Sleeping {
                                repos,
                                name: job_config.name,
                                d: Duration::from_secs(10),
                            }
                        }
                    },
                },
                Self::Sleeping { mut repos, name, d } => {
                    // sleep(d).await;
                    // Self::TryLock { repos, name }

                    let mut cancel_signal_rx = repos.cancel_signal_rx.take().unwrap();

                    let x = tokio::select! {
                        _ = sleep(d) => {
                            println!("do_stuff_async() completed first");
                            true
                        }
                        _ = &mut cancel_signal_rx => {
                            println!("more_async_work() completed first");
                            false
                        }
                    };
                    if x {
                        repos.cancel_signal_rx = Some(cancel_signal_rx);
                        Self::TryLock { repos, name }
                    } else {
                        Self::Done
                    }

                }

                //     println!("START");
                //     match ex.cancel_signal_rx.try_recv() {
                //         Ok(_) => {
                //             return Ok(None);
                //         }
                //         Err(_e) => {}
                //     }
                //     interval.tick().await;
                //     Ok(Some(Create {
                //         job_config: ex.job_config.clone(),
                //     }))
                // }
                Self::TryLock { mut repos, name } => {
                    let instance = repos.instance.clone(); // try to avoid
                                                           // TOD no hardcoed ttl here
                    match repos
                        .lock_repo
                        .acquire_lock(name.clone(), instance, Duration::from_secs(10))
                        .await
                    {
                        Err(_) => Self::Sleeping {
                            repos,
                            name,
                            d: Duration::from_secs(10),
                        },
                        Ok(LockStatus::AlreadyLocked) => Self::Sleeping {
                            repos,
                            name,
                            d: Duration::from_secs(10),
                        },
                        Ok(LockStatus::Acquired(lock)) => Self::Run { repos, name, lock },
                    }
                }

                Self::Run {
                    mut repos,
                    name,
                    lock,
                } => {
                    let job_config = repos.job_repo.get_job(name.clone()).await?.unwrap();
                    if job_config.run_job_now().unwrap() {
                        let job_fut = repos.action.call(job_config.state);
                        tokio::select! {
                        sxx = job_fut => {
                            match sxx {
                             Ok(xxx) => {
                            let last_run = Utc::now().timestamp_millis();
                            let _x = repos.job_repo.save_state(name.clone(), last_run, xxx).await.unwrap();
                            Self::Sleeping {
                                repos,
                                name,
                                d: Duration::from_secs(10),
                            }
                        },
                        Err(_) => Self::Done,

                        }
                            }
                        _ = lock => {
                                Self::Done
                        }
                            // TODO Unlock

                    }
                    } else {
                        Self::Sleeping {
                            repos,
                            name,
                            d: Duration::from_secs(10),
                        }

                    }
                }
                Self::Done => break,
            }
        }
        Ok(())
    }
}

//
// if job_config.clone().run_job_now()? {
//     let name = job_config.clone().name;
//     let mut lock_data = ex.lock_data.clone();
//     lock_data.expires = (Utc::now().timestamp_millis() as u64)
//         .add(lock_data.ttl.as_millis() as u64);
//     if ex.lock_repo.acquire_lock(name.clone(), lock_data).await? {
//         let refresh_lock_fut = ex.lock_repo.refresh_lock(name.clone());
//         let mut action = ex.action.lock().await;
//         let job_fut = action.call(job_config.state);
//         tokio::select! {
//                 refreshed = refresh_lock_fut => {
//                 match refreshed {
//                     Ok(_x) => Ok(()),
//                     Err(e) => Err(anyhow!(e)),
//                     }
//                 }
//                 state = job_fut => {
//                 match state {
//                     Ok(s) => {
//                         let last_run = Utc::now().timestamp_millis();
//                         let _x = ex.job_repo.save_state(name, last_run, s).await?;
//                         ex.job_config.last_run = last_run;
//                         Ok(())
//                     }
//                     Err(e) => Err(anyhow!(e)),
//                 }
//             }
//         }?;
//     }
// }

// println!("RUN");
// let job_config = repos.job_repo.get_job(name.clone()).await?.unwrap();
//
// if job_config.clone().run_job_now()? {
// let name = job_config.clone().name;
// let mut lock_data = ex.lock_data.clone();
// lock_data.expires = (Utc::now().timestamp_millis() as u64)
// .add(lock_data.ttl.as_millis() as u64);
// if ex.lock_repo.acquire_lock(name.clone(), lock_data).await? {
// let refresh_lock_fut = ex.lock_repo.refresh_lock(name.clone());
// let mut action = ex.action.lock().await;
// let job_fut = action.call(job_config.state);
// tokio::select! {
// refreshed = refresh_lock_fut => {
// match refreshed {
// Ok(_x) => Ok(()),
// Err(e) => Err(anyhow!(e)),
// }
// }
// state = job_fut => {
// match state {
// Ok(s) => {
// let last_run = Utc::now().timestamp_millis();
// let _x = ex.job_repo.save_state(name, last_run, s).await?;
// ex.job_config.last_run = last_run;
// Ok(())
// }
// Err(e) => Err(anyhow!(e)),
// }
// }
// }?;
// }
// }
// }
