use crate::job::{JobConfig, JobName, JobRepo};
use anyhow::anyhow;
use async_trait::async_trait;
use mongodb::bson::{doc, Document};
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};

#[derive(Clone)]
pub struct MongoRepo {
    client: mongodb::Client,
}

impl MongoRepo {
    pub async fn init(s: impl AsRef<str>) -> Result<MongoRepo> {
        dbg!("into");
        let client = Client::with_uri_str(s.as_ref()).await?;
        dbg!("done");
        Ok(MongoRepo { client })
    }
}
#[async_trait]
impl JobRepo for MongoRepo {
    async fn create_or_update_job(&mut self, job: JobConfig) -> anyhow::Result<bool> {
        println!("create_job");
        let c = self.client.clone();
        match c
            .database("example")
            .collection::<JobConfig>("jobs")
            .insert_one(&job, None)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    async fn get_job(&mut self, name: JobName) -> anyhow::Result<Option<JobConfig>> {
        let c = self.client.clone();
        let jc = c
            .database("example")
            .collection::<JobConfig>("identity")
            .find_one(doc! {"name":name.as_ref().to_string()}, None)
            .await?;
        Ok(jc)
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
