use std::future::Future;
use crate::job::{JobConfig, JobName, JobRepo};
use crate::lock::{LockRepo, LockStatus};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use pickledb::PickleDb;
use std::ops::{Add, Deref};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time;
use std::time::Duration;
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};


#[derive(Clone)]
pub struct Repo {
    pub(crate) db: Arc<RwLock<PickleDb>>,
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
    async fn create_or_update_job(&mut self, job: JobConfig) -> Result<bool> {
        //println!("create_or_update: {:?}",&job);
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(true))
            .map_err(|e| anyhow!(e.to_string()))?
    }
    async fn get_job(&mut self, name: JobName) -> Result<Option<JobConfig>> {
        let j = self.db.write().await.get::<JobConfig>(name.as_ref());
        //dbg!(&j);
        Ok(j)
    }
    async fn save_state(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> Result<bool> {
        // println!("Save state: {},{},{:?}",name,last_run,state);
        let name1 = name.clone();
        let mut job = self
            .get_job(name.into())
            .await
            .map_err(|e| anyhow!(e))?
            .ok_or(anyhow!("job not found"))?;

        job.state = state;
        job.last_run = last_run;
        self.db
            .write()
            .await
            .set(name1.as_ref(), &job)
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(true)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct LockData {
    pub owner: String,
    pub expires: i64,
    pub version: i8,
}


struct MyLock<'a> {
    fut: BoxFuture<'a,Result<()>>
}

#[async_trait]
impl LockRepo for Repo {
    type Lock = MyLock<'static>;
    async fn acquire_lock(&mut self, name: JobName, owner: String, ttl: Duration) -> Result<LockStatus<Self::Lock>> {
        match self
            .db
            .read()
            .await
            .get::<LockData>(name.0.as_str())
        {
            Some(data) if data.expires > Utc::now().timestamp() => Ok(LockStatus::AlreadyLocked),
            _ => {
                let expires = Utc::now().timestamp() + ttl.as_secs() as i64;
                let secs = ttl.as_secs() as i64;
                let name2 = "name.clone().0.as_str()";
                let x = self
                    .db
                    .write()
                    .await
                    .set(name.0.as_str(), &LockData{
                        owner,
                        expires,
                        version: 0,
                    }).unwrap();

                //x.map(|| LockStatus::Acquired(MyLock{}))
                let f = async {

                    loop {
                        println!("Loop lock");
                        sleep(Duration::from_secs(10)).await;

                        //let expires = Utc::now().timestamp() + secs;
                        let expires = Utc::now().timestamp() + 10;

                        self
                            .db
                            .write()
                            .await
                            .set(name2, &LockData{
                                owner: "owner".to_owned(),
                                expires,
                                version: 0,
                            }).unwrap();

                    }

                }.boxed();
                Ok(LockStatus::Acquired(MyLock{fut: f}))
            }
        }
    }
}

impl Drop for MyLock {
    fn drop(&mut self) {
        println!("DROPING LOCK Future NOW");
        //self.fut.drop();
        println!("DROPPED LOCK Future NOW");
    }
}

impl Future for MyLock {

    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("POLL LOCK NOW");
        //self.fut.deref().poll(cx)
        Poll::Pending
    }
}