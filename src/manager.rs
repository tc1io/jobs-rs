use crate::error::Error;
use crate::executor::Executor;
use crate::job::Status::{Registered, Running, Suspended};
use crate::job::{Job, JobAction, JobName, JobRepo, Status};
use crate::lock::LockRepo;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::sleep;
use tokio_retry::Action;

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
    pub fn register(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
        self.jobs.push(Job::new(name.into(), action));
    }

    pub async fn start_all(&mut self) -> Result<(), Error> {
        let mut job_repo = self.job_repo.clone();
        let lock_repo = self.lock_repo.clone();
        // tokio::task::spawn(async move {
        loop {
            for job in self
                .jobs
                .iter_mut()
                .filter(|j| Job::get_registered_or_running(&j.status))
            {
                let (tx, mut rx) = oneshot::channel::<JobName>();
                // match rx.try_recv() {
                //     Ok(n) => {
                //         self.jobs2
                //             // .map_err(|e| GeneralError {
                //             //     description: e.to_string(),
                //             // })?
                //             // .unwrap()
                //             .iter_mut()
                //             .filter(|j| j.name == n)
                //             .map(|job| job.status = Suspended);
                //         // {
                //         //     job.status = Suspended;
                //         // }
                //     }
                //     Err(_e) => {}
                // };
                job.status = Running(tx);
                let mut ex = Executor::new(
                    job.name.clone(),
                    job.action.clone(),
                    job_repo.clone(),
                    lock_repo.clone(),
                );
                tokio::spawn(async move {
                    ex.run().await;
                });
                // }
                sleep(Duration::from_secs(2)).await;
            }
        }
        Ok(())
    }
    pub async fn stop_by_name(self, name: String) -> Result<(), Error> {
        for job in self
            .jobs
            // .lock()
            // .map_err(|e| GeneralError {
            //     description: e.to_string(),
            // })?
            .into_iter()
            .filter(|j| j.name == name.clone().into())
        {
            return match job.status {
                Running(s) => {
                    s.send(job.name.clone()).unwrap();
                    Ok(())
                }
                _ => Ok(()),
            };
        }
        Ok(())
    }
}
