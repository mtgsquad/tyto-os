#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use uefi::{prelude::*, proto::console::gop::GraphicsOutput};

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let mut framebuffer = unsafe {
        if let Ok(gop) = system_table
            .boot_services()
            .locate_protocol::<GraphicsOutput>()
        {
            &mut *gop.get()
        } else {
            return Status::NO_MEDIA;
        }
    }
    .frame_buffer();

    let framebuffer_ptr = framebuffer.as_mut_ptr();

    // clear the screen
    for i in 0..framebuffer.size() {
        unsafe {
            framebuffer_ptr.add(i).write_volatile(0x00);
        }
    }

    Status::SUCCESS
}
