use crate::error::Error;
use crate::job::Status::{Registered, Suspended};
use async_trait::async_trait;
use derive_more::{Display, From, Into};
use std::os::unix::prelude::ExitStatusExt;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

#[async_trait]
pub trait JobAction {
    async fn call(&mut self, name: String, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[derive(Default, Clone, From, Into, Eq, Hash, PartialEq, Debug)]
pub struct JobName(pub String);

// #[derive(Clone)]
// pub struct Job {
//     pub name: JobName,
//     // pub state: Vec<u8>,
//     pub action: Arc<Mutex<dyn JobAction + Send + Sync>>,
// }

// impl Job {
//     pub fn new_with_action(name: JobName, action: impl JobAction + Send + Sync + 'static) -> Self {
//         Job {
//             name,
//             // state: Vec::new(),
//             action: Arc::new(Mutex::new(action)),
//         }
//     }
// }
#[derive(Clone, Debug)]
pub enum Status {
    Registered,
    Suspended,
    Running(Sender<()>),
    // Errored,
    // Cancelled,
}

// impl Clone for Status {
//     fn clone(&self) -> Self {
//         match self {
//             Registered => Registered,
//             Suspended => Suspended,
//             Status::Running(sender) => Status::Running(*sender),
//         }
//     }
// }

#[derive(Default, Clone)]
pub struct Config {
    name: JobName,
}

#[derive(Clone)]
pub struct Job {
    pub name: JobName,
    pub action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    pub status: Status,
    // state: Vec<u8>,
    // config: Config,
}

impl Job {
    pub fn new(name: JobName, action: impl JobAction + Send + Sync + 'static) -> Self {
        Job {
            name: name.clone(),
            action: Arc::new(Mutex::new(action)),
            status: Status::Registered,
            // config: Config { name },
            // state: Vec::new(),
        }
    }
    // pub fn set_status_running(&mut self, tx: Sender<()>) {
    //     self.status = Status::Running(tx);
    // }
    // pub async fn run(&mut self) -> Result<(), Error> {
    //     // dbg!("inside run");
    //     let mut action = self.action.lock().await;
    //     // other logic will be added
    //     let _xx = action.call(self.name.clone().into(), Vec::new()).await?;
    //     Ok(())
    // }
}

#[async_trait]
pub trait JobRepo {
    // async fn create_job(&mut self, job: JobInfo) -> Result<bool, JobError>;
    // async fn get_job(&mut self, name: &str) -> Result<Option<JobInfo>, JobError>;
    // async fn save_state(&mut self, name: &str, state: Vec<u8>) -> Result<bool, JobError>;
}
