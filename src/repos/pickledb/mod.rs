mod job;
mod lock;

use std::future::Future;
use crate::job::JobRepo;
use crate::lock::LockRepo;
use pickledb::PickleDb;
use std::ops::Add;
use std::sync::Arc;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Repo {
    pub(crate) db: Arc<RwLock<PickleDb>>,
}

impl Repo {
    pub fn new(db: PickleDb) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}