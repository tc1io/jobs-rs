use crate::job::{JobAction, JobRepo};
use crate::lock::LockRepo;

pub struct JobManager<J, L>
where
    J: JobRepo,
    L: LockRepo,
{
    job_repo: J,
    lock_repo: L,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> JobManager<J, L> {
    pub fn new(job_repo: J, lock_repo: L) -> Self {
        JobManager {
            job_repo,
            lock_repo,
        }
    }

    pub fn register(self, job_action: impl JobAction) -> Self {
        self
    }
}
