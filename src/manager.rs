use std::time::Duration;
use crate::executor::Executor;
use crate::job::Status::Running;
use crate::job::{Job, JobAction, JobName, JobRepo, Schedule};
use crate::lock::LockRepo;
use log::{info, trace, warn};
use rand::Rng;
use tokio::sync::oneshot;
use crate::{Error,Result};

/// JobManager holds the job + lock repo along with the list of jobs
pub struct JobManager<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    instance: String,
    job_repo: J,
    lock_repo: L,
    jobs: Vec<Job>,
}

impl<J: JobRepo + Clone + Send + Sync + 'static, L: LockRepo + Clone + Send + Sync + 'static>
    JobManager<J, L>
{
    pub fn new(instance: String, job_repo: J, lock_repo: L) -> Self {
        JobManager {
            instance,
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
        trace!("job: {:?} registered", name);
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
            let action = job.action.take().unwrap();
            let schedule = job.schedule.clone();

            job.status = Running(tx);
            let instance = self.instance.clone();
            let mut rng = rand::thread_rng();
            let delay = Duration::from_millis(rng.gen_range(10..100));
            tokio::spawn(async move {
                trace!("start executor for job {}",&name);
                let mut ex = Executor::new(instance,name.clone(), action, schedule, job_repo, lock_repo, rx,delay);
                match ex.run().await {
                    Ok(()) => trace!("job {} stopped",name),
                    Err(e) => warn!("job {} stopped with an error: {:?}", name,e),
                };
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
                        .map_err(|()| Error::CancelFailed(name))?;
                    Ok(())
                }
                _ => Ok(()),
            };
        }
        Ok(())
    }
}
