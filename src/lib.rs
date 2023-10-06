use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule as CronSchedule;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::fmt::{Error, Formatter};
use std::ops::Add;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration as Dur, UNIX_EPOCH};
use std::{fmt, format, println};
use tokio::time::{sleep, Duration};
mod job;
#[derive(Debug)]
pub enum JobError {
    DatabaseError(String),
    LockError(String),
    JobRunError(String),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub expr: String,
}

#[async_trait]
pub trait JobRunner {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}
#[derive(Clone)]
pub struct Job {
    pub job_info: JobInfo,
    pub runner: Arc<RwLock<dyn JobRunner + Sync + Send + 'static>>,
    // pub lock: Lock,
}

pub struct JobManager<R, L> {
    pub job_repo: R,
    pub lock_repo: L,
    jobs: Vec<Job>,
}

// TODO: decouple job_config and job_info
// #[derive(Clone, Serialize, Deserialize)]
// pub struct JobConfig {
//
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub schedule: Schedule,
    pub state: Vec<u8>,
    pub enabled: bool,
    pub last_run: i64,
    pub lock_ttl: Duration,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LockData {
    // pub status: String,
    pub job_name: String,
    pub ttl: Duration,
    pub expires: i64,
    pub version: i8,
}

#[async_trait]
pub trait JobsRepo {
    async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}

#[async_trait]
pub trait LockRepo {
    async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
}

impl<R: JobsRepo, L: LockRepo> JobManager<R, L> {
    pub fn new(job_repo: R, lock_repo: L) -> Self {
        Self {
            job_repo,
            lock_repo,
            jobs: Vec::new(),
        }
    }
    pub async fn register(
        &mut self,
        name: &'static str,
        schedule: Schedule,
        job_runner: impl JobRunner + Sync + Send + 'static,
    ) -> Result<(), JobError> {
        // TODO: do something with existing job.. maybe change to create_or_update()....
        let existing_job = self.job_repo.get_job(name).await?;

        let state = Vec::<u8>::new();
        let job_info = JobInfo {
            name: name.to_string(),
            schedule: schedule.clone(),
            state: state.clone(),
            enabled: true,
            last_run: DateTime::<Utc>::default().timestamp_millis(),
            lock_ttl: Duration::new(10, 0), // TODO: get it from client
        };
        let job = Job {
            job_info: job_info.clone(),
            runner: Arc::new(RwLock::new(job_runner)),
        };
        self.jobs.push(job);

        // let _r = self.job_repo.create_job(job_info).await?;

        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), JobError> {
        loop {
            for mut job in self.jobs.clone() {
                self.run(job).await?;
            }
            sleep(Duration::from_secs(10)).await;
        }
    }
    async fn run(&mut self, job: Job) -> Result<(), JobError> {
        dbg!("new run");
        let name = job.clone().job_info.clone().name;
        let ji = self
            .job_repo
            .get_job(name.clone().as_str())
            .await?
            .unwrap_or(job.clone().job_info);

        let _r = self.job_repo.create_job(ji.clone()).await?;

        if ji.clone().should_run_now()? {

            println!("yes");
            let mut w = job
                .runner
                .write()
                .map_err(|e| JobError::LockError(e.to_string()))?;

            let lock_data = job.clone().init_lock_data();
            let acquire_lock = self.lock_repo.acquire_lock(lock_data.clone()).await?;

            if acquire_lock {
                println!("acquired");
                let refresh_lock = self.lock_repo.refresh_lock(lock_data);
                let job_runner = w.call(job.job_info.state.clone());

                let f = tokio::select! {
                    refreshed = refresh_lock => {
                        dbg!(refreshed);
                        Ok(())
                    }
                    bar = job_runner => {
                        match bar {
                            Ok(state) => {
                                println!("before saving state");
                                self.job_repo.save_state(name.as_str(), state).await;
                                Ok(())
                                }
                            Err(_) => Err(4),
                        }

                    }
                }
                .map_err(|e| JobError::JobRunError(e.to_string()))?;
            }
        }
        Ok(())
    }
}

impl Job {
    fn init_lock_data(&self) -> LockData {
        return LockData {
            job_name: self.job_info.name.clone(),
            ttl: self.job_info.lock_ttl,
            expires: Utc::now()
                .timestamp_millis()
                .add(self.job_info.lock_ttl.as_millis() as i64),
            version: 0,
        };
    }
}

impl JobInfo {
    fn should_run_now(self) -> Result<bool, JobError> {
        if !self.enabled {
            return Ok(false);
        }
        dbg!("{:?}", self.last_run);
        if self.last_run.eq(&0) {
            return Ok(true);
        }
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

// // async fn lock_refresher() -> Result<(), Error> {
// //     // use a future loop function instead
// //     // use select along with timer
// //
// //     loop {
// //         println!("refreshing lock");
// //         // Err(Error::,)
// //         sleep(Duration::from_secs(10)).await;
// //         // Ok(())
// //         // sleep(Duration::from_millis(100));
// //         // println!("done");
// //     }
// //     Ok(())
// }
//     pub async fn run(&mut self) -> Result<(), Error> {
//         println!("Run");
//         // let job = self.job.as_ref().unwrap().clone();
//         let name = &self.job_info.as_ref().unwrap().name;
//         let ji = self
//             .job_repo
//             .get_job_info(name.as_str())
//             .await
//             .unwrap()
//             .unwrap()
//             .clone();
//         // let name = ji.clone().name;
//
//         if ji.clone().should_run_now().await.unwrap() {
//             println!("yes");
//
//             // let schedule = CronSchedule::from_str(ji.schedule.expr.as_str()).unwrap();
//             // let zz = schedule
//             //     .upcoming(Utc)
//             //     .next()
//             //     .map(|t| t.timestamp_millis() > Utc::now().timestamp_millis())
//             //     .unwrap_or(false);
//             // dbg!("{:?}", zz);
//             // let duration_since_epoch = SystemTime::now()
//             //     .duration_since(SystemTime::UNIX_EPOCH)
//             //     .expect("Duration since UNI epoch must be > 0");
//             //
//             // println!("{:?}", duration_since_epoch.as_millis().clone());
//             // if xx > Utc::now().timestamp_millis() {
//             //     println!("{:?}", Utc::now().timestamp_millis());
//             // }
//             // for datetime in xx.take(1) {
//             //     println!("-> {}", datetime);
//             // }
//
//             // if let Ok(next) = parse(ji.schedule.expr.as_str(), &Utc::now()) {
//             //     println!("{:?}", next.timestamp_millis());
//             //     if next > Utc::now() {
//             //         println!("when: {:?}", next);
//             //     }
//             // }
//
//             // TODO: add get_job_info().......
//             // let ji2 = self
//             //     .job_repo
//             //     .get_job_info(name.as_ref())
//             //     .await
//             //     .unwrap()
//             //     .unwrap();
//
//             // let (tx1, rx1) = oneshot::channel();
//             // let (tx2, rx2) = oneshot::channel();
//
//             // let lock_handle = tokio::spawn(async move {
//             let xx = lock_refresher();
//             let yy = self.job.as_mut().unwrap().call(ji.clone().state);
//
//             // async move{
//             //     xx.await;
//             //     yy.await;
//             // }
//             // let _ = tx1.send("done");
//             // });
//
//             // let job_handle = tokio::spawn(async move {
//             // let state = job.clone().call(ji.clone().state).await.unwrap();
//             // let _ = tx2.send(state);
//             // });
//
//             let f = tokio::select! {
//                 foo = xx => {
//                     match foo {
//                         Ok(_) => Err(1),
//                         Err(_) => Err(2),
//                     }
//                 }
//                 bar = yy => {
//                     match bar {
//                         Ok(state) => {
//                             println!("before saving state");
//                             self.job_repo.save_state(ji.name, state).await;
//                             Ok(())
//                             }
//                         Err(_) => Err(4),
//                     }
//                 }
//                 // val = rx1 => {
//                 //     println!("stop signal received from refresh job. stopping job now!!");
//                 //     job_handle.abort();
//                 //     println!("job stopped!!");
//                 // }
//                 // new_state = rx2 => {
//                 //     let s = new_state.unwrap();
//                 //     println!("stop signal received from job. stopping refresh job now!!");
//                 //     self.job_repo.save_state(name, s.clone()).await.unwrap();
//                 //     lock_handle.abort();
//                 //     println!("refresh job stopped!!");
//                 // }
//             };
//             println!("{:?}", f);
//             println!("all done!!!");
//         }
//
//         Ok(())
//     }
// }
