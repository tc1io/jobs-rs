use async_trait::async_trait;
use mongodb::bson::doc;
use crate::Error;
use crate::job::{JobConfig, JobName, JobRepo};
use crate::repos::mongo::MongoRepo;

#[async_trait]
impl JobRepo for MongoRepo {
    async fn create_or_update_job(&mut self, job: JobConfig) -> crate::Result<bool> {
        println!("create_job");
        self.client
            .database("example")
            .collection::<JobConfig>("job")
            .insert_one(&job, None)
            .await
            .map(|_| Ok(true))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn get_job(&mut self, name: JobName) -> crate::Result<Option<JobConfig>> {
        let c = self.client.clone();
        let jc = c
            .database("example")
            .collection::<JobConfig>("job")
            .find_one(doc! {"name":name.as_ref().to_string()}, None)
            .await
            .map_err(|e| Error::Repo(e.to_string()))?;
        Ok(jc)
    }

    async fn save_state(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> crate::Result<()> {
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
