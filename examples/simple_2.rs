use async_trait::async_trait;
use jobs::job::Schedule;
use jobs::repos::pickledb::Repo;
use jobs::{job::JobAction, manager::JobManager, repos};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let db_client = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let lock_client = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let db_repo = repos::pickledb::Repo::new(db_client);
    let lock_repo = repos::pickledb::Repo::new(lock_client);

    let job = JobImplementer {};

    let mut manager = JobManager::<Repo, Repo>::new(db_repo, lock_repo);
    manager.register(
        String::from("project-updater"),
        job.clone(),
        Schedule {
            expr: "* */3 * * * *".to_string(),
        },
    );
    manager.register(
        String::from("project-puller"),
        job,
        Schedule {
            expr: "* */2 * * * *".to_string(),
        },
    );

    let _ = manager.start_all().await.unwrap();
    sleep(Duration::from_secs(4)).await;
    manager
        .stop_by_name("project-puller".to_string())
        .await
        .unwrap();
    sleep(Duration::from_secs(10)).await;
}

#[derive(Clone)]
struct JobImplementer {}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    id: i32,
    lifecycle_state: String,
    updated: String,
}

#[async_trait]
impl JobAction for JobImplementer {
    async fn call(&mut self, name: String, _state: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        dbg!(name);
        let state = Vec::new();
        Ok(state)
    }
}
