use crate::executor::{Executor, State};
use crate::job::Status::Running;
use crate::job::{Job, JobAction, JobName, JobRepo, Schedule};
use crate::lock::LockRepo;
use anyhow::{anyhow, Result};
use log::{info, warn};
use tokio::sync::oneshot;

/// JobManager holds the job + lock repo along with the list of jobs
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
    /// register will add the job to the vector of jobs in JobManager
    /// ```rust,ignore
    ///     let mut manager = JobManager::<Repo, Repo>::new(db_repo, lock_repo);
    ///     manager.register(
    ///         String::from("project-updater"),
    ///         job.clone(),
    ///         Schedule {
    ///             expr: "* */3 * * * *".to_string(),
    ///        },
    ///     );
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

    /// start_all will spawn the jobs and run the job for ever until the job is stopped or aborted
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
            dbg!("before spwan");
            tokio::spawn(async move {
                let mut ex = Executor::new(name.clone(), action, schedule, job_repo, lock_repo, rx);
                let mut state = State::new();
                match state.execute(&mut ex).await {
                    Ok(maybe_state) => {
                        if maybe_state.map(|s| state = s).is_none() {
                            info!(
                                "received empty state. Aborting job: {:?}",
                                name.clone().to_string()
                            );
                            return;
                        }
                    }
                    Err(e) => {
                        warn!(
                            "error: {:?}. Aborting job: {:?}",
                            e.to_string(),
                            name.clone()
                        );
                        return;
                    }
                }
            });
        }
        Ok(())
    }
    /// stop_by_name will stop the job which is started as part of start_all
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
