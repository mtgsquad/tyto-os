#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod device;
pub mod interrupt;
pub mod task;

/// Initialize the kernel.
pub fn init() {
    use spin::Once;

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        device::init();
        task::init();
        interrupt::init();
    });
}
