use crate::error::Error;
use crate::job::Schedule;
use crate::job::{Job, JobAction, JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use futures::FutureExt;
use std::sync::Arc;
use std::thread::sleep;
// use std::time;
use crate::error::Error::GeneralError;
use crate::executor::State::{Create, Run, Start, Timer};
use std::time::{Duration, Instant};
use tokio::select;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;
use tokio::sync::{oneshot, Mutex};
use tokio::time;
use tokio::time::Timeout;
use tokio_timer::Delay;

// #[derive(Clone)]
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    // pub job: Job,
    job_name: JobName,
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    job_config: JobConfig,
    cancel_signal_rx: Receiver<()>,
    job_repo: J,
    lock_repo: L,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        job_name: JobName,
        action: Arc<Mutex<dyn JobAction + Send + Sync>>,
        schedule: Schedule,
        job_repo: J,
        lock_repo: L,
        cancel_signal_rx: Receiver<()>,
    ) -> Self {
        Executor {
            job_name: job_name.clone(),
            action,
            job_config: JobConfig::new(job_name, schedule),
            cancel_signal_rx,
            job_repo,
            lock_repo,
        }
    }
    pub async fn run(&mut self) -> Option<State> {
        Some(Start())
        // dbg!("inside run");
        // let mut interval = time::interval(time::Duration::from_secs(
        //     self.job_config.clone().check_interval_sec,
        // ));
        // // loop {
        // match self.cancel_signal_rx.try_recv() {
        //     Ok(_) => {
        //         dbg!("received stopped signal....", self.job_name.clone());
        //         return Ok(None);
        //     }
        //     Err(e) => {}
        // }
        // interval.tick().await; // TODO: waits for timer.. should be a better solution
        //
        // let mut action = self.action.lock().await;
        // let job_config = self.job_config.clone();
        // let _ji = self.job_repo.create_job(job_config).await?;
        //
        // let _xx = action
        //     .call(self.job_name.clone().into(), Vec::new())
        //     .await?;
        // // }
        // Ok(None)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Start(),
    Create(),
    Run(),
}

impl State {
    pub fn init() -> State {
        Start()
    }
    pub async fn execute<J: JobRepo + Sync + Send + Clone, L: LockRepo + Sync + Send + Clone>(
        &mut self,
        ex: &mut Executor<J, L>,
    ) -> Result<Option<State>, Error> {
        return match self {
            Start() => {
                let mut interval = time::interval(time::Duration::from_secs(
                    ex.job_config.clone().check_interval_sec,
                ));
                match ex.cancel_signal_rx.try_recv() {
                    Ok(_) => {
                        dbg!("received stopped signal....", ex.job_name.clone());
                        return Ok(None);
                    }
                    Err(e) => {}
                }
                interval.tick().await; // TODO: waits for timer.. should be a better solution
                Ok(Some(Create()))
            }
            Create() => {
                let job_config = ex.job_config.clone();
                let _ji = ex.job_repo.create_job(job_config).await?;
                Ok(Some(Run()))
            }
            Run() => {
                dbg!("run.. returning check");
                let mut action = ex.action.lock().await;

                let _xx = action.call(ex.job_name.clone().into(), Vec::new()).await?;
                Ok(Some(Start()))
            }
        };
    }
}

// ======= kept for reference.. Aravind will remove
//         dbg!("inside run");
//         // let mut action = self.job.action.lock().await;
//         // other logic will be added
//         let name = self.job.clone().name.clone();
//         let name1 = name.clone();
//         let ji = self
//             .job_repo
//             .create_job(Job {
//                 name,
//                 state: vec![],
//                 // action: Arc::new(()),
//                 schedule: Schedule { expr: "".to_string() },
//                 enabled: false,
//                 last_run: 0,
//             })
//             .await?;
//         if ji {
//             println!("job created")
//         }
//
//        match self
//             .job_repo
//             .get_job(name1.clone())
//             .await? {
//            None => {}
//            Some(value) => {
//                // Job::should_run_now(value).unwrap().expect("TODO: panic message")
//            }
//        }
//         // let ji = self
//         //     .job_repo
//         //     .create_job(ji.clone())
//         //     .await?;
//
//         // let _r = self.job_repo.create_job(ji.clone()).await?;
// >>>>>>> master
