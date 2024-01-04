use crate::job::Status::{Registered, Running};
use crate::{Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use derive_more::Into;
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::future::Future;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::oneshot::Sender;

#[derive(Default, Clone, Into, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct JobName(pub String);

impl AsRef<str> for JobName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

// TODO remove?
impl Display for JobName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct JobConfig {
    pub name: JobName,
    pub check_interval: Duration,
    pub lock_ttl: Duration,
    pub schedule: Schedule,
    pub enabled: bool,
}

impl JobConfig {
    pub fn new(name: impl Into<String>, schedule: Schedule) -> Self {
        JobConfig {
            name: JobName(name.into()),
            schedule,
            check_interval: Duration::from_secs(60),
            lock_ttl: Duration::from_secs(20),
            enabled: true,
        }
    }
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }
    pub fn with_lock_ttl(mut self, ttl: Duration) -> Self {
        self.lock_ttl = ttl;
        self
    }
}

#[derive(Clone, Debug)]
pub struct JobData {
    pub name: JobName,
    pub check_interval: Duration,
    pub lock_ttl: Duration,
    pub state: Vec<u8>,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_run: DateTime<Utc>,
}

impl JobData {
    pub fn due(&self, now: DateTime<Utc>) -> bool {
        if !self.enabled {
            trace!("job not enabled");
            return false;
        }

        trace!("last run is {:?}", self.last_run);
        let next_scheduled_run = self
            .schedule
            .after(&self.last_run)
            .next()
            .unwrap_or_else(|| DateTime::default());
        trace!("next schedule run is {:?}", next_scheduled_run);
        if next_scheduled_run.lt(&now) {
            trace!("run is due");
            return true;
        }
        trace!("run is not due");
        false
    }
}

impl From<JobConfig> for JobData {
    fn from(value: JobConfig) -> Self {
        Self {
            name: value.name,
            check_interval: value.check_interval,
            lock_ttl: value.lock_ttl,
            state: Vec::default(),
            schedule: value.schedule,
            enabled: value.enabled,
            last_run: DateTime::default(),
        }
    }
}

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub(crate) enum Status {
    Registered,
    Suspended,
    Running(Sender<()>),
}

pub struct Job {
    pub data: JobConfig,
    pub action: Option<Box<dyn JobAction + Send + Sync>>,
    pub status: Status,
}
impl Job {
    pub fn new(data: JobConfig, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            data,
            action: Some(Box::new(action)),
            status: Registered,
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

pub enum LockStatus<LOCK> {
    Acquired(JobData, LOCK),
    AlreadyLocked,
}
#[async_trait]
pub trait Repo {
    type Lock: Future<Output = Result<()>> + Send;
    // Transactionally create job config entry if it does not exist.
    async fn create(&mut self, job: JobData) -> Result<()>;
    // Obtain job data by name without locking
    async fn get(&mut self, name: JobName) -> Result<Option<JobData>>;
    // Save state without unlocking so jobs can do intermediate commits.
    async fn commit(&mut self, name: JobName, state: Vec<u8>) -> Result<()>;
    // Save the job state after the job ran and release the lock.
    async fn save(&mut self, name: JobName, last_run: DateTime<Utc>, state: Vec<u8>) -> Result<()>;
    // Get the job data if the lock can be obtained. Return job data and the lock future.
    async fn lock(&mut self, name: JobName, owner: String) -> Result<LockStatus<Self::Lock>>;
}
