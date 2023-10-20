use crate::job::Status::{Registered, Running};
use crate::lock::LockData;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use derive_more::Into;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
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
    pub last_run: u16,
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
            lock: LockData {
                expires: Default::default(),
                version: Default::default(),
            },
        }
    }
    pub fn run_job_now(self) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }
        if self.last_run.eq(&0) {
            return Ok(true);
        }
        let last_run =
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(self.last_run as u64));
        let schedule = CronSchedule::from_str(self.schedule.expr.as_str())?;
        let next_scheduled_run = schedule
            .after(&last_run)
            .next()
            .map_or_else(|| 0, |t| t.timestamp_millis());
        if next_scheduled_run.lt(&Utc::now().timestamp_millis()) {
            return Ok(true);
        }
        Ok(false)
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
        // if !self.enabled {
        //     return Ok(false);
        // }
        // dbg!("", self.last_run);
        // if self.last_run.eq(&0) {
        //     return Ok(true);
        // }
        // let dt = UNIX_EPOCH + Dur::from_millis(self.last_run as u64);
        // // let date_time = DateTime::<Utc>::(self.last_run, 0).unwrap();
        // let date_time = DateTime::<Utc>::from(dt);
        // dbg!("", date_time);
        // let schedule = CronSchedule::from_str(self.schedule.expr.as_str()).unwrap();
        // let ff = schedule.after(&date_time).next().unwrap_or(Utc::now());
        // // .next()
        // // .map(|t| t.timestamp_millis())
        // // .unwrap();
        // dbg!("", ff);
        // let next_scheduled_run = schedule
        //     .after(&date_time)
        //     .next()
        //     .map(|t| t.timestamp_millis())
        //     .unwrap_or(0);
        // dbg!(
        //     "{:?}-----{:?}---- {:?}",
        //     self.last_run,
        //     next_scheduled_run,
        //     Utc::now().timestamp_millis()
        // );
        // if next_scheduled_run.lt(&Utc::now().timestamp_millis()) {
        //     return Ok(true);
        // }
        Ok(false)
        //     name,
        //     action: Arc::new(Mutex::new(action)),
        //     status: Registered,
        // config: Config { name },
        // state: Vec::new(),
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
