pub mod ps2;
pub mod serial;

pub(crate) fn init() {
    ps2::init();
    serial::init();
}
