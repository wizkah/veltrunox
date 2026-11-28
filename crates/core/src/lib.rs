//! Domain types and pure operations shared by server and client.

mod analysis;
mod auth;
mod bookmark;
mod bookmark_import;
mod playback;
mod reason;
mod recording;
mod stems;
mod tempo_pitch;
mod units;
mod waveform;

pub use analysis::{
    AnalysisInfo, AnalysisProgress, Confidence, KeyMode, LibraryEntry, MusicalKey, PitchClass,
    RecordingAnalysis,
};
pub use auth::{
    ApiTokenId, Capability, InvalidUser, PasswordPolicy, Role, TokenName, TokenScope, UserId,
    UserName,
};
pub use bookmark::{
    BookmarkId, BookmarkName, Bookmarks, EntryRef, Label, NewBookmarks, NewLabel, NewRegion,
    Region, Version, VersionConflict, check_version,
};
pub use bookmark_import::{
    ConflictKind, ImportConflict, ImportDiff, ImportedBookmarks, ImportedLabel, ImportedRegion,
    KnownEntry, plan_import,
};
pub use playback::{
    Intent, PlayMode, PlaybackSettings, PlaybackState, PositionReference, TransportError,
    end_playback, end_region, pause, play, play_region, seek, select_recording, set_key,
    set_play_mode, set_speed, set_stem_mix, set_tuning, set_volume,
};
pub use reason::{Reason, Refusal};
pub use recording::{InvalidRecording, Recording, RecordingId, RecordingTitle};
pub use stems::{
    MAX_STEMS, Section, SplitModel, StemChannel, StemMix, StemName, StemSource, section_regions,
};
pub use tempo_pitch::{Bpm, TargetSpeed, pitch_shift_semitones, speed_for_target_tempo};
pub use units::{InvalidValue, KeyShift, Millis, Speed, TimeRange, TuningHz, Volume};
pub use waveform::{FINEST_BLOCK_FRAMES, Peak, Waveform, WaveformBuilder, WaveformLevel};
