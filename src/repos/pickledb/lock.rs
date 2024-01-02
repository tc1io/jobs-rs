use serde::{Deserialize, Serialize};
use futures::future::BoxFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use async_trait::async_trait;
use std::time::Duration;
use chrono::Utc;
use tokio::time::sleep;
use futures::FutureExt;
use crate::job::JobName;
use crate::lock::{LockRepo, LockStatus};
use crate::repos::pickledb::Repo;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct LockData {
    pub owner: String,
    pub expires: i64,
    pub version: i8,
}


pub struct MyLock {
    repo: Repo,
    fut: Option<BoxFuture<'static, crate::Result<()>>>
}

#[async_trait]
impl LockRepo for Repo {
    type Lock = MyLock;
    async fn acquire_lock(&mut self, name: JobName, owner: String, ttl: Duration) -> crate::Result<LockStatus<Self::Lock>> {
        dbg!("1");
        let x = {

        let mut m = self
            .db
            .read()
            .await;

            m.get::<LockData>(name.0.as_str())
        };

        match x
        {
            Some(data) if data.expires > Utc::now().timestamp() => Ok(LockStatus::AlreadyLocked),
            Some( _ ) | None => {
                dbg!("2");
                let expires = Utc::now().timestamp() + ttl.as_secs() as i64;
                let secs = ttl.as_secs() as i64;
                let name2 = name.0.clone();
                dbg!("2a");
                {

               let x = self
                    .db
                    .write()
                    .await
                    .set(name.0.as_str(), &LockData{
                        owner,
                        expires,
                        version: 0,
                    }).unwrap();
                }

                dbg!("2.1");


                //x.map(|| LockStatus::Acquired(MyLock{}))
                let db = self.db.clone();
                let f = async move {

                    loop {
                        dbg!("Loop lock");
                        sleep(Duration::from_secs(1)).await;

                        //let expires = Utc::now().timestamp() + secs;
                        let expires = Utc::now().timestamp() + 10;

                            db
                            .write()
                            .await
                            .set(name2.as_str(), &LockData{
                                owner: "owner".to_owned(),
                                expires,
                                version: 0,
                            }).unwrap();
                        // TODO error return

                    }

                }.boxed();
                dbg!("3");
                Ok(LockStatus::Acquired(MyLock{repo: self.clone(),fut: Some(f)}))
            }
        }
    }
}

impl Drop for MyLock {
    fn drop(&mut self) {
        {
            println!("DROPING LOCK Future NOW");
            let f = self.fut.take();
            println!("DROPPED LOCK Future NOW");
        }
               println!("AFTER Scope block");
        // self.repo
        //     .db
        //     .write()
        //     .await
        //     .set(name.0.as_str(), &LockData{
        //         owner,
        //         expires,
        //         version: 0,
        //     }).unwrap();

    }
}

impl Future for MyLock {

    type Output = crate::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("POLL LOCK NOW");
        match self.fut.as_mut() {
            None => Poll::Ready(Ok(())),
            Some(f) => f.as_mut().poll_unpin(cx)
        }
    }
}
