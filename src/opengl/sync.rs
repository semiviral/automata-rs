use gl::types::GLsync;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use crate::ring::RingIndex;

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum SyncStatus {
    AlreadySignaled = 37146,
    TimeoutExpired = 37147,
    ConditionSatisfied = 37148,
    WaitFailed = 37149,
}

pub struct FenceSync {
    sync: GLsync,
}

impl FenceSync {
    fn generate_sync(flags: u32) -> GLsync {
        unsafe { gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, flags) }
    }

    pub fn new(flags: u32) -> Self {
        Self {
            sync: Self::generate_sync(flags),
        }
    }

    pub fn wait_gpu(&self, timeout: u64, flags: u32) {
        unsafe { gl::WaitSync(self.sync, flags, timeout) };
    }

    pub fn wait_cpu(&self, timeout: u64, flags: u32) -> SyncStatus {
        unsafe { SyncStatus::try_from(gl::ClientWaitSync(self.sync, flags, timeout)).unwrap() }
    }

    pub fn busy_wait_cpu(&self) {
        loop {
            if let SyncStatus::AlreadySignaled | SyncStatus::ConditionSatisfied =
                self.wait_cpu(1, gl::SYNC_FLUSH_COMMANDS_BIT as u32)
            {
                break;
            }
        }
    }

    pub fn regenerate(&mut self, flags: u32) {
        unsafe {
            gl::DeleteSync(self.sync);
            self.sync = Self::generate_sync(flags);
        }
    }
}

impl Drop for FenceSync {
    fn drop(&mut self) {
        unsafe { gl::DeleteSync(self.sync) };
    }
}

pub struct RingFenceSync {
    ring: RingIndex,
    fences: Vec<FenceSync>,
}

impl RingFenceSync {
    pub fn new(count: usize) -> Self {
        Self {
            ring: RingIndex::new(count),
            fences: {
                let mut vec = Vec::with_capacity(count);

                for _ in 0..count {
                    vec.push(FenceSync::new(0));
                }

                vec
            },
        }
    }

    pub fn index(&self) -> usize {
        self.ring.index()
    }

    pub fn next_index(&self) -> usize {
        self.ring.next_index()
    }

    pub fn wait_current(&self) {
        self.fences[self.ring.index()].busy_wait_cpu();
    }

    pub fn wait_enter_next(&mut self) {
        self.fences[self.ring.next_index()].busy_wait_cpu();
        self.ring.increment();
    }

    pub fn fence_current(&mut self) {
        self.fences[self.ring.index()] = FenceSync::new(0);
    }
}
