#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use log::info;

pub mod device;
pub mod interrupt;
pub mod task;

/// Initialize the kernel.
pub fn init() {
    use spin::Once;

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        info!("Initializing the kernel.");

        device::init();
        task::init();
        interrupt::init();

        info!("Kernel initialized.");
    });
}
