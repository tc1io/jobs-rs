use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::FutureExt;
use jobs::{JobError, JobInfo, JobManager, JobRunner, JobsRepo, LockData, LockRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::ops::Add;
use std::sync::Arc;
// use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep, Duration};

#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut db2 = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut project_db = PickleDb::load(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
    .unwrap();
    let mut project_db2 = PickleDb::load(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
    .unwrap();

    let repo = DbRepo {
        db: Arc::new(RwLock::new(db)),
    };
    let repo2 = DbRepo {
        db: Arc::new(RwLock::new(db2)),
    };

    let schedule = Schedule {
        expr: "* */2 * * * *".to_string(),
    };
    let foo_job = FooJob {
        name: "".to_string(),
        db: project_db,
        project: Project {
            name: "".to_string(),
            id: 0,
            lifecycle_state: "".to_string(),
            updated: "".to_string(),
        },
    };
    let foo_job2 = FooJob2 {
        name: "my-job2".to_string(),
        db: project_db2,
    };

    let mut manager = JobManager::<DbRepo, DbRepo>::new(repo, repo2);
    manager.register("dummy", schedule.clone(), foo_job).await;
    // manager.register("my-job2", schedule, foo_job2).await;
    // manager.run().await.unwrap();
    manager.start().await.unwrap();
}

#[derive(Clone)]
pub struct DbRepo {
    db: Arc<RwLock<PickleDb>>,
}

#[async_trait]
impl LockRepo for DbRepo {
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, JobError> {
        println!("acquire lock");
        let mut acquire = false;
        // TODO: try functional approach
        let existing_lock = self
            .db
            .write()
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
        self.db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .set(lock_data.job_name.as_str(), &lock_data)
            .map_err(|e| JobError::DatabaseError(e.to_string()))?;
        Ok(true)
    }
    async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError> {
        println!("refresh lock...");
        let mut refresh_interval = interval(Duration::from_secs(lock_data.ttl.as_secs() / 2));
        loop {
            refresh_interval.tick().await;
            println!("refreshing lock");

            // TODO: try functional approach
            match self
                .db
                .write()
                .await
                // .map_err(|e| JobError::DatabaseError(e.to_string()))?
                .get::<LockData>(lock_data.job_name.as_str())
            {
                // match existing_lock {
                Some(mut lock) => {
                    if lock.expires < Utc::now().timestamp_millis() {
                        println!("lock expired. unable to refresh. Try again");
                        Err(JobError::LockError(
                            format!("lock expired. unable to refresh").to_string(),
                        ))
                        // Ok(false)
                    } else {
                        lock.expires = Utc::now()
                            .timestamp_millis()
                            .add(lock.ttl.as_millis() as i64);
                        lock.version = lock.version.add(1);
                        let foo = self
                            .db
                            .write()
                            .await
                            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
                            .set(lock.job_name.as_str(), &lock)
                            .map_err(|e| JobError::DatabaseError(e.to_string()))
                            .unwrap();
                        println!("lock refreshed");
                        Ok(true)
                    }
                }
                None => {
                    println!("lock not found. unable to refresh. Try again");
                    Err(JobError::LockError(format!(
                        "lock not found. unable to refresh"
                    )))
                }
            }?;
            dbg!("here");
            sleep(Duration::from_secs(lock_data.ttl.as_secs() / 2)).await;
        }
    }
}

#[async_trait]
impl JobsRepo for DbRepo {
    async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError> {
        println!("create_job");
        self.db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .set(&job.name, &job)
            .map(|_| Ok(true))
            .map_err(|e| JobError::DatabaseError(e.to_string()))?
    }

    async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError> {
        Ok(self
            .db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .get::<JobInfo>(name))
    }

    async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError> {
        let mut job = self.get_job(name).await.unwrap().unwrap();
        job.state = state;
        job.last_run = Utc::now().timestamp_millis();
        dbg!("{:?}", job.last_run);
        self.db
            .write()
            .await
            // .map_err(|e| JobError::DatabaseError(e.to_string()))?
            .set(name, &job)
            .unwrap();
        println!("state saved");
        Ok(true)
    }
}
struct FooJob {
    name: String,
    db: PickleDb,
    project: Project,
}

struct FooJob2 {
    name: String,
    db: PickleDb,
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
impl JobRunner for FooJob2 {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("starting job2");
        sleep(Duration::from_secs(10)).await;
        println!("finishing job2");
        Ok(state)
    }
}
#[async_trait]
impl JobRunner for FooJob {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    // async fn call(&self, state: Vec<u8 >) -> Result<Vec<u8>, JobError> {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("starting job1");
        let mut sleep_interval = interval(Duration::from_secs(1));
        let all_data = self.db.get_all();
        for a in all_data {
            // println!("inside iterator");
            let mut data = self.db.get::<Project>(&a).unwrap();
            if data.lifecycle_state == "DELETED" {
                data.updated = "DONE".to_string();
                self.db
                    .set(format!("{:?}", data.id.clone()).as_str(), &data)
                    .unwrap();
                println!("{:?}", data);
                // sleep(Duration::from_secs(1)).await;
                sleep_interval.tick().await;
            }
        }
        // sleep(Duration::from_secs(10)).await;
        println!("finishing job1");
        Ok(state)
    }
}

#[tokio::test]
#[cfg(test)]
async fn test_acquire_lock() {
    let mut lrepo = DbRepo {
        db: PickleDb::new(
            "test.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ),
    };

    let lock_data = LockData {
        job_name: "test_job".to_string(),
        ttl: Duration::from_secs(10),
        expires: Utc::now().timestamp_millis(),
        version: 0,
    };

    let result = lrepo.acquire_lock(lock_data).await;
    assert!(result.is_ok());
}

// #[tokio::test]
// #[cfg(test)]
// async fn test_add_lock() {
//     // Create a new instance of LkRepo with a PickleDb
//     let mut lrepo = Db2Repo {
//         repo: PickleDb::new(
//             "test.db",
//             PickleDbDumpPolicy::AutoDump,
//             SerializationMethod::Json,
//         ),
//     };
//
//     // Create a LockInfo instance to pass to the add_lock function
//     let lock = LockData {
//         job_name: "job1".to_string(),
//         status: "processing".to_string(),
//         ttl: Default::default(),
//     };
//     let result = lrepo.add_lock(lock).await;
//     assert!(result.is_ok());
// }
