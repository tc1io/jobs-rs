use crate::{JobError, Schedule};
use crate::JobManager;
use core::fmt::Error;
use crate::LockInfo;
use crate::Job;
use crate::JobInfo;
use crate::LockRepo;
use crate::JobsRepo;

// Mock implementation of JobsRepo for testing
struct MockJobsRepo;
#[async_trait::async_trait]
impl JobsRepo for MockJobsRepo {
    async fn create_job(&mut self, job_info: JobInfo) -> Result<bool, JobError> {
        Ok(true)
    }

    async fn get_job_info(&mut self, name: &str) -> Result<Option<JobInfo>, JobError> {
        Ok(None)
    }

    async fn save_state(&mut self, name: String, state: Vec<u8>) -> Result<bool, JobError> {
        Ok(true)
    }
}

// Mock implementation of LockRepo for testing
struct MockLockRepo;
#[async_trait::async_trait]
impl LockRepo for MockLockRepo {
    async fn lock_refresher1(&self) -> Result<(), JobError> {
        Ok(())
    }

    async fn add_lock(&mut self, li: LockInfo) -> Result<bool, JobError> {
        Ok(true)
    }

    async fn get_lock(&mut self, name: &str) -> Result<Option<LockInfo>, JobError> {
        Ok(None)
    }
}

#[cfg(test)]
use super::*;
#[tokio::test]
async fn test_job_call() {
    // Create a mock implementation of the Job trait for testing
    struct MockJob;
    #[async_trait::async_trait]
    impl Job for MockJob {
        async fn call(&self, state: Vec<u8>) -> Result<Vec<u8>, JobError> {
            // implement a mock behavior here for testing
            Ok(state)
        }
    }
    // Create a JobManager instance and register a job
    let mut manager = JobManager::new(MockJobsRepo, MockLockRepo);
    manager
        .register("test_job".to_string(), Schedule { expr: "* * * * * * *".to_string() }, MockJob)
        .await;

    // Run the job and verify the result
    let result = manager.run().await;
    assert!(result.is_ok());
}