use core::fmt::Write;
use embedded_graphics_core::prelude::Size;

pub(crate) trait Terminal: Write {
    fn get_dimensions(&self) -> Size;
    fn clear(&mut self);
}
