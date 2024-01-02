use async_trait::async_trait;
use jobs::job::Schedule;
use jobs::repos::pickledb::Repo;
use jobs::Result;
use jobs::{job::JobAction, manager::JobManager, repos};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::process;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
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
    let project_db = PickleDb::load(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
    .unwrap();
    let db_repo = repos::pickledb::Repo::new(db_client);
    let lock_repo = repos::pickledb::Repo::new(lock_client);

    let job = JobImplementer {
        db: Arc::new(Mutex::new(project_db)),
    };

    let pid = process::id();

    let mut manager = JobManager::<Repo, Repo>::new(pid.to_string(), db_repo, lock_repo);
    manager.register(
        String::from("project-updater"),
        job.clone(),
        Schedule {
            expr: "0 * * * * *".to_string(),
        },
    );
    let _ = manager.start_all().await.unwrap();
    sleep(Duration::from_secs(4)).await;
    manager
        .stop_by_name(String::from("project-updater"))
        .await
        .unwrap();
    sleep(Duration::from_secs(300)).await;
}

#[derive(Clone)]
struct JobImplementer {
    db: Arc<Mutex<PickleDb>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Project {
    name: String,
    id: i32,
    lifecycle_state: String,
    updated: String,
}

#[async_trait]
impl JobAction for JobImplementer {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>> {
        let s: String = if state.len() == 0 {
            String::default()
        } else {
            String::from_utf8(state).unwrap()
        };
        println!("IN: {}", &s);

        let s = format!("{}.foo", s);

        // let all_data = self
        //     .db
        //     .lock()
        //     .map_err(|e| anyhow!(e.to_string()))?
        //     .get_all();
        // for a in all_data {
        //     let maybe_project = self
        //         .db
        //         .lock()
        //         .map_err(|e| anyhow!(e.to_string()))?
        //         .get::<Project>(&a);
        //
        //     if let Some(mut project) = maybe_project {
        //         if project.lifecycle_state == "DELETED" {
        //             project.updated = "DONE".to_string();
        //             self.db
        //                 .lock()
        //                 .map_err(|e| anyhow!(e.to_string()))?
        //                 .set(format!("{:?}", project.id.clone()).as_str(), &project)?;
        //             println!("{:?}", project);
        //         }
        //     }
        // }
        sleep(Duration::from_secs(1)).await;
        dbg!("work-1");
        sleep(Duration::from_secs(2)).await;
        dbg!("work-2");
        sleep(Duration::from_secs(3)).await;
        dbg!("work-3");
        sleep(Duration::from_secs(4)).await;
        dbg!("work-4");
        sleep(Duration::from_secs(5)).await;
        dbg!("work-5");
        let state = Vec::from(s.as_bytes());
        Ok(state)
    }
}
