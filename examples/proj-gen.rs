use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let mut db = PickleDb::new(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    for i in 1..5 {
        println!("{:?}", i);
        let mut lifecycle_state = "CREATED".to_string();
        if i % 2 != 0 {
            lifecycle_state = "DELETED".to_string();
        }
        let proj = Project {
            name: format!("project-{:?}", i.clone()),
            id: i.clone(),
            lifecycle_state,
            updated: "".to_string(),
        };
        db.set(format!("{:?}", i.clone()).as_str(), &proj).unwrap();
    }
    let all_data = db.get_all();
    for a in all_data {
        let data = db.get::<Project>(&a).unwrap();
        println!("{:?}", data);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    id: i32,
    lifecycle_state: String,
    updated: String,
}
