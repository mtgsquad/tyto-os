use alloc::{vec, vec::Vec};
use boot_lib::KernelArgs;
use core::ptr::NonNull;

use embedded_graphics_core::{pixelcolor::Rgb888, prelude::IntoStorage};
use uefi::proto::console::gop::ModeInfo;

use crate::data::{IRQLock, LateInit};

pub(crate) static GLOBAL_FRAMEBUFFER: IRQLock<LateInit<FramebufferDisplay>> =
    IRQLock::new(LateInit::new());

pub(crate) struct FramebufferDisplay {
    pub(crate) mode: ModeInfo,
    pub(crate) buffer: Vec<u32>,
    pub(crate) base: NonNull<u32>,
    pub(crate) size: u64,
}

impl FramebufferDisplay {
    pub(crate) fn new(base: NonNull<u32>, mode: ModeInfo) -> Self {
        let size = mode.resolution().1 * mode.stride();
        Self {
            size: size as u64,
            base,
            buffer: vec![0; size],
            mode,
        }
    }

    pub(crate) fn flush(&mut self) {
        unsafe {
            self.base
                .as_ptr()
                .copy_from_nonoverlapping(self.buffer.as_slice().as_ptr(), self.size as _);
        }
    }

    pub(crate) fn scroll_up(&mut self, height: usize, bg: Rgb888) {
        let high = self.mode.stride() * height;
        let low = self.mode.stride() * self.mode.resolution().1;
        self.buffer[0..(high - 1)].fill(bg.into_storage());
        self.buffer.copy_within(high..low, 0)
    }

    pub(crate) fn fill(&mut self, color: Rgb888) {
        self.buffer.fill(color.into_storage());
    }

    pub(crate) fn write(&mut self, pos: usize, color: Rgb888) {
        self.buffer[pos] = color.into_storage();
    }
}

pub(crate) fn init(args: &mut KernelArgs) {
    GLOBAL_FRAMEBUFFER.lock().init(|| {
        FramebufferDisplay::new(
            NonNull::new(args.framebuffer_addr as *mut u32).unwrap(),
            args.framebuffer_info,
        )
    });
}
