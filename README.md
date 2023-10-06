# Job Manager

Job Manager is a Rust library for managing asynchronous jobs and their scheduling. It provides a flexible and extensible framework for handling jobs and their execution.

## Features

- Asynchronous job execution.
- Job scheduling using cron-like expressions.
- Flexible job runner implementations.
- Extensible for custom job and lock repositories.
- Lock management for job synchronization.

## Usage

To use Job Manager, We need to define a custom job runners, create a job manager, and register jobs for execution.

### Defining a Custom Job Runner

We can define a custom job runner by implementing the `JobRunner` trait. This trait requires us to implement the `call` method, which contains our custom job logic.

```rust
use async_trait::async_trait;
use std::fmt::Error;

#[async_trait]
pub trait JobRunner {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error>;
}
```

### Creating a Job Manager

To create a Job Manager, initialize it with a job repository (`JobsRepo`) and a lock repository (`LockRepo`).

```rust
use my_job_manager::{JobManager, JobsRepo, LockRepo};

let job_manager = JobManager::new(MyJobRepo, MyLockRepo);
```

### Registering a Job

Registering a job involves providing a name, schedule, and the job runner implementation.

```rust
let schedule = Schedule { expr: "*/5 * * * * * *".to_string() };
job_manager.register("my_job", schedule, MyJobRunner).await?;
```

### Starting the Job Manager

Once we've registered your jobs, we can start the Job Manager to execute them.

```rust
job_manager.start().await?;
```

## Example Usage

Here's a complete example of using Job Manager to manage and execute asynchronous jobs:

```rust
use async_trait::async_trait;
use my_job_manager::{JobManager, JobsRepo, LockRepo, Schedule};
use std::fmt::Error;
use std::time::Duration;
use tokio::time::sleep;

// Define a custom job runner that implements the JobRunner trait.
struct MyJobRunner;

#[async_trait]
impl my_job_manager::JobRunner for MyJobRunner {
    async fn call(&mut self, state: Vec<u8>) -> Result<Vec<u8>, Error> {
        // Implement custom job logic here.
        Ok(state)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize job manager with a job repository and lock repository.
    let job_manager = JobManager::new(MyJobRepo, MyLockRepo);

    // Register a job with the job manager.
    let schedule = Schedule { expr: "*/5 * * * * * *".to_string() };
    job_manager.register("my_job", schedule, MyJobRunner).await?;

    // Start the job manager, which will execute registered jobs.
    job_manager.start().await?;

    // Keep the program running.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
```

## License

This project is licensed under the [XXXXXXXX](LICENSE) LICENSE.

## Author

Aravind Ragavendran,
Ashesh Mishra

Feel free to modify this README to suit your project's needs, including adding more details, usage examples, and contact information.