use crossbeam_channel::{Receiver, RecvTimeoutError, SendError, Sender};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

/// Internal struct used to wrap job data.
struct Job {
    work: Box<dyn FnOnce()>,
    completion: Arc<AtomicBool>,
}

impl Job {
    /// Consumes the job and executes the contained function.
    fn execute(self) {
        self.work.call_once(());
        self.completion.store(true, Ordering::Release);
    }
}

unsafe impl Send for Job {}

/// Wrapper type representing the completion of a successfully queued job.
pub struct JobCompletion(Arc<AtomicBool>);

impl JobCompletion {
    /// Returns true is the job's function has been executed; otherwise, false.
    pub fn is_complete(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Wrapper type representing the components of a bounded worker pool.
struct BoundedWorkerPool {
    workers: Mutex<Vec<(Arc<AtomicBool>, Arc<AtomicBool>, JoinHandle<()>)>>,
    job_sender: Sender<Job>,
    job_receiver: Arc<Receiver<Job>>,
    die: Arc<AtomicBool>,
}

impl BoundedWorkerPool {
    pub fn new() -> Self {
        let threads = Mutex::new(Vec::new());
        let (job_sender, job_receiver) = crossbeam_channel::unbounded();
        let job_receiver = Arc::new(job_receiver);
        let die = Arc::new(AtomicBool::new(false));

        Self {
            workers: threads,
            job_sender,
            job_receiver,
            die,
        }
    }

    /// Thread-safe function used to spawn worker threads, as well as keep them alive and
    /// kill them if need-be.
    fn worker(
        global_die: Arc<AtomicBool>,
        local_die: Arc<AtomicBool>,
        has_job: Arc<AtomicBool>,
        jobs: Arc<Receiver<Job>>,
    ) {
        debug!("Spawned thread successfully, entering receiver loop.");

        while !global_die.load(Ordering::Acquire) && !local_die.load(Ordering::Acquire) {
            match jobs.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(job) => {
                    has_job.store(true, Ordering::Release);
                    job.execute();
                    has_job.store(false, Ordering::Release)
                }
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }
}

lazy_static::lazy_static! {
    static ref POOL: BoundedWorkerPool = BoundedWorkerPool::new();
}

/// Stops all workers currently alive in the pool, and reinitializes
/// the pool with the given count of workers.
pub fn set_worker_count(worker_count: usize) {
    stop_workers();

    let mut threads_lock = POOL.workers.lock().unwrap();

    for count in 0..worker_count {
        debug!("Spawning bounded invocation pool thread #{}.", count);

        let local_die = Arc::new(AtomicBool::new(false));
        let has_job = Arc::new(AtomicBool::new(false));
        let thread = std::thread::spawn({
            let die_clone = Arc::clone(&POOL.die);
            let local_die_clone = Arc::clone(&local_die);
            let has_job_clone = Arc::clone(&has_job);
            let job_receiver_clone = Arc::clone(&POOL.job_receiver);

            || {
                BoundedWorkerPool::worker(
                    die_clone,
                    local_die_clone,
                    has_job_clone,
                    job_receiver_clone,
                )
            }
        });

        threads_lock.push((local_die, has_job, thread));
    }
}

/// Stops and removes all workers from the pool.
pub fn stop_workers() {
    let mut threads_lock = POOL.workers.lock().unwrap();

    POOL.die.store(true, Ordering::Release);
    for (local_die, has_job, _) in threads_lock.drain(0..) {
        local_die.store(true, Ordering::Release);
        has_job.store(false, Ordering::Release);
    }
}

/// Queues work onto the pool, returning a completion
pub fn queue(work: Box<dyn FnOnce()>) -> Result<JobCompletion, ()> {
    let completion = Arc::new(AtomicBool::new(false));
    let completion2 = Arc::clone(&completion);

    POOL.job_sender
        .send(Job { work, completion })
        .map_or(Err(()), |_| Ok(JobCompletion(completion2)))
}
