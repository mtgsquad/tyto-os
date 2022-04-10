#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use uefi::{prelude::*, proto::console::gop::GraphicsOutput};

#[entry]
fn main(_image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    tyto::init();

    let mut framebuffer = if let Ok(gop) = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
    {
        unsafe { &mut *gop.get() }
    } else {
        return Status::NO_MEDIA;
    }
    .frame_buffer();

    // clear the screen
    for i in 0..framebuffer.size() {
        unsafe { framebuffer.write_value(i, 0x00) }
    }

    Status::SUCCESS
}
