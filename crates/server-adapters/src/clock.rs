use std::time::Instant;
use veltrunox_core::Millis;
use veltrunox_server_app::Clock;

/// Server time: milliseconds since the server started. Monotonic.
pub struct SystemClock {
    started: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Millis {
        Millis::new(u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX))
    }
}
