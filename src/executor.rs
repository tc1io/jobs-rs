use crate::error::Error;
use crate::job::Schedule;
use crate::job::{Job, JobAction, JobConfig, JobName, JobRepo};
use crate::lock::{LockData, LockRepo};
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Mutex;

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
    pub async fn run(&mut self) -> Result<(), Error> {
        dbg!("inside run");
        loop {
            let mut action = self.action.lock().await;
            match self
                .job_repo
                .get_job(self.job_name.clone().into())
                .await? {
                None => Ok({
                    self
                        .job_repo
                        .create_job(JobConfig {
                            name: self.job_name.clone().into(),
                            state: vec![],
                            schedule: Schedule { expr: "".to_string() },
                            enabled: false,
                            last_run: 0,
                            lock: LockData { expires: 0, version: 0 },
                        }).await?;
                }),
                Some(val) => Ok({ self.job_repo.update_job(val).await?; }),
                Some(val) => Err({ self.job_repo.update_job(val).await?; })
            }.expect("TODO: panic message");

            dbg!("inside run -1 ");
            let _xx = action
                .call(self.job_name.clone().into(), Vec::new())
                .await?;
            self.cancel_signal_rx.try_recv();
            dbg!("done with call()");
        }

//         loop {
//             let mut action = self.action.lock().await;
//             let job_config = self.job_config.clone();
//             let _ji = self.job_repo.create_job(job_config).await?;
//
//             let _xx = action
//                 .call(self.job_name.clone().into(), Vec::new())
//                 .await?;
//             self.cancel_signal_rx.try_recv();
//             dbg!("done with call()");
//         }
// >>>>>>> master
        // =======
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
        Ok(())
    }
}
