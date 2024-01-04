use thiserror::Error;

mod executor;
mod job;
mod manager;
mod repos;

pub mod schedule;

pub use repos::pickledb::PickleDbRepo;

pub use job::JobAction;
pub use job::JobConfig;
pub use manager::JobManager;

use job::JobName;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    #[error("invalid cron expression {expression:?}: {msg:?})")]
    InvalidCronExpression { expression: String, msg: String },
    #[error("Job is missing: {0}")]
    JobNotFound(JobName),
    #[error("Repository error: {0}")]
    Repo(String),
    #[error("Loack refresh failed: {0}")]
    LockRefreshFailed(String),
    #[error("canceling job {0} failed")]
    CancelFailed(JobName),

    #[error("TODO")]
    TODO,
}

pub type Result<T> = std::result::Result<T, Error>;
