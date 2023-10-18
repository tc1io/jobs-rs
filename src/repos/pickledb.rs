use crate::error::Error;
use crate::job::{Job, JobConfig, JobName, JobRepo};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use pickledb::PickleDb;
use std::sync::Arc;
use std::thread::sleep;
use tokio::sync::RwLock;
use tokio::time::interval;
use crate::lock::{LockData, LockRepo};


#[derive(Clone)]
pub struct Repo {
    pub(crate) db: Arc<RwLock<PickleDb>>,
}

impl Repo {
    pub fn new(db: PickleDb) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}

#[async_trait]
impl JobRepo for Repo {
    async fn create_job(&mut self, job: JobConfig) -> Result<bool, Error> {
        println!("create job");
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true)) // TODO
            .map_err(|e| Error::GeneralError {
                description: "job creation failed".to_string(),
            })?

    }
    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>, Error> {
        Ok(self
            .db
            .write()
            .await
            .get::<JobConfig>(name.as_ref()))
    }

    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
        let name1 = name.clone();
        let mut job = self.get_job(name.into()).await.unwrap().unwrap();

        job.state = state;
        // let name1 = name.clone()
        // job.last_run = Utc::now().timestamp_millis();
        self.db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .set((name1.as_ref() ), &job)
            .unwrap();
        println!("state saved");
        Ok(true)
    }
}

#[async_trait]
impl LockRepo for Repo {
    async fn acquire_lock(&mut self, job: JobConfig) -> Result<bool,Error > {
        println!("acquire lock");
        let mut acquire = false;
       // TODO: try functional approach
        let existing_lock = self
            .db
            .read()
            .await
            .get::<JobConfig>(job.name.as_ref());
        // .unwrap();
        // .map_err(|e| JobError::DatabaseError(e.to_string()))?

        match existing_lock {
            Some(lock) => {
                if lock.lock.expires < Utc::now().timestamp_millis() {
                    acquire = true;
                }
            }
            None => acquire = true,
        }
        if acquire {
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true))
            .map_err(|e| Error::GeneralError { description: "lock error".to_string() })?
        } else {
            Ok(false)
        }
    }

    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, Error> {
    //     let mut refresh_interval = interval(Duration::from_secs(lock_data.ttl.as_secs() / 2));
    //     loop {
    //         refresh_interval.tick().await;
    //         println!("refreshing lock");
    //         // TODO: try functional approach
    //         let existing_lock = self
    //             .db
    //             .read()
    //             .await
    //             // .map_err(|e| JobError::DatabaseError(e.to_string()))?
    //             .get::<LockData>(lock_data.job_name.as_str());
    //         match existing_lock {
    //             // match existing_lock {
    //             Some(mut lock) => {
    //                 dbg!("some");
    //                 if lock.expires < Utc::now().timestamp_millis() {
    //                     println!("lock expired. unable to refresh. Try again");
    //                     Err(JobError::LockError(
    //                         format!("lock expired. unable to refresh").to_string(),
    //                     ))
    //                     // Ok(false)
    //                 } else {
    //                     dbg!("some else");
    //                     lock.expires = Utc::now()
    //                         .timestamp_millis()
    //                         .add(lock.ttl.as_millis() as i64);
    //                     lock.version = lock.version.add(1);
    //                     dbg!(lock.job_name.as_str());
    //                     self.db
    //                         .write()
    //                         .await
    //                         .set(lock.job_name.as_str(), &lock)
    //                         .map_err(|e| JobError::DatabaseError(e.to_string()))?;
    //                     // .map_err(|e| JobError::DatabaseError(e.to_string()))?
    //                     // .set(lock.job_name.as_str(), &lock)
    //                     // .await
    //                     // .map_err(|e| JobError::DatabaseError(e.to_string()))?;
    //                     // sleep_interval.tick().await;
    //                     // let foo = self
    //                     //     .db
    //                     //     .write()
    //                     //     .await
    //                     //     // .map_err(|e| JobError::DatabaseError(e.to_string()))?
    //                     //     .set::<LockData>(lock.job_name.as_str(), &lock)
    //                     //     // .map_err(|e| JobError::DatabaseError(e.to_string()))
    //                     //     .unwrap();
    //                     println!("lock refreshed");
    //                     Ok(true)
    //                 }
    //             }
    //             None => {
    //                 println!("lock not found. unable to refresh. Try again");
    //                 // Err(JobError::LockError(format!(
    //                 //     "lock not found. unable to refresh"
    //                 // )))
    //             }
    //         }?;
    //         dbg!("here");
    //         sleep(Duration::from_secs(lock_data.ttl.as_secs() / 2));
    //     }
    // }
}

