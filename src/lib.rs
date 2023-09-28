// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

#[async_trait]
pub trait Job {
    async fn call(&self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

pub struct JobManager {
    job_repo: Box<dyn JobsRepo>,
    job: Option<Rc<dyn Job>>,
    job_info: Option<JobInfo>,
}

// #[derive(Clone)]
// pub struct JobConfig {
//     pub job: Option<Rc<dyn Job>>,
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub schedule: Schedule,
    pub state: Vec<u8>,
}

#[async_trait]
pub trait JobsRepo {
    async fn create_job(&mut self, job_info: JobInfo) -> Result<bool, Error>;
    async fn get_job_info(&mut self, name: String) -> Result<Option<JobInfo>, Error>;
    async fn save_state(&mut self, name: String, state: Vec<u8>) -> Result<bool, Error>;
}

impl JobManager {
    pub fn new(job_repo: impl JobsRepo + 'static) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job_info: None,
            job: None,
        }
    }

    pub async fn register(&mut self, name: String, schedule: Schedule, job: impl Job + 'static) {
        let state = Vec::<u8>::new();
        let job_info = JobInfo {
            name,
            schedule,
            state,
        };
        self.job = Some(Rc::new(job));
        self.job_info = Some(job_info.clone());
        self.job_repo
            .create_job(job_info)
            .await
            .expect("TODO: panic message");
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        println!("Run");

        let ji = self.job_info.as_ref().unwrap();

        // get job info from db

        // if not present - return to caller

        // From DB...
        let state = Vec::<u8>::new();
        let new_state = self
            .job
            .as_ref()
            .unwrap()
            .call(ji.clone().state)
            .await
            .unwrap();
        self.job_repo
            .as_mut()
            .save_state(ji.clone().name, new_state)
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
