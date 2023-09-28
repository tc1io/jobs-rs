use async_trait::async_trait;
use jobs::{Job, JobInfo, JobManager, JobsRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    let repo = DbRepo { db };

    let schedule = Schedule {
        expr: "* * * 3 * * *".to_string(),
    };
    let foo_job = FooJob {
        name: "".to_string(),
    };

    let mut manager = JobManager::new(repo);
    manager
        .register("dummy".to_string(), schedule, foo_job)
        .await;
    manager.run().await.unwrap();
}

pub struct DbRepo {
    db: PickleDb,
}

#[async_trait]
impl JobsRepo for DbRepo {
    async fn create_job(&mut self, ji: JobInfo) -> Result<bool, Error> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        let name = &ji.name;
        self.db.set(name.as_str(), &ji).unwrap();
        Ok(true)
        // todo!()
    }

    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, Error> {
        if let Some(value) = self.db.get(name) {
            Ok(value)
        } else {
            Ok(None)
        }
    }

    async fn save_state(&mut self, name: String, state: Vec<u8>) -> Result<bool, Error> {
        let mut job = self
            .get_job_info(name.as_str().clone())
            .await
            .unwrap()
            .unwrap();
        job.state = state;
        self.db.set(name.as_str(), &job).unwrap();
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
    async fn call(&self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("starting job");
        thread::sleep(Duration::from_secs(10));
        println!("finising job");
        Ok(state)
    }
}
