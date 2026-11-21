use crate::hub::Hub;
use std::sync::Arc;
use veltrunox_server_app::{
    BookmarkRepository, BookmarkService, Clock, PlaybackBroadcast, PlaybackService,
    RecordingRepository,
};

/// Everything the endpoints share.
#[derive(Clone)]
pub struct AppState {
    pub recordings: Arc<dyn RecordingRepository>,
    pub bookmarks: Arc<BookmarkService>,
    pub playback: Arc<PlaybackService>,
    pub hub: Arc<Hub>,
    pub clock: Arc<dyn Clock>,
}

impl AppState {
    /// Wires the use cases to one store and one clock.
    pub fn new<S>(store: Arc<S>, clock: Arc<dyn Clock>) -> Self
    where
        S: RecordingRepository + BookmarkRepository + 'static,
    {
        Self::with_audio(store, clock, None)
    }

    /// Like `new`, and every published state also goes to the audio playback.
    pub fn with_audio<S>(
        store: Arc<S>,
        clock: Arc<dyn Clock>,
        audio: Option<Arc<dyn PlaybackBroadcast>>,
    ) -> Self
    where
        S: RecordingRepository + BookmarkRepository + 'static,
    {
        let hub = Arc::new(Hub::new(veltrunox_core::PlaybackState::initial(
            clock.now(),
        )));
        let broadcast: Arc<dyn PlaybackBroadcast> = match audio {
            Some(audio) => Arc::new(Fanout(vec![hub.clone(), audio])),
            None => hub.clone(),
        };
        let playback = Arc::new(PlaybackService::new(
            store.clone(),
            clock.clone(),
            broadcast,
        ));
        let bookmarks = Arc::new(BookmarkService::new(store.clone(), hub.clone()));
        Self {
            recordings: store,
            bookmarks,
            playback,
            hub,
            clock,
        }
    }
}

/// Passes every notice on to all receivers.
struct Fanout(Vec<Arc<dyn PlaybackBroadcast>>);

impl PlaybackBroadcast for Fanout {
    fn state_changed(&self, state: &veltrunox_core::PlaybackState) {
        self.0.iter().for_each(|target| target.state_changed(state));
    }

    fn bookmarks_changed(&self, recording: veltrunox_core::RecordingId) {
        self.0
            .iter()
            .for_each(|target| target.bookmarks_changed(recording));
    }
}
