use crate::error::Error;
use crate::executor::Executor;
use crate::job::{Job, JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use tokio::sync::oneshot;
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
            let jobs: Vec<Job> = self.jobs.clone().into_iter().collect();
            for mut job in jobs {
                let mut ex =
                    Executor::new(job.clone(), self.job_repo.clone(), self.lock_repo.clone());
                tokio::task::spawn(async {
                    let (tx, rx) = oneshot::channel();
                    tokio::task::spawn(async move {
                        ex.run().await;
                        tx.send("done");
                    });
                    rx.await
                        .map_err(|e| Error::GeneralError {
                            description: String::from("not done"),
                        })
                        .unwrap();
                });
            }
        }
    }
}
