//! Notes about the channel for whoever debugs it: the browser console on `wasm32`,
//! standard error elsewhere. Only reasons are noted, never user data or tokens.

#[cfg(target_arch = "wasm32")]
pub(crate) fn note(text: &str) {
    web_sys::console::warn_1(&format!("veltrunox: {text}").into());
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn note(text: &str) {
    eprintln!("veltrunox: {text}");
}
