use crate::error::Error;
use crate::job::Schedule;
use crate::job::{Job, JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Mutex;

// #[derive(Clone)]
pub struct Executor<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    // pub job: Job,
    job_name: JobName,
    action: Arc<Mutex<dyn JobAction + Send + Sync>>,
    cancel_signal_rx: Receiver<()>,
}

impl<J: JobRepo + Clone + Send + Sync, L: LockRepo + Clone + Send + Sync> Executor<J, L> {
    pub fn new(
        job_name: JobName,
        action: Arc<Mutex<dyn JobAction + Send + Sync>>,
        job_repo: J,
        lock_repo: L,
        cancel_signal_rx: Receiver<()>,
    ) -> Self {
        Executor {
            job_repo,
            lock_repo,
            job_name,
            action,
            cancel_signal_rx,
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        // dbg!("inside run");
        let mut action = self.action.lock().await;
        // .map_err(|e| GeneralError {
        //     description: e.to_string(),
        // })?;
        // other logic will be added
        let _xx = action
            .call(self.job_name.clone().into(), Vec::new())
            .await?;
        self.cancel_signal_rx.try_recv();
        dbg!("done with call()");
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
