// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule as CronSchedule;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::time::{Duration as Dur, SystemTime, UNIX_EPOCH};
use std::{fmt, println};
use tokio::time::{sleep, Duration};
mod test;

#[derive(Debug)]
pub enum JobError {
    DatabaseError(String),
    LockError(String),
}

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
    pub lock_repo: Box<dyn LockRepo + Sync + Send + 'static>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub schedule: Schedule,
    pub state: Vec<u8>,
    pub enabled: bool,
    pub last_run: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub status: String,
    pub job_id: String,
    pub ttl: Duration,
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

impl fmt::Display for LockInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize how JobInfo is formatted as a string here
        write!(f, "status: {}, job_id: {}", self.status, self.job_id)
    }
}

#[async_trait]
pub trait JobsRepo {
    async fn create_job(&mut self, job_info: JobInfo) -> Result<bool, JobError>;
    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    async fn save_state(&mut self, name: String, state: Vec<u8>) -> Result<bool, JobError>;
}

#[async_trait]
pub trait LockRepo {
    async fn lock_refresher1(&self) -> Result<(), JobError>;
    async fn add_lock(&mut self, li: LockInfo) -> Result<bool, JobError>;
    async fn get_lock(&mut self, name: &str) -> Result<Option<LockInfo>, JobError>;
}

impl<R: JobsRepo, T: Job> JobManager<R, T> {
    pub fn new(job_repo: R, lock_repo: impl LockRepo + Sync + Send + 'static) -> Self {
        Self {
            job_repo,
            job_info: None,
            job: None,
            lock_repo: Box::new((lock_repo)),
        }
    }
    // pub async fn register(
    //     &mut self,
    //     name: String,
    //     schedule: Schedule,
    //     job: impl Job + Sync + Send + 'static,
    // ) {

    pub async fn register(&mut self, name: String, schedule: Schedule, job: T) {
        let name1 = name.clone();

        let state = Vec::<u8>::new();
        // pub fn now() -> Timestamp {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration since UNI epoch must be > 0");
        // Timestamp {
        //     epoch: 1,
        //     unixmillis: duration_since_epoch.as_millis() as u64,
        // }
        // }
        // println!("{:?}", duration_since_epoch.as_millis().clone());
        // println!("{:?}", Utc::now());
        let job_info = JobInfo {
            name,
            schedule,
            state,
            enabled: true,
            last_run: DateTime::<Utc>::default().timestamp_millis(),
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
        let r = self.job_repo.create_job(job_info).await;
        match r {
            Ok(_) => {
                println!("job created successful");
                // Ok(())
            }
            Err(err) => Err(JobError::DatabaseError("Failed to create job".to_string()))
                .expect("TODO: panic message"),
            // Err(err)
        };

        let l_info = LockInfo {
            status: "locked".to_string(),
            job_id: "dummy".to_string(),
            ttl: Default::default(),
        };
        self.lock_repo
            .add_lock(l_info)
            .await
            .expect("TODO: panic message");
        match self
            .lock_repo
            .get_lock(name1.as_str())
            .await
            .expect("TODO: panic message")
        {
            // Some(result) => println!("Result:{} ", result),
            Some(result) => println!("Result: {}", result),
            None => println!("lock not found!"),
        }
    }

    //     pub async fn run(&mut self) -> Result<(), JobError> {
    //         println!("Run");
    //         let job = self.job.as_ref().unwrap().clone();
    //         let ji = self.job_info.as_ref().unwrap().clone();
    //         let xx = job.clone();
    //
    //         let (tx1, rx1) = oneshot::channel();
    //         let (tx2, rx2) = oneshot::channel();
    //
    //
    //
    //
    //         let lock_handle = tokio::spawn(async  {
    //                 // self.lock_repo.lock_refresher1().await.expect("TODO: panic message");
    //                 lock_refresher().await.expect("TODO: panic message").clone();
    //                     let _ = tx1.send("done");
    //         });
    //
    //         let job_handle = tokio::spawn(async move {
    //             let new_state = job.clone().call(ji.clone().state).await.unwrap();
    //             let _ = tx2.send("done");
    //         });
    //
    //
    //         // self.job_repo
    //         //     .as_mut()
    //         //     .save_state(ji.clone().name, new_state)
    //         //     .await
    //         //     .unwrap();
    //
    //         tokio::select! {
    //             val = rx1 => {
    //                 job_handle.abort();
    //                 println!("rx1 completed first with {:?}", val);
    //             }
    //             val = rx2 => {
    //                 lock_handle.abort();
    //                 println!("rx2 completed first with {:?}", val);
    //             }
    // =======
    pub async fn start(&mut self) -> Result<(), Error> {
        loop {
            self.run().await?;
            sleep(Duration::from_secs(10)).await;
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        println!("Run");
        // let job = self.job.as_ref().unwrap().clone();
        let name = &self.job_info.as_ref().unwrap().name;
        let ji = self
            .job_repo
            .get_job_info(name.as_str())
            .await
            .unwrap()
            .unwrap()
            .clone();
        // let name = ji.clone().name;

        if ji.clone().should_run_now().await.unwrap() {
            println!("yes");

            // let schedule = CronSchedule::from_str(ji.schedule.expr.as_str()).unwrap();
            // let zz = schedule
            //     .upcoming(Utc)
            //     .next()
            //     .map(|t| t.timestamp_millis() > Utc::now().timestamp_millis())
            //     .unwrap_or(false);
            // dbg!("{:?}", zz);
            // let duration_since_epoch = SystemTime::now()
            //     .duration_since(SystemTime::UNIX_EPOCH)
            //     .expect("Duration since UNI epoch must be > 0");
            //
            // println!("{:?}", duration_since_epoch.as_millis().clone());
            // if xx > Utc::now().timestamp_millis() {
            //     println!("{:?}", Utc::now().timestamp_millis());
            // }
            // for datetime in xx.take(1) {
            //     println!("-> {}", datetime);
            // }

            // if let Ok(next) = parse(ji.schedule.expr.as_str(), &Utc::now()) {
            //     println!("{:?}", next.timestamp_millis());
            //     if next > Utc::now() {
            //         println!("when: {:?}", next);
            //     }
            // }

            // TODO: add get_job_info().......
            // let ji2 = self
            //     .job_repo
            //     .get_job_info(name.as_ref())
            //     .await
            //     .unwrap()
            //     .unwrap();

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
                        Ok(state) => {
                            println!("before saving state");
                            self.job_repo.save_state(ji.name, state).await;
                            Ok(())
                            }
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
        }

        Ok(())
    }
}

impl JobInfo {
    async fn should_run_now(self) -> Result<bool, Error> {
        if !self.enabled {
            return Ok(false);
        }
        dbg!("{:?}", self.last_run);
        // if self.last_run.eq(&0) {
        //     return Ok(true);
        // }
        let dt = UNIX_EPOCH + Dur::from_millis(self.last_run as u64);
        // let date_time = DateTime::<Utc>::(self.last_run, 0).unwrap();
        let date_time = DateTime::<Utc>::from(dt);
        dbg!("", date_time);
        let schedule = CronSchedule::from_str(self.schedule.expr.as_str()).unwrap();
        let ff = schedule.after(&date_time).next().unwrap_or(Utc::now());
        // .next()
        // .map(|t| t.timestamp_millis())
        // .unwrap();
        dbg!("", ff);
        let next_scheduled_run = schedule
            .after(&date_time)
            .next()
            .map(|t| t.timestamp_millis())
            .unwrap_or(0);
        dbg!(
            "{:?}-----{:?}---- {:?}",
            self.last_run,
            next_scheduled_run,
            Utc::now().timestamp_millis()
        );
        if next_scheduled_run.lt(&Utc::now().timestamp_millis()) {
            return Ok(true);
        }
        Ok(false)
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
