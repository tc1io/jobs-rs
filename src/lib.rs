// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt, println};
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
    pub job_repo: Box<dyn JobsRepo + Sync + Send + 'static>,
    job: Option<Arc<dyn Job + Sync + Send + 'static>>,
    job_info: Option<JobInfo>,
    pub lock_repo: Box<dyn LockRepo + Sync + Send + 'static>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub schedule: Schedule,
    pub state: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub status: String,
    job_id: String,
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

#[async_trait]
pub trait LockRepo {
    async fn lock_refresher1(&self) -> Result<(), Error>;
    async fn add_lock(&mut self, li: LockInfo) -> Result<bool, Error>;
    async fn get_lock(&mut self, name: &str) -> Result<Option<LockInfo>, Error>;
}

    impl JobManager {
    pub fn new(job_repo: impl JobsRepo + Sync + Send + 'static, lock_repo: impl LockRepo + Sync + Send + 'static) -> Self {
        Self {
            job_repo: Box::new(job_repo),
            job_info: None,
            job: None,
            lock_repo: Box::new((lock_repo)),
        }
    }
    pub async fn register(
        &mut self,
        name: String,
        schedule: Schedule,
        job: impl Job + Sync + Send + 'static,
    ) {
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

        self.job = Some(Arc::new(job));
        self.job_info = Some(job_info.clone());
        self.job_repo
            .create_job(job_info)
            .await
            .expect("TODO: panic message");
        let l_info = LockInfo {
            status: "locked".to_string(),
            job_id: "dummy".to_string(),
        };
        self.lock_repo
            .add_lock(l_info)
            .await
            .expect("TODO: panic message");
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        println!("Run");
        let job = self.job.as_ref().unwrap().clone();
        let ji = self.job_info.as_ref().unwrap().clone();
        let xx = job.clone();

        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();


        let lock_handle = tokio::spawn(async  {
                // self.lock_repo.lock_refresher1().await.expect("TODO: panic message");
                lock_refresher().await.expect("TODO: panic message").clone();
                    let _ = tx1.send("done");
        });

        let job_handle = tokio::spawn(async move {
            let new_state = job.clone().call(ji.clone().state).await.unwrap();
            let _ = tx2.send("done");
        });



        // self.job_repo
        //     .as_mut()
        //     .save_state(ji.clone().name, new_state)
        //     .await
        //     .unwrap();

        tokio::select! {
            val = rx1 => {
                job_handle.abort();
                println!("rx1 completed first with {:?}", val);
            }
            val = rx2 => {
                lock_handle.abort();
                println!("rx2 completed first with {:?}", val);
            }
        }
        println!("all done!!!");

        Ok(())
    }
}

async fn lock_refresher() -> Result<(), Error> {
    loop {
        println!("refreshing lock");
        sleep(Duration::from_secs(5)).await;
        // sleep(Duration::from_millis(100));
        println!("done");
    }
    Ok(())
}
