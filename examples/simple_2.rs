use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::{FutureExt, TryFutureExt};
use jobs::job::{Job, JobName, Schedule};
use jobs::{
    error::Error, job::JobAction, job::JobRepo, lock::LockRepo, manager::JobManager, repos,
};

use jobs::lock::LockData;
use jobs::repos::pickledb::Repo;
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
    let mut db_repo = repos::pickledb::Repo::new(db_client);
    let mut lock_repo = repos::pickledb::Repo::new(lock_client);

    let job1 = JobImplementer {
        // name: "".to_string(),
    };
    let job2 = JobImplementer {
        // name: "".to_string(),
    };

    let mut manager = JobManager::<Repo, Repo>::new(db_repo, lock_repo);
    // manager.register(
    //     String::from("project-updater"),
    //     job1,
    //     Schedule {
    //         expr: "* */3 * * * *".to_string(),
    //     },
    // );
    manager.register(
        String::from("project-puller"),
        job2,
        Schedule {
            expr: "* */2 * * * *".to_string(),
        },
    );
    let _ = manager.start_all().await.unwrap();
    sleep(Duration::from_secs(4)).await;
    manager.stop_by_name("project-puller".to_string()).await;
    sleep(Duration::from_secs(10)).await;
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]

impl LockRepo for DbRepo {
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error> {
        todo!()
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
        dbg!(name);
        let state = Vec::new();
        Ok(state)
    }
}
