// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use std::fmt::Error;
use example;

pub struct Schedule {}

pub trait Job {
    fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

pub struct JobManager {
    job_repo: Box<dyn JobsRepo>,
    job: Option<Box<dyn Job>>,
    db_repo: ox<dyn Repository>
    // schedule: Schedule,
    // config: Option<JobConfig>,
}

pub struct JobConfig {
    job: Option<Box<dyn Job>>,
    schedule: Schedule,
}

pub trait JobsRepo {
    fn create_job(&mut self, name: &str, schedule: Schedule) -> Result<(), Error>;
}

impl JobManager {
    pub fn new(job_repo: impl JobsRepo + 'static, db_repo: impl Repository + 'static ) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job: None,
            db_repo: Box::new(db_repo),
        }
    }

    pub fn register(&mut self, name: &str, schedule: Schedule, job: impl Job + 'static) {
        // let job_config = JobConfig {
        //     job: Box::new(job),
        //     schedule: schedule,
        // };
        self.job_repo.create_job(name, schedule);
        self.job = Some(Box::new(job));
        // self.job = Some(job)
    }

    pub async fn run(&mut self) -> Result<(), i32> {
        println!("Run");
        self.db_repo.
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
