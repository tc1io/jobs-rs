use hyper;
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use hyper_rustls::HttpsConnector;
use jobs::manager::JobManager;
use jobs::repos;
use jobs::repos::pickledb::PickleDbRepo;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{oneshot, Mutex};
use tokio::time;
use warp::Filter;

#[derive(Debug)]
struct MyError(String);

impl warp::reject::Reject for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl std::error::Error for MyError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_s = env::var("PORT").unwrap_or("8080".into());
    let port: u16 = port_s.parse().expect("parse port env var to u16 failed");
    let https = HttpsConnector::with_native_roots();
    let db_client = PickleDb::new(
        "example.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let lock_client = PickleDb::new(
        "lock.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );
    let project_db = PickleDb::load(
        "project.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
    .unwrap();

    let db_repo = repos::pickledb::PickleDbRepo::new(db_client);
    let lock_repo = repos::pickledb::PickleDbRepo::new(lock_client);

    let job = JobImplementer {
        db: Arc::new(Mutex::new(project_db)),
    };

    let mut manager = JobManager::<PickleDbRepo, PickleDbRepo>::new(db_repo, lock_repo);
    let client: Client<_, Body> = Client::builder().build(https);

    // let (tx, rx) = oneshot::channel();
    let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();
    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    let (_, server) = warp::serve(routes(client, job.clone(), manager))
        .try_bind_with_graceful_shutdown(sock_addr, async {
            rx.await.ok();
        })?;
    tokio::spawn(server);
    let _ = signal::ctrl_c().await;
    let _ = tx.send(());
    tokio::time::sleep(time::Duration::from_secs(1)).await;

    Ok(())
}

pub fn routes(
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
    job: JobImplementer,
    manager: JobManager<PickleDbRepo, PickleDbRepo>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let hello = warp::path("hello")
        .and(warp::get())
        .and_then(say_hello)
        .recover(handle_rejection);
    let reg = warp::path("register")
        .and(warp::get())
        .and_then(manage)
        .recover(handle_rejection);

    reg
}

#[derive(Clone)]
pub struct JobImplementer {
    db: Arc<Mutex<PickleDb>>,
}

pub async fn say_hello() -> Result<impl warp::Reply, warp::Rejection> {
    let value = sayy_hello()
        .await
        .map_err(|err| warp::reject::custom(MyError(err.to_string())))?;
    Ok(warp::reply::json(&value))
}

pub async fn sayy_hello() -> Result<String, Box<dyn std::error::Error>> {
    Ok("Hello from test handle...!!!".to_string())
}

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    let code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
    let message = "Internal Server Error"; // This is a &'static str

    Ok(warp::reply::with_status(message, code))
}

pub async fn manage() -> Result<impl warp::Reply, warp::Rejection> {
    let value = manage_jobs()
        .await
        .map_err(|err| warp::reject::custom(MyError(err.to_string())))?;
    Ok(warp::reply::json(&value))
}

pub async fn manage_jobs() -> Result<String, Box<dyn std::error::Error>> {
    Ok("Hello from job manager...!!!".to_string())
}
