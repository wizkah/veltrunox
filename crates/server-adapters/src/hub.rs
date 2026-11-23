//! Distributes playback state and bookmark notices to the connected clients.

use tokio::sync::{broadcast, watch};
use veltrunox_core::{PlaybackState, RecordingId};
use veltrunox_server_app::PlaybackBroadcast;

/// How much of an upload the server has received so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadProgress {
    pub upload_id: String,
    pub received_bytes: u64,
}

/// Holds the latest state, so a joining client reads it directly. Clients that
/// are slow skip intermediate states; each one is complete anyway.
pub struct Hub {
    state: watch::Sender<PlaybackState>,
    bookmarks: broadcast::Sender<RecordingId>,
    analysis: broadcast::Sender<RecordingId>,
    library: broadcast::Sender<()>,
    uploads: broadcast::Sender<UploadProgress>,
    audio_output: watch::Sender<bool>,
}

impl Hub {
    pub fn new(initial: PlaybackState) -> Self {
        Self {
            state: watch::channel(initial).0,
            bookmarks: broadcast::channel(64).0,
            analysis: broadcast::channel(64).0,
            library: broadcast::channel(16).0,
            uploads: broadcast::channel(64).0,
            // Until a device is opened there is no sound.
            audio_output: watch::channel(false).0,
        }
    }

    pub fn subscribe_state(&self) -> watch::Receiver<PlaybackState> {
        self.state.subscribe()
    }

    pub fn subscribe_audio_output(&self) -> watch::Receiver<bool> {
        self.audio_output.subscribe()
    }

    pub fn set_audio_output(&self, available: bool) {
        self.audio_output.send_replace(available);
    }

    pub fn subscribe_analysis(&self) -> broadcast::Receiver<RecordingId> {
        self.analysis.subscribe()
    }

    pub fn subscribe_library(&self) -> broadcast::Receiver<()> {
        self.library.subscribe()
    }

    pub fn subscribe_uploads(&self) -> broadcast::Receiver<UploadProgress> {
        self.uploads.subscribe()
    }

    /// Tells the clients how far an upload has come; nobody listening is fine.
    pub fn upload_progress(&self, upload_id: &str, received_bytes: u64) {
        let _ = self.uploads.send(UploadProgress {
            upload_id: upload_id.to_owned(),
            received_bytes,
        });
    }

    pub fn subscribe_bookmarks(&self) -> broadcast::Receiver<RecordingId> {
        self.bookmarks.subscribe()
    }
}

impl PlaybackBroadcast for Hub {
    fn state_changed(&self, state: &PlaybackState) {
        self.state.send_replace(*state);
    }

    fn bookmarks_changed(&self, recording: RecordingId) {
        // No receiver means no client is connected; nobody needs the notice.
        let _ = self.bookmarks.send(recording);
    }

    fn analysis_changed(&self, recording: RecordingId) {
        let _ = self.analysis.send(recording);
    }

    fn library_changed(&self) {
        let _ = self.library.send(());
    }
}
