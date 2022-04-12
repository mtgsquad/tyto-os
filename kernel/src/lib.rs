#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use log::info;
use spin::{Lazy, Mutex};
use task::{executor::Executor, Task};

pub static EXECUTOR: Lazy<Mutex<Executor>> = Lazy::new(|| Mutex::new(Executor::new()));

pub mod device;
pub mod interrupt;
pub mod task;

/// Initialize the kernel.
pub async fn init() {
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

pub fn kernel_main(mut args: boot_lib::KernelArgs<'static>) -> ! {
    for i in 0..args.fb.size() {
        unsafe {
            args.fb.write_byte(i, 0x00);
        }
    }

    EXECUTOR.lock().spawn(Task::new(init()));

    EXECUTOR.lock().run();
}
