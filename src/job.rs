use crate::error::Error;
use async_trait::async_trait;
use derive_more::{Display, From, Into};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, name: String, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[derive(Default, Clone, Into, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct JobName {
    name: String,
}

impl From<String> for JobName {
    fn from(name: String) -> Self {
        JobName { name }
    }
}

impl<'a> Into<&'a str> for &'a JobName {
    fn into(self) -> &'a str {
        &self.name
    }
}





#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct Job {
    pub name: JobName,
    pub state: Vec<u8>,
    // pub action: Arc<Mutex<dyn JobAction>>,
}

impl Job {
    pub fn new_with_action(name: String, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            name: name.into(),
            state: Vec::new(),
            // action: Arc::new(Mutex::new(action)),
        }
    }
}

#[async_trait]
pub trait JobRepo {
    async fn create_job(&mut self, job: Job) -> Result<bool, Error>;
    async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
