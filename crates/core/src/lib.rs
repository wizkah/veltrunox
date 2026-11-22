//! Domain types and pure operations shared by server and client.

mod analysis;
mod bookmark;
mod playback;
mod recording;
mod tempo_pitch;
mod units;
mod waveform;

pub use analysis::{
    AnalysisInfo, AnalysisProgress, Confidence, KeyMode, LibraryEntry, MusicalKey, PitchClass,
    RecordingAnalysis,
};
pub use bookmark::{
    BookmarkId, BookmarkName, Bookmarks, Label, Region, Version, VersionConflict, check_version,
};
pub use playback::{
    Intent, PlaybackSettings, PlaybackState, PositionReference, TransportError, end_playback,
    pause, play, seek, select_recording, set_key, set_loop, set_speed, set_tuning, set_volume,
};
pub use recording::{InvalidRecording, Recording, RecordingId, RecordingTitle};
pub use tempo_pitch::{Bpm, TargetSpeed, pitch_shift_semitones, speed_for_target_tempo};
pub use units::{InvalidValue, KeyShift, Millis, Speed, TimeRange, TuningHz, Volume};
pub use waveform::{FINEST_BLOCK_FRAMES, Peak, Waveform, WaveformLevel};
