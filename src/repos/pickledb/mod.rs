mod job;

use pickledb::PickleDb;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct PickleDbRepo {
    pub(crate) db: Arc<RwLock<PickleDb>>,
}

impl PickleDbRepo {
    pub fn new(db: PickleDb) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}
