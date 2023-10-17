use jobs::job::{Job, JobName, JobRepo, Schedule};
use jobs::{lock, repos};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use jobs::lock::LockData;
use lock::LockRepo;

#[tokio::main]
async fn main() {
    let mut ldb = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut lrepo = repos::pickledb::Repo::new(ldb);
    // let name  = JobName {
    //     name: "dummmy".to_string(),
    // };
    //
    // lrepo.acquire_lock( LockData {
    //     job_name: "dummy".to_string(),
    //     expires: 0,
    //     version: 0,
    // })
    //     .await
    //     .unwrap();
}

