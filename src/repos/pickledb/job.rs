use async_trait::async_trait;
use crate::Error;
use crate::job::{JobConfig, JobName, JobRepo};
use crate::repos::pickledb::Repo;

#[async_trait]
impl JobRepo for Repo {
    async fn create_or_update_job(&mut self, job: JobConfig) -> crate::Result<bool> {
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true))
            .map_err(|e| Error::Repo(e.to_string()))?
    }
    async fn get_job(&mut self, name: JobName) -> crate::Result<Option<JobConfig>> {
        let j = self.db.write().await.get::<JobConfig>(name.as_ref());
        //dbg!(&j);
        Ok(j)
    }
    async fn save_state(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> crate::Result<()> {
        println!("Save state: {},{},{:?}",name,last_run,state);
        let name1 = name.clone();
        let name2 = name.clone();
        let mut job = self
            .get_job(name.into())
            .await
            .map_err(|e| Error::Repo(e.to_string()))?
            .ok_or(Error::JobNotFound(name1))?;

        job.state = state;
        job.last_run = last_run;
        self.db
            .write()
            .await
            .set(name2.as_ref(), &job)
            .map_err(|e| Error::Repo(e.to_string()))?;
        Ok(())
    }
}
