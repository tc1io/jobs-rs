use crate::error::Error;
use crate::executor::Executor;
use crate::job::Status::{Registered, Running, Suspended};
use crate::job::{Job, JobAction, JobName, JobRepo, Status};
use crate::lock::LockRepo;
use std::sync::Arc;
// use tokio::prelude::future;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
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
    pub async fn register(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
        self.jobs.lock().await.push(Job::new(name.into(), action))
    }
    pub async fn start_all(&mut self) -> Result<(), Error> {
        let ll = self.jobs.lock().await;
        dbg!(ll.len());
        let (tx, mut rx) = mpsc::channel::<String>(1);
        let mut jobs = Arc::clone(&self.jobs);

        let mut j = self.job_repo.clone();
        let l = self.lock_repo.clone();
        tokio::task::spawn(async move {
            loop {
                for job in jobs
                    .lock()
                    .await
                    .iter_mut()
                    .filter(|j| j.registered_or_running())
                {
                    sleep(Duration::from_secs(2)).await;
                    match rx.try_recv() {
                        Ok(n) => {
                            dbg!("received");
                            job.status = Suspended;
                            continue;
                        } //remove_job_from_list;
                        Err(e) => {
                            if e == TryRecvError::Empty {
                                dbg!("empty");
                                // let mut tasks = Vec::with_capacity(self.jobs.len());
                                // let (tx, rx) = mpsc::channel::<()>(1);
                                job.status = Running(tx.clone());
                                let mut ex = Executor::new(job.clone(), j.clone(), l.clone());
                                // tasks.push(
                                tokio::task::spawn(async move {
                                    ex.run().await;
                                });
                            }
                        } // TODO: handle other error case
                    }
                }
            }
        });
        // let mut outputs = Vec::with_capacity(tasks.len());
        // for task in tasks {
        //     outputs.push(task.await.unwrap());
        // }
        // println!("{:?}", outputs);
        Ok(())
    }
    pub async fn stop_by_name(&mut self, name: String) -> Result<(), Error> {
        for job in self.jobs.lock().await.iter_mut() {
            let n = JobName(name.clone());
            if job.name.clone() == n {
                dbg!("stopping");
                dbg!(job.clone().status);
                // job.status = Suspended
                return match job.clone().status {
                    Running(s) => {
                        dbg!("status: running");
                        s.send(name).await.unwrap();
                        Ok(())
                    }
                    _ => Ok(()),
                };
            }
        }
        Ok(())
    }
}
