//! The server as a library: the headless program and the desktop app wire and
//! serve it the same way.

pub mod config;
mod core;
pub mod embedded;
mod serve;
mod tls;
mod web;

pub use crate::core::{Seed, open_store, start_core};
pub use serve::{listen, with_cors};
pub use tls::{TlsError, prepare as prepare_tls};
pub use web::with_web_app;
