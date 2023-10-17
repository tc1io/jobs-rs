use jobs::job::{Job, JobName, JobRepo, Schedule};
use jobs::{
   repos,
};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};


#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let mut repo = repos::pickledb::Repo::new(db);
    let name  = JobName {
        name: "dummmy".to_string(),
    };
    repo.create_job(Job {
        // name: JobName { name: "dummy".to_string() },
        name: name.clone(),
        state: vec![],
        schedule: Schedule {
            expr: "".to_string(),
        },
        enabled: false,
        last_run: 0,
    })
    .await
    .unwrap();

    repo.save_state( name.clone(), vec![])
        .await
        .unwrap();

    match repo
        .get_job(name)
        .await{
        Ok(Val) => {
            println!("{:?}", Val)
        }
        Err(_) => {
            println!("Not found")
        }
    }
}
