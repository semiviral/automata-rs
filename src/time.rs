use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct DeltaTime(pub std::time::Duration);

pub struct Stopwatch {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::ZERO,
        }
    }

    pub fn start_new() -> Self {
        let mut sw = Self::new();
        sw.start();

        sw
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.elapsed = self.elapsed();
        self.start = None;
    }

    pub fn restart(&mut self) {
        self.elapsed = self.elapsed();
        self.start = Some(Instant::now());
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.start = None;
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(start) => start.elapsed(),
            None => self.elapsed,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        let dur = self.elapsed();
        (dur.as_secs() * 1000) + (dur.subsec_nanos() / 1000000) as u64
    }
}
