// use std::fmt::Error;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::{FutureExt, TryFutureExt};
// use jobs::{JobError, JobInfo, JobManager, JobRepo, JobRunner, LockData, LockRepo, Schedule};
use jobs::{error::Error, job::JobAction, job::JobRepo, lock::LockRepo, manager::JobManager};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::Arc;
use chrono::format::Item;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep, Duration};
use jobs::job::{Job, JobName};
use jobs::lock::LockData;
use serde::{Deserializer};

#[tokio::main]
async fn main() {
    let mut db_client = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut lock_client = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let db_repo = DbRepo {
        db: Arc::new(RwLock::new(db_client)),
    };
    let lock_repo = DbRepo {
        db: Arc::new(RwLock::new(lock_client)),
    };

    let job1 = JobImplementer {
        // name: "".to_string(),
    };
    let job2 = JobImplementer {
        // name: "".to_string(),
    };

    let mut manager = JobManager::<DbRepo, DbRepo>::new(db_repo, lock_repo);
    manager.register(String::from("project-updater"), job1).await;
    manager.register(String::from("project-puller"), job2).await;
    let _ = manager.start().await.unwrap();
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]
impl LockRepo for DbRepo {
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error> {
        println!("acquire lock");
        let mut acquire = false;
        // TODO: try functional approach
        let existing_lock = self
            .db
            .read()
            .await
            .get::<LockData>(lock_data.job_name.as_str());
        // .unwrap();
        // .map_err(|e| JobError::DatabaseError(e.to_string()))?
        match existing_lock {
            Some(lock) => {
                if lock.expires < Utc::now().timestamp_millis() {
                    acquire = true;
                }
            }
            None => acquire = true,
        }
        // if acquire {
        self.db
            .write()
            .await
            .set(lock_data.job_name.as_str(), &lock_data)
            .map(|_| Ok(true))
            .map_err(|e| Error::GeneralError { description: "lock error".to_string() })?
        // }
    }


    async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, Error> {
        let mut refresh_interval = interval(Duration::from_secs(lock_data.ttl.as_secs() / 2));
        loop {
            refresh_interval.tick().await;
            println!("refreshing lock");
            // TODO: try functional approach
            let existing_lock = self
                .db
                .read()
                .await
                // .map_err(|e| JobError::DatabaseError(e.to_string()))?
                .get::<LockData>(lock_data.job_name.as_str());
            match existing_lock {
                // match existing_lock {
                Some(mut lock) => {
                    dbg!("some");
                    if lock.expires < Utc::now().timestamp_millis() {
                        println!("lock expired. unable to refresh. Try again");
                        Err(Error::GeneralError{ description: "lock error".to_string() })
                        // Ok(false)
                    } else {
                        dbg!("some else");
                        lock.expires = Utc::now()
                            .timestamp_millis()
                            .add(lock.ttl.as_millis() as i64);
                        lock.version = lock.version.add(1);
                        dbg!(lock.job_name.as_str());
                        self.db
                            .write()
                            .await
                            .set(lock.job_name.as_str(), &lock)
                            .map(|_| Ok(true))
                            .map_err(|e| Error::GeneralError { description: "lock error".to_string() })
                        // .map_err(|e| JobError::DatabaseError(e.to_string()))?
                        // .set(lock.job_name.as_str(), &lock)
                        // .await
                        // .map_err(|e| JobError::DatabaseError(e.to_string()))?;
                        // sleep_interval.tick().await;
                        // let foo = self
                        //     .db
                        //     .write()
                        //     .await
                        //     // .map_err(|e| JobError::DatabaseError(e.to_string()))?
                        //     .set::<LockData>(lock.job_name.as_str(), &lock)
                        //     // .map_err(|e| JobError::DatabaseError(e.to_string()))
                        //     .unwrap();
                        // println!("lock refreshed");
                    }
                }
                None => {
                    println!("lock not found. unable to refresh. Try again");
                    // Err(Error::GeneralError(format!(
                    //     "lock not found. unable to refresh"
                    // )))
                }
            }?;
            dbg!("here");
            sleep(Duration::from_secs(lock_data.ttl.as_secs() / 2)).await;
        }
    }
}


#[async_trait]
impl JobRepo for DbRepo {
    async fn create_job(&mut self, job: Job) -> Result<bool, Error> {
        dbg!("2");
        println!("create_job");
        let name = (&job.name).into();
        self.db
            .write()
            .await
            .set(name, &job)
            .map(|_| Ok(true))
            .map_err(|e| Error::GeneralError { description: "job creation failed".to_string() })?
    }

    async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error> {
        Ok(self
            .db
            .write()
            .await
            // .map_err(|e| Error::GeneralError { description: "".to_string() })?
            .get::<Job>((&name).into()))
        }

    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
        let mut job = self.get_job(name.clone()).await.unwrap().unwrap();
        job.state = state;
        // let name1 = name.clone()
        job.last_run = Utc::now().timestamp_millis();
        dbg!("{:?}", job.last_run);
        self.db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .set((&name).into(), &job)
            .unwrap();
        println!("state saved");
        Ok(true)
    }
    }
struct JobImplementer {
    // name: String,
    // db: PickleDb,
    // project: Project,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    id: i32,
    lifecycle_state: String,
    updated: String,
}

#[async_trait]
impl JobAction for JobImplementer {
    async fn call(&mut self, name: String, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        dbg!("call");
        dbg!(name);
        let state = Vec::new();
        Ok(state)
    }
}
