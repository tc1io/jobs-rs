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
use tokio::sync::RwLock;
use tokio::time::{interval, sleep, Duration};
use jobs::job::{Job, JobName};

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
    manager.register(String::from("project-updater"), job1);
    manager.register(String::from("project-puller"), job2);
    let _ = manager.start().await.unwrap();
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]
impl LockRepo for DbRepo {}
#[async_trait]
impl JobRepo for DbRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError> {
    //     println!("create_job");
    //     self.db
    //         .write()
    //         .await
    //         // .map_err(|e| JobError::DatabaseError(e.to_string()))?
    //         .set(&job.name, &job)
    //         .map(|_| Ok(true))
    //         .map_err(|e| JobError::DatabaseError(e.to_string()))?
    // }

    async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error> {
        Ok(self
            .db
            .write()
            .await
            .map_err(|e| Error::GeneralError { description: "".to_string() })?
            .get::<Job>(name))
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
