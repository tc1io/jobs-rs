use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
// use mongodb::bson::doc;
// use crate::Error;
use crate::job::{JobData, JobName, LockStatus, Repo};
use crate::repos::mongo::MongoRepo;
use crate::repos::pickledb::PickleDbRepo;
use futures::FutureExt;

pub struct MyLock {
    repo: PickleDbRepo,
    fut: Option<BoxFuture<'static, crate::Result<()>>>,
}

#[async_trait]
impl Repo for MongoRepo {
    type Lock = MyLock;

    async fn create(&mut self, job: JobData) -> crate::Result<()> {
        todo!()
    }

    async fn get(&mut self, name: JobName) -> crate::Result<Option<JobData>> {
        todo!()
    }

    async fn commit(
        &mut self,
        name: JobName,
        last_run: DateTime<Utc>,
        state: Vec<u8>,
    ) -> crate::Result<()> {
        todo!()
    }

    async fn save(
        &mut self,
        name: JobName,
        last_run: DateTime<Utc>,
        state: Vec<u8>,
    ) -> crate::Result<()> {
        todo!()
    }

    async fn lock(
        &mut self,
        name: JobName,
        owner: String,
    ) -> crate::Result<LockStatus<Self::Lock>> {
        todo!()
    }
    // async fn create_or_update_job(&mut self, job: JobConfig) -> crate::Result<bool> {
    //     println!("create_job");
    //     self.client
    //         .database("example")
    //         .collection::<JobConfig>("job")
    //         .insert_one(&job, None)
    //         .await
    //         .map(|_| Ok(true))
    //         .map_err(|e| Error::Repo(e.to_string()))?
    // }
    //
    // async fn get_job(&mut self, name: JobName) -> crate::Result<Option<JobConfig>> {
    //     let c = self.client.clone();
    //     let jc = c
    //         .database("example")
    //         .collection::<JobConfig>("job")
    //         .find_one(doc! {"name":name.as_ref().to_string()}, None)
    //         .await
    //         .map_err(|e| Error::Repo(e.to_string()))?;
    //     Ok(jc)
    // }
    //
    // async fn save_state(&mut self, name: JobName, last_run: i64, state: Vec<u8>) -> crate::Result<()> {
    //     // TODO exactly solve this need to clone just because there is an error later
    //     let mut job = self
    //         .get_job(name.clone().into())
    //         .await
    //         .map_err(|e| Error::Repo(e.to_string()))?
    //         .ok_or(Error::JobNotFound(name))?;
    //
    //     job.state = state;
    //     job.last_run = last_run;
    //
    //     self.client
    //         .clone()
    //         .database("example")
    //         .collection::<JobConfig>("job")
    //         .insert_one(&job, None)
    //         .await
    //         .map(|_| Ok(()))
    //         .map_err(|e| Error::Repo(e.to_string()))?
    // }
}
impl Future for MyLock {
    type Output = crate::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("POLL LOCK NOW");
        match self.fut.as_mut() {
            None => Poll::Ready(Ok(())),
            Some(f) => f.as_mut().poll_unpin(cx),
        }
    }
}
