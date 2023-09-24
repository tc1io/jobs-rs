use jobs::JobManager;





#[tokio::main]
async fn main() {

    let repo = RedisJobRep::new()


    let job1 = ???;
    let job2 = ???;


    let mut manager = JobManager::new(repo);

    manager.register(job1);
    manager.register(job2);

    manager.run().await.unwrap();


}