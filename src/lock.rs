use std::{dbg, format, println};
use async_trait::async_trait;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use crate::error::Error;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LockData {
    // pub status: String,
    pub job_name: String,
    // pub ttl: Duration,
    pub expires: i64,
    pub version: i8,
}
#[async_trait]
pub trait LockRepo {
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;
}
