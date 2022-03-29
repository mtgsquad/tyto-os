#![no_std]
#![no_main]
#![feature(const_ptr_offset)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub mod colors;
pub mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();

        // set the screen to black
        for (i, byte) in framebuffer.buffer_mut().iter_mut().enumerate() {
            if (0..info.horizontal_resolution * info.bytes_per_pixel).contains(&i) {
                *byte = 0x55;
            } else {
                *byte = colors::BLACK;
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
