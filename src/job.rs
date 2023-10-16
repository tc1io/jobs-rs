use crate::error::Error;
use crate::job::Status::{Registered, Running};
use async_trait::async_trait;
use derive_more::{Display, From, Into};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, name: String, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[derive(Default, Clone, From, Into, Eq, Hash, PartialEq, Debug)]
pub struct JobName(pub String);

#[derive(Debug)]
pub enum Status {
    Registered,
    Suspended,
    Running(Sender<JobName>),
    // Errored,
    // Cancelled,
}

// impl Clone for Status {
//     fn clone(&self) -> Self {
//         match self {
//             Registered => Registered,
//             Suspended => Suspended,
//             Running(sender) => Running(sender.into()),
//         }
//     }
// }

#[derive(Default, Clone)]
pub struct Config {
    name: JobName,
}

pub struct Job {
    pub name: JobName,
    pub action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    pub status: Status,
    // state: Vec<u8>,
    // config: Config,
}

impl Job {
    pub fn new(name: JobName, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            name: name.clone(),
            action: Arc::new(Mutex::new(action)),
            status: Registered,
            // config: Config { name },
            // state: Vec::new(),
        }
    }
    pub fn get_registered_or_running(s: &Status) -> bool {
        match s {
            Registered => true,
            Running(_s) => true,
            _ => false,
        }
    }
}

#[async_trait]
pub trait JobRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    // async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
