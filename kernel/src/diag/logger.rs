use core::{fmt, fmt::Write};

use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor, WebColors};
use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::{
    data::CrateMutex, device::serial::SERIAL1, graphics::framebuffer_term::FramebufferTextRender,
};

pub static GLOBAL_LOGGER: CrateMutex<DefaultLogger> = CrateMutex::new(DefaultLogger::new(None));

pub struct DefaultLogger {
    term: Option<FramebufferTextRender>,
}

// TODO Buffering

impl Log for CrateMutex<DefaultLogger> {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Some(mut serial) = SERIAL1.try_lock() {
            serial
                .write_fmt(format_args!(
                    "[{}] {}\n",
                    record.level().as_str().chars().next().unwrap(),
                    record.args()
                ))
                .expect("Could not write log message to serial port");
        }

        if self.is_locked() {
            return;
        }

        if let Some(term) = self.lock().term.as_mut() {
            term.write_fmt_colored(
                format_args!(
                    "[{}] {}\n",
                    record.level().as_str().chars().next().unwrap(),
                    record.args()
                ),
                match record.level() {
                    Level::Error => Rgb888::CSS_TOMATO,
                    Level::Warn => Rgb888::CSS_LIGHT_SALMON,
                    Level::Info => Rgb888::WHITE,
                    Level::Debug => Rgb888::CSS_ANTIQUE_WHITE,
                    Level::Trace => Rgb888::CSS_AZURE,
                },
            )
            .expect("Could not write log record to framebuffer")
        }
    }

    fn flush(&self) {
        // TODO
    }
}

impl DefaultLogger {
    pub const fn new(term: Option<FramebufferTextRender>) -> Self {
        Self { term }
    }

    pub fn reinit_with_framebuffer_term(&mut self, term: FramebufferTextRender) {
        self.term = Some(term)
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        if let Some(mut serial) = SERIAL1.try_lock() {
            serial
                .write_fmt(args)
                .expect("Could not write log message to serial port");
        }

        if let Some(term) = self.term.as_mut() {
            term.write_fmt_colored(args, Rgb888::WHITE).unwrap();
        }
    }
}

pub(crate) fn init() {
    if log::set_logger(&GLOBAL_LOGGER).is_ok() {
        log::set_max_level(LevelFilter::Debug)
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            $crate::diag::logger::GLOBAL_LOGGER.lock().print(format_args!($($arg)*));
        }
    };
}
