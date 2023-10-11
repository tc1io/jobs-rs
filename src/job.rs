use crate::error::Error;
use async_trait::async_trait;
use derive_more::{Display, From, Into};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[derive(Default, Clone, From, Into, Eq, Hash, PartialEq, Debug)]
pub struct JobName(String);
#[derive(Clone)]
pub struct Job {
    pub name: JobName,
    pub action: Arc<Mutex<dyn JobAction>>,
}

impl Job {
    pub fn new_with_action(action: impl JobAction + 'static) -> Self {
        Job {
            name: JobName::default(),
            action: Arc::new(Mutex::new(action)),
        }
    }
}

#[async_trait]
pub trait JobRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    // async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
