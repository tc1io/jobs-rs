use crate::error::Error;
use crate::job::{JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use async_trait::async_trait;
use pickledb::PickleDb;
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
    async fn create_or_update_job(&mut self, job: JobConfig) -> Result<bool, Error> {
        dbg!("create job");
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true)) // TODO
            .map_err(|e| Error::GeneralError {
                description: e.to_string(),
            })?
    }
    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>, Error> {
        Ok(self.db.write().await.get::<JobConfig>(name.as_ref()))
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
            .set(name1.as_ref(), &job)
            .unwrap();
        println!("state saved");
        Ok(true)
    }
}

#[async_trait]
impl LockRepo for Repo {
    async fn acquire_lock(&mut self, _lock_data: LockData) -> Result<bool, Error> {
        // println!("acquire lock");
        // let mut acquire = false;
        // TODO: try functional approach
        // let existing_lock = self
        //     .db
        //     .read()
        //     .await
        //     .get::<LockData>(lock_data.job_name);
        // // .unwrap();
        // // .map_err(|e| JobError::DatabaseError(e.to_string()))?
        // match existing_lock {
        //     Some(lock) => {
        //         if lock.expires < Utc::now().timestamp_millis() {
        //             acquire = true;
        //         }
        //     }
        //     None => acquire = true,
        // }
        // if acquire {
        // self.db
        //     .write()
        //     .await
        //     .set(lock_data.job_name.as_str(), &lock_data)
        //     .map(|_| Ok(true))
        //     .map_err(|e| Error::GeneralError { description: "lock error".to_string() })?
        // } else {
        //     Ok(false)
        // }
        Ok(false)
    }
}
