use crate::executor::{Executor, State};
use crate::job::Status::Running;
use crate::job::{Job, JobAction, JobName, JobRepo, Schedule};
use crate::lock::LockRepo;
use anyhow::{anyhow, Result};
use log::{info, warn};
use tokio::sync::oneshot;

pub struct JobManager<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    jobs: Vec<Job>,
}

impl<J: JobRepo + Clone + Send + Sync + 'static, L: LockRepo + Clone + Send + Sync + 'static>
    JobManager<J, L>
{
    pub fn new(job_repo: J, lock_repo: L) -> Self {
        JobManager {
            job_repo: job_repo.clone(),
            lock_repo: lock_repo.clone(),
            jobs: Vec::new(),
        }
    }
    pub fn register(
        &mut self,
        name: String,
        action: impl JobAction + Send + Sync + 'static,
        schedule: Schedule,
    ) {
        self.jobs
            .push(Job::new(JobName(name.clone().into()), action, schedule)); // TODO: add validation during registration??
        info!("job: {:?} registered", name);
    }

    pub async fn start_all(&mut self) -> Result<()> {
        for job in self
            .jobs
            .iter_mut()
            .filter(|j| Job::get_registered_or_running(&j.status))
        {
            let (tx, rx) = oneshot::channel();
            let job_repo = self.job_repo.clone();
            let lock_repo = self.lock_repo.clone();
            let name = job.name.clone();
            let action = job.action.clone();
            let schedule = job.schedule.clone();

            job.status = Running(tx);
            tokio::spawn(async move {
                let mut ex = Executor::new(name.clone(), action, schedule, job_repo, lock_repo, rx);
                let mut state = State::init();
                loop {
                    match state.execute(&mut ex).await {
                        Ok(maybe_state) => {
                            if maybe_state.map(|s| state = s).is_none() {
                                info!(
                                    "received empty state. Aborting job: {:?}",
                                    name.clone().to_string()
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "error: {:?}. Aborting job: {:?}",
                                e.to_string(),
                                name.clone()
                            );
                            break;
                        }
                    }
                }
            });
        }
        Ok(())
    }
    pub async fn stop_by_name(self, name: String) -> Result<()> {
        for job in self
            .jobs
            .into_iter()
            .filter(|j| j.name == JobName(name.clone().into()))
        {
            return match job.status {
                Running(s) => {
                    info!("received stop signal. Stopping job: {:?}", name.clone());
                    let _xx = s
                        .send(())
                        .map_err(|()| anyhow!("send cancel signal failed"))?;
                    Ok(())
                }
                _ => Ok(()),
            };
        }
        Ok(())
    }
}
