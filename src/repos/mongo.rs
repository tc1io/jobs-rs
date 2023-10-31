use crate::job::{JobConfig, JobName, JobRepo};
use anyhow::anyhow;
use async_trait::async_trait;
use mongodb::bson::{self, doc, Document};
use mongodb::options::FindOneOptions;
use mongodb::Client;
use mongodb::Collection;
// pub trait Repository<T> {
//     fn insert(&self, item: T) -> Result<(), mongodb::error::Error>;
//     fn find(&self, filter: bson::Document) -> Result<Option<T>, mongodb::error::Error>;
//     fn update(
//         &self,
//         filter: bson::Document,
//         update: bson::Document,
//     ) -> Result<(), mongodb::error::Error>;
//     fn delete(&self, filter: bson::Document) -> Result<(), mongodb::error::Error>;
// }
#[derive(Clone)]
pub struct MongoRepo {
    client: mongodb::Client,
}

impl MongoRepo {
    pub fn new(client: mongodb::Client) -> Self {
        MongoRepo { client }
    }
}
#[async_trait]
impl JobRepo for MongoRepo {
    async fn create_or_update_job(&mut self, job: JobConfig) -> anyhow::Result<bool> {
        println!("create_job");
        self.client
            .database("foo")
            .collection::<JobConfig>("job")
            .insert_one(&job, None)
            .await
            .map(|_| Ok(true))
            .map_err(|e| anyhow!(e.to_string()))?
    }

    async fn get_job(&mut self, name: JobName) -> anyhow::Result<Option<JobConfig>> {
        todo!()
    }

    async fn save_state(
        &mut self,
        name: JobName,
        last_run: i64,
        state: Vec<u8>,
    ) -> anyhow::Result<bool> {
        todo!()
    }
}

// impl Repository<YourData> for MongoRepository {
//     fn insert(&self, item: YourData) -> Result<(), mongodb::error::Error> {
//         self.collection
//             .insert_one(bson::to_document(&item)?, None)
//             .map(|_| ())
//     }
//
//     fn find(&self, filter: bson::Document) -> Result<Option<YourData>, mongodb::error::Error> {
//         self.collection
//             .find_one(filter, None)
//             .map(|doc| doc.and_then(|doc| bson::from_document(doc).ok()))
//     }
//
//     fn update(
//         &self,
//         filter: bson::Document,
//         update: bson::Document,
//     ) -> Result<(), mongodb::error::Error> {
//         self.collection.update_one(filter, update, None).map(|_| ())
//     }
//
//     fn delete(&self, filter: bson::Document) -> Result<(), mongodb::error::Error> {
//         self.collection.delete_one(filter, None).map(|_| ())
//     }
// }
