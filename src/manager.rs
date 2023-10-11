use crate::executor::Executor;
use crate::job::{JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::collections::HashMap;

pub struct JobManager<J, L>
where
    J: JobRepo,
    L: LockRepo,
{
    job_repo: J,
    lock_repo: L,
    executors: HashMap<JobName, Executor>,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> JobManager<J, L> {
    pub fn new(job_repo: J, lock_repo: L) -> Self {
        JobManager {
            job_repo,
            lock_repo,
            executors: HashMap::new(),
        }
    }

    pub fn register(self, name: String, job: impl JobAction) -> Self {
        self
    }
}
