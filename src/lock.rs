use anyhow::Result;
use async_trait::async_trait;

// use chrono::u;
use serde::{Deserialize, Serialize};
use std::{dbg, format, println};
use std::thread::sleep;
use std::time::Duration;
use chrono::Utc;
use tokio::time::interval;
use crate::job::JobConfig;


#[derive(Clone, Serialize, Deserialize, Debug)]
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
