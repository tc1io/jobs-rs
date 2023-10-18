use crate::error::Error;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{dbg, format, println};
use std::thread::sleep;
use tokio::time::interval;
use crate::job::JobConfig;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LockData {
    pub expires: i64,
    pub version: i8,
    // pub ttl: Duration,
}
#[async_trait]
pub trait LockRepo {
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
    // async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;

    async fn acquire_lock(&mut self, jc: JobConfig) -> Result<bool,Error >;
    // async fn refresh_lock(&mut self, lock_data: JobConfig) -> Result<bool, Error>;
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
}
