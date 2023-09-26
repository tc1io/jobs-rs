use jobs::{Job, JobConfig, JobManager, JobsRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;

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

    // let repo = RedisJobRep::new()

    // let job1 = ???;
    // let job2 = ???;

    let mut manager = JobManager::new(repo);
    //
    manager.register("dummy".to_string(), schedule, foo_job);
    // manager.register(job2);
    //
    manager.run().await.unwrap();
}

pub struct DbRepo {
    db: PickleDb,
}

impl JobsRepo for DbRepo {
    fn create_job(&mut self, jc: JobConfig) -> Result<bool, Error> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        let name = jc.name;
        self.db.set(name.as_str(), &jc.schedule).unwrap();
        Ok(true)
        // todo!()
    }

    fn get_job_config(&mut self, name: &str) -> Result<Option<jobs::JobConfig>, Error> {
        todo!()
    }
}
struct FooJob {
    name: String,
}

impl Job for FooJob {
    // type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    fn call(&self, state: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>> {
        Box::pin(async move {
            let state = Vec::<u8>::new();
            Ok(state)
        })
    }
}
