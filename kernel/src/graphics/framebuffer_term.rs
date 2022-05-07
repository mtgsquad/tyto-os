use alloc::string::{String, ToString};
use core::{
    fmt::{Arguments, Write},
    iter::{repeat, FromIterator},
};
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use noto_sans_mono_bitmap::{get_bitmap, BitmapHeight, FontWeight};

use super::framebuffer::{FramebufferDisplay, GLOBAL_FRAMEBUFFER};

const TAB_SIZE: i64 = 4;
const PADDING_PX: i64 = 3;
const FONT_SIZE: BitmapHeight = BitmapHeight::Size20;

pub(crate) struct FramebufferTextRender {
    font_size: i64,
    current_pos: (i64, i64),
    bg: Rgb888,
}

// TODO Support ANSI control codes

impl FramebufferTextRender {
    pub(crate) fn new(bg: Rgb888) -> Self {
        let mut framebuffer = GLOBAL_FRAMEBUFFER.lock();
        framebuffer.fill(bg);
        framebuffer.flush();
        Self {
            font_size: FONT_SIZE.val() as _,
            current_pos: (PADDING_PX, PADDING_PX),
            bg,
        }
    }

    pub(crate) fn write_fmt_colored(
        &mut self,
        args: Arguments<'_>,
        color: Rgb888,
    ) -> core::fmt::Result {
        let str = args.to_string();
        self.write_str_colored(&str, color)
    }

    pub(crate) fn write_str_colored(&mut self, string: &str, color: Rgb888) -> core::fmt::Result {
        for ch in string.chars() {
            self.write_char_colored_impl(ch, color)?
        }

        GLOBAL_FRAMEBUFFER.lock().flush();

        Ok(())
    }

    pub(crate) fn write_char_colored(&mut self, ch: char, color: Rgb888) -> core::fmt::Result {
        self.write_char_colored_impl(ch, color)
            .map(|_| GLOBAL_FRAMEBUFFER.lock().flush())
    }

    fn write_char_colored_impl(&mut self, ch: char, color: Rgb888) -> core::fmt::Result {
        if ch.is_control() {
            match ch {
                '\n' => self.advance_line(&mut **GLOBAL_FRAMEBUFFER.lock()),
                '\r' => self.current_pos.0 = 0,
                '\t' => {
                    self.write_str_colored(
                        &String::from_iter(repeat(' ').take(TAB_SIZE as usize)),
                        color,
                    )?;
                }
                _ => {}
            }
        } else {
            self.draw_char(ch, color)
        }
        Ok(())
    }

    fn alloc_place_for_char(&mut self, framebuffer: &mut FramebufferDisplay, width: i64) {
        let res = framebuffer.mode.resolution();
        if self.current_pos.0 + width + PADDING_PX * 2 >= res.0 as i64
            || self.current_pos.1 + self.font_size + PADDING_PX * 2 >= res.1 as i64
        {
            self.advance_line(framebuffer);
        }
    }

    fn advance_line(&mut self, framebuffer: &mut FramebufferDisplay) {
        let res = framebuffer.mode.resolution();
        let line_size = self.font_size + PADDING_PX;
        if self.current_pos.1 + line_size >= res.1 as i64 {
            // let amount = self.current_pos.1 + line_size - (res.1 as i64) + 1;
            framebuffer.scroll_up((line_size + PADDING_PX) as usize, self.bg);
            self.current_pos.1 -= line_size + PADDING_PX;
        } else {
            self.current_pos.1 += line_size + PADDING_PX;
        }
        self.current_pos.0 = PADDING_PX;
    }

    fn draw_char(&mut self, ch: char, color: Rgb888) {
        let bmch = match get_bitmap(ch, FontWeight::Regular, FONT_SIZE) {
            Some(x) => x,
            None => return,
        };
        let mut framebuffer = GLOBAL_FRAMEBUFFER.lock();
        self.alloc_place_for_char(&mut **framebuffer, bmch.width() as i64);
        for i in 0..bmch.height() {
            for j in 0..bmch.width() {
                let stride = framebuffer.mode.stride();
                let framebuffer_idx = (self.current_pos.0 + j as i64) as usize
                    + (self.current_pos.1 + i as i64) as usize * stride;

                let new_r = (color.r() as u64 * bmch.bitmap()[i][j] as u64) / u8::MAX as u64;
                let new_g = (color.g() as u64 * bmch.bitmap()[i][j] as u64) / u8::MAX as u64;
                let new_b = (color.b() as u64 * bmch.bitmap()[i][j] as u64) / u8::MAX as u64;

                framebuffer.write(
                    framebuffer_idx,
                    Rgb888::new(new_r as _, new_g as _, new_b as _),
                );
            }
        }

        self.current_pos.0 += bmch.width() as i64;
    }
}

impl Write for FramebufferTextRender {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str_colored(s, Rgb888::WHITE)
    }

    fn write_char(&mut self, ch: char) -> core::fmt::Result {
        self.write_char_colored(ch, Rgb888::WHITE)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> core::fmt::Result {
        self.write_fmt_colored(args, Rgb888::WHITE)
    }
}
