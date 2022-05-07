pub(crate) mod keyboard;

use ps2::Controller;
use spin::Mutex;

pub(crate) static CONTROLLER: Mutex<Controller> = unsafe { Mutex::new(Controller::new()) };

pub(crate) fn init() {
    keyboard::init();
}
