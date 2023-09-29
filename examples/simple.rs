use async_trait::async_trait;
use jobs::{Job, JobInfo, JobManager, JobsRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut project_db = PickleDb::load(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
    .unwrap();

    let repo = DbRepo { db };

    let schedule = Schedule {
        expr: "* * * 3 * * *".to_string(),
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

    let mut manager = JobManager::<DbRepo, FooJob>::new(repo);
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
impl Job for FooJob {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("starting job");
        let all_data = self.db.get_all();
        for a in all_data {
            // println!("inside iterator");
            let mut data = self.db.get::<Project>(&a).unwrap();
            if data.lifecycle_state == "DELETED" {
                data.updated = "DONE".to_string();
                // self.db
                //     .set(format!("{:?}", data.id.clone()).as_str(), &data)
                //     .unwrap();
                println!("{:?}", data);
                sleep(Duration::from_secs(100)).await;
            }
        }
        // sleep(Duration::from_secs(10)).await;
        println!("finising job");
        Ok(state)
    }
}
