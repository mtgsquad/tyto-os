use core::ops::Deref;
use spin::Mutex;

mod late_init;
pub(crate) use late_init::LateInit;

mod irq_lock;
pub(crate) use irq_lock::IRQLock;

/// A hacky workaround to be able to implement traits for `Mutex<T>`
pub(crate) struct CrateMutex<T>(pub(crate) Mutex<T>);

impl<T> CrateMutex<T> {
    pub(crate) const fn new(t: T) -> Self {
        Self(Mutex::new(t))
    }
}

impl<T> Deref for CrateMutex<T> {
    type Target = Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
