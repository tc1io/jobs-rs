mod job;
mod lock;

use crate::job::JobRepo;
use mongodb::Client;
use crate::{Error, Result};

#[derive(Clone)]
pub struct MongoRepo {
    client: mongodb::Client,
}

impl MongoRepo {
    pub async fn init(s: impl AsRef<str>) -> Result<MongoRepo> {
        dbg!("into");
        let client = Client::with_uri_str(s.as_ref()).await.map_err(|e| Error::Repo(e.to_string()))?;
        dbg!("done");
        Ok(MongoRepo { client })
    }
}
