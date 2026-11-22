use crate::analysis_worker::{AnalysisQueue, FileAnalyzer};
use crate::hooks::{AudioHooks, LengthCache};
use crate::hub::Hub;
use std::sync::Arc;
use veltrunox_core::{PlaybackState, Volume};
use veltrunox_server_app::{
    AccountService, AnalysisRepository, AnalysisService, BookmarkRepository, BookmarkService,
    Clock, MediaCatalog, PlaybackBroadcast, PlaybackService, RecordingRepository,
    WaveformRepository,
};

/// Everything the endpoints share.
#[derive(Clone)]
pub struct AppState {
    pub recordings: Arc<dyn RecordingRepository>,
    pub waveforms: Arc<dyn WaveformRepository>,
    pub bookmarks: Arc<BookmarkService>,
    pub playback: Arc<PlaybackService>,
    pub hub: Arc<Hub>,
    pub clock: Arc<dyn Clock>,
    pub analysis: Arc<AnalysisService>,
    pub analysis_queue: Arc<AnalysisQueue>,
    /// `None` leaves the server open: everybody may do everything.
    pub accounts: Option<Arc<AccountService>>,
}

impl AppState {
    /// Wires the use cases to one store and one clock.
    pub fn new<S>(store: Arc<S>, clock: Arc<dyn Clock>) -> Self
    where
        S: RecordingRepository
            + BookmarkRepository
            + AnalysisRepository
            + WaveformRepository
            + MediaCatalog
            + 'static,
    {
        Self::build(
            store,
            clock,
            Volume::default(),
            None::<fn(AudioHooks) -> Arc<dyn PlaybackBroadcast>>,
        )
    }

    /// Like `new`, and every published state also goes to the audio playback,
    /// which `start_audio` creates from the hooks it reports back through.
    /// `volume` is what the audio device has when the server starts.
    pub fn with_audio<S>(
        store: Arc<S>,
        clock: Arc<dyn Clock>,
        volume: Volume,
        start_audio: impl FnOnce(AudioHooks) -> Arc<dyn PlaybackBroadcast>,
    ) -> Self
    where
        S: RecordingRepository
            + BookmarkRepository
            + AnalysisRepository
            + WaveformRepository
            + MediaCatalog
            + 'static,
    {
        Self::build(store, clock, volume, Some(start_audio))
    }

    fn build<S>(
        store: Arc<S>,
        clock: Arc<dyn Clock>,
        volume: Volume,
        start_audio: Option<impl FnOnce(AudioHooks) -> Arc<dyn PlaybackBroadcast>>,
    ) -> Self
    where
        S: RecordingRepository
            + BookmarkRepository
            + AnalysisRepository
            + WaveformRepository
            + MediaCatalog
            + 'static,
    {
        let lengths = Arc::new(LengthCache::new(store.clone()));
        let initial = PlaybackState {
            volume,
            ..PlaybackState::initial(clock.now())
        };
        let hub = Arc::new(Hub::new(initial));
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
            initial,
            store.clone(),
            lengths,
            clock.clone(),
            broadcast,
        ));
        let bookmarks = Arc::new(BookmarkService::new(store.clone(), hub.clone()));
        let analysis = Arc::new(AnalysisService::new(
            store.clone(),
            store.clone(),
            store.clone(),
            Arc::new(FileAnalyzer),
            broadcast_for_analysis,
        ));
        let analysis_queue = Arc::new(AnalysisQueue::start(analysis.clone()));
        Self {
            recordings: store.clone(),
            waveforms: store,
            bookmarks,
            playback,
            hub,
            clock,
            analysis,
            analysis_queue,
            accounts: None,
        }
    }

    /// Requires sign-in (or guest access) and checks the rights of every action.
    pub fn with_accounts(mut self, accounts: Arc<AccountService>) -> Self {
        self.accounts = Some(accounts);
        self
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
