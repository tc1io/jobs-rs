// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

pub trait Job {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    fn call(&self, state: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
}

pub struct JobManager {
    pub job_repo: Box<dyn JobsRepo>,
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
}

impl fmt::Display for JobInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize how JobInfo is formatted as a string here
        write!(f, "name: {}, schedule {}" , self.name, self.schedule.expr)
    }
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize how JobInfo is formatted as a string here
        write!(f, "expr: {}", self.expr)
    }
}

#[async_trait]
pub trait JobsRepo {
    async fn create_job(&mut self, job_info: JobInfo) -> Result<bool, Error>;
    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, Error>;
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
        let name1 = name.clone();
        let job_info = JobInfo { name, schedule };

        match self.job_repo.get_job_info(name1.as_str()).await.expect("TODO: panic message") {
            // Some(result) => println!("Result:{} ", result),
            Some(result) => println!("Result:{} ", result),
            None => println!("job not found!"),
        }

        self.job = Some(Rc::new(job));
        self.job_repo
            .create_job(job_info)
            .await
            .expect("TODO: panic message");


    }

    pub async fn run(&self) -> Result<(), i32> {
        println!("Run");

        // get job info from db

        // if not present - return to caller

        // From DB...
        let state = Vec::<u8>::new();
        let j = self
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
