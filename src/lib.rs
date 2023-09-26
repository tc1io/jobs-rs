// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Clone, Serialize)]
pub struct Schedule {
    pub expr: String,
}

pub trait Job {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    fn call(&self, state: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
}

pub struct JobManager {
    job_repo: Box<dyn JobsRepo>,
    job_config: Option<JobConfig>,
}

#[derive(Clone)]
pub struct JobConfig {
    pub name: String,
    pub schedule: Schedule,
    pub job: Option<Rc<dyn Job>>,
}

pub trait JobsRepo {
    fn create_job(&mut self, job_config: JobConfig) -> Result<bool, Error>;
    fn get_job_config(&mut self, name: &str) -> Result<Option<JobConfig>, Error>;
}

impl JobManager {
    pub fn new(job_repo: impl JobsRepo + 'static) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job_config: None,
        }
    }

    pub fn register(&mut self, name: String, schedule: Schedule, job: impl Job + 'static) {
        let job_config = JobConfig {
            name: name,
            schedule: schedule,
            job: Some(Rc::new(job)),
        };
        self.job_config = Some(job_config.clone());
        self.job_repo.create_job(job_config);
    }

    pub async fn run(&self) -> Result<(), i32> {
        println!("Run");

        // get job info from db

        // if not present - return to caller

        // From DB...
        let state = Vec::<u8>::new();
        let j = self
            .job_config
            .as_ref()
            .unwrap()
            .job
            .as_ref()
            .unwrap()
            .call(state.clone())
            .await
            .unwrap();

        // match job.call(state).await {
        //     Err(e) => todo!(),
        //     Ok(s) => todo!(), // save new state
        // }
        //

        Ok(())
    }
}
