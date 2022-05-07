use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

pub(crate) static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();

    Mutex::new(serial_port)
});

pub(crate) fn init() {
    Lazy::force(&SERIAL1);
}
