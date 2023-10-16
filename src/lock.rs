use std::{dbg, format, println};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};
use crate::error::Error;

#[async_trait]
pub trait LockRepo {
    async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;
    async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, Error>;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LockData {
    // pub status: String,
    pub job_name: String,
    pub ttl: Duration,
    pub expires: i64,
    pub version: i8,
}