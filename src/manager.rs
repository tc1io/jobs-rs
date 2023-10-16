use crate::error::Error;
use crate::error::Error::GeneralError;
use crate::executor::Executor;
use crate::job::Status::{Registered, Running, Suspended};
use crate::job::{Job, JobAction, JobName, JobRepo, Status};
use crate::lock::LockRepo;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;
use tokio_retry::Action;

pub struct JobManager<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl<J: JobRepo + Clone + Send + Sync + 'static, L: LockRepo + Clone + Send + Sync + 'static>
    JobManager<J, L>
{
    pub fn new(job_repo: J, lock_repo: L) -> Self {
        JobManager {
            job_repo: job_repo.clone(),
            lock_repo: lock_repo.clone(),
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn register(
        &mut self,
        name: String,
        action: impl JobAction + Send + Sync + 'static,
    ) -> Result<(), Error> {
        Ok(self
            .jobs
            .lock()
            .map_err(|e| GeneralError {
                description: e.to_string(),
            })?
            .push(Job::new(name.into(), action)))
    }
    pub async fn start_all(&mut self) -> Result<(), Error> {
        let (tx, mut rx) = mpsc::channel::<JobName>(1);
        let mut jobs = Arc::clone(&self.jobs);

        let mut job_repo = self.job_repo.clone();
        let lock_repo = self.lock_repo.clone();
        tokio::task::spawn(async move {
            loop {
                match rx.try_recv() {
                    Ok(n) => {
                        for job in jobs
                            .lock()
                            .map_err(|e| GeneralError {
                                description: e.to_string(),
                            })
                            .unwrap()
                            .iter_mut()
                            .filter(|j| j.name == n)
                        {
                            job.status = Suspended;
                        }
                    }
                    Err(_e) => {}
                };
                for job in jobs
                    .lock()
                    .map_err(|e| GeneralError {
                        description: e.to_string(),
                    })
                    .unwrap()
                    .iter_mut()
                    .filter(|j| j.get_registered_or_running())
                {
                    job.status = Running(tx.clone());
                    let mut ex = Executor::new(job.clone(), job_repo.clone(), lock_repo.clone());
                    tokio::task::spawn(async move {
                        ex.run().await;
                    });
                }
                // sleep(Duration::from_secs(2)).await;
            }
        });
        Ok(())
    }
    pub async fn stop_by_name(&mut self, name: String) -> Result<(), Error> {
        for job in self
            .jobs
            .lock()
            .map_err(|e| GeneralError {
                description: e.to_string(),
            })?
            .iter_mut()
        {
            if job.name.clone() == name.clone().into() {
                return match job.clone().status {
                    Running(s) => {
                        s.send(job.name.clone()).await.unwrap();
                        Ok(())
                    }
                    _ => Ok(()),
                };
            }
        }
        Ok(())
    }
}
