#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub mod interrupts;
pub mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();

        serial_println!("{}", framebuffer.buffer().len());

        for y in 0..info.vertical_resolution {
            for x in 0..info.horizontal_resolution {
                let byte = &mut framebuffer.buffer_mut()[((y + x) * info.horizontal_resolution)];

                *byte = 0xFF;
            }
        }
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
