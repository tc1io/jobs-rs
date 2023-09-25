use jobs::{Job, JobManager, JobsRepo, Schedule};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::fmt::Error;

#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    let repo = DbRepo { db };

    // let repo = RedisJobRep::new()

    // let job1 = ???;
    // let job2 = ???;

    let mut manager = JobManager::new(repo);
    //
    // manager.register(job1);
    // manager.register(job2);
    //
    manager.run().await.unwrap();
}

pub struct DbRepo {
    db: PickleDb,
}

impl JobsRepo for DbRepo {
    fn create_job(&mut self, name: String, schedule: jobs::Schedule) -> Result<(), Error> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        self.db.set("key1", &100).unwrap();
        todo!()
    }
}

pub struct FooJob {
    name: String,
}

impl jobs::Job for FooJob {
    fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        println!("inside call");
        let state = Vec::<u8>::new();
        Ok(state)
    }
}
