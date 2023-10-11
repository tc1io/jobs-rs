use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Job {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[async_trait]
pub trait JobRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    // async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
