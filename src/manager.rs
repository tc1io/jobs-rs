use crate::error::Error;
use crate::executor::Executor;
use crate::job::Status::{Registered, Running};
use crate::job::{Job, JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
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
        self.jobs.push(Job::new(name.into(), action))
    }
    pub async fn start_all(&mut self) -> Result<(), Error> {
        loop {
            let mut tasks = Vec::with_capacity(self.jobs.len());
            for job in self.jobs.as_mut_slice() {
                job.status = Running;
                let mut ex =
                    Executor::new(job.clone(), self.job_repo.clone(), self.lock_repo.clone());
                tasks.push(tokio::task::spawn(async move {
                    ex.run().await;
                }));
            }
            // let mut outputs = Vec::with_capacity(tasks.len());
            // for task in tasks {
            //     outputs.push(task.await.unwrap());
            // }
            // println!("{:?}", outputs);
        }
    }
}
