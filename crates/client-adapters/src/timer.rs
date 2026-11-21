//! Timer adapter: a thread per delay natively (delays are rare and long), the
//! browser's `setTimeout` on `wasm32`.

use std::time::Duration;
use veltrunox_client_app::Timer;

pub struct SystemTimer;

#[cfg(not(target_arch = "wasm32"))]
impl Timer for SystemTimer {
    fn after(&self, delay: Duration, action: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            action();
        });
    }
}

#[cfg(target_arch = "wasm32")]
impl Timer for SystemTimer {
    fn after(&self, delay: Duration, action: Box<dyn FnOnce() + Send>) {
        let millis = u32::try_from(delay.as_millis()).unwrap_or(u32::MAX);
        // The timeout fires on its own; the handle is not needed to cancel it.
        gloo_timers::callback::Timeout::new(millis, action).forget();
    }
}
