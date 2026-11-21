use crate::hub::Hub;
use std::sync::Arc;
use veltrunox_server_app::{
    BookmarkRepository, BookmarkService, Clock, PlaybackService, RecordingRepository,
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
        let hub = Arc::new(Hub::new(veltrunox_core::PlaybackState::initial(
            clock.now(),
        )));
        let playback = Arc::new(PlaybackService::new(
            store.clone(),
            clock.clone(),
            hub.clone(),
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
