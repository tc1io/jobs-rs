mod job;

use crate::{Error, Result};
use mongodb::Client;

#[derive(Clone)]
pub struct MongoRepo {
    client: Client,
}

impl MongoRepo {
    pub async fn init(s: impl AsRef<str>) -> Result<MongoRepo> {
        dbg!("into");
        let client = Client::with_uri_str(s.as_ref())
            .await
            .map_err(|e| Error::Repo(e.to_string()))?;
        dbg!("done");
        Ok(MongoRepo { client })
    }
}
