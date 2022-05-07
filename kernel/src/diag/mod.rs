use crate::graphics::{framebuffer::FramebufferDisplay, framebuffer_term::FramebufferTextRender};
use core::ptr::NonNull;
use embedded_graphics::{pixelcolor::Rgb888, prelude::RgbColor};
use uefi::proto::console::gop::ModeInfo;

pub(crate) mod logger;
pub(crate) mod terminal;

pub(crate) fn init() {
    logger::init();
}

pub(crate) fn reinit_with_framebuffer(addr: NonNull<u8>, mode: ModeInfo) {
    logger::GLOBAL_LOGGER
        .lock()
        .reinit_with_framebuffer_term(FramebufferTextRender::new(Rgb888::BLACK));
}
