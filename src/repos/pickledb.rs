use crate::error::Error;
use crate::job::{Job, JobName, JobRepo};
use async_trait::async_trait;
use chrono::Utc;
use pickledb::PickleDb;
use std::sync::Arc;
use tokio::sync::RwLock;

// #[derive(Clone)]
// pub struct Repo {
//     pub(crate) db: Arc<RwLock<PickleDb>>,
// }
//
// impl Repo {
//     pub fn new(db: PickleDb) -> Self {
//         Self {
//             db: Arc::new(RwLock::new(db)),
//         }
//     }
// }
//
// #[async_trait]
// impl JobRepo for Repo {
//     async fn create_job(&mut self, job: Job) -> Result<bool, Error> {
//         println!("create job");
//         self.db
//             .write()
//             .await
//             .set(job.name.as_ref(), &job)
//             .map(|_| Ok(true)) // TODO
//             .map_err(|e| Error::GeneralError {
//                 description: "job creation failed".to_string(),
//             })?
//
//     }
//     async fn get_job(&mut self, name: JobName) -> Result<Option<Job>, Error> {
//         Ok(self
//             .db
//             .write()
//             .await
//             .get::<Job>((&name).into()))
//     }
//
//     async fn save_state(&mut self, name: JobName, state: Vec<u8>) -> Result<bool, Error> {
//         let mut job = self.get_job(name.clone()).await.unwrap().unwrap();
//         job.state = state;
//         // let name1 = name.clone()
//         // job.last_run = Utc::now().timestamp_millis();
//         self.db
//             .write()
//             .await
//             // .map_err(|e| JobError::DatabaseError(e.to_string()))?
//             .set((&name).into(), &job)
//             .unwrap();
//         println!("state saved");
//         Ok(true)
//     }
// }
//
//
