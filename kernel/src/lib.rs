#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]

extern crate alloc;

use boot_lib::KernelArgs;
use log::info;
use spin::{Lazy, Mutex};
use task::executor::Executor;

pub(crate) static EXECUTOR: Lazy<Mutex<Executor>> = Lazy::new(|| Mutex::new(Executor::new()));

pub(crate) mod data;
pub(crate) mod device;
pub(crate) mod diag;
pub(crate) mod graphics;
pub(crate) mod interrupt;
pub(crate) mod task;

pub fn kernel_main(mut args: KernelArgs) -> ! {
    graphics::init(&mut args);
    diag::init();

    info!("Tyto kernel v{} on x86_64", env!("CARGO_PKG_VERSION"));

    info!("Initializing the kernel.");

    interrupt::init();
    device::init();
    task::init();

    info!("Kernel initialized.");

    EXECUTOR.lock().run();
}
