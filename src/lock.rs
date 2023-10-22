use anyhow::Result;
use async_trait::async_trait;

use crate::job::JobName;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub struct LockData {
    pub expires: u64,
    pub version: i8,
    pub ttl: Duration,
}

impl LockData {
    pub fn new() -> Self {
        LockData {
            expires: 0,
            version: 0,
            ttl: Duration::from_secs(10),
        }
    }
}
#[async_trait]
pub trait LockRepo {
    async fn acquire_lock(&mut self, name: JobName, lock_data: LockData) -> Result<bool>;
    async fn refresh_lock(&mut self, name: JobName) -> Result<bool>;
}
