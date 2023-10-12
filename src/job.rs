use crate::error::Error;
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
pub struct JobName(String);
#[derive(Clone)]
pub struct Job {
    pub name: JobName,
    pub state: Vec<u8>,
    pub action: Arc<Mutex<dyn JobAction + Send + Sync>>,
}

impl Job {
    pub fn new_with_action(name: JobName, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            name,
            state: Vec::new(),
            action: Arc::new(Mutex::new(action)),
        }
    }
}
pub enum Status {
    Registered,
    Suspended,
    Running(Sender<()>),
    Errored,
    Cancelled,
}

pub struct Config {
    name: JobName,
}

pub struct Entry {
    pub name: JobName,
    config: Config,
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    status: Status,
    state: Vec<u8>,
}

impl Entry {
    pub fn new(name: JobName, action: impl JobAction + Send + Sync + 'static) -> Self {
        Entry {
            name: name.clone(),
            config: Config { name },
            action: Arc::new(Mutex::new(action)),
            status: Status::Registered,
            state: Vec::new(),
        }
    }
    pub fn set_status_running(&mut self, tx: Sender<()>) {
        self.status = Status::Running(tx);
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        dbg!("inside run");
        let mut action = self.action.lock().await;
        // other logic will be added
        let _xx = action.call(self.name.clone().into(), Vec::new()).await?;
        Ok(())
    }
}

#[async_trait]
pub trait JobRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    // async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
