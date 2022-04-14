use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker};
use log::warn;
use spin::{Lazy, Mutex};

static SCANCODE_QUEUE: Lazy<Mutex<ArrayQueue<u8>>> = Lazy::new(|| Mutex::new(ArrayQueue::new(100)));
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler.
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if SCANCODE_QUEUE.lock().push(scancode).is_err() {
        warn!("Scancode queue full; dropping keyboard input");
    } else {
        WAKER.wake();
    }
}

/// A stream of scancodes.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScancodeStream;

impl Stream for ScancodeStream {
    type Item = u8;

    /// Returns the scancode from the queue if available, or `None` otherwise.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        // fast path
        if let Some(scancode) = SCANCODE_QUEUE.lock().pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match SCANCODE_QUEUE.lock().pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub(crate) fn init() {}
