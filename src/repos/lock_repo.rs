// <<<<<<< HEAD
// // use std::sync::{Arc};
// // use async_trait::async_trait;
// // use chrono::Utc;
// // use pickledb::PickleDb;
// // use crate::error::Error;
// // use crate::lock::{LockData, LockRepo};
// // use crate::repos::pickledb::Repo;
// // use tokio::sync::RwLock;
// //
// //
// //
// // #[derive(Clone)]
// // pub struct LRepo {
// //     pub(crate) db: Arc<RwLock<PickleDb>>,
// // }
// //
// // impl LRepo {
// //     pub fn new(lock: PickleDb) -> Self {
// //         Self {
// //             db: Arc::new(RwLock::new(lock)),
// //         }
// //     }
// // }
// // #[async_trait]
// // impl LockRepo for LRepo {
// //     async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool,Error > {
// //         // println!("acquire lock");
// //         // let mut acquire = false;
// //         // // TODO: try functional approach
// //         // let existing_lock = self
// //         //     .db
// //         //     .read()
// //         //     .await
// //         //     .get::<LockData>(lock_data.job_name);
// //         // // .unwrap();
// //         // // .map_err(|e| JobError::DatabaseError(e.to_string()))?
// //         // match existing_lock {
// //         //     Some(lock) => {
// //         //         if lock.expires < Utc::now().timestamp_millis() {
// //         //             acquire = true;
// //         //         }
// //         //     }
// //         //     None => acquire = true,
// //         // }
// //         // if acquire {
// //         // self.db
// //         //     .write()
// //         //     .await
// //         //     .set(lock_data.job_name.as_str(), &lock_data)
// //         //     .map(|_| Ok(true))
// //         //     .map_err(|e| Error::GeneralError { description: "lock error".to_string() })?
// //         // } else {
// //         //     Ok(false)
// //         // }
// //         Ok(false)
// //     }
// // }
// =======
// use crate::error::Error;
// use crate::lock::{LockData, LockRepo};
// use async_trait::async_trait;
// use chrono::Utc;
// use pickledb::PickleDb;
// use std::sync::Arc;
// // use crate::repos::pickledb::Repo;
// use tokio::sync::RwLock;
//
// #[derive(Clone)]
// pub struct LRepo {
//     pub(crate) db: Arc<RwLock<PickleDb>>,
// }
//
// impl LRepo {
//     pub fn new(lock: PickleDb) -> Self {
//         Self {
//             db: Arc::new(RwLock::new(lock)),
//         }
//     }
// }
// #[async_trait]
// impl LockRepo for LRepo {
//     // async fn acquire_lock(&mut self, lock_data: LockData) -> Result<bool, Error> {
//     //     println!("acquire lock");
//     //     let mut acquire = false;
//     //     // TODO: try functional approach
//     //     let existing_lock = self
//     //         .db
//     //         .read()
//     //         .await
//     //         .get::<LockData>(lock_data.job_name.as_str());
//     //     // .unwrap();
//     //     // .map_err(|e| JobError::DatabaseError(e.to_string()))?
//     //     match existing_lock {
//     //         Some(lock) => {
//     //             if lock.expires < Utc::now().timestamp_millis() {
//     //                 acquire = true;
//     //             }
//     //         }
//     //         None => acquire = true,
//     //     }
//     //     if acquire {
//     //         self.db
//     //             .write()
//     //             .await
//     //             .set(lock_data.job_name.as_str(), &lock_data)
//     //             .map(|_| Ok(true))
//     //             .map_err(|e| Error::GeneralError {
//     //                 description: "lock error".to_string(),
//     //             })?
//     //     } else {
//     //         Ok(false)
//     //     }
//     // }
// }
// >>>>>>> master
