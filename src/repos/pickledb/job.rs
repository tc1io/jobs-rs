use crate::job::{JobData, JobName, LockStatus, Repo};
use crate::repos::pickledb::PickleDbRepo;
use crate::Error;
use crate::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::trace;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::{Duration, UNIX_EPOCH};
use tokio::time::sleep;
use AsRef;

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
struct JobDto {
    pub name: JobName,
    pub check_interval: u64,
    pub lock_ttl: u64,
    pub state: Vec<u8>,
    pub schedule: String,
    pub enabled: bool,
    pub last_run: u64,
    pub owner: String,
    pub expires: i64,
    pub version: i8,
}

impl From<JobData> for JobDto {
    fn from(value: JobData) -> Self {
        Self {
            name: value.name,
            check_interval: value.check_interval.as_secs(),
            lock_ttl: value.lock_ttl.as_secs(),
            state: value.state,
            schedule: value.schedule.to_string(),
            enabled: value.enabled,
            last_run: value.last_run.timestamp() as u64,
            owner: "".to_string(),
            expires: 0,
            version: 0,
        }
    }
}

impl TryFrom<JobDto> for JobData {
    type Error = Error;

    fn try_from(value: JobDto) -> std::result::Result<Self, Self::Error> {
        let schedule = CronSchedule::from_str(value.schedule.as_str()).map_err(|e| {
            Error::InvalidCronExpression {
                expression: value.schedule,
                msg: e.to_string(),
            }
        })?;
        Ok(Self {
            name: value.name,
            check_interval: Duration::from_secs(value.check_interval),
            lock_ttl: Duration::from_secs(value.lock_ttl),
            state: value.state,
            schedule,
            enabled: value.enabled,
            last_run: DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(value.last_run)),
        })
    }
}

pub struct MyLock {
    repo: PickleDbRepo,
    fut: Option<BoxFuture<'static, Result<()>>>,
}

#[async_trait]
impl Repo for PickleDbRepo {
    type Lock = MyLock;

    async fn create(&mut self, job_config: JobData) -> Result<()> {
        let job: JobDto = job_config.into();
        self.db
            .write()
            .await
            .set(job.name.as_ref(), &job)
            .map(|_| Ok(()))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn get(&mut self, name: JobName) -> Result<Option<JobData>> {
        let j = self.db.write().await.get::<JobDto>(name.as_ref());

        match j {
            None => Ok(None),
            Some(d) => {
                let jd: Result<JobData> = d.try_into();
                match jd {
                    Ok(k) => Ok(Some(k)),
                    Err(e) => Err(e),
                }
            }
        }
    }

    async fn commit(&mut self, name: JobName, state: Vec<u8>) -> crate::Result<()> {
        todo!()
    }

    async fn save(
        &mut self,
        name: JobName,
        last_run: DateTime<Utc>,
        state: Vec<u8>,
    ) -> crate::Result<()> {
        let mut w = self.db.write().await;

        let mut j = w.get::<JobDto>(name.as_ref()).ok_or(Error::TODO)?;
        j.last_run = last_run.timestamp() as u64;
        j.owner = String::default();
        j.state = state;
        j.expires = 0;
        j.version = 0;

        w.set(name.as_ref(), &j)
            .map_err(|e| Error::Repo(e.to_string()))
    }

    async fn lock(&mut self, name: JobName, owner: String) -> Result<LockStatus<Self::Lock>> {
        let mut w = self.db.write().await;

        let mut j = w.get::<JobDto>(name.as_ref()).ok_or(Error::TODO)?;
        if j.expires > Utc::now().timestamp() {
            Ok(LockStatus::AlreadyLocked)
        } else {
            j.owner = owner;
            j.expires = Utc::now().timestamp() + 10;
            j.version = 0;
            w.set(name.as_ref(), &j)
                .map_err(|e| Error::Repo(e.to_string()))
                .unwrap();

            let name = j.name.clone();
            let owner = j.owner.clone();
            let db = self.db.clone();
            let ttl_secs = 5; // TODO derive ttl from config j.xxx

            let f = async move {
                trace!("starting lock refresh");
                loop {
                    let refresh_interval = Duration::from_secs(ttl_secs / 2);
                    sleep(refresh_interval).await;
                    let mut w = db.write().await;
                    let mut j = w.get::<JobDto>(name.as_ref()).ok_or(Error::TODO).unwrap();
                    j.expires = Utc::now().timestamp() + ttl_secs as i64;
                    match w.set(name.0.as_str(), &j) {
                        Ok(()) => {}
                        Err(e) => return Err(Error::LockRefreshFailed(e.to_string())),
                    }
                    trace!("lock refreshed");
                }
            }
            .boxed();

            let lock = MyLock {
                repo: self.clone(),
                fut: Some(f),
            };

            let job_config: JobData = j.try_into()?;
            Ok(LockStatus::Acquired(job_config, lock))
        }
    }
}

impl Drop for MyLock {
    fn drop(&mut self) {
        {
            let f = self.fut.take();
        }
        trace!("droped lock refresh future");
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
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //trace!("droped lock refresh future");
        match self.fut.as_mut() {
            None => Poll::Ready(Ok(())),
            Some(f) => f.as_mut().poll_unpin(cx),
        }
    }
}
