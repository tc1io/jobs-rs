use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use jobs::{JobError, JobInfo, JobManager, JobRunner, JobsRepo, Lock, LockRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, Duration};

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

    let repo = DbRepo { db };
    let repo2 = DbRepo { db: db2 };

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
    manager.register("my-job2", schedule, foo_job2).await;
    // manager.run().await.unwrap();
    manager.start().await.unwrap();
}

pub struct DbRepo {
    db: PickleDb,
}

#[async_trait]
impl LockRepo for DbRepo {
    // async fn lock_refresher1(&self) -> Result<(), JobError> {
    //     // loop {
    //     println!("refreshing lock");
    //     sleep(Duration::from_secs(5)).await;
    //     println!("done");
    //     // }
    //     Ok(())
    // }
    async fn acquire_lock(&mut self, lock: Lock) -> Result<bool, JobError> {
        println!("acquire lock");
        // // let job_lock = Arc::new(Mutex::new(()));
        // let job_semaphore = Arc::new(Semaphore::new(2));
        // let _lock = job_lock.lock().await;
        // println!("Job {} is processing", lock.job_id);
        // let s = &lock.status;

        // start a mutex..on file

        let existing_lock = self.db.get::<Lock>(lock.job_name);
        match existing_lock {
            Some(lock) => {
                if lock.ttl < time.now() {
                    self.db.set(lock.job_name, &lock)?
                }
            }
            None => println!("lock empty"),
        }
        self.db.set(lock.job_name, &lock)?;
        println!("Release the lock and permit");
        // drop(job_semaphore);
        Ok(true)
        // todo!()
    }

    // async fn get_lock(&mut self, job_id: &str) -> Result<Option<Lock>, JobError> {
    //     if let Some(value) = self.db.get(job_id) {
    //         Ok(value)
    //     } else {
    //         Ok(None)
    //     }
    // }
}

#[async_trait]
impl JobsRepo for DbRepo {
    async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        // let name = &job.name;
        // let _b = self.db.set(&job.name, &job);
        match self.db.set(&job.name, &job) {
            Ok(..) => Ok(true),
            Err(err) => {
                Err(JobError::DatabaseError("Job creation failed".to_string()))
                    .expect("TODO: panic message")
                // eprintln!("JobManager run failed: {:?}", err);
                // Err(err)
            }
        }
    }

    async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError> {
        // if let Some(value) = self.db.get::<JobInfo>(name) {
        //     Ok(Some(value))
        // } else {
        //     Ok(None)
        // }
        Ok(self.db.get::<JobInfo>(name))
    }

    async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError> {
        let mut job = self.get_job(name).await.unwrap().unwrap();
        job.state = state;
        job.last_run = Utc::now().timestamp_millis();
        dbg!("{:?}", job.last_run);
        self.db.set(name, &job).unwrap();
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
        println!("job2 started");
        println!("job2 finished");
        Ok(state)
    }
}
#[async_trait]
impl JobRunner for FooJob {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    // async fn call(&self, state: Vec<u8 >) -> Result<Vec<u8>, JobError> {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("starting job");
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
                sleep(Duration::from_secs(1)).await;
            }
        }
        // sleep(Duration::from_secs(10)).await;
        println!("finishing job");
        Ok(state)
    }
}

#[tokio::test]
#[cfg(test)]
async fn test_lock_refresher1() {
    let lrepo = Db2Repo {
        repo: PickleDb::new(
            "test.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ),
    };
    let result = lrepo.lock_refresher1().await;
    assert!(result.is_ok());
}
#[tokio::test]
#[cfg(test)]
async fn test_add_lock() {
    // Create a new instance of LkRepo with a PickleDb
    let mut lrepo = Db2Repo {
        repo: PickleDb::new(
            "test.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ),
    };

    // Create a LockInfo instance to pass to the add_lock function
    let lock = Lock {
        job_name: "job1".to_string(),
        status: "processing".to_string(),
        ttl: Default::default(),
    };
    let result = lrepo.add_lock(lock).await;
    assert!(result.is_ok());
}
