use std::sync::atomic::{AtomicBool, Ordering};

pub struct CancellationState {
    pub is_cancelled: AtomicBool,
}

impl Default for CancellationState {
    fn default() -> Self {
        Self {
            is_cancelled: AtomicBool::new(false),
        }
    }
}

impl CancellationState {
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.is_cancelled.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }
}
