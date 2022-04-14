use core::ops::Deref;
use spin::Mutex;

mod late_init;
pub use late_init::LateInit;

mod irq_lock;
pub use irq_lock::{IRQLock, InterruptGuard};

/// A hacky workaround to be able to implement traits for `Mutex<T>`
pub struct CrateMutex<T>(pub Mutex<T>);

impl<T> CrateMutex<T> {
    pub const fn new(t: T) -> Self {
        Self(Mutex::new(t))
    }
}

impl<T> Deref for CrateMutex<T> {
    type Target = Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
