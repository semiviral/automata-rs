use gl::types::GLsync;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

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
