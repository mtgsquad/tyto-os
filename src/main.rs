#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::{fmt::Write, panic::PanicInfo};
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

pub mod interrupts;
pub mod serial;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    // let mode = Graphics640x480x16::new();
    // mode.set_mode();
    // mode.clear_screen(Color16::Black);
    // mode.draw_line((80, 60), (80, 420), Color16::White);
    // mode.draw_line((80, 60), (540, 60), Color16::White);
    // mode.draw_line((80, 420), (540, 420), Color16::White);
    // mode.draw_line((540, 420), (540, 60), Color16::White);
    // mode.draw_line((80, 90), (540, 90), Color16::White);
    // for (offset, character) in "Hello World!".chars().enumerate() {
    //     mode.draw_character(270 + offset * 8, 72, character, Color16::White)
    // }

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
