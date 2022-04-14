use core::{
    arch::asm,
    sync::atomic::{AtomicU64, Ordering},
};

use log::{error, info};
use pic8259::ChainedPics;
use spin::Lazy;
use x86_64::{
    instructions::port::Port,
    registers::control::Cr2,
    set_general_handler,
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
        paging::{PageSize, Size2MiB},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::data::IRQLock;

pub const PIC_OFFSET: u8 = 32;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum IntIdx {
    Timer = PIC_OFFSET,
    Keyboard,
}

impl IntIdx {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

static TIMER_VAL: AtomicU64 = AtomicU64::new(0);

pub static DOUBLE_FAULT_STACK: [u8; Size2MiB::SIZE as usize] = [0; Size2MiB::SIZE as usize];

static TSS: Lazy<IRQLock<TaskStateSegment>> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[1] = VirtAddr::new(&DOUBLE_FAULT_STACK as *const _ as u64);

    IRQLock::new(tss)
});
static GDT: Lazy<GlobalDescriptorTable> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    gdt.add_entry(Descriptor::kernel_code_segment());
    gdt.add_entry(Descriptor::kernel_data_segment());
    gdt.add_entry(Descriptor::user_code_segment());
    gdt.add_entry(Descriptor::user_data_segment());
    gdt.add_entry(Descriptor::tss_segment(unsafe {
        &*(&*TSS.lock() as *const _)
    }));

    gdt
});
static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    set_general_handler!(&mut idt, general_handler);

    idt.page_fault.set_handler_fn(page_fault);
    idt.double_fault.set_handler_fn(double_fault);
    idt.breakpoint.set_handler_fn(breakpoint);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault);
    idt[IntIdx::Timer.as_u8() as _].set_handler_fn(timer);
    idt[IntIdx::Keyboard.as_u8() as _].set_handler_fn(keyboard);

    idt
});

pub static PICS: IRQLock<ChainedPics> =
    IRQLock::new(unsafe { ChainedPics::new(PIC_OFFSET, PIC_OFFSET + 8) });

/// Loads the GDT and fixes the segments
fn load_gdt() {
    // FIXME
    GDT.load();

    unsafe {
        asm!(
            "push 0x08",
            "mov AX, 0x10",
            "mov DS, AX",
            "mov ES, AX",
            "mov FS, AX",
            "mov GS, AX",
            "mov SS, AX"
        )
    }
}

const PIC_TERM_COUNT: u16 = 5966; // Should fire roughly each 5 ms

/// Initialize GDT, IDT and PIC
pub fn init() {
    load_gdt();
    IDT.load();
    unsafe {
        let mut pic = PICS.lock();
        pic.initialize();
        pic.write_masks(!3, !3);
    }
    // FIXME
    // Configure the PIC

    let mut port1 = Port::new(0x43);
    let mut port2 = Port::new(0x40);
    unsafe {
        port1.write(0b00110100u8);
        port2.write((PIC_TERM_COUNT & 0xff) as u8);
        port2.write((PIC_TERM_COUNT >> 8) as u8);
    }
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, code: PageFaultErrorCode) {
    error!("Page fault occured");
    error!("{:#?}", frame);
    error!("Code: {:?}", code);
    error!("CR2: {:#x}", Cr2::read().as_u64());
    panic!("Page Fault!")
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _code: u64) -> ! {
    error!("Double fault occured");
    error!("{:#?}", frame);
    panic!("Double Fault!")
}

extern "x86-interrupt" fn breakpoint(_frame: InterruptStackFrame) {
    info!("Waiting for debugger");
    unsafe {
        asm!("2: jmp 2b");
    }
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, flag: u64) {
    info!("General Protection Fault: {:#x}", flag);
    info!("{:#?}", frame);
    panic!("GPF");
}

/// Timer fires each 5 seconds
extern "x86-interrupt" fn timer(_frame: InterruptStackFrame) {
    let old = TIMER_VAL.fetch_add(1, Ordering::SeqCst);
    if (old + 1) % 200 == 0 {
        info!("TIMER SECOND {}", (old + 1) / 200);
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(IntIdx::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard(_frame: InterruptStackFrame) {
    let code: u8 = unsafe { Port::new(0x60).read() };
    crate::device::ps2::keyboard::add_scancode(code);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(IntIdx::Keyboard.as_u8());
    }
}

fn general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    info!("irq {} Err: {:?}", index, error_code);
    info!("{:?}", stack_frame);
    unsafe {
        PICS.lock().notify_end_of_interrupt(index);
    }
}
