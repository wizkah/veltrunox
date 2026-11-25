//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.
//!
//! All functions are `async` on purpose: the browser runs async functions on the
//! main thread, while synchronous ones need a worker pool, which needs threads.

use crate::clock::local_now;
use crate::frb_generated::StreamSink;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
use veltrunox_client_adapters::{EventChannel, HttpServerLink, SystemTimer, open_channel};
use veltrunox_client_app::{
    ApiToken, CreatedApiToken, ExchangeFormat, ImportOptions, ImportReport, ImportStatus,
    RecordingImport, end_api_token, export_bookmarks, import_bookmarks, load_api_tokens,
    make_api_token,
};
use veltrunox_client_app::{
    ChannelAccess, ChannelEvent, ConnectionMonitor, ConnectionStatus, LinkError, LinkEvent,
    ScrubThrottle, ServerEvent, SessionView, ViewChange, WaveformCache, add_label, add_recording,
    add_region, analyse_again, convert_label_to_region, convert_region_to_label, delete_label,
    delete_recording, delete_region, edit_label, edit_region, load_bookmarks, load_library,
    load_waveform, rename_recording,
};
use veltrunox_client_app::{
    Credentials, ManagedUser, NewUser, Rights, SignedIn, UserChange, add_user, change_own_password,
    delete_user, edit_user, load_access_status, load_rights, load_users, set_guest_access,
    set_password_policy, set_up_first_admin, sign_in, sign_out,
};
use veltrunox_core::{
    AnalysisProgress, ApiTokenId, BookmarkId, BookmarkName, Bpm, Capability, EntryRef, Intent,
    InvalidValue, KeyMode, KeyShift, LibraryEntry, Millis, PasswordPolicy, PlayMode, Reason,
    RecordingId, RecordingTitle, Refusal, Role, Speed, TargetSpeed, TimeRange, TokenScope,
    TuningHz, UserId, Version, Volume, speed_for_target_tempo,
};

/// A recording as the user interface needs it.
pub struct RecordingItem {
    pub id: u64,
    pub title: String,
    /// Length of the audio in milliseconds; `None` until the recording has been scanned.
    pub length_ms: Option<u64>,
    pub analysis_state: AnalysisState,
    /// How far the running analysis has come, 0 to 100; only meaningful in the state `Running`.
    pub analysis_percent: u8,
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
            AnalysisProgress::Running(_) => (AnalysisState::Running, None),
            AnalysisProgress::Failed(reason) => (AnalysisState::Failed, Some(reason.clone())),
            AnalysisProgress::Done => (AnalysisState::Done, None),
        };
        Self {
            id: entry.recording.id.value(),
            title: entry.recording.title.as_str().to_owned(),
            length_ms: entry.length.map(Millis::value),
            analysis_state,
            analysis_percent: match entry.analysis.progress {
                AnalysisProgress::Running(percent) => percent,
                _ => 0,
            },
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

/// Why the server refused something. The `code` is stable and the interface
/// words it (`args` are the numbers its text needs); `message` is English, for
/// codes the interface does not know.
#[derive(Debug, Clone)]
pub struct ReasonInfo {
    pub code: String,
    pub args: Vec<u64>,
    pub message: String,
}

impl From<Reason> for ReasonInfo {
    fn from(reason: Reason) -> Self {
        Self {
            code: reason.code,
            args: reason.args,
            message: reason.message,
        }
    }
}

/// Why uploading, renaming or deleting a recording failed.
#[derive(Debug)]
pub enum RecordingActionError {
    /// The server refused it.
    Refused { reason: ReasonInfo },
    /// Anything else (no connection, an invalid title, ...), described in English.
    Failed { detail: String },
}

/// What became of an uploaded file.
pub struct UploadedRecordingItem {
    pub id: u64,
    pub title: String,
    /// The same audio was in the library already; nothing was added.
    pub existing: bool,
}

/// Sends an audio file to the server. `upload_id` is a fresh id of 1 to 64 letters, digits, `-`
/// or `_`; the bytes the server has received arrive as `SessionUpdate::UploadProgress` under it.
/// The refusal reason (not audio, too large, not allowed)
/// comes back as a code for the interface; the analysis result arrives as a session update.
pub async fn upload_recording(
    server_url: String,
    file_name: String,
    upload_id: String,
    content: Vec<u8>,
) -> Result<UploadedRecordingItem, RecordingActionError> {
    let link = link(&server_url);
    let uploaded = add_recording(&link, &file_name, &upload_id, content)
        .await
        .map_err(action_failure)?;
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
) -> Result<(), RecordingActionError> {
    let title = RecordingTitle::new(&title).map_err(|error| RecordingActionError::Refused {
        reason: error.reason().into(),
    })?;
    rename_recording(&link(&server_url), RecordingId::new(recording_id), &title)
        .await
        .map_err(action_failure)
}

/// Deletes a recording with its bookmarks and analysis (admin only).
pub async fn delete_recording_for_good(
    server_url: String,
    recording_id: u64,
) -> Result<(), RecordingActionError> {
    delete_recording(&link(&server_url), RecordingId::new(recording_id))
        .await
        .map_err(action_failure)
}

/// The server's reason when it refused, else the error's description.
fn action_failure(error: LinkError) -> RecordingActionError {
    match error {
        LinkError::Refused(reason) => RecordingActionError::Refused {
            reason: reason.into(),
        },
        other => RecordingActionError::Failed {
            detail: other.to_string(),
        },
    }
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

/// What a failed marker write reports as its reason when someone else changed the marker
/// meanwhile; the interface tells the user in its own words (it knows this word as
/// `conflictReason` in `bridge_marker_source.dart`).
const CONFLICT_REASON: &str = "conflict";

fn write_failure(error: &LinkError) -> String {
    match error {
        LinkError::Conflict { .. } => CONFLICT_REASON.to_owned(),
        other => other.to_string(),
    }
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
    .map_err(|error| write_failure(&error))
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
    .map_err(|error| write_failure(&error))
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
        .map_err(|error| write_failure(&error))
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
    .map_err(|error| write_failure(&error))
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
    .map_err(|error| write_failure(&error))
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
        .map_err(|error| write_failure(&error))
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
    .map_err(|error| write_failure(&error))
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
    .map_err(|error| write_failure(&error))
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
    /// Recordings were added, renamed or removed; load the library again.
    LibraryChanged,
    /// The server has received this many bytes of the upload with this id.
    UploadProgress {
        upload_id: String,
        received_bytes: u64,
    },
    Rejected {
        reason: ReasonInfo,
    },
    /// The server accepted the sign-in of the channel; these are the rights of this client.
    Rights {
        rights: RightsInfo,
    },
    /// Whether the server has an audio device; without one it plays nothing.
    AudioOutput {
        available: bool,
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
        ViewChange::LibraryChanged => Some(SessionUpdate::LibraryChanged),
        ViewChange::UploadProgress {
            upload_id,
            received_bytes,
        } => Some(SessionUpdate::UploadProgress {
            upload_id,
            received_bytes,
        }),
        ViewChange::Rejected(reason) => Some(SessionUpdate::Rejected {
            reason: reason.into(),
        }),
        ViewChange::Rights(rights) => Some(SessionUpdate::Rights {
            rights: RightsInfo::from(&rights),
        }),
        ViewChange::AudioOutput => view
            .audio_output()
            .map(|available| SessionUpdate::AudioOutput { available }),
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
    Refused { reason: ReasonInfo },
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
            LinkError::Refused(reason) => Self::Refused {
                reason: reason.into(),
            },
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

/// The signed-in account changes its own password with its current one. Afterwards every
/// sign-in of the account has ended, this one too. Refusals (wrong current password, a new one
/// that breaks the rules) carry their reason.
pub async fn change_my_password(
    server_url: String,
    current_password: String,
    new_password: String,
) -> Result<(), AccessError> {
    change_own_password(&link(&server_url), &current_password, &new_password)
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

/// The file formats of the bookmark exchange.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExchangeFormatChoice {
    Json,
    Csv,
}

impl From<ExchangeFormatChoice> for ExchangeFormat {
    fn from(choice: ExchangeFormatChoice) -> Self {
        match choice {
            ExchangeFormatChoice::Json => Self::Json,
            ExchangeFormatChoice::Csv => Self::Csv,
        }
    }
}

/// The bookmarks of one recording, or of the whole library when `recording_id` is `None`, as the
/// text of a file in the chosen format. Failures are an `AccessError`: not signed in, a refusal
/// with a reason, or the server cannot be reached.
pub async fn export_bookmarks_text(
    server_url: String,
    recording_id: Option<u64>,
    format: ExchangeFormatChoice,
) -> Result<String, AccessError> {
    export_bookmarks(
        &link(&server_url),
        recording_id.map(RecordingId::new),
        format.into(),
    )
    .await
    .map_err(AccessError::from)
}

/// What became of one recording of an import.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImportStatusInfo {
    /// Applied, or, for a preview, would be applied.
    Ready,
    /// An entry's stored original changed or is gone.
    Conflict,
    /// Entries are invalid; see the issues.
    Invalid,
    UnknownRecording,
}

/// What kind of bookmark an import issue points at.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImportEntryKind {
    Label,
    Region,
}

/// A problem with one entry of an import: the `index`-th label or region of the file's set for
/// the recording, counted from 0.
pub struct ImportIssueInfo {
    pub kind: ImportEntryKind,
    pub index: u32,
    pub reason: ReasonInfo,
}

/// The outcome for one recording; the counts are zero unless the status is `Ready`.
pub struct RecordingImportInfo {
    pub recording_id: u64,
    pub status: ImportStatusInfo,
    pub added: u32,
    pub changed: u32,
    pub unchanged: u32,
    pub removed: u32,
    pub issues: Vec<ImportIssueInfo>,
}

/// The server's answer to an import, one entry per recording of the file in its order.
pub struct ImportReportInfo {
    pub dry_run: bool,
    pub recordings: Vec<RecordingImportInfo>,
}

impl From<&RecordingImport> for RecordingImportInfo {
    fn from(found: &RecordingImport) -> Self {
        Self {
            recording_id: found.recording.value(),
            status: match found.status {
                ImportStatus::Ready => ImportStatusInfo::Ready,
                ImportStatus::Conflict => ImportStatusInfo::Conflict,
                ImportStatus::Invalid => ImportStatusInfo::Invalid,
                ImportStatus::UnknownRecording => ImportStatusInfo::UnknownRecording,
            },
            added: found.counts.added,
            changed: found.counts.changed,
            unchanged: found.counts.unchanged,
            removed: found.counts.removed,
            issues: found
                .issues
                .iter()
                .map(|issue| {
                    let (kind, index) = match issue.entry {
                        EntryRef::Label(index) => (ImportEntryKind::Label, index),
                        EntryRef::Region(index) => (ImportEntryKind::Region, index),
                    };
                    ImportIssueInfo {
                        kind,
                        index: u32::try_from(index).unwrap_or(u32::MAX),
                        reason: issue.reason.clone().into(),
                    }
                })
                .collect(),
        }
    }
}

impl From<&ImportReport> for ImportReportInfo {
    fn from(report: &ImportReport) -> Self {
        Self {
            dry_run: report.dry_run,
            recordings: report.recordings.iter().map(Into::into).collect(),
        }
    }
}

/// Sends the text of an export file. With `dry_run` the server only says what it would do; the
/// same call without it replaces the bookmarks of the recordings the file lists. With
/// `overwrite`, entries whose stored original changed meanwhile are taken too. A file that
/// cannot be read at all is an `AccessError::Refused` with a reason.
pub async fn import_bookmarks_text(
    server_url: String,
    content: String,
    format: ExchangeFormatChoice,
    dry_run: bool,
    overwrite: bool,
) -> Result<ImportReportInfo, AccessError> {
    import_bookmarks(
        &link(&server_url),
        &content,
        format.into(),
        ImportOptions { dry_run, overwrite },
    )
    .await
    .map(|report| ImportReportInfo::from(&report))
    .map_err(AccessError::from)
}

/// How far an API token reaches at most.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenScopeChoice {
    /// Reads library, bookmarks and exports.
    ReadOnly,
    /// Also edits, replaces and imports bookmarks.
    EditBookmarks,
}

impl From<TokenScopeChoice> for TokenScope {
    fn from(choice: TokenScopeChoice) -> Self {
        match choice {
            TokenScopeChoice::ReadOnly => Self::ReadOnly,
            TokenScopeChoice::EditBookmarks => Self::EditBookmarks,
        }
    }
}

impl From<TokenScope> for TokenScopeChoice {
    fn from(scope: TokenScope) -> Self {
        match scope {
            TokenScope::ReadOnly => Self::ReadOnly,
            TokenScope::EditBookmarks => Self::EditBookmarks,
        }
    }
}

/// An API token as listed; its text is known only when it is made.
pub struct ApiTokenItem {
    pub id: u64,
    pub name: String,
    pub scope: TokenScopeChoice,
    /// Unix seconds.
    pub created_at: u64,
    /// Unix seconds; `None` until the token was used.
    pub last_used: Option<u64>,
}

impl From<&ApiToken> for ApiTokenItem {
    fn from(token: &ApiToken) -> Self {
        Self {
            id: token.id.value(),
            name: token.name.clone(),
            scope: token.scope.into(),
            created_at: token.created_at,
            last_used: token.last_used,
        }
    }
}

/// A new token with its text, shown only once.
pub struct CreatedApiTokenItem {
    pub token: String,
    pub api_token: ApiTokenItem,
}

impl From<&CreatedApiToken> for CreatedApiTokenItem {
    fn from(created: &CreatedApiToken) -> Self {
        Self {
            token: created.token.clone(),
            api_token: ApiTokenItem::from(&created.api_token),
        }
    }
}

/// The API tokens of the signed-in user.
pub async fn list_api_tokens(server_url: String) -> Result<Vec<ApiTokenItem>, AccessError> {
    load_api_tokens(&link(&server_url))
        .await
        .map(|tokens| tokens.iter().map(ApiTokenItem::from).collect())
        .map_err(AccessError::from)
}

/// Makes an API token for the signed-in user (not for a guest).
pub async fn create_api_token(
    server_url: String,
    name: String,
    scope: TokenScopeChoice,
) -> Result<CreatedApiTokenItem, AccessError> {
    make_api_token(&link(&server_url), &name, scope.into())
        .await
        .map(|created| CreatedApiTokenItem::from(&created))
        .map_err(AccessError::from)
}

/// Ends one of the signed-in user's API tokens.
pub async fn revoke_api_token(server_url: String, token_id: u64) -> Result<(), AccessError> {
    end_api_token(&link(&server_url), ApiTokenId::new(token_id))
        .await
        .map_err(AccessError::from)
}
