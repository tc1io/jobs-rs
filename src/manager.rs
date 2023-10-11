use crate::executor::Executor;
use crate::job::{JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::collections::HashMap;

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
    pub fn register(&mut self, name: String, job: impl JobAction + 'static) {
        self.executors.insert(
            name.into(),
            Executor::new(self.job_repo.clone(), self.lock_repo.clone(), job),
        );
    }
}
