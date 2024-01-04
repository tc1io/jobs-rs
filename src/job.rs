use crate::job::Status::{Registered, Running};
use crate::{Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
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

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Schedule {
    pub expr: String, // TODO: consider alias
}

impl Schedule {
    pub fn minutely() -> CronSchedule {
        CronSchedule::from_str("0 * * * * *").expect("minutely cron expression should parse")
    }
}

// TODO remove?
impl Display for JobName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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

#[derive(Clone, Debug)]
pub struct JobData {
    pub name: JobName,
    pub check_interval: Duration,
    pub lock_ttl: Duration,
    pub state: Vec<u8>,
    pub schedule: CronSchedule,
    pub enabled: bool,
    pub last_run: i64,
}

#[derive(Clone)]
pub struct JobConfig {
    pub name: JobName,
    pub check_interval: Duration,
    pub lock_ttl: Duration,
    pub schedule: CronSchedule,
    pub enabled: bool,
}

impl JobConfig {
    pub fn new(name: impl Into<String>, schedule: CronSchedule) -> Self {
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

impl From<JobConfig> for JobData {
    fn from(value: JobConfig) -> Self {
        Self {
            name: value.name,
            check_interval: value.check_interval,
            lock_ttl: value.lock_ttl,
            state: Vec::default(),
            schedule: value.schedule,
            enabled: value.enabled,
            last_run: 0,
        }
    }
}

impl JobData {
    pub fn due(&self, now: DateTime<Utc>) -> bool {
        if !self.enabled {
            trace!("job not enabled");
            return false;
        }
        if self.last_run.eq(&0) {
            trace!("last run is 0");
            return true;
        }
        let last_run =
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(self.last_run as u64));
        trace!("last run is {}", last_run);
        let next_scheduled_run = self
            .schedule
            .after(&last_run)
            .next()
            .map_or_else(|| 0, |t| t.timestamp_millis());
        trace!("next schedule run is {}", next_scheduled_run);
        if next_scheduled_run.lt(&now.timestamp_millis()) {
            trace!("run is due");
            return true;
        }
        trace!("run is not due");
        false
    }
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
    // TODO
    async fn commit(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> Result<()>;
    // Save the job state after the job ran and release the lock.
    async fn save(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> Result<()>;
    // Get the job data if the lock can be obtained. Return job data and the lock future.
    async fn lock(&mut self, name: JobName, owner: String) -> Result<LockStatus<Self::Lock>>;
}
