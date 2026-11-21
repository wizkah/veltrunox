//! Client use cases and the ports they need. Free of network and UI types.

mod connection;

pub use connection::{ConnectionMonitor, ConnectionStatus, LinkEvent};

use std::future::Future;
use veltrunox_core::Recording;

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("server unreachable: {0}")]
    Unreachable(String),
    #[error("unexpected server response: {0}")]
    InvalidResponse(String),
}

/// Port: requests to the server. The future is not required to be `Send`
/// because the browser target runs single-threaded.
pub trait ServerLink {
    fn fetch_recordings(&self) -> impl Future<Output = Result<Vec<Recording>, LinkError>>;
}

/// Use case: load the recording list from the server.
pub async fn load_library(link: &impl ServerLink) -> Result<Vec<Recording>, LinkError> {
    link.fetch_recordings().await
}
