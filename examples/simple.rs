use jobs::JobManager;

#[tokio::main]
async fn main() {

    let mut manager = JobManager::new();

    manager.register();

    manager.run().await.unwrap();


}