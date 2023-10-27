use crate::executor::State::{Create, Run, Start};
use crate::job::Schedule;
use crate::job::{JobAction, JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use anyhow::{anyhow, Result};
use chrono::Utc;
use log::info;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Mutex;
use tokio::time;
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    job_config: JobConfig,
    cancel_signal_rx: Receiver<()>,
    job_repo: J,
    lock_repo: L,
    lock_data: LockData,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        job_name: JobName,
        action: Arc<Mutex<dyn JobAction + Send + Sync>>,
        schedule: Schedule,
        job_repo: J,
        lock_repo: L,
        cancel_signal_rx: Receiver<()>,
    ) -> Self {
        Executor {
            action,
            job_config: JobConfig::new(job_name, schedule),
            cancel_signal_rx,
            job_repo,
            lock_repo,
            lock_data: LockData::new(),
        }
    }
    pub async fn run(&mut self) -> Result<()> {
        let mut state = State::new();
        while state.execute(self).await?.map(|s| state = s).is_some() {} // execute until the state is None.
        info!(
            "received empty state. Aborting job: {:?}",
            self.job_config.name.clone().to_string()
        );
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum State {
    Start,
    Create(JobConfig),
    Run(JobConfig),
}

impl State {
    pub fn new() -> State {
        Start
    }

    pub async fn execute<J: JobRepo + Sync + Send + Clone, L: LockRepo + Sync + Send + Clone>(
        &mut self,
        ex: &mut Executor<J, L>,
    ) -> Result<Option<State>> {
        let mut interval = time::interval(time::Duration::from_secs(
            ex.job_config.clone().check_interval_sec,
        ));
        interval.tick().await; // The first tick completes immediately
        return match self {
            Start => {
                match ex.cancel_signal_rx.try_recv() {
                    Ok(_) => {
                        return Ok(None);
                    }
                    Err(_e) => {}
                }
                interval.tick().await;
                Ok(Some(Create(ex.job_config.clone())))
            }
            Create(job_config) => {
                if let Some(jc) = ex.job_repo.get_job(job_config.name.clone().into()).await? {
                    job_config.state = jc.state;
                    job_config.last_run = jc.last_run
                }
                ex.job_repo.create_or_update_job(job_config.clone()).await?;
                Ok(Some(Run(ex.job_config.clone())))
            }
            Run(job_config) => {
                if job_config.clone().run_job_now()? {
                    let name = job_config.clone().name;
                    let mut lock_data = ex.lock_data.clone();
                    lock_data.expires = (Utc::now().timestamp_millis() as u64)
                        .add(lock_data.ttl.as_millis() as u64);
                    if ex.lock_repo.acquire_lock(name.clone(), lock_data).await? {
                        let refresh_lock_fut = ex.lock_repo.refresh_lock(name.clone());
                        let mut action = ex.action.lock().await;
                        let job_fut = action.call(Vec::new());
                        tokio::select! {
                                refreshed = refresh_lock_fut => {
                                match refreshed {
                                    Ok(_x) => Ok(()),
                                    Err(e) => Err(anyhow!(e)),
                                    }
                                }
                                state = job_fut => {
                                match state {
                                    Ok(s) => {
                                        let last_run = Utc::now().timestamp_millis();
                                        let _x = ex.job_repo.save_state(name, last_run, s).await?;
                                        ex.job_config.last_run = last_run;
                                        Ok(())
                                    }
                                    Err(e) => Err(anyhow!(e)),
                                }
                            }
                        }?;
                    }
                }
                Ok(Some(Start))
            }
        };
    }
}
