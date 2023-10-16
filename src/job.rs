use crate::error::Error;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use cron::Schedule as CronSchedule;
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration as Dur, UNIX_EPOCH};
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Schedule {
    pub expr: String,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct Job {
    pub name: JobName,
    pub state: Vec<u8>,
    // pub action: Arc<Mutex<dyn JobAction>>,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_run: i64,
    // pub lock_ttl: Duration,
}

impl AsRef<str> for JobName {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

impl Job {
    pub fn new_with_action(name: String, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            name: name.into(),
            state: Vec::new(),
            // action: Arc::new(Mutex::new(action)),
            schedule: Schedule {
                expr: "".to_string(),
            },
            enabled: false,
            last_run: 0,
            // lock_ttl: (),
        }
    }
    pub fn should_run_now(self) -> Result<bool, Error> {
        if !self.enabled {
            return Ok(false);
        }
        dbg!("", self.last_run);
        if self.last_run.eq(&0) {
            return Ok(true);
        }
        let dt = UNIX_EPOCH + Dur::from_millis(self.last_run as u64);
        // let date_time = DateTime::<Utc>::(self.last_run, 0).unwrap();
        let date_time = DateTime::<Utc>::from(dt);
        dbg!("", date_time);
        let schedule = CronSchedule::from_str(self.schedule.expr.as_str()).unwrap();
        let ff = schedule.after(&date_time).next().unwrap_or(Utc::now());
        // .next()
        // .map(|t| t.timestamp_millis())
        // .unwrap();
        dbg!("", ff);
        let next_scheduled_run = schedule
            .after(&date_time)
            .next()
            .map(|t| t.timestamp_millis())
            .unwrap_or(0);
        dbg!(
            "{:?}-----{:?}---- {:?}",
            self.last_run,
            next_scheduled_run,
            Utc::now().timestamp_millis()
        );
        if next_scheduled_run.lt(&Utc::now().timestamp_millis()) {
            return Ok(true);
        }
        Ok(false)
    }
}

#[async_trait]
pub trait JobRepo {
    async fn create_job(&mut self, job: Job) -> Result<bool, Error>;
    async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error>;
    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error>;
}
