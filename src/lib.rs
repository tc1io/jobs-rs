// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use serde::{Deserialize, Serialize};
use std::fmt::Error;
use mongodb::bson::Document;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;


#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

// #[derive(Clone, Serialize, Deserialize)]
// pub struct JobData {
//     // Define your document fields here
//     name: String,
//     // state: String,
// }


pub trait Job {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    fn call(&self, state: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
}

pub struct JobManager {
    job_repo: Box<dyn JobsRepo>,
    job: Option<Rc<dyn Job>>,
    job_info: Option<JobInfo>,
}

#[derive(Clone)]
pub struct JobConfig {
    pub job: Option<Rc<dyn Job>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub schedule: Schedule,
}

pub trait JobsRepo {

    // fn create(&self, document: &JobData) -> Result<(), mongodb::error::Error>;
    // fn read(&self, filter: Document) -> Result<Option<JobData>, mongodb::error::Error>;
    // fn find_by_jobname(&self, name: &str) -> Result<Option<JobData>, mongodb::error::Error>;

    fn create_job(&mut self, job_info: JobInfo) -> Result<bool, mongodb::bson::ser::Error>;
    fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, Error>;

}

impl JobManager {
    pub fn new(job_repo: impl JobsRepo + 'static) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job_info: None,
            job: None,
        }
    }

    pub fn register(&mut self, name: &str, schedule: Schedule, job: impl Job + 'static) {
        println!("inside register...");
        println!("{}", name.clone());
        let job_info = JobInfo { name: name.to_string(), schedule };
        self.job = Some(Rc::new(job));
        self.job_repo.create_job(job_info);


        // let jd = JobData {
        //     name: name.to_string(),
        // };
        // self.job_repo.create(&jd);
        // self.job = Some(job)
    }
        pub async fn run(&self) -> Result<(), i32> {
            println!("Run");

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

