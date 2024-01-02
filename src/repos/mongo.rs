use crate::job::{JobConfig, JobName, JobRepo};
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::Client;
use crate::{Error,Result};

#[derive(Clone)]
pub struct MongoRepo {
    client: mongodb::Client,
}

impl MongoRepo {
    pub async fn init(s: impl AsRef<str>) -> Result<MongoRepo> {
        dbg!("into");
        let client = Client::with_uri_str(s.as_ref()).await.map_err(|e| Error::Repo(e.to_string()))?;
        dbg!("done");
        Ok(MongoRepo { client })
    }
}
#[async_trait]
impl JobRepo for MongoRepo {
    async fn create_or_update_job(&mut self, job: JobConfig) -> Result<bool> {
        println!("create_job");
        self.client
            .database("example")
            .collection::<JobConfig>("job")
            .insert_one(&job, None)
            .await
            .map(|_| Ok(true))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>> {
        let c = self.client.clone();
        let jc = c
            .database("example")
            .collection::<JobConfig>("job")
            .find_one(doc! {"name":name.as_ref().to_string()}, None)
            .await
            .map_err(|e| Error::Repo(e.to_string()))?;
        Ok(jc)
    }

    async fn save_state(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> Result<()> {
        // TODO exactly solve this need to clone just because there is an error later
        let mut job = self
            .get_job(name.clone().into())
            .await
            .map_err(|e| Error::Repo(e.to_string()))?
            .ok_or(Error::JobNotFound(name))?;

        job.state = state;
        job.last_run = last_run;

        self.client
            .clone()
            .database("example")
            .collection::<JobConfig>("job")
            .insert_one(&job, None)
            .await
            .map(|_| Ok(()))
            .map_err(|e| Error::Repo(e.to_string()))?
    }
}
