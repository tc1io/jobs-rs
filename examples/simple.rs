use async_trait::async_trait;
use jobs::{Job, JobInfo, JobManager, JobsRepo, Schedule, LockRepo, LockInfo, JobError};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{fmt, thread};
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::{Mutex, Semaphore};
#[tokio::main]
async fn main()-> Result<(), JobError> {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut lrepo = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    let repo = DbRepo { db };
    let lc_repo = LkRepo { lrepo };

    let schedule = Schedule {
        expr: "* * * 3 * * *".to_string(),
    };
    let foo_job = FooJob {
        name: "".to_string(),
    };

    let mut manager = JobManager::new(repo, lc_repo);
    manager
        .register("dummy".to_string(), schedule, foo_job)
        .await;

    let result = manager.run().await;
    match result {
        Ok(_) => {
            println!("JobManager run successful");
            Ok(())
        }
        // Err(err) => {
        //     Err(MyError::DatabaseError("JobManager run failed".to_string())).expect("TODO: panic message")
        //     // eprintln!("JobManager run failed: {:?}", err);
        //     // Err(err)
        // }
        Err(err) => {
            eprintln!("JobManager run failed: {:?}", err);
            Err(err)
        }
    }
}

pub struct DbRepo {
    db: PickleDb,
}

pub struct LkRepo {
    lrepo: PickleDb,
}
#[async_trait]
impl jobs::LockRepo for LkRepo {
    async fn lock_refresher1(&self) -> Result<(), JobError> {
        // loop {
            println!("refreshing lock");
            sleep(Duration::from_secs(5)).await;
            println!("done");
        // }
    Ok(())
    }
    async fn add_lock(&mut self, lock: jobs::LockInfo) -> Result<bool, JobError> {
        println!("adding lock");
        let job_lock = Arc::new(Mutex::new(()));
        let job_semaphore = Arc::new(Semaphore::new(2));
        let _lock = job_lock.lock().await;
        println!("Job {} is processing", lock.job_id);
        let s = &lock.status;
        self.lrepo.set(s.as_str(), &lock).expect("TODO: panic message");
        println!("Release the lock and permit");
        drop(job_semaphore);
        Ok(true)
        // todo!()
    }

    async fn get_lock(&mut self, job_id: &str) -> Result<Option<LockInfo>, JobError> {
        if let Some(value) = self.lrepo.get(job_id) {
            Ok(value)
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl JobsRepo for DbRepo {
    async fn create_job(&mut self, ji: JobInfo) -> Result<bool, JobError> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        let name = &ji.name;
        let b= self.db.set("name", &ji);
        match b {
            Ok(..) => {
                Ok(true)
            }
            Err(err) => {
                Err(JobError::DatabaseError("Job creation failed".to_string())).expect("TODO: panic message")
                // eprintln!("JobManager run failed: {:?}", err);
                // Err(err)
            }
        }

        // todo!()
    }

    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, JobError> {
        if let Some(value) = self.db.get(name) {
            Ok(value)
        } else {
            Ok(None)
        }
    }

    async fn save_state(&mut self, name: String, state: Vec<u8>) -> Result<bool, JobError> {
        let mut job = self
            .get_job_info(name.as_str().clone())
            .await
            .unwrap()
            .unwrap();
        job.state = state;
        self.db.set(name.as_str(), &job);
        println!("state saved");
        Ok(true)
    }
}
struct FooJob {
    name: String,
}

#[async_trait]
impl Job for FooJob {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    async fn call(&self, state: Vec<u8 >) -> Result<Vec<u8>, JobError> {
        println!("starting job");
        thread::sleep(Duration::from_secs(10));
        println!("finising job");
        Ok(state)
    }
}

#[tokio::test]
#[cfg(test)]
async fn test_lock_refresher1() {
    let lrepo = LkRepo {
        lrepo: PickleDb::new("test.db", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json),
    };
    let result = lrepo.lock_refresher1().await;
    assert!(result.is_ok());
}
#[tokio::test]
#[cfg(test)]
async fn test_add_lock() {
    // Create a new instance of LkRepo with a PickleDb
    let mut lrepo = LkRepo {
        lrepo: PickleDb::new("test.db", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json),
    };

    // Create a LockInfo instance to pass to the add_lock function
    let lock = LockInfo {
        job_id: "job1".to_string(),
        status: "processing".to_string(),
        ttl: Default::default(),
    };
    let result = lrepo.add_lock(lock).await;
    assert!(result.is_ok());
}
