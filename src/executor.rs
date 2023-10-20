use crate::executor::State::{Create, Run, Start};
use crate::job::Schedule;
use crate::job::{JobAction, JobConfig, JobName, JobRepo};
use crate::lock::LockRepo;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Mutex;
use tokio::time;
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_name: JobName,
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    job_config: JobConfig,
    cancel_signal_rx: Receiver<()>,
    job_repo: J,
    lock_repo: L,
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
            job_name: job_name.clone(),
            action,
            job_config: JobConfig::new(job_name, schedule),
            cancel_signal_rx,
            job_repo,
            lock_repo,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Start(),
    Create(),
    Run(),
}

impl State {
    pub fn init() -> State {
        Start()
    }
    pub async fn execute<J: JobRepo + Sync + Send + Clone, L: LockRepo + Sync + Send + Clone>(
        &mut self,
        ex: &mut Executor<J, L>,
    ) -> Result<Option<State>> {
        return match self {
            Start() => {
                let mut interval = time::interval(time::Duration::from_secs(
                    ex.job_config.clone().check_interval_sec,
                ));
                match ex.cancel_signal_rx.try_recv() {
                    Ok(_) => {
                        dbg!("received stopped signal....", ex.job_name.clone());
                        return Ok(None);
                    }
                    Err(_e) => {}
                }
                interval.tick().await; // TODO: waits for timer.. should be a better solution
                Ok(Some(Create()))
            }
            Create() => {
                let mut job_config = ex.job_config.clone();
                if let Some(jc) = ex.job_repo.get_job(job_config.name.clone().into()).await? {
                    job_config.state = jc.state;
                    job_config.last_run = jc.last_run
                }
                ex.job_repo.create_or_update_job(job_config.clone()).await?;

                Ok(Some(Run()))
            }
            Run() => {
                dbg!("run.. returning check");
                let mut action = ex.action.lock().await;
                let _xx = action.call(ex.job_name.clone().into(), Vec::new()).await?;
                Ok(Some(Start()))
            }
        };
    }
}
