# jobs-rs
Background job control crate



# Design

- get lock for a job
- start sth that refreshes the ttl of the lock in intervals
- start the job function until it completes

- get lock for a job
- start sth that refreshes the ttl of the lock in intervals
  - start and keep running, stop (stop signal?, must not stop if job is running, must stop if job stops or crashes)
  - it is concurrent
- start the job function until it completes


execute_job(stopSignaleChannel) {

    toucherFuture = LockToucher::new()

    runtime.spawn(toucherFuture) -> spand in background

    jobFuture = the-job

    runtime.spawn(jobFuture) -> spand in

    select!{
    if tocuher exists ->
       drop job

    if job exist =>
    match jobGFuture.await {
      Ok(newState) => save state
      Err(err) => {
         // stop toucher
         log err
    }
    }

}