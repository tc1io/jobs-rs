use crate::job::{Job, JobAction, JobRepo};
use crate::lock::LockRepo;

pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    job: Job,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(job_repo: J, lock_repo: L, job_action: impl JobAction + 'static) -> Self {
        Executor {
            job_repo,
            lock_repo,
            job: Job::new_with_action(job_action),
        }
    }
}
