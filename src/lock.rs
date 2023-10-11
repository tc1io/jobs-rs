use async_trait::async_trait;

#[async_trait]
pub trait LockRepo {
    // async fn refresh_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
    // async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, JobError>;
}
