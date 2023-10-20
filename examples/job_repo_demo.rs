use jobs::job::{Job, JobConfig, JobName, JobRepo, Schedule};
use jobs::{
   repos,
};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use jobs::lock::{LockData, LockRepo};


#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut repo = repos::pickledb::Repo::new(db);
    let job_name = JobName("test".to_string());
    let job_name1 = job_name.clone();

    repo.create_or_update_job(JobConfig {
        // name: JobName { name: "dummy".to_string() },
        name: job_name.clone(),
        check_interval_sec: 0,
        state: vec![],
        schedule: Schedule {
            expr: "".to_string(),
        },
        enabled: false,
        last_run: 0,
        lock: LockData {
            expires: 0,
            version: 0,
            // lock_ttl: (),
            ttl: Default::default(),
        },
    })
        .await
        .unwrap();

    repo.save_state(job_name, vec![])
        .await
        .unwrap();

    // match repo
    //     .get_job(job_name1.into())
    //     .await {
    //     Ok(Val) => {
    //         println!("{:?}", Val)
    //     }
    //     Err(_) => {
    //         println!("Not found")
    //     }
    // }

    repo.acquire_lock(JobConfig {

        name: job_name1.into(),
        check_interval_sec: 0,
        state: vec![],
        schedule: Schedule {
            expr: "".to_string(),
        },
        enabled: false,
        last_run: 0,
        lock: LockData {
            expires: 0,
            version: 0,
            ttl: Default::default(),
        },
        // lock_ttl: (),
    })
        .await.unwrap();
}


