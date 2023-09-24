

// func(context.Context, func(state []byte) error, []byte) ([]byte, error)

// func(context.Context, state []byte) ([]byte, error)

pub struct Schedule{}

pub trait Job {

}


pub struct JobManager{
    job: Option<Box<dyn Job>>,
}




impl JobManager {
    pub fn new() -> Self {
        Self{
            job: None
        }
    }

    pub fn register(&mut self,schedule: Schedule, job: impl Job) {
        self.job = Some(job)

    }


    pub async fn run(&mut self) -> Result<(), i32> {
        println!("Run");

        // let job = self.job.unwrap();

        // From DB...
        // let state = Vec::<u8>::new();

        // match job.call(state).await {
        //     Err(e) => todo!(),
        //     Ok(s) => todo!(), // save new state
        // }
        //

        Ok(())
    }
}