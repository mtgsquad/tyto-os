use core::ops::{Deref, DerefMut};

use spin::Once;

pub(crate) struct LateInit<T>(Once<T>);

impl<T> LateInit<T> {
    pub(crate) const fn new() -> Self {
        Self(Once::new())
    }

    pub(crate) fn init(&self, init: impl FnOnce() -> T) {
        self.0.call_once(init);
    }

    pub(crate) fn into_inner(self) -> Once<T> {
        self.0
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // panics if not initialized
        self.0.get().unwrap()
    }
}

impl<T> DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // panics if not initialized
        self.0.get_mut().unwrap()
    }
}
