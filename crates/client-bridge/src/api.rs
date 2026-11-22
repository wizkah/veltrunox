//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.
//!
//! All functions are `async` on purpose: the browser runs async functions on the
//! main thread, while synchronous ones need a worker pool, which needs threads.

use crate::clock::local_now;
use crate::frb_generated::StreamSink;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
use veltrunox_client_adapters::{EventChannel, HttpServerLink, SystemTimer, open_channel};
use veltrunox_client_app::{
    ChannelAccess, ChannelEvent, ConnectionMonitor, ConnectionStatus, LinkError, LinkEvent,
    ScrubThrottle, ServerEvent, SessionView, ViewChange, WaveformCache, add_label, add_recording,
    add_region, analyse_again, convert_label_to_region, convert_region_to_label, delete_label,
    delete_recording, delete_region, edit_label, edit_region, load_bookmarks, load_length,
    load_library, load_waveform, rename_recording,
};
use veltrunox_client_app::{
    Credentials, ManagedUser, NewUser, Rights, SignedIn, UserChange, add_user, delete_user,
    edit_user, load_access_status, load_rights, load_users, set_guest_access, set_password_policy,
    set_up_first_admin, sign_in, sign_out,
};
use veltrunox_core::{
    AnalysisProgress, BookmarkId, BookmarkName, Bpm, Capability, Intent, InvalidValue, KeyMode,
    KeyShift, LibraryEntry, Millis, PasswordPolicy, PlayMode, RecordingId, RecordingTitle, Role,
    Speed, TargetSpeed, TimeRange, TuningHz, UserId, Version, Volume, speed_for_target_tempo,
};

/// A recording as the user interface needs it.
pub struct RecordingItem {
    pub id: u64,
    pub title: String,
    pub analysis_state: AnalysisState,
    /// Why the analysis failed; only set in the state `Failed`.
    pub analysis_failure: Option<String>,
    /// The estimated values; only set once the analysis is done.
    pub analysis: Option<AnalysisValues>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnalysisState {
    None,
    Queued,
    Running,
    Failed,
    Done,
}

/// Tuning, key and tempo as estimated from the audio.
#[derive(Clone)]
pub struct AnalysisValues {
    pub tuning_hz: f64,
    /// Pitch class of the key's tonic, 0 = C … 11 = B.
    pub key_tonic: u8,
    pub key_minor: bool,
    pub tempo_bpm: f64,
}

/// Loads the recording list from the server at `server_url`.
pub async fn list_recordings(server_url: String) -> Result<Vec<RecordingItem>, String> {
    let link = link(&server_url);
    let entries = load_library(&link)
        .await
        .map_err(|error| error.to_string())?;
    Ok(entries.iter().map(RecordingItem::from).collect())
}

impl From<&LibraryEntry> for RecordingItem {
    fn from(entry: &LibraryEntry) -> Self {
        let (analysis_state, analysis_failure) = match &entry.analysis.progress {
            AnalysisProgress::None => (AnalysisState::None, None),
            AnalysisProgress::Queued => (AnalysisState::Queued, None),
            AnalysisProgress::Running => (AnalysisState::Running, None),
            AnalysisProgress::Failed(reason) => (AnalysisState::Failed, Some(reason.clone())),
            AnalysisProgress::Done => (AnalysisState::Done, None),
        };
        Self {
            id: entry.recording.id.value(),
            title: entry.recording.title.as_str().to_owned(),
            analysis_state,
            analysis_failure,
            analysis: entry.analysis.result.map(|found| AnalysisValues {
                tuning_hz: found.tuning.hertz(),
                key_tonic: found.key.tonic.index(),
                key_minor: found.key.mode == KeyMode::Minor,
                tempo_bpm: found.tempo.value(),
            }),
        }
    }
}

/// Asks the server to analyse a recording again; the result arrives as a session update.
pub async fn reanalyse_recording(server_url: String, recording_id: u64) -> Result<(), String> {
    let link = link(&server_url);
    analyse_again(&link, RecordingId::new(recording_id))
        .await
        .map_err(|error| error.to_string())
}

/// What became of an uploaded file.
pub struct UploadedRecordingItem {
    pub id: u64,
    pub title: String,
    /// The same audio was in the library already; nothing was added.
    pub existing: bool,
}

/// Sends an audio file to the server. The refusal reason (not audio, too large, not allowed)
/// comes back as text for the user; the analysis result arrives as a session update.
pub async fn upload_recording(
    server_url: String,
    file_name: String,
    content: Vec<u8>,
) -> Result<UploadedRecordingItem, String> {
    let link = link(&server_url);
    let uploaded = add_recording(&link, &file_name, content)
        .await
        .map_err(|error| user_reason(&error))?;
    Ok(UploadedRecordingItem {
        id: uploaded.id.value(),
        title: uploaded.title.as_str().to_owned(),
        existing: uploaded.existing,
    })
}

/// Gives a recording another title (admin only).
pub async fn rename_recording_to(
    server_url: String,
    recording_id: u64,
    title: String,
) -> Result<(), String> {
    let title = RecordingTitle::new(&title).map_err(|error| error.to_string())?;
    rename_recording(&link(&server_url), RecordingId::new(recording_id), &title)
        .await
        .map_err(|error| user_reason(&error))
}

/// Deletes a recording with its bookmarks and analysis (admin only).
pub async fn delete_recording_for_good(
    server_url: String,
    recording_id: u64,
) -> Result<(), String> {
    delete_recording(&link(&server_url), RecordingId::new(recording_id))
        .await
        .map_err(|error| user_reason(&error))
}

/// The server's own reason when it refused, else the error's description.
fn user_reason(error: &LinkError) -> String {
    match error {
        LinkError::Refused(reason) => reason.clone(),
        other => other.to_string(),
    }
}

/// The length of a recording in milliseconds from its stored scan; `None` while there is none
/// (not scanned yet) or the server cannot be reached.
pub async fn recording_length_ms(server_url: String, recording_id: u64) -> Option<u64> {
    let link = link(&server_url);
    load_length(&link, RecordingId::new(recording_id))
        .await
        .ok()
        .map(Millis::value)
}

/// One resolution of a waveform: per block the minimum, then the maximum, as signed bytes.
pub struct WaveformLevelData {
    pub frames_per_block: u32,
    pub peaks: Vec<u8>,
}

/// The peaks of a recording from coarse to fine, with the length they cover.
pub struct WaveformData {
    pub length_ms: u64,
    pub sample_rate: u32,
    pub levels: Vec<WaveformLevelData>,
}

/// Waveforms fetched so far; they do not change once a recording is scanned.
static WAVEFORMS: LazyLock<Mutex<WaveformCache>> =
    LazyLock::new(|| Mutex::new(WaveformCache::default()));

/// The waveform of a recording. It is fetched once and then kept in memory. Fails while the
/// recording has no scan or the server cannot be reached.
pub async fn recording_waveform(
    server_url: String,
    recording_id: u64,
) -> Result<WaveformData, String> {
    let link = link(&server_url);
    let found = load_waveform(&link, &WAVEFORMS, RecordingId::new(recording_id))
        .await
        .map_err(|error| error.to_string())?;
    Ok(WaveformData {
        length_ms: found.length.value(),
        sample_rate: found.waveform.sample_rate,
        levels: found
            .waveform
            .levels
            .iter()
            .map(|level| WaveformLevelData {
                frames_per_block: level.frames_per_block,
                peaks: level
                    .peaks
                    .iter()
                    .flat_map(|peak| [peak.min.to_le_bytes()[0], peak.max.to_le_bytes()[0]])
                    .collect(),
            })
            .collect(),
    })
}

/// A point marker of a recording.
pub struct LabelItem {
    pub id: u64,
    /// The version this client saw; a change or deletion must name it.
    pub version: u64,
    pub name: String,
    pub position_ms: u64,
}

/// A named span of a recording.
pub struct RegionItem {
    pub id: u64,
    pub version: u64,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

/// All labels and regions of one recording.
pub struct BookmarkSet {
    pub labels: Vec<LabelItem>,
    pub regions: Vec<RegionItem>,
}

/// Loads the labels and regions of a recording.
pub async fn list_bookmarks(server_url: String, recording_id: u64) -> Result<BookmarkSet, String> {
    let link = link(&server_url);
    let found = load_bookmarks(&link, RecordingId::new(recording_id))
        .await
        .map_err(|error| error.to_string())?;
    Ok(BookmarkSet {
        labels: found
            .labels
            .iter()
            .map(|label| LabelItem {
                id: label.id.value(),
                version: label.version.value(),
                name: label.name.as_str().to_owned(),
                position_ms: label.position.value(),
            })
            .collect(),
        regions: found
            .regions
            .iter()
            .map(|region| RegionItem {
                id: region.id.value(),
                version: region.version.value(),
                name: region.name.as_str().to_owned(),
                start_ms: region.range.start().value(),
                end_ms: region.range.end().value(),
            })
            .collect(),
    })
}

fn bookmark_name(name: &str) -> Result<BookmarkName, String> {
    BookmarkName::new(name).map_err(|error| error.to_string())
}

fn invalid(error: InvalidValue) -> String {
    error.to_string()
}

fn range_of(start_ms: u64, end_ms: u64) -> Result<TimeRange, String> {
    TimeRange::new(
        Millis::position(start_ms).map_err(invalid)?,
        Millis::position(end_ms).map_err(invalid)?,
    )
    .map_err(invalid)
}

/// Adds a label; the server tells every client to reload the bookmarks.
pub async fn create_label(
    server_url: String,
    recording_id: u64,
    name: String,
    position_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    add_label(
        &link,
        RecordingId::new(recording_id),
        &bookmark_name(&name)?,
        Millis::position(position_ms).map_err(invalid)?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// Renames or moves a label seen at `expected_version`.
pub async fn change_label(
    server_url: String,
    id: u64,
    expected_version: u64,
    name: String,
    position_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    edit_label(
        &link,
        BookmarkId::new(id),
        Version::new(expected_version),
        &bookmark_name(&name)?,
        Millis::position(position_ms).map_err(invalid)?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// Deletes a label seen at `expected_version`.
pub async fn remove_label(
    server_url: String,
    id: u64,
    expected_version: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    delete_label(&link, BookmarkId::new(id), Version::new(expected_version))
        .await
        .map_err(|error| error.to_string())
}

/// Adds a region from `start_ms` up to `end_ms`.
pub async fn create_region(
    server_url: String,
    recording_id: u64,
    name: String,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    add_region(
        &link,
        RecordingId::new(recording_id),
        &bookmark_name(&name)?,
        range_of(start_ms, end_ms)?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// Renames or resizes a region seen at `expected_version`.
pub async fn change_region(
    server_url: String,
    id: u64,
    expected_version: u64,
    name: String,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    edit_region(
        &link,
        BookmarkId::new(id),
        Version::new(expected_version),
        &bookmark_name(&name)?,
        range_of(start_ms, end_ms)?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// Deletes a region seen at `expected_version`.
pub async fn remove_region(
    server_url: String,
    id: u64,
    expected_version: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    delete_region(&link, BookmarkId::new(id), Version::new(expected_version))
        .await
        .map_err(|error| error.to_string())
}

/// Replaces a label seen at `expected_version` by a region in one step.
pub async fn label_to_region(
    server_url: String,
    id: u64,
    expected_version: u64,
    name: String,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    convert_label_to_region(
        &link,
        BookmarkId::new(id),
        Version::new(expected_version),
        &bookmark_name(&name)?,
        range_of(start_ms, end_ms)?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// Replaces a region seen at `expected_version` by a label in one step.
pub async fn region_to_label(
    server_url: String,
    id: u64,
    expected_version: u64,
    name: String,
    position_ms: u64,
) -> Result<(), String> {
    let link = link(&server_url);
    convert_region_to_label(
        &link,
        BookmarkId::new(id),
        Version::new(expected_version),
        &bookmark_name(&name)?,
        Millis::position(position_ms).map_err(|error| error.to_string())?,
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// The speed that plays a recording of `recording_bpm` at `target_bpm`.
pub struct TargetTempo {
    pub speed: f64,
    /// `false` if the speed lies outside the supported range; it must not be set then.
    pub supported: bool,
}

pub async fn speed_for_target_bpm(
    target_bpm: f64,
    recording_bpm: f64,
) -> Result<TargetTempo, String> {
    target_tempo(target_bpm, recording_bpm)
}

fn target_tempo(target_bpm: f64, recording_bpm: f64) -> Result<TargetTempo, String> {
    let target = Bpm::new(target_bpm).map_err(|error| error.to_string())?;
    let recording = Bpm::new(recording_bpm).map_err(|error| error.to_string())?;
    Ok(match speed_for_target_tempo(target, recording) {
        TargetSpeed::Supported(speed) => TargetTempo {
            speed: speed.factor(),
            supported: true,
        },
        TargetSpeed::OutOfRange { factor } => TargetTempo {
            speed: factor,
            supported: false,
        },
    })
}

/// Whether the server accepts this tuning; it must not be sent otherwise.
pub async fn tuning_supported(tuning_hz: f64) -> bool {
    tuning_in_range(tuning_hz)
}

fn tuning_in_range(tuning_hz: f64) -> bool {
    TuningHz::new(tuning_hz).is_ok()
}

/// State of the real-time channel as the user interface needs it.
#[derive(Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
}

/// The shared playback state as the user interface needs it.
#[derive(Clone)]
pub struct PlaybackView {
    pub revision: u64,
    pub recording_id: Option<u64>,
    /// Position at the moment this view was made; use `playback_position_ms` while running.
    pub position_ms: u64,
    pub running: bool,
    pub speed: f64,
    pub tuning_hz: f64,
    pub key_shift_semitones: i8,
    /// The region being played, if any.
    pub play_range: Option<TimeSpan>,
    pub play_mode: PlayModeChoice,
    pub scrubbing: bool,
    pub volume_percent: u8,
}

/// A span of the recording in milliseconds.
#[derive(Clone, Copy)]
pub struct TimeSpan {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// What happens at the end of the region being played.
#[derive(Clone, Copy)]
pub enum PlayModeChoice {
    /// Playback runs to the end of the recording.
    ToEnd,
    /// Playback stops at the end of the region.
    ToRegionEnd,
    /// Playback wraps from the end of the region to its start.
    Loop,
}

/// Everything the real-time channel tells the user interface.
#[derive(Clone)]
pub enum SessionUpdate {
    Connection(ConnectionState),
    Playback(PlaybackView),
    BookmarksChanged {
        recording_id: u64,
    },
    /// The analysis of this recording changed; load the library again.
    AnalysisChanged {
        recording_id: u64,
    },
    Rejected {
        reason: String,
    },
    /// The server accepted the sign-in of the channel; these are the rights of this client.
    Rights {
        rights: RightsInfo,
    },
}

/// What the user asks the shared player to do.
pub enum PlayerCommand {
    SelectRecording {
        recording_id: u64,
    },
    Play,
    Pause,
    Seek {
        position_ms: u64,
    },
    SetSpeed {
        speed: f64,
    },
    SetTuning {
        tuning_hz: f64,
    },
    SetKey {
        key_shift_semitones: i8,
    },
    /// Plays the region from its start, in one step.
    PlayRegion {
        start_ms: u64,
        end_ms: u64,
    },
    SetPlayMode {
        play_mode: PlayModeChoice,
    },
    SetVolume {
        volume_percent: u8,
    },
}

/// The latest server state and clock estimate of this client; shared between the
/// channel callback and the calls from the interface.
static VIEW: LazyLock<Mutex<SessionView>> = LazyLock::new(|| Mutex::new(SessionView::new()));

fn view() -> MutexGuard<'static, SessionView> {
    VIEW.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The one real-time channel of this client; a new `watch_session` replaces it.
/// `flutter_rust_bridge` drops values returned from stream functions, so the
/// channel has to be owned here to stay open.
static CHANNEL: Mutex<Option<EventChannel>> = Mutex::new(None);

fn with_channel<T>(action: impl FnOnce(&mut Option<EventChannel>) -> T) -> T {
    action(&mut CHANNEL.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Opens the real-time channel and pushes connection changes, playback state,
/// bookmark notices and rejections to `updates`. The channel reconnects by itself
/// and measures the clock offset when connected and from time to time.
pub async fn watch_session(
    server_url: String,
    updates: StreamSink<SessionUpdate>,
) -> Result<(), String> {
    // Close the previous channel first so it cannot report into the new session.
    with_channel(|channel| *channel = None);
    *view() = SessionView::new();
    let status_sink = updates.clone();
    let monitor = Mutex::new(ConnectionMonitor::new(move |status| {
        // A closed sink means the UI stopped listening; nothing is left to report.
        let _ = status_sink.add(SessionUpdate::Connection(ConnectionState::from(status)));
    }));
    let channel = open_channel(
        &server_url,
        channel_access(),
        Arc::new(SystemTimer),
        local_now,
        move |event| match event {
            ChannelEvent::Link(link_event) => {
                if link_event == LinkEvent::Greeted {
                    // The server sends the current state next; its revisions may
                    // have started over if it restarted.
                    view().forget_state();
                }
                if let Ok(mut monitor) = monitor.lock() {
                    monitor.push(link_event);
                }
            }
            ChannelEvent::Server(server_event) => forward(server_event, &updates),
        },
    );
    with_channel(|slot| *slot = Some(channel));
    Ok(())
}

fn forward(event: ServerEvent, updates: &StreamSink<SessionUpdate>) {
    let now = local_now();
    let mut view = view();
    let update = match view.handle(event, now) {
        ViewChange::Nothing => return,
        ViewChange::PlaybackState => view.state().map(|state| {
            SessionUpdate::Playback(PlaybackView::from_state(state, view.server_time(now)))
        }),
        ViewChange::BookmarksChanged(recording) => Some(SessionUpdate::BookmarksChanged {
            recording_id: recording.value(),
        }),
        ViewChange::AnalysisChanged(recording) => {
            // A new scan may have replaced the peaks.
            WAVEFORMS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .forget(recording);
            Some(SessionUpdate::AnalysisChanged {
                recording_id: recording.value(),
            })
        }
        ViewChange::Rejected(reason) => Some(SessionUpdate::Rejected { reason }),
        ViewChange::Rights(rights) => Some(SessionUpdate::Rights {
            rights: RightsInfo::from(&rights),
        }),
        // Not shown yet: the interface needs a place for it first (GUI work).
        ViewChange::AudioOutput => None,
    };
    if let Some(update) = update {
        let _ = updates.add(update);
    }
}

impl PlaybackView {
    fn from_state(state: &veltrunox_core::PlaybackState, server_now: Millis) -> Self {
        Self {
            revision: state.revision,
            recording_id: state.recording.map(RecordingId::value),
            position_ms: state.position_at(server_now).value(),
            running: state.running,
            speed: state.settings.speed.factor(),
            tuning_hz: state.settings.tuning.hertz(),
            key_shift_semitones: state.settings.key.semitones(),
            play_range: state.play_range.map(|range| TimeSpan {
                start_ms: range.start().value(),
                end_ms: range.end().value(),
            }),
            play_mode: state.play_mode.into(),
            scrubbing: state.scrubbing,
            volume_percent: state.volume.percent(),
        }
    }
}

/// The playback position now, extrapolated from the latest state; `None` before the first state.
pub async fn playback_position_ms() -> Option<u64> {
    view().position_now(local_now()).map(Millis::value)
}

/// Limits the seek intents of the position handle while it is dragged.
static SCRUB: Mutex<Option<ScrubThrottle>> = Mutex::new(None);

fn with_scrub<T>(action: impl FnOnce(&mut ScrubThrottle) -> T) -> T {
    let mut guard = SCRUB.lock().unwrap_or_else(PoisonError::into_inner);
    action(guard.get_or_insert_with(|| ScrubThrottle::new(ScrubThrottle::DEFAULT_INTERVAL)))
}

/// The position handle is dragged to `position_ms`; the server hears it at most about 20 times a
/// second and keeps the sound muted meanwhile.
pub async fn scrub_to(position_ms: u64) -> Result<(), String> {
    let position = Millis::position(position_ms).map_err(|error| error.to_string())?;
    match with_scrub(|throttle| throttle.drag(position, local_now())) {
        Some(intent) => send_intent(intent),
        None => Ok(()),
    }
}

/// The position handle is let go at `position_ms`: always sent, so the last position is never lost.
pub async fn end_scrub(position_ms: u64) -> Result<(), String> {
    let position = Millis::position(position_ms).map_err(|error| error.to_string())?;
    send_intent(with_scrub(|throttle| throttle.release(position)))
}

/// Sends a command to the shared player. Out-of-range values are rejected here
/// and the server rejects them again; the answer arrives as a session update.
pub async fn send_command(command: PlayerCommand) -> Result<(), String> {
    send_intent(command.into_intent().map_err(|error| error.to_string())?)
}

fn send_intent(intent: Intent) -> Result<(), String> {
    with_channel(|channel| match channel {
        Some(channel) => channel
            .send_intent(intent)
            .map_err(|error| error.to_string()),
        None => Err("not connected".to_owned()),
    })
}

impl PlayerCommand {
    fn into_intent(self) -> Result<Intent, veltrunox_core::InvalidValue> {
        Ok(match self {
            Self::SelectRecording { recording_id } => {
                Intent::SelectRecording(RecordingId::new(recording_id))
            }
            Self::Play => Intent::Play,
            Self::Pause => Intent::Pause,
            Self::Seek { position_ms } => Intent::Seek {
                position: Millis::position(position_ms)?,
                scrubbing: false,
            },
            Self::SetSpeed { speed } => Intent::SetSpeed(Speed::new(speed)?),
            Self::SetTuning { tuning_hz } => Intent::SetTuning(TuningHz::new(tuning_hz)?),
            Self::SetKey {
                key_shift_semitones,
            } => Intent::SetKey(KeyShift::new(key_shift_semitones)?),
            Self::PlayRegion { start_ms, end_ms } => Intent::PlayRegion(TimeRange::new(
                Millis::position(start_ms)?,
                Millis::position(end_ms)?,
            )?),
            Self::SetPlayMode { play_mode } => Intent::SetPlayMode(play_mode.into()),
            Self::SetVolume { volume_percent } => Intent::SetVolume(Volume::new(volume_percent)?),
        })
    }
}

impl From<PlayMode> for PlayModeChoice {
    fn from(mode: PlayMode) -> Self {
        match mode {
            PlayMode::ToEnd => Self::ToEnd,
            PlayMode::ToRegionEnd => Self::ToRegionEnd,
            PlayMode::Loop => Self::Loop,
        }
    }
}

impl From<PlayModeChoice> for PlayMode {
    fn from(choice: PlayModeChoice) -> Self {
        match choice {
            PlayModeChoice::ToEnd => Self::ToEnd,
            PlayModeChoice::ToRegionEnd => Self::ToRegionEnd,
            PlayModeChoice::Loop => Self::Loop,
        }
    }
}

impl From<ConnectionStatus> for ConnectionState {
    fn from(status: ConnectionStatus) -> Self {
        match status {
            ConnectionStatus::Connected => Self::Connected,
            ConnectionStatus::Disconnected => Self::Disconnected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tuning_is_supported_between_300_and_500_hertz() {
        assert!(tuning_in_range(300.0) && tuning_in_range(432.5) && tuning_in_range(500.0));
        assert!(!tuning_in_range(299.9) && !tuning_in_range(600.0) && !tuning_in_range(f64::NAN));
    }

    #[test]
    fn commands_become_intents() {
        let intent = PlayerCommand::Seek { position_ms: 1_500 }.into_intent();
        assert_eq!(
            intent,
            Ok(Intent::Seek {
                position: Millis::new(1_500),
                scrubbing: false
            })
        );
    }

    #[test]
    fn a_volume_command_is_checked_and_becomes_an_intent() {
        assert_eq!(
            PlayerCommand::SetVolume { volume_percent: 40 }.into_intent(),
            Ok(Intent::SetVolume(Volume::new(40).unwrap()))
        );
        assert!(
            PlayerCommand::SetVolume {
                volume_percent: 101
            }
            .into_intent()
            .is_err()
        );
    }

    #[test]
    fn a_target_tempo_gives_the_speed() {
        let target = target_tempo(90.0, 120.0).unwrap();
        assert!(target.supported);
        assert!((target.speed - 0.75).abs() < 1e-12);
        let too_fast = target_tempo(300.0, 100.0).unwrap();
        assert!(!too_fast.supported);
        assert!(target_tempo(0.0, 100.0).is_err());
    }

    #[test]
    fn a_region_needs_an_end_after_its_start() {
        assert!(range_of(1_000, 2_000).is_ok());
        assert!(range_of(2_000, 2_000).is_err());
        assert!(range_of(3_000, 2_000).is_err());
    }

    #[test]
    fn a_bookmark_name_must_not_be_blank() {
        assert!(bookmark_name(" Chorus ").is_ok());
        assert!(bookmark_name("  ").is_err());
    }

    #[test]
    fn out_of_range_values_are_refused_before_sending() {
        assert!(
            PlayerCommand::SetSpeed { speed: 3.0 }
                .into_intent()
                .is_err()
        );
        assert!(
            PlayerCommand::SetKey {
                key_shift_semitones: 20
            }
            .into_intent()
            .is_err()
        );
    }
}

/// How this client presents itself to the server: set by signing in or by choosing guest
/// access, used by every request and by the real-time channel.
static ACCESS: Mutex<Access> = Mutex::new(Access::None);

#[derive(Clone)]
enum Access {
    None,
    Token(String),
    Guest,
}

fn access() -> Access {
    ACCESS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

fn set_access(new: Access) {
    *ACCESS.lock().unwrap_or_else(PoisonError::into_inner) = new;
}

fn link(server_url: &str) -> HttpServerLink {
    let token = match access() {
        Access::Token(token) => Some(token),
        Access::None | Access::Guest => None,
    };
    HttpServerLink::with_token(server_url, token)
}

fn channel_access() -> ChannelAccess {
    match access() {
        Access::None => ChannelAccess::Open,
        Access::Token(token) => ChannelAccess::Token(token),
        Access::Guest => ChannelAccess::Guest,
    }
}

/// What may be known before signing in.
/// Why a sign-in or an administration call failed.
#[derive(Debug)]
pub enum AccessError {
    /// Wrong user name or password, or the sign-in is no longer valid.
    NotSignedIn,
    /// The role may not do this.
    NotAllowed,
    /// Too many failed sign-ins; try again after this many seconds.
    TooManyAttempts { retry_after_seconds: u64 },
    /// The server refused the values; its reason is meant for the person.
    Refused { reason: String },
    /// The server cannot be reached or answered unexpectedly.
    Unavailable { detail: String },
}

impl From<LinkError> for AccessError {
    fn from(error: LinkError) -> Self {
        match error {
            LinkError::Unauthorized => Self::NotSignedIn,
            LinkError::Forbidden => Self::NotAllowed,
            LinkError::TooManyAttempts {
                retry_after_seconds,
            } => Self::TooManyAttempts {
                retry_after_seconds,
            },
            LinkError::Refused(reason) => Self::Refused { reason },
            other => Self::Unavailable {
                detail: other.to_string(),
            },
        }
    }
}

/// What may be known before signing in.
pub struct AccessStatusInfo {
    /// No user exists yet: offer to set up the first admin.
    pub setup_needed: bool,
    /// Offer guest access.
    pub guest_enabled: bool,
    /// The rules for new passwords.
    pub password_policy: PasswordPolicyChoice,
}

/// How demanding the rules for new passwords are.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PasswordPolicyChoice {
    /// At least 6 characters.
    Standard,
    /// At least 10 characters with lowercase, uppercase, digit and special character.
    Strict,
}

impl From<PasswordPolicy> for PasswordPolicyChoice {
    fn from(policy: PasswordPolicy) -> Self {
        match policy {
            PasswordPolicy::Standard => Self::Standard,
            PasswordPolicy::Strict => Self::Strict,
        }
    }
}

impl From<PasswordPolicyChoice> for PasswordPolicy {
    fn from(policy: PasswordPolicyChoice) -> Self {
        match policy {
            PasswordPolicyChoice::Standard => Self::Standard,
            PasswordPolicyChoice::Strict => Self::Strict,
        }
    }
}

/// The role of a person.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoleChoice {
    Admin,
    User,
    Guest,
}

/// Who the server takes this client for and what it may do. Hide what is not allowed;
/// the server refuses it anyway.
#[derive(Clone)]
pub struct RightsInfo {
    /// Absent for a guest.
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub role: RoleChoice,
    pub can_control_playback: bool,
    pub can_edit_bookmarks: bool,
    pub can_upload: bool,
    pub can_manage_recordings: bool,
    pub can_manage_users: bool,
}

/// A completed sign-in. Keep `token` (for example in browser storage) to resume it with
/// `resume_sign_in`; it stays valid for 30 days without use.
pub struct SignInResult {
    pub token: String,
    pub rights: RightsInfo,
}

/// A user account as the admin sees it.
pub struct UserItem {
    pub id: u64,
    pub user_name: String,
    pub role: RoleChoice,
    pub active: bool,
}

impl From<Role> for RoleChoice {
    fn from(role: Role) -> Self {
        match role {
            Role::Admin => Self::Admin,
            Role::User => Self::User,
            Role::Guest => Self::Guest,
        }
    }
}

impl From<RoleChoice> for Role {
    fn from(role: RoleChoice) -> Self {
        match role {
            RoleChoice::Admin => Self::Admin,
            RoleChoice::User => Self::User,
            RoleChoice::Guest => Self::Guest,
        }
    }
}

impl From<&Rights> for RightsInfo {
    fn from(rights: &Rights) -> Self {
        Self {
            user_id: rights.user_id.map(UserId::value),
            user_name: rights.user_name.clone(),
            role: rights.role.into(),
            can_control_playback: rights.allows(Capability::PlayControl),
            can_edit_bookmarks: rights.allows(Capability::EditBookmarks),
            can_upload: rights.allows(Capability::Upload),
            can_manage_recordings: rights.allows(Capability::ManageRecordings),
            can_manage_users: rights.allows(Capability::ManageUsers),
        }
    }
}

impl From<&ManagedUser> for UserItem {
    fn from(user: &ManagedUser) -> Self {
        Self {
            id: user.id.value(),
            user_name: user.user_name.clone(),
            role: user.role.into(),
            active: user.active,
        }
    }
}

fn signed_in(done: SignedIn) -> SignInResult {
    set_access(Access::Token(done.token.clone()));
    SignInResult {
        token: done.token,
        rights: RightsInfo::from(&done.rights),
    }
}

/// What the server tells before sign-in: first start, guest access.
pub async fn access_status(server_url: String) -> Result<AccessStatusInfo, AccessError> {
    let status = load_access_status(&link(&server_url))
        .await
        .map_err(AccessError::from)?;
    Ok(AccessStatusInfo {
        setup_needed: status.setup_needed,
        guest_enabled: status.guest_enabled,
        password_policy: status.password_policy.into(),
    })
}

/// Creates the first admin and signs in; fails once a user exists.
pub async fn set_up_admin(
    server_url: String,
    user_name: String,
    password: String,
) -> Result<SignInResult, AccessError> {
    let credentials = Credentials {
        user_name,
        password,
    };
    set_up_first_admin(&link(&server_url), &credentials)
        .await
        .map(signed_in)
        .map_err(AccessError::from)
}

/// Signs in with user name and password.
pub async fn sign_in_with_password(
    server_url: String,
    user_name: String,
    password: String,
) -> Result<SignInResult, AccessError> {
    let credentials = Credentials {
        user_name,
        password,
    };
    sign_in(&link(&server_url), &credentials)
        .await
        .map(signed_in)
        .map_err(AccessError::from)
}

/// Resumes an earlier sign-in with the token it returned; fails if it has expired.
pub async fn resume_sign_in(server_url: String, token: String) -> Result<RightsInfo, AccessError> {
    let probe = HttpServerLink::with_token(&server_url, Some(token.clone()));
    let rights = load_rights(&probe).await.map_err(AccessError::from)?;
    set_access(Access::Token(token));
    Ok(RightsInfo::from(&rights))
}

/// Enters as a guest; fails while the server does not let guests in.
pub async fn continue_as_guest(server_url: String) -> Result<RightsInfo, AccessError> {
    let rights = load_rights(&HttpServerLink::new(&server_url))
        .await
        .map_err(AccessError::from)?;
    set_access(Access::Guest);
    Ok(RightsInfo::from(&rights))
}

/// Signs out: the token stops working on the server and this client forgets it.
/// Close the session channel afterwards by watching again.
pub async fn sign_out_now(server_url: String) -> Result<(), AccessError> {
    let result = match access() {
        Access::Token(_) => sign_out(&link(&server_url))
            .await
            .map_err(AccessError::from),
        Access::None | Access::Guest => Ok(()),
    };
    set_access(Access::None);
    with_channel(|channel| *channel = None);
    result
}

/// The rights of the current sign-in or guest access.
pub async fn current_rights(server_url: String) -> Result<RightsInfo, AccessError> {
    load_rights(&link(&server_url))
        .await
        .map(|rights| RightsInfo::from(&rights))
        .map_err(AccessError::from)
}

/// All users; admin only.
pub async fn list_users(server_url: String) -> Result<Vec<UserItem>, AccessError> {
    load_users(&link(&server_url))
        .await
        .map(|users| users.iter().map(UserItem::from).collect())
        .map_err(AccessError::from)
}

/// Adds a user; admin only.
pub async fn create_user(
    server_url: String,
    user_name: String,
    password: String,
    role: RoleChoice,
) -> Result<UserItem, AccessError> {
    let user = NewUser {
        user_name,
        password,
        role: role.into(),
    };
    add_user(&link(&server_url), &user)
        .await
        .map(|created| UserItem::from(&created))
        .map_err(AccessError::from)
}

/// Changes the fields that are present; admin only.
pub async fn change_user(
    server_url: String,
    user_id: u64,
    role: Option<RoleChoice>,
    active: Option<bool>,
    password: Option<String>,
) -> Result<(), AccessError> {
    let change = UserChange {
        role: role.map(Role::from),
        active,
        password,
    };
    edit_user(&link(&server_url), UserId::new(user_id), &change)
        .await
        .map_err(AccessError::from)
}

/// Deletes a user; admin only.
pub async fn remove_user(server_url: String, user_id: u64) -> Result<(), AccessError> {
    delete_user(&link(&server_url), UserId::new(user_id))
        .await
        .map_err(AccessError::from)
}

/// Chooses the rules for new passwords; admin only.
pub async fn switch_password_policy(
    server_url: String,
    policy: PasswordPolicyChoice,
) -> Result<(), AccessError> {
    set_password_policy(&link(&server_url), policy.into())
        .await
        .map_err(AccessError::from)
}

/// Lets guests in or not; admin only.
pub async fn switch_guest_access(server_url: String, enabled: bool) -> Result<(), AccessError> {
    set_guest_access(&link(&server_url), enabled)
        .await
        .map_err(AccessError::from)
}
