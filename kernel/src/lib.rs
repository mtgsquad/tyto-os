#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use log::info;

pub mod device;
pub mod interrupt;
pub mod spin;
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

#[allow(clippy::not_unsafe_ptr_arg_deref)] // we know that the pointer lasts for 'static
pub fn kernel_main(args: *mut boot_lib::KernelArgs) -> ! {
    let args = unsafe { &mut *args };

    init();

    loop {}
}
