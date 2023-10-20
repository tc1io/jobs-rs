use crate::job::Status::{Registered, Running};
use crate::lock::LockData;
use anyhow::Result;
use async_trait::async_trait;
use derive_more::Into;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, name: String, state: Vec<u8>) -> Result<Vec<u8>>;
}

#[derive(Default, Clone, Into, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct JobName(pub String);

impl Display for JobName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum Status {
    Registered,
    Suspended,
    Running(Sender<()>),
}

pub struct Job {
    pub name: JobName,
    pub action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    pub schedule: Schedule,
    pub status: Status,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Schedule {
    pub expr: String, // TODO: consider alias
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct JobConfig {
    pub name: JobName,
    pub check_interval_sec: u64,
    pub state: Vec<u8>,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_run: i64,
    pub lock: LockData,
}

impl JobConfig {
    pub fn new(name: JobName, schedule: Schedule) -> Self {
        JobConfig {
            name,
            check_interval_sec: 2, // checks every 5sec
            state: Default::default(),
            schedule,
            enabled: true,
            last_run: Default::default(),
            lock: LockData { expires: 0, version: 0, ttl: Default::default() },

        }
    }
}

impl AsRef<str> for JobName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
impl Job {
    pub fn new(
        name: JobName,
        action: impl JobAction + Send + Sync + 'static,
        schedule: Schedule,
    ) -> Self {
        Job {
            name: JobName(name.into()),
            action: Arc::new(Mutex::new(action)),
            schedule,
            status: Registered,
        }
    }
    pub fn should_run_now(self) -> Result<bool> {
        Ok(false)
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
    async fn create_or_update_job(&mut self, job: JobConfig) -> Result<bool>;
    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>>;
    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool>;
}
