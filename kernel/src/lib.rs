#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use boot_lib::KernelArgs;
use log::info;
use spin::{Lazy, Mutex, Once};
use task::{executor::Executor, Task};
use uefi::proto::console::gop::FrameBuffer;

pub static EXECUTOR: Lazy<Mutex<Executor>> = Lazy::new(|| Mutex::new(Executor::new()));
pub static KERNEL_ARGS: Once<KernelArgs> = Once::new();

pub mod device;
pub mod interrupt;
pub mod task;

/// Initialize the kernel.
pub async fn init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        info!("Initializing the kernel.");

        device::init();
        task::init();
        interrupt::init();

        EXECUTOR.lock().spawn(Task::new(
            device::ps2::keyboard::print_keypresses::print_keypresses(),
        ));

        info!("Kernel initialized.");
    });
}

pub fn clear_screen(framebuffer: &mut FrameBuffer) {
    for i in 0..framebuffer.size() {
        unsafe { framebuffer.write_byte(i, 0x00) }
    }
}

pub fn kernel_main(mut args: KernelArgs<'static>) -> ! {
    clear_screen(&mut args.fb);

    KERNEL_ARGS.call_once(|| args);

    let mut executor = EXECUTOR.lock();

    executor.spawn(Task::new(init()));

    executor.run();
}
