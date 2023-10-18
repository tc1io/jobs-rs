use crate::error::Error;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{dbg, format, println};
use crate::job::JobConfig;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LockData {
    pub expires: i64,
    pub version: i8,
    // pub lock_ttl: Duration,
}
#[async_trait]
pub trait LockRepo {
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
    // async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;

    async fn acquire_lock(&mut self, jc: JobConfig) -> Result<bool,Error >;
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
}
