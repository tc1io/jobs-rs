use std::future::Future;
use std::time;
use anyhow::Result;
use async_trait::async_trait;

use crate::job::JobName;

//#[async_trait]
//pub trait Lock {
//    async fn unlock() -> Result<()>;
//}

pub enum LockStatus<L> {
    Acquired(L),
    AlreadyLocked
}

#[async_trait]
pub trait LockRepo {
    type Lock: Future<Output=Result<()>> + Send;
    async fn acquire_lock(&mut self, name: JobName, owner: String, ttl: time::Duration) -> Result<LockStatus<Self::Lock>>;
}
