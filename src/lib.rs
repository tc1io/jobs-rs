// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use std::fmt::Error;
use mongodb::bson::Document;
use serde::Serialize;
use serde::Deserialize;


pub struct Schedule {}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobData {
    // Define your document fields here
    name: String,
    // state: String,
}


pub trait Job {
    fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

pub struct JobManager {
    job_repo: Box<dyn JobsRepo>,
    job: Option<Box<dyn Job>>,
    // schedule: Schedule,
    // config: Option<JobConfig>,
}

pub struct JobConfig {
    job: Option<Box<dyn Job>>,
    schedule: Schedule,
}

pub trait JobsRepo {
    fn create_job(&mut self, name: &str, schedule: Schedule) -> Result<(), Error>;
    fn create(&self, document: &JobData) -> Result<(), mongodb::error::Error>;
    // fn read(&self, filter: Document) -> Result<Option<JobData>, mongodb::error::Error>;
    // fn find_by_jobname(&self, name: &str) -> Result<Option<JobData>, mongodb::error::Error>;
}

impl JobManager {
    pub fn new(job_repo: impl JobsRepo + 'static ) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job: None,
        }
    }

    pub fn register(&mut self, name: &str, schedule: Schedule, job: impl Job + 'static) {
        // let schedule1 = schedule.clone();
        // let job_config = JobConfig {
        //     job: None,
        //     schedule:schedule1,
        // };
        println!( "inside register...");
        println!("{}", name.clone());

        self.job_repo.create_job(name, schedule);
        self.job = Some(Box::new(job));

        let jd = JobData {
            name: name.to_string(),
        };
        self.job_repo.create(&jd);
        // self.job = Some(job)
    }

    pub async fn run(&mut self) -> Result<(), i32> {
        println!("Run");
        // From DB...
        let state = Vec::<u8>::new();
        let j = self.job.as_mut().unwrap().call(state).unwrap();

        // match job.call(state).await {
        //     Err(e) => todo!(),
        //     Ok(s) => todo!(), // save new state
        // }
        //

        Ok(())
    }
}
