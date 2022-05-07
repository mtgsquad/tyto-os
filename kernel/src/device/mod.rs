pub(crate) mod ps2;
pub(crate) mod serial;

pub(crate) fn init() {
    ps2::init();
    serial::init();
}
