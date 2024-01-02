use std::future::Future;
use std::time;
use async_trait::async_trait;
use crate::job::JobName;
use crate::{Result};

pub enum LockStatus<LOCK> {
    Acquired(LOCK),
    AlreadyLocked
}

#[async_trait]
pub trait LockRepo {
    type Lock: Future<Output=Result<()>> + Send;
    async fn acquire_lock(&mut self, name: JobName, owner: String, ttl: time::Duration) -> Result<LockStatus<Self::Lock>>;
}
