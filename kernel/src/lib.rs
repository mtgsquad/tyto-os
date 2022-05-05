#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

extern crate alloc;

use core::ptr::NonNull;

use boot_lib::KernelArgs;
use data::{IRQLock, LateInit};
use log::info;
use spin::{Lazy, Mutex, Once};
use task::executor::Executor;

pub static EXECUTOR: Lazy<Mutex<Executor>> = Lazy::new(|| Mutex::new(Executor::new()));
pub static KERNEL_ARGS: LateInit<IRQLock<KernelArgs>> = LateInit::new();

pub mod data;
pub mod device;
pub mod diag;
pub mod graphics;
pub mod interrupt;
pub mod task;

/// Initialize the kernel.
pub fn init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        info!("Initializing the kernel.");

        device::init();
        task::init();

        info!("Kernel initialized.");
    });
}

pub fn kernel_main(args: KernelArgs<'static>) -> ! {
    KERNEL_ARGS.init(|| IRQLock::new(args));

    diag::init();

    info!("Tyto kernel v{} on x86_64", env!("CARGO_PKG_VERSION"));

    info!("Initializing arch specific structures");

    interrupt::init();

    info!("Initializing framebuffer");

    {
        let mut args = KERNEL_ARGS.lock();

        diag::reinit_with_framebuffer(
            NonNull::new(args.framebuffer.as_mut_ptr()).unwrap(),
            args.framebuffer_info,
        );
    }

    init();

    {
        let mut args = KERNEL_ARGS.lock();
        for thing in 0..args.framebuffer.size() {
            unsafe {
                args.framebuffer.write_byte(thing, 0xFF);
            }
        }
    }

    EXECUTOR.lock().run();
}
