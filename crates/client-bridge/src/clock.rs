//! The client's own clock in milliseconds. Only differences to the server clock
//! matter, so the epoch is irrelevant; the browser has no `std::time::Instant`.

use veltrunox_core::Millis;

#[cfg(not(target_arch = "wasm32"))]
pub fn local_now() -> Millis {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Millis::new(u64::try_from(since_epoch.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(target_arch = "wasm32")]
pub fn local_now() -> Millis {
    Millis::new(js_sys::Date::now() as u64)
}
