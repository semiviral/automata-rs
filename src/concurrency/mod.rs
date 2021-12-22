use crossbeam_channel::{Receiver, RecvError, RecvTimeoutError, SendError, Sender};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

/// Internal struct used to wrap job data.
struct Job {
    work: Box<dyn FnOnce() + Send>,
    completion: Arc<AtomicBool>,
}

impl Job {
    /// Consumes the job and executes the contained function.
    fn execute(self) {
        self.work.call_once(());
        self.completion.store(true, Ordering::Relaxed);
    }
}

/// Wrapper type representing the completion of a successfully queued job.
pub struct JobCompletion(Arc<AtomicBool>);

impl JobCompletion {
    /// Returns true is the job's function has been executed; otherwise, false.
    pub fn is_complete(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

lazy_static::lazy_static! {
    static ref POOL: Mutex<(Vec<JoinHandle<()>>, (Sender<Job>, Receiver<Job>))> = Mutex::new((Vec::new(), crossbeam_channel::unbounded()));
}

/// Stops all workers currently alive in the pool, and reinitializes
/// the pool with the given count of workers.
pub fn set_worker_count(worker_count: usize) {
    stop_workers();

    let mut pool_lock = POOL.lock().unwrap();

    for worker_num in 0..worker_count {
        let job_receiver_clone = pool_lock.1 .1.clone();

        pool_lock.0.push(std::thread::spawn({
            move || {
                debug!("Worker #{} spawned.", worker_num);

                loop {
                    match job_receiver_clone.recv() {
                        Ok(job) => job.execute(),
                        Err(RecvError) => break,
                    }
                }

                debug!("Worker #{} killed.", worker_num);
            }
        }));
    }
}

/// Stops and removes all workers from the pool.
pub fn stop_workers() {
    let mut pool_lock = POOL.lock().unwrap();

    pool_lock.0.clear();
    pool_lock.1 = crossbeam_channel::unbounded();
}

/// Queues work onto the pool, returning a completion
pub fn queue(work: Box<dyn FnOnce() + Send>) -> Result<JobCompletion, ()> {
    let completion = Arc::new(AtomicBool::new(false));
    let completion_clone = Arc::clone(&completion);

    POOL.lock()
        .unwrap()
        .1
         .0
        .send(Job { work, completion })
        .map_or(Err(()), |_| Ok(JobCompletion(completion_clone)))
}
