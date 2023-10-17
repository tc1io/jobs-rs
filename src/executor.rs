use std::sync::Arc;
use crate::error::Error;
use crate::job::{Job, JobAction, JobConfig, JobRepo};
use crate::lock::{LockData, LockRepo};
use crate::job::Schedule;

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
        job_action: impl JobAction + Send + Sync + 'static + std::marker::Send,
    ) -> Self {
        Executor {
            job_repo,
            lock_repo,
            job: Job::new_with_action(name, job_action),
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        dbg!("inside run");
        // let mut action = self.job.action.lock().await;
        // other logic will be added
        let name = self.job.clone().name.clone();
        let name1 = name.clone();
        let ji = self
            .job_repo
            .create_job(JobConfig {
                name,
                state: vec![],
                // action: Arc::new(()),
                schedule: Schedule { expr: "".to_string() },
                enabled: false,
                last_run: 0,
                lock: LockData { expires: 0, version: 0 },
            })
            .await?;
        if ji {
            println!("job created")
        }

       match self
            .job_repo
            .get_job(name1.clone())
            .await? {
           None => {}
           Some(value) => {
               // Job::should_run_now(value).unwrap().expect("TODO: panic message")
           }
       }
        // let ji = self
        //     .job_repo
        //     .create_job(ji.clone())
        //     .await?;

        // let _r = self.job_repo.create_job(ji.clone()).await?;
        Ok(())
    }
}
