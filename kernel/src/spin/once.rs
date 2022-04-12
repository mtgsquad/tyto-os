use core::sync::atomic::{AtomicBool, Ordering};

pub struct Once<F: FnOnce()> {
    init: AtomicBool,
    f: F,
}

impl<F: FnOnce()> Once<F> {
    pub const fn new(f: F) -> Self {
        Self {
            init: AtomicBool::new(false),
            f,
        }
    }

    pub fn call_once(&self, f: F)
    where
        F: FnOnce(),
    {
        if !self
            .init
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap()
        {
            f()
        }
    }
}
