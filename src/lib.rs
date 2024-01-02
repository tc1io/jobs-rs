pub mod executor;
pub mod job;
pub mod lock;
pub mod manager;
pub mod repos;

use thiserror::Error;
use crate::job::JobName;

#[derive(Error, Debug)]
pub enum Error {

    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    #[error("invalid cron expression {expression:?}: {msg:?})")]
    InvalidCronExpression {
        expression: String,
        msg: String,
    },
    #[error("Job is missing: {0}")]
    JobNotFound(JobName),
    #[error("Repository error: {0}")]
    Repo(String),
    #[error("canceling job {0} failed")]
    CancelFailed(String),

    #[error("TODO")]
    TODO,
}

pub type Result<T> = std::result::Result<T,Error>;
