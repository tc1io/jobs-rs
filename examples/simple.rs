use async_trait::async_trait;
use jobs::{schedule, PickleDbRepo};
use jobs::{Error, Result};
use jobs::{JobAction, JobConfig, JobManager};
use mongodb::Client;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::process;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .map_err(|e| Error::Repo(e.to_string()))
        .unwrap();

    let repo = jobs::MongoRepo::new(client).unwrap();

    let db_client = PickleDb::new(
        "jobs.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let db_repo = PickleDbRepo::new(db_client);

    let counter_db = PickleDb::new(
        "counter.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    let job = JobImplementer {
        db: Arc::new(Mutex::new(counter_db)),
    };

    //let mut manager = JobManager::<PickleDbRepo>::new(process::id().to_string(), db_repo);
    let mut manager = JobManager::new(process::id().to_string(), repo);

    manager.register(
        JobConfig::new("project-updater", schedule::minutely())
            .with_check_interval(Duration::from_secs(3)),
        job.clone(),
    );
    let _ = manager.start_all().await.unwrap();
    sleep(Duration::from_secs(120)).await;

    // manager
    //     .stop_by_name(String::from("project-updater"))
    //     .await
    //     .unwrap();
    // sleep(Duration::from_secs(30)).await;
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct State(i32);

#[derive(Clone)]
struct JobImplementer {
    db: Arc<Mutex<PickleDb>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Counter(i32);

#[async_trait]
impl JobAction for JobImplementer {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>> {
        let mut data: State = if state.len() == 0 {
            State(0)
        } else {
            serde_json::from_slice(&state).unwrap()
        };

        println!("Count: {}", data.0);

        sleep(Duration::from_secs(1)).await;

        data.0 += 1;

        match self.db.lock().unwrap().set("counter", &Counter(data.0)) {
            Ok(()) => Ok(serde_json::to_vec(&data).unwrap()),
            Err(_e) => Err(Error::TODO),
        }
    }
}
