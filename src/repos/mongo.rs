use crate::job::{JobData, JobName, LockStatus, Repo};
use crate::{Error, Result};
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use cron::Schedule;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::trace;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::{Duration, UNIX_EPOCH};
use tokio::time::sleep;

#[derive(Clone)]
pub struct MongoRepo {
    client: Client,
}

impl MongoRepo {
    pub fn new(client: Client) -> Result<MongoRepo> {
        Ok(MongoRepo { client })
    }
}

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
struct JobDto {
    pub _id: String,
    pub check_interval: u64,
    pub lock_ttl: u64,
    pub state: String,
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
            _id: value.name.0,
            check_interval: value.check_interval.as_secs(),
            lock_ttl: value.lock_ttl.as_secs(),
            state: STANDARD.encode(&value.state),
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
        let schedule = Schedule::from_str(value.schedule.as_str()).map_err(|e| {
            Error::InvalidCronExpression {
                expression: value.schedule,
                msg: e.to_string(),
            }
        })?;
        let state = STANDARD.decode(&value.state).map_err(|e| Error::TODO)?;
        Ok(Self {
            name: JobName(value._id),
            check_interval: Duration::from_secs(value.check_interval),
            lock_ttl: Duration::from_secs(value.lock_ttl),
            state,
            schedule,
            enabled: value.enabled,
            last_run: DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(value.last_run)),
        })
    }
}

pub struct Lock {
    fut: BoxFuture<'static, Result<()>>,
}

#[async_trait]
impl Repo for MongoRepo {
    type Lock = Lock;

    async fn create(&mut self, data: JobData) -> Result<()> {
        let job: JobDto = data.into();
        self.client
            .database("jobs")
            .collection::<JobDto>("job")
            .insert_one(&job, None)
            .await
            .map(|_| Ok(()))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn get(&mut self, name: JobName) -> Result<Option<JobData>> {
        let j = self
            .client
            .database("jobs")
            .collection::<JobDto>("job")
            .find_one(doc! {"_id":name.as_ref().to_string()}, None)
            .await
            .map_err(|e| Error::Repo(e.to_string()))?;

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

    async fn commit(&mut self, name: JobName, state: Vec<u8>) -> Result<()> {
        let opts: UpdateOptions = UpdateOptions::builder().upsert(false).build();
        let update_doc = doc! { "$set": doc! { "state": STANDARD.encode(&state) }};
        self.client
            .database("jobs")
            .collection::<JobDto>("job")
            .update_one(doc! {"_id":name.as_str()}, update_doc, opts)
            .await
            .map(|_| Ok(()))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn save(&mut self, name: JobName, last_run: DateTime<Utc>, state: Vec<u8>) -> Result<()> {
        let opts: UpdateOptions = UpdateOptions::builder().upsert(false).build();

        let update_doc = doc! { "$set": doc! {
            "state": STANDARD.encode(&state),
            "last_run": last_run.timestamp(),
            "owner": String::default(),
            "expires": 0,
        }};

        self.client
            .database("jobs")
            .collection::<JobDto>("job")
            .update_one(doc! {"_id":name.as_str()}, update_doc, opts)
            .await
            .map(|_| Ok(()))
            .map_err(|e| Error::Repo(e.to_string()))?
    }

    async fn lock(
        &mut self,
        name: JobName,
        owner: String,
        ttl: Duration,
    ) -> Result<LockStatus<Self::Lock>> {
        let opts = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();

        //let filter_doc = doc! {"_id":name.as_str(), "expires": {"$lt" : "78238723"}  };
        let filter_doc = doc! {"_id":name.as_str()  };

        let update_doc = doc! { "$set": doc! {
            "owner": owner,
            "expires": Utc::now().timestamp() + ttl.as_secs() as i64
        }};

        match self
            .client
            .database("jobs")
            .collection::<JobDto>("job")
            .find_one_and_update(filter_doc, update_doc, opts)
            .await
        {
            Ok(Some(res)) => {
                let name = res._id.clone();
                let owner = res.owner.clone();
                let db = self.client.clone();

                let jd: Result<JobData> = res.try_into();
                match jd {
                    Ok(k) => {
                        let fut = async move {
                            trace!("starting lock refresh");
                            loop {
                                let refresh_interval = Duration::from_secs(ttl.as_secs() / 2);
                                sleep(refresh_interval).await;

                                let opts: UpdateOptions = UpdateOptions::builder().upsert(false).build();
                                let expires = Utc::now().timestamp() + ttl.as_secs() as i64;
                                let update_doc = doc! { "$set": doc! { "owner" : owner.clone(), "expires": expires }};
                                match db
                                    .database("jobs")
                                    .collection::<JobDto>("job")
                                    .update_one(doc! {"_id":name.as_str()}, update_doc, opts)
                                    .await {
                                    Ok(_) => {}
                                    Err(e) => return Err(Error::LockRefreshFailed(e.to_string())),
                                }
                                trace!("lock refreshed");
                            }
                        }
                            .boxed();

                        let lock = Lock { fut };
                        Ok(LockStatus::Acquired(k, lock))
                    }
                    Err(e) => Err(e),
                }
            }
            Ok(None) => Err(Error::TODO),
            Err(e) => Err(Error::Repo(e.to_string())),
        }
    }
}

impl Future for Lock {
    type Output = crate::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.fut.poll_unpin(cx)
    }
}
