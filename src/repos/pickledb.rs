use crate::job::{JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use pickledb::PickleDb;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    async fn acquire_lock(&mut self, name: JobName, lock_data: LockData) -> Result<bool> {
        println!("acquire lock");

        if self
            .db
            .read()
            .await
            .get::<LockData>(name.0.as_str())
            .map_or_else(|| true, |data| data.expires < Utc::now().timestamp_millis())
        {
            return self
                .db
                .write()
                .await
                .set(name.0.as_str(), &lock_data)
                .map(|_| Ok(true))
                .map_err(|e| anyhow!(e.to_string()))?;
        }
        Ok(false)
    }
    async fn refresh_lock(&mut self, name: JobName) -> Result<bool> {
        // let mut refresh_interval = interval(Duration::from_secs(jc.lock.ttl.as_secs() / 2));
        loop {
            // refresh_interval.tick().await;

            let mut lock_data = self
                .db
                .read()
                .await
                .get::<LockData>(name.as_ref())
                .ok_or(anyhow!("lock not found or lock expired"))?;

            if lock_data.expires < Utc::now().timestamp_millis() {
                return Err(anyhow!("lock expired"));
            } else {
                lock_data.expires = Utc::now()
                    .timestamp_millis()
                    .add(lock_data.ttl.as_millis() as i64);
                lock_data.version = lock_data.version.add(1);
                self.db
                    .write()
                    .await
                    .set(name.0.as_str(), &lock_data)
                    .map_err(|e| anyhow!(e.to_string()))?;
                println!("lock refreshed");
            }

            // println!("refreshing lock");
            // match self.db.read().await.get::<LockData>(name.as_ref()) {
            //     Some(mut lock) => {
            //         dbg!("some");
            //         if lock.lock.expires < Utc::now().timestamp_millis() {
            //             println!("lock expired. unable to refresh. Try again");
            //             // Err(JobError::LockError(
            //             //     format!("lock expired. unable to refresh").to_string(),
            //             // ))
            //             // Ok(false)
            //         } else {
            //             dbg!("some else");
            //             lock.lock.expires = Utc::now()
            //                 .timestamp_millis()
            //                 .add(lock.lock.ttl.as_millis() as i64);
            //             lock.lock.version = lock.lock.version.add(1);
            //             dbg!(lock.name.0.as_str());
            //             self.db
            //                 .write()
            //                 .await
            //                 .set(lock.name.0.as_str(), &lock)
            //                 .map_err(|e| anyhow!(e.to_string()))?;
            //             println!("lock refreshed");
            //         }
            //     }
            //     None => {
            //         println!("lock not found. unable to refresh. Try again");
            //     }
        }
        // dbg!("here");
        // sleep(Duration::from_secs(jc.lock.ttl.as_secs() / 2)).await;
    }
}
