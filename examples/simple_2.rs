use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::FutureExt;
// use jobs::{JobError, JobInfo, JobManager, JobRepo, JobRunner, LockData, LockRepo, Schedule};
use jobs::{error::Error, job::JobAction, job::JobRepo, lock::LockRepo, manager::JobManager};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep, Duration};

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
    manager
        .register(String::from("project-updater"), job1)
        .await;
    manager.register(String::from("project-puller"), job2).await;
    let _ = manager.start_all().await.unwrap();
    sleep(Duration::from_secs(6)).await;
    dbg!("hheee");
    manager.stop_by_name("project-puller".to_string()).await;
    sleep(Duration::from_secs(20)).await;
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]
impl LockRepo for DbRepo {}
#[async_trait]
impl JobRepo for DbRepo {}
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
        // sleep(Duration::from_secs(5)).await;
        let state = Vec::new();
        Ok(state)
    }
}
