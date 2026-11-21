//! Server adapters: storage and HTTP endpoints.

mod http;
mod memory;

pub use http::router;
pub use memory::InMemoryRecordingRepository;
