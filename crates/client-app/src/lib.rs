//! Client use cases and the ports they need. Free of network and UI types.

mod connection;
mod reconnect;
mod session;

pub use connection::{ConnectionMonitor, ConnectionStatus, LinkEvent};
pub use reconnect::{CLOCK_SYNC_INTERVAL, ReconnectPolicy, Timer};
pub use session::{ScrubThrottle, ServerEvent, SessionView, ViewChange};

/// What the real-time channel reports: its own lifecycle or a message from the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelEvent {
    Link(LinkEvent),
    Server(ServerEvent),
}

use std::future::Future;
use veltrunox_core::{Bookmarks, LibraryEntry, Millis, RecordingId};

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("server unreachable: {0}")]
    Unreachable(String),
    #[error("unexpected server response: {0}")]
    InvalidResponse(String),
    #[error("not connected")]
    NotConnected,
}

/// Port: requests to the server. The future is not required to be `Send`
/// because the browser target runs single-threaded.
pub trait ServerLink {
    fn fetch_recordings(&self) -> impl Future<Output = Result<Vec<LibraryEntry>, LinkError>>;
    fn fetch_bookmarks(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Bookmarks, LinkError>>;
    /// Asks the server to analyse the recording again.
    fn request_analysis(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// The length of the recording as stored by its scan; an error while there is none.
    fn fetch_length(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Millis, LinkError>>;
}

/// Use case: load the recording list from the server.
pub async fn load_library(link: &impl ServerLink) -> Result<Vec<LibraryEntry>, LinkError> {
    link.fetch_recordings().await
}

/// Use case: load the bookmarks of a recording, for example after a change notice.
pub async fn load_bookmarks(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<Bookmarks, LinkError> {
    link.fetch_bookmarks(recording).await
}

/// Use case: start the analysis of a recording again; the result arrives as a notice.
pub async fn analyse_again(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<(), LinkError> {
    link.request_analysis(recording).await
}

/// Use case: the length of a recording, for the position control.
pub async fn load_length(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<Millis, LinkError> {
    link.fetch_length(recording).await
}
