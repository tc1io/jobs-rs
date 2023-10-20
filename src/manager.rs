use crate::executor::{Executor, State};
use crate::job::Status::Running;
use crate::job::{Job, JobAction, JobName, JobRepo, Schedule};
use crate::lock::LockRepo;
use anyhow::{anyhow, Result};
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
            .push(Job::new(JobName(name.clone().into()), action, schedule));
    }

    pub async fn start_all(&mut self) -> Result<()> {
        let job_repo = self.job_repo.clone();
        let lock_repo = self.lock_repo.clone();
        for job in self
            .jobs
            .iter_mut()
            .filter(|j| Job::get_registered_or_running(&j.status))
        {
            let (tx, rx) = oneshot::channel();
            job.status = Running(tx);
            let name = job.name.clone();
            let action = job.action.clone();
            let schedule = job.schedule.clone();
            let j = job_repo.clone();
            let l = lock_repo.clone();
            tokio::spawn(async move {
                let mut ex = Executor::new(name, action, schedule, j, l, rx);
                let mut state = State::init();
                loop {
                    match state.execute(&mut ex).await {
                        Ok(maybe_state) => {
                            if maybe_state.map(|s| state = s).is_none() {
                                println!("empty state. Aborting job!!");
                                break;
                            }
                        }
                        Err(e) => {
                            println!("error: {:?}. Aborting job", e);
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
                    dbg!("sending the stop signal now....");
                    let xx = s
                        .send(())
                        .map_err(|e| anyhow!("send cancel signal failed"))?;
                    Ok(())
                }
                _ => Ok(()),
            };
        }
        Ok(())
    }
}
