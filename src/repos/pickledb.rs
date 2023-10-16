use crate::error::Error;
use crate::job::{Job, JobName, JobRepo};
use async_trait::async_trait;
use pickledb::PickleDb;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Repo {
    db: Arc<RwLock<PickleDb>>,
}

impl Repo {
    pub fn new(db: PickleDb) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}

#[async_trait]
impl JobRepo for Repo {
    async fn create_job(&mut self, job: Job) -> Result<bool, Error> {
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true)) // TODO
            .map_err(|e| Error::GeneralError {
                description: "job creation failed".to_string(),
            })?
    }

    async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error> {
        todo!()
    }

    async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
        todo!()
    }
    // async fn create_job(&mut self, job: Job) -> Result<bool, Error> {
    //     dbg!("2");
    //     println!("create_job");
    //     let name = (&job.name).into();
    //     self.db
    //         .write()
    //         .await
    //         .set(name, &job)
    //         .map(|_| Ok(true))
    //         .map_err(|e| Error::GeneralError { description: "job creation failed".to_string() })?
    // }
    //
    // async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error> {
    //     Ok(self
    //         .db
    //         .write()
    //         .await
    //         // .map_err(|e| Error::GeneralError { description: "".to_string() })?
    //         .get::<Job>((&name).into()))
    // }
    //
    // async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
    //     let mut job = self.get_job(name.clone()).await.unwrap().unwrap();
    //     job.state = state;
    //     // let name1 = name.clone()
    //     job.last_run = Utc::now().timestamp_millis();
    //     dbg!("{:?}", job.last_run);
    //     self.db
    //         .write()
    //         .await
    //         // .map_err(|e| JobError::DatabaseError(e.to_string()))?
    //         .set((&name).into(), &job)
    //         .unwrap();
    //     println!("state saved");
    //     Ok(true)
    // }
}
// struct JobImplementer {
//     // name: String,
//     // db: PickleDb,
//     // project: Project,
// }
