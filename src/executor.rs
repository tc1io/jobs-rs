use crate::error::Error;
use crate::job::{Job, JobAction, JobRepo};
use crate::lock::LockRepo;

#[derive(Clone)]
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    pub job: Job,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        name: String,
        job_repo: J,
        lock_repo: L,
        job_action: impl JobAction + 'static,
    ) -> Self {
        Executor {
            job_repo,
            lock_repo,
            job: Job::new_with_action(name, job_action),
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        dbg!("inside run");
        let mut action = self.job.action.lock().await;
        // other logic will be added
        let name = self.job.clone().name;
        let ji = self
            .job_repo
            .get_job(name.clone())
            .await?;
        // if ji.clone().should_run_now().await.unwrap() {
        //    println!("yes");
        // }
        let _xx = action
            .call(self.job.name.clone().into(), Vec::new())
            .await?;
        Ok(())
    }
}
