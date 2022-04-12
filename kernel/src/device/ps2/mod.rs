pub mod keyboard;

use ps2::Controller;
use spin::{Lazy, Mutex};
pub static CONTROLLER: Lazy<Mutex<Controller>> =
    Lazy::new(|| unsafe { Mutex::new(Controller::new()) });

pub(crate) fn init() {
    Lazy::force(&CONTROLLER);

    keyboard::init();
}
