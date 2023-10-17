// use std::fmt::Error;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::{FutureExt, TryFutureExt};
// use jobs::{JobError, JobInfo, JobManager, JobRepo, JobRunner, LockData, LockRepo, Schedule};
use jobs::{error::Error, job::JobAction, job::JobRepo, lock::LockRepo, manager::JobManager, repos};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep, Duration};
use jobs::job::{Job, JobConfig, JobName};
use jobs::lock::LockData;
use jobs::repos::pickledb::Repo;


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
    // let db_repo = DbRepo {
    //     db: Arc::new(RwLock::new(db_client)),
    // };
    // let lock_repo = DbRepo {
    //     db: Arc::new(RwLock::new(lock_client)),
    // };

    let mut db_repo = repos::pickledb::Repo::new(db_client);
    let mut lock_repo = repos::pickledb::Repo::new(lock_client);

    let job1 = JobImplementer {
        // name: "".to_string(),
    };
    let job2 = JobImplementer {
        // name: "".to_string(),
    };

    let mut manager = JobManager::<Repo, Repo>::new(db_repo, lock_repo);
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
        todo!()
    }
}
// #[async_trait]
// impl JobRepo for DbRepo {
//     async fn create_job(&mut self, job: JobConfig) -> Result<bool, Error> {
//         dbg!("2");
//         println!("create_job");
//         let name = job.name.as_ref();
//         self.db
//             .write()
//             .await
//             .set(name, &job)
//             .map(|_| Ok(true))
//             .map_err(|e| Error::GeneralError { description: "job creation failed".to_string() })?
//     }
//
//     async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>, Error> {
//         Ok(self
//             .db
//             .write()
//             .await
//             // .map_err(|e| Error::GeneralError { description: "".to_string() })?
//             .get::<JobConfig>(name.as_ref())
//         }
//
//     async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
//         let mut job = self.get_job(name.clone()).await.unwrap().unwrap();
//         job.state = state;
//         // let name1 = name.clone()
//         job.last_run = Utc::now().timestamp_millis();
//         dbg!("{:?}", job.last_run);
//         self.db
//             .write()
//             .await
//             // .map_err(|e| JobError::DatabaseError(e.to_string()))?
//             .set((&name).into(), &job)
//             .unwrap();
//         println!("state saved");
//         Ok(true)
//     }
//     }
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
