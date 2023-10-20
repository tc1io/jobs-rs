use anyhow::Result;
use async_trait::async_trait;

// use chrono::u;
use crate::job::JobConfig;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;
use std::{dbg, format, println};
use tokio::time::interval;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LockData {
    pub expires: i64,
    pub version: i8,
    pub ttl: Duration,
}
#[async_trait]
pub trait LockRepo {
    async fn acquire_lock(&mut self, jc: JobConfig) -> Result<bool>;
    async fn refresh_lock(&mut self, lock_data: JobConfig) -> Result<bool>;
}
