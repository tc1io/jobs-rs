use crate::error::Error;
use crate::error::Error::GeneralError;
use crate::job::{Job, JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::sync::Arc;
use tokio::sync::Mutex;

// #[derive(Clone)]
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    // pub job: Job,
    job_name: JobName,
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        job_name: JobName,
        action: Arc<Mutex<dyn JobAction + Send + Sync>>,
        job_repo: J,
        lock_repo: L,
    ) -> Self {
        Executor {
            job_repo,
            lock_repo,
            job_name,
            action,
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        // dbg!("inside run");
        let mut action = self.action.lock().await;
        // .map_err(|e| GeneralError {
        //     description: e.to_string(),
        // })?;
        // other logic will be added
        let _xx = action
            .call(self.job_name.clone().into(), Vec::new())
            .await?;
        Ok(())
    }
}
