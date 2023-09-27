use async_trait::async_trait;
use jobs::JobInfo;
use jobs::{Job, JobManager, JobsRepo, Schedule};
use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;

use mongodb::bson::{self, doc, Document};
use mongodb::options::FindOneOptions;
use mongodb::Client;
use mongodb::Collection;
// use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    println!("db connection ....");
    let db = client.database("xxx_db");
    // let collection = db.collection("xxx_collection");
    let repo = DBRepository::new(client);

    // let repo = DbRepo { db };
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
    manager.register("dummy", schedule, foo_job);
    // manager.register(job2);
    //
    manager.run().await.unwrap();
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

pub struct DBRepository {
    client: mongodb::Client,
    // collection: mongodb::Collection<bson::Document>,
}

impl DBRepository {
    pub fn new(client: mongodb::Client) -> Self {
        DBRepository { client }
    }
}

#[async_trait]
impl JobsRepo for DBRepository {
    async fn create_job(&mut self, ji: JobInfo) -> Result<bool, Error> {
        // TODO: do it without jobs ext - jobs::Schedule
        println!("create_job");
        // let name = &ji.name;
        // let document = bson::to_document(&ji);
        self.client
            .database("foo")
            .collection::<JobInfo>("job")
            .insert_one(&ji, None)
            .await
            .expect("TODO: panic message");
        // Ok(())
        // self.db.set(name.as_str(), &ji).unwrap();

        Ok(true)
        // todo!()
    }
    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, Error> {
        todo!()
    }
    // fn read(&self, filter: Document) -> Result<Option<JobData>, mongodb::error::Error> {
    //     self.collection.find_one(filter, None)?.map(|doc| bson::from_document(doc).unwrap()).transpose()
    // }

    // fn find_by_jobname(&self, name: &str) -> Result<Option<JobData>, mongodb::error::Error> {
    //    let filter = doc! {"name": name};
    //    let options = FindOneOptions::default();
    //
    //    if let Some(document) = self.collection.find_one(filter, options).{
    //        let jobdata: JobData = bson::from_document(document).unwrap();
    //        Ok(Some(jobdata))
    //    } else {
    //        Ok(None)
    //    }
}
