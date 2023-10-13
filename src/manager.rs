use crate::error::Error;
use crate::error::Error::GeneralError;
use crate::executor::Executor;
use crate::job;
use crate::job::{Job, JobAction, JobName, JobRepo};
use crate::lock::LockRepo;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio_retry::Action;

pub struct JobManager<J, L>
where
    J: JobRepo + Sync + Send + Clone,
    L: LockRepo + Sync + Send + Clone,
{
    job_repo: J,
    lock_repo: L,
    // executors: HashMap<JobName, Executor<J, L>>,
    // jobs: HashMap<JobName, Entry>,
    jobs: Vec<Job>,
    // execs: Vec<Executor<J, L>>,
    // exec: Executor<J, L>,
}

impl<J: JobRepo + Clone + Send + Sync + 'static, L: LockRepo + Clone + Send + Sync + 'static>
    JobManager<J, L>
{
    pub fn new(
        job_repo: J,
        lock_repo: L,
        name: String,
        job_action: impl JobAction + Send + Sync + 'static,
    ) -> Self {
        JobManager {
            job_repo: job_repo.clone(),
            lock_repo: lock_repo.clone(),
            jobs: Vec::new(),
        }
    }
    // pub fn register(&mut self, name: String, job_action: impl JobAction + Send + Sync + 'static) {
    //     self.executors.insert(
    //         name.clone().into(),
    //         Executor::new(
    //             name.into(),
    //             self.job_repo.clone(),
    //             self.lock_repo.clone(),
    //             job_action,
    //         ),
    //     );
    // }
    pub fn register2(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
        self.jobs.push(Job::new(name.into(), action))
        // self.jobs
        //     .insert(name.clone().into(), Entry::new(name.into(), action));
    }
    // pub fn register3(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
    //     self.execs.push(Executor::new(
    //         name.into(),
    //         self.job_repo.clone(),
    //         self.lock_repo.clone(),
    //         action,
    //     ));
    // }
    // pub async fn start_all(&mut self, name: String) -> Result<(), Error> {
    //     dbg!("1");
    //     loop {
    //         let mut executors: Vec<(JobName, Executor<J, L>)> =
    //             self.executors.clone().into_iter().collect();
    //         for (k, mut v) in executors {
    //             let xx = tokio::task::spawn(async {
    //                 let (tx, rx) = oneshot::channel();
    //                 // dbg!("2");
    //                 tokio::task::spawn(async move {
    //                     v.run().await.unwrap();
    //                     tx.send("done");
    //                 });
    //                 rx.await
    //                     .map_err(|e| Error::GeneralError {
    //                         description: String::from("not done"),
    //                     })
    //                     .unwrap();
    //             });
    //         }
    //     }
    // }
    pub async fn start_all(&mut self) -> Result<(), Error> {
        loop {
            let jobs: Vec<Job> = self.jobs.clone().into_iter().collect();
            for mut job in jobs {
                let mut ex =
                    Executor::new(job.clone(), self.job_repo.clone(), self.lock_repo.clone());
                tokio::task::spawn(async {
                    let (tx, rx) = oneshot::channel();
                    // dbg!(x.name);
                    tokio::task::spawn(async move {
                        ex.run().await;
                        tx.send("done");
                    });
                    rx.await
                        .map_err(|e| Error::GeneralError {
                            description: String::from("not done"),
                        })
                        .unwrap();
                    // let n: JobName = name.into();
                });
            }
        }
    }
}
