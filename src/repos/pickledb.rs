use std::ops::Add;
use crate::job::{JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use pickledb::PickleDb;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::Utc;
use tokio::time::{interval, sleep};

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
    async fn create_or_update_job(&mut self, job: JobConfig) -> Result<bool> {
        dbg!("create job");
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true))
            .map_err(|e| anyhow!(e.to_string()))?
    }
    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>> {
        Ok(self.db.write().await.get::<JobConfig>(name.as_ref()))
    }
    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool> {
        let name1 = name.clone();
        let mut job = self.get_job(name.into()).await.unwrap().unwrap();

        job.state = state;
        // let name1 = name.clone()
        // job.last_run = Utc::now().timestamp_millis();
        self.db
            .write()
            .await
            .set(name1.as_ref(), &job)
            .map_err(|e| anyhow!(e.to_string()))?;
        println!("state saved");
        Ok(true)
    }
}

#[async_trait]
impl LockRepo for Repo {

    async fn acquire_lock(&mut self, jc: JobConfig) -> Result<bool> {
        println!("acquire lock");
        let mut acquire = false;

        // TODO: try functional approach
        let existing_lock = self
            .db
            .read()
            .await
            .get::<JobConfig>(jc.name.as_ref());

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
                .set(jc.name.as_ref(), &jc)
                .map(|_| Ok(true))
                .map_err(|e| anyhow!(e.to_string()))?
        } else {
            Ok(false)
        }
    }
    async fn refresh_lock(&mut self, jc: JobConfig) -> Result<bool> {
        // let mut refresh_interval = interval(Duration::from_secs(jc.lock.ttl.as_secs() / 2));
        loop {
            // refresh_interval.tick().await;
            println!("refreshing lock");
            let existing_lock = self
                .db
                .read()
                .await
                .get::<JobConfig>(jc.name.as_ref());
            match existing_lock {
                // match existing_lock {
                Some(mut lock) => {
                    dbg!("some");
                    if lock.lock.expires < Utc::now().timestamp_millis() {
                        println!("lock expired. unable to refresh. Try again");
                        // Err(JobError::LockError(
                        //     format!("lock expired. unable to refresh").to_string(),
                        // ))
                        // Ok(false)
                    } else {
                        dbg!("some else");
                        lock.lock.expires = Utc::now()
                            .timestamp_millis()
                            .add(lock.lock.ttl.as_millis() as i64);
                        lock.lock.version = lock.lock.version.add(1);
                        dbg!(lock.name.0.as_str());
                        self.db
                            .write()
                            .await
                            .set(lock.name.0.as_str(), &lock)
                            .map_err(|e| anyhow!(e.to_string()))?;
                        println!("lock refreshed");

                    }
                }
                None => {
                    println!("lock not found. unable to refresh. Try again");
                }
            };
            dbg!("here");
            sleep(Duration::from_secs(jc.lock.ttl.as_secs() / 2)).await;
        }
    }
}
