use crate::error::Error;
use crate::error::Error::GeneralError;
use crate::executor::Executor;
use crate::job;
use crate::job::{Entry, JobAction, JobName, JobRepo};
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
    executors: HashMap<JobName, Executor<J, L>>,
    jobs: HashMap<JobName, Entry>,
    execs: Vec<Executor<J, L>>,
    exec: Executor<J, L>,
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
            executors: HashMap::new(),
            jobs: HashMap::new(),
            execs: Vec::new(),
            exec: Executor::new(name.into(), job_repo.clone(), lock_repo.clone(), job_action),
        }
    }
    pub fn register(&mut self, name: String, job_action: impl JobAction + Send + Sync + 'static) {
        self.executors.insert(
            name.clone().into(),
            Executor::new(
                name.into(),
                self.job_repo.clone(),
                self.lock_repo.clone(),
                job_action,
            ),
        );
    }
    // pub fn register2(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
    //     self.jobs
    //         .insert(name.clone().into(), Entry::new(name.into(), action));
    // }
    // pub fn register3(&mut self, name: String, action: impl JobAction + Send + Sync + 'static) {
    //     self.execs.push(Executor::new(
    //         name.into(),
    //         self.job_repo.clone(),
    //         self.lock_repo.clone(),
    //         action,
    //     ));
    // }
    pub async fn start_all(&mut self, name: String) -> Result<(), Error> {
        dbg!("1");
        loop {
            // let mut exec = self.exec.clone();
            // tokio::spawn(async move { exec.run().await });
            // ---- works
            let mut executors: Vec<(JobName, Executor<J, L>)> =
                self.executors.clone().into_iter().collect();
            for (k, mut v) in executors {
                let xx = tokio::task::spawn(async {
                    let (tx, rx) = oneshot::channel();
                    // dbg!("2");
                    tokio::task::spawn(async move {
                        v.run().await.unwrap();
                        tx.send("done");
                    });
                    rx.await
                        .map_err(|e| Error::GeneralError {
                            description: String::from("not done"),
                        })
                        .unwrap();
                });
            }
        }

        // let execs: Vec<(JobName, Executor<J, L>)> = self
        //     .executors
        //     .clone()
        //     .into_iter()
        //     .filter(|(n, _)| n == JobName::from(name))
        //     .collect();
        // let mut items = Vec::new();
        // for (k, mut v) in self.executors.clone() {
        //     items.push(v.run())
        // }

        // let n: JobName = name.into();
        // let exec: Vec<Executor<J, L>> = self
        //     .execs
        //     .clone()
        //     .into_iter()
        //     .filter(|e| e.job.name == name.clone().into())
        //     .collect();
        //
        // let mut ex: &Executor<J, L> = exec.get(1).ok_or(Error::GeneralError {
        //     description: String::from("not found"),
        // })?;
        // ex.run().await.unwrap();

        // let mut executors: Vec<(JobName, Executor<J, L>)> = self
        //     .executors
        //     .clone()
        //     .into_iter()
        //     .filter(|(j, _)| j == n)
        //     .collect();
        // dbg!(executors);
        // for (_k, mut v) in self.executors.into_iter().collect() {
        //     let (tx, rx) = oneshot::channel();
        // tokio::spawn(
        // let ex = Executor::new(
        //         "".into(),
        //         self.job_repo.clone(),
        //         self.lock_repo.clone(),
        //         job_action,
        // );
        //     ex.run(rx)
        //
        //     rx

        // );

        // self.executors.insert(String::from("").into(), tx)
        // }
        Ok(())
    }
    // pub async fn start(&mut self, name: String) -> Result<(), Error> {
    //     let (tx, rx) = oneshot::channel::<()>();
    //     // let n: JobName = name.into();
    //
    //     // let ex = Executor::new(name.into(), self.clone().job_repo, self.clone().lock_repo, xx.)
    //
    //     let xx = self
    //         .jobs
    //         .entry(name.into())
    //         .and_modify(|e| e.set_status_running(tx));
    //     // dbg!(xx);
    //     // .and_modify(|e| e.set_status_running(tx))
    //     // .key();
    //     // let yy = self.jobs.get(xx);
    //     // match yy {
    //     //     Some(mut v) => {
    //     //         v.run().await?;
    //     //         // tokio::spawn(async { vv.await });
    //     //         rx.await.map_err(|e| Error::GeneralError {
    //     //             description: e.to_string(),
    //     //         })?;
    //     //         Ok(())
    //     //     }
    //     //     None => Err(Error::GeneralError {
    //     //         description: String::from("not found"),
    //     //     }),
    //     // }?;
    //     //
    //     Ok(())
    // }
}
