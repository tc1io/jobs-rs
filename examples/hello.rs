use hyper;
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use hyper_rustls::HttpsConnector;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::signal;
use tokio::sync::oneshot;
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
    let client: Client<_, Body> = Client::builder().build(https);

    // let (tx, rx) = oneshot::channel();
    let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();
    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    let (_, server) =
        warp::serve(routes(client)).try_bind_with_graceful_shutdown(sock_addr, async {
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
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let hello = warp::path("hello")
        .and(warp::get())
        .and_then(say_hello)
        .recover(handle_rejection);

    hello
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
