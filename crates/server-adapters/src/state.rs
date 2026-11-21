use crate::analysis_worker::{AnalysisQueue, FileAnalyzer};
use crate::hooks::{AudioHooks, LengthCache};
use crate::hub::Hub;
use std::sync::Arc;
use veltrunox_server_app::{
    AnalysisRepository, AnalysisService, BookmarkRepository, BookmarkService, Clock, MediaCatalog,
    PlaybackBroadcast, PlaybackService, RecordingRepository,
};

/// Everything the endpoints share.
#[derive(Clone)]
pub struct AppState {
    pub recordings: Arc<dyn RecordingRepository>,
    pub bookmarks: Arc<BookmarkService>,
    pub playback: Arc<PlaybackService>,
    pub hub: Arc<Hub>,
    pub clock: Arc<dyn Clock>,
    pub analysis: Arc<AnalysisService>,
    pub analysis_queue: Arc<AnalysisQueue>,
}

impl AppState {
    /// Wires the use cases to one store and one clock.
    pub fn new<S>(store: Arc<S>, clock: Arc<dyn Clock>) -> Self
    where
        S: RecordingRepository + BookmarkRepository + AnalysisRepository + MediaCatalog + 'static,
    {
        Self::build(
            store,
            clock,
            None::<fn(AudioHooks) -> Arc<dyn PlaybackBroadcast>>,
        )
    }

    /// Like `new`, and every published state also goes to the audio playback,
    /// which `start_audio` creates from the hooks it reports back through.
    pub fn with_audio<S>(
        store: Arc<S>,
        clock: Arc<dyn Clock>,
        start_audio: impl FnOnce(AudioHooks) -> Arc<dyn PlaybackBroadcast>,
    ) -> Self
    where
        S: RecordingRepository + BookmarkRepository + AnalysisRepository + MediaCatalog + 'static,
    {
        Self::build(store, clock, Some(start_audio))
    }

    fn build<S>(
        store: Arc<S>,
        clock: Arc<dyn Clock>,
        start_audio: Option<impl FnOnce(AudioHooks) -> Arc<dyn PlaybackBroadcast>>,
    ) -> Self
    where
        S: RecordingRepository + BookmarkRepository + AnalysisRepository + MediaCatalog + 'static,
    {
        let lengths = Arc::new(LengthCache::default());
        let hub = Arc::new(Hub::new(veltrunox_core::PlaybackState::initial(
            clock.now(),
        )));
        let broadcast: Arc<dyn PlaybackBroadcast> = match start_audio {
            Some(start) => {
                let audio = start(AudioHooks {
                    hub: hub.clone(),
                    lengths: lengths.clone(),
                });
                Arc::new(Fanout(vec![hub.clone(), audio]))
            }
            None => hub.clone(),
        };
        let broadcast_for_analysis = broadcast.clone();
        let playback = Arc::new(PlaybackService::new(
            store.clone(),
            lengths,
            clock.clone(),
            broadcast,
        ));
        let bookmarks = Arc::new(BookmarkService::new(store.clone(), hub.clone()));
        let analysis = Arc::new(AnalysisService::new(
            store.clone(),
            store.clone(),
            Arc::new(FileAnalyzer),
            broadcast_for_analysis,
        ));
        let analysis_queue = Arc::new(AnalysisQueue::start(analysis.clone()));
        Self {
            recordings: store,
            bookmarks,
            playback,
            hub,
            clock,
            analysis,
            analysis_queue,
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

    fn analysis_changed(&self, recording: veltrunox_core::RecordingId) {
        self.0
            .iter()
            .for_each(|target| target.analysis_changed(recording));
    }
}
