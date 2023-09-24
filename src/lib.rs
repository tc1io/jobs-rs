
pub struct JobManager{}

impl JobManager {
    pub fn new() -> Self {
        Self{}
    }

    pub fn register(&mut self) {

    }


    pub async fn run(&mut self) -> Result<(), i32> {
        println!("Run");
        Ok(())
    }
}