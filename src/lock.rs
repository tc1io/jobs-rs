use anyhow::Result;
use async_trait::async_trait;

use crate::job::JobName;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize, Debug, Default, Copy)]
pub struct LockData {
    pub expires: i64,
    pub version: i8,
    pub ttl: Duration,
}
#[async_trait]
pub trait LockRepo {
    async fn acquire_lock(&mut self, name: JobName, lock_data: LockData) -> Result<bool>;
    async fn refresh_lock(&mut self, name: JobName) -> Result<bool>;
}
