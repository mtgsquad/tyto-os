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

        for y in 0..info.horizontal_resolution * info.bytes_per_pixel {
            for x in 0..info.vertical_resolution * info.bytes_per_pixel {
                let color = if x % 2 == 0 { 0x00 } else { 0xFF };
                framebuffer.buffer_mut()[(y + x) / info.bytes_per_pixel] = color;
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
