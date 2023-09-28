// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

#[async_trait]
pub trait Job {
    async fn call(&self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
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
    pub state: Vec<u8>,
}

impl fmt::Display for JobInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize how JobInfo is formatted as a string here
        write!(f, "name: {}, schedule {}", self.name, self.schedule.expr)
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
        let name1 = name.clone();
        // let job_info = JobInfo { name, schedule, state: vec![] };
        //
        // match self.job_repo.get_job_info(name1.as_str()).await.expect("TODO: panic message") {
        //     // Some(result) => println!("Result:{} ", result),
        //     Some(result) => println!("Result:{} ", result),
        //     None => println!("job not found!"),
        // }

        let state = Vec::<u8>::new();
        let job_info = JobInfo {
            name,
            schedule,
            state,
        };

        match self
            .job_repo
            .get_job_info(name1.as_str())
            .await
            .expect("TODO: panic message")
        {
            // Some(result) => println!("Result:{} ", result),
            Some(result) => println!("Result:{} ", result),
            None => println!("job not found!"),
        }

        self.job = Some(Rc::new(job));
        self.job_info = Some(job_info.clone());
        self.job_repo
            .create_job(job_info)
            .await
            .expect("TODO: panic message");
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        println!("Run");
        let job = self.job.as_ref().unwrap();
        let ji = self.job_info.as_ref().unwrap();

        // let (tx1, rx1) = oneshot::channel();
        // let (tx2, rx2) = oneshot::channel();

        let handle = tokio::spawn(async {
            lock_refresher().await.expect("TODO: panic message");
        });

        // if not present - return to caller

        // From DB...
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
        handle.abort();

        // tokio::select! {
        //     val = rx1 => {
        //         println!("rx1 completed first with {:?}", val);
        //     }
        //     val = rx2 => {
        //         println!("rx2 completed first with {:?}", val);
        //     }
        // }
        // handle.abort();

        println!("all done!!!");

        // match job.call(state).await {
        //     Err(e) => todo!(),
        //     Ok(s) => todo!(), // save new state
        // }
        //

        Ok(())
    }
}

async fn lock_refresher() -> Result<(), Error> {
    // loop {
    println!("refreshing lock");
    sleep(Duration::from_secs(100)).await;
    // }
    Ok(())
}
