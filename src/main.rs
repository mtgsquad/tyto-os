#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::{fmt::Write, panic::PanicInfo};

pub mod interrupts;
pub mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();

        for (i, byte) in framebuffer.buffer_mut().iter_mut().enumerate() {
            let y = i % info.horizontal_resolution;
            let x = i % info.vertical_resolution;

            *byte = if x % 2 == 0 { 0x00 } else { 0xFF };
        }
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    {
        let mut serial_port = unsafe { uart_16550::SerialPort::new(0x3F8) };
        serial_port.init();
        write!(serial_port, "Kernel panic: {}", info).expect("Printing to serial failed");
    }

    loop {}
}
