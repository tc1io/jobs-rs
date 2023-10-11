use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::FutureExt;
// use jobs::{JobError, JobInfo, JobManager, JobRepo, JobRunner, LockData, LockRepo, Schedule};
use jobs::{job::JobRepo, lock::LockRepo, manager::JobManager};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::Error;
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

    let mut manager = JobManager::<DbRepo, DbRepo>::new(db_repo, lock_repo);
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]
impl LockRepo for DbRepo {}
#[async_trait]
impl JobRepo for DbRepo {}
struct FooJob {
    name: String,
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
