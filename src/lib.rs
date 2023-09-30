// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use tokio::time::{sleep, Duration};

#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

#[async_trait]
pub trait Job {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

pub struct JobManager<R, T> {
    pub job_repo: R,
    job: Option<T>,
    job_info: Option<JobInfo>,
}

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

impl<R: JobsRepo, T: Job> JobManager<R, T> {
    pub fn new(job_repo: R) -> Self {
        Self {
            job_repo,
            job_info: None,
            job: None,
        }
    }

    pub async fn register(&mut self, name: String, schedule: Schedule, job: T) {
        let name1 = name.clone();

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
            Some(result) => println!("Result:{} ", result),
            None => println!("job not found!"),
        }

        self.job = Some(job);
        self.job_info = Some(job_info.clone());
        self.job_repo
            .create_job(job_info)
            .await
            .expect("TODO: panic message");
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        loop {
            self.run().await?;
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        println!("Run");
        // let job = self.job.as_ref().unwrap().clone();
        let ji = self.job_info.as_ref().unwrap().clone();
        let name = ji.clone().name;

        // TODO: add get_job_info().......

        // let (tx1, rx1) = oneshot::channel();
        // let (tx2, rx2) = oneshot::channel();

        // let lock_handle = tokio::spawn(async move {
        let xx = lock_refresher();
        let yy = self.job.as_mut().unwrap().call(ji.clone().state);

        // async move{
        //     xx.await;
        //     yy.await;
        // }
        // let _ = tx1.send("done");
        // });

        // let job_handle = tokio::spawn(async move {
        // let state = job.clone().call(ji.clone().state).await.unwrap();
        // let _ = tx2.send(state);
        // });

        let f = tokio::select! {
            foo = xx => {
                match foo {
                    Ok(_) => Err(1),
                    Err(_) => Err(2),
                }
            }
            bar = yy => {
                match bar {
                    Ok(state) => Ok(state),
                    Err(_) => Err(4),
                }
            }
            // val = rx1 => {
            //     println!("stop signal received from refresh job. stopping job now!!");
            //     job_handle.abort();
            //     println!("job stopped!!");
            // }
            // new_state = rx2 => {
            //     let s = new_state.unwrap();
            //     println!("stop signal received from job. stopping refresh job now!!");
            //     self.job_repo.save_state(name, s.clone()).await.unwrap();
            //     lock_handle.abort();
            //     println!("refresh job stopped!!");
            // }
        };
        println!("{:?}", f);
        println!("all done!!!");

        Ok(())
    }
}

async fn lock_refresher() -> Result<(), Error> {
    // use a future loop function instead
    // use select along with timer

    loop {
        println!("refreshing lock");
        // Err(Error::,)
        sleep(Duration::from_secs(10)).await;
        // Ok(())
        // sleep(Duration::from_millis(100));
        // println!("done");
    }
    Ok(())
}
