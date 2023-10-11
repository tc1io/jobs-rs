use crate::error::Error;
use crate::executor::Executor;
use crate::job::{JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::collections::HashMap;
use tokio::task::JoinSet;

pub struct JobManager<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    executors: HashMap<JobName, Executor<J, L>>,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> JobManager<J, L> {
    pub fn new(job_repo: J, lock_repo: L) -> Self {
        JobManager {
            job_repo,
            lock_repo,
            executors: HashMap::new(),
        }
    }
    pub fn register(&mut self, name: String, job_action: impl JobAction + 'static) {
        self.executors.insert(
            name.clone().into(),
            Executor::new(
                name.into(),
                self.job_repo.clone(),
                self.lock_repo.clone(),
                job_action,
            ),
        );
    }
    pub async fn start(&mut self) -> Result<(), Error> {
        dbg!("1");
        let mut executors: Vec<(JobName, Executor<J, L>)> =
            self.executors.clone().into_iter().collect();
        let mut items = Vec::new();
        for (k, v) in executors.as_mut_slice() {
            dbg!(k);
            items.push(v.run())
        }
        for mut item in items {
            // item.await?;
            tokio::join!(item);
            // item.job.action.lock().await.call(Vec::new()).await?;
        }
        Ok(())
    }
}
