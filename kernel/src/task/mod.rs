use alloc::boxed::Box;
use core::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;

/// A task that can be executed with an [`Executor`](executor::Executor).
pub struct Task<'a> {
    pub id: TaskId,
    future: Pin<Box<dyn Future<Output = ()> + 'a + Send + Sync>>,
}

impl Debug for Task<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("future", &"{...}")
            .finish()
    }
}

impl<'a> Task<'a> {
    pub fn new(future: impl Future<Output = ()> + 'a + Send + Sync) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
}

impl Future for Task<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.future.as_mut().poll(cx)
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn init() {
    // we don't need to do anything here...
}
