use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use jobs::{JobError, JobInfo, JobManager, JobRunner, JobsRepo, LockInfo, LockRepo, Schedule};
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
    let mut lrepo = PickleDb::new(
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
    let lc_repo = LkRepo { lrepo };

    let schedule = Schedule {
        expr: "* */2 * * * *".to_string(),
    };
    // if let Ok(next) = parse(schedule.expr.as_str(), &Utc::now()) {
    //     println!("when: {}", next);
    // }
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
    let foo_job2 = FooJob {
        name: "".to_string(),
        db: project_db2,
        project: Project {
            name: "".to_string(),
            id: 0,
            lifecycle_state: "".to_string(),
            updated: "".to_string(),
        },
    };

    // let mut manager = JobManager::new(repo, lc_repo);
    // manager
    //     .register("dummy".to_string(), schedule, foo_job)
    //     .await;
    //
    // let result = manager.run().await.unwrap();
    //     match result {
    //         Ok(_) => {
    //             println!("JobManager run successful");
    //             Ok(())
    //         }
    //         // Err(err) => {
    //         //     Err(MyError::DatabaseError("JobManager run failed".to_string())).expect("TODO: panic message")
    //         //     // eprintln!("JobManager run failed: {:?}", err);
    //         //     // Err(err)
    //         // }
    //         Err(err) => {
    //             eprintln!("JobManager run failed: {:?}", err);
    //             Err(err)
    //         }
    //     }
    // =======
    let mut manager = JobManager::<DbRepo>::new(repo, lc_repo);
    manager.register("dummy", schedule, foo_job).await;
    // manager.run().await.unwrap();
    manager.start().await.unwrap();
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
        self.lrepo
            .set(s.as_str(), &lock)
            .expect("TODO: panic message");
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

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    id: i32,
    lifecycle_state: String,
    updated: String,
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
    let lrepo = LkRepo {
        lrepo: PickleDb::new(
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
    let mut lrepo = LkRepo {
        lrepo: PickleDb::new(
            "test.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ),
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
