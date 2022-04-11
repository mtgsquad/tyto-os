use log::info;
use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const KEYBOARD_INTERRUPT_OFFSET: usize = 33;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // idt[KEYBOARD_INTERRUPT_OFFSET].set_handler_fn(keyboard);

    idt
});

extern "x86-interrupt" fn keyboard(_frame: InterruptStackFrame) {
    use crate::device::ps2;

    let code = ps2::CONTROLLER
        .lock()
        .keyboard()
        .resend_last_byte()
        .unwrap();

    ps2::keyboard::add_scancode(code);
}

pub(crate) fn init() {
    info!("Initializing the IDT.");

    // IDT.load();

    info!("IDT initialized.");
}
