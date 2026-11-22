//! Analysis results of a recording: reference tuning, key and tempo.

use crate::tempo_pitch::Bpm;
use crate::units::TuningHz;

/// Pitch class of a key's tonic: 0 = C, 1 = C♯/D♭, … 11 = B.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchClass(u8);

impl PitchClass {
    pub const COUNT: u8 = 12;

    /// Wraps any number of semitones into 0..12.
    pub fn from_semitones(semitones: i32) -> Self {
        Self(semitones.rem_euclid(i32::from(Self::COUNT)) as u8)
    }

    pub fn index(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

/// The sounding key of a recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicalKey {
    pub tonic: PitchClass,
    pub mode: KeyMode,
}

/// How much an estimate can be trusted: best score divided by the second best,
/// so 1.0 means "no better than the runner-up".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(ratio: f64) -> Self {
        Self(if ratio.is_finite() {
            ratio.max(1.0)
        } else {
            1.0
        })
    }

    pub fn ratio(self) -> f64 {
        self.0
    }
}

/// Estimated values of one recording. Analysis results are never edited;
/// playback settings compensate differences (see `pitch_shift_semitones`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordingAnalysis {
    pub tuning: TuningHz,
    pub tuning_confidence: Confidence,
    pub key: MusicalKey,
    pub key_confidence: Confidence,
    pub tempo: Bpm,
    pub tempo_confidence: Confidence,
}

/// Where a recording stands in the analysis.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AnalysisProgress {
    /// Nothing stored, nothing planned.
    #[default]
    None,
    Queued,
    /// Under way; the value is how far it has come, in percent (0 to 100).
    Running(u8),
    /// The analysis could not be made; the reason is for the user to read.
    Failed(String),
    Done,
}

/// The analysis side of a library entry: the status and, once done, the result.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnalysisInfo {
    pub progress: AnalysisProgress,
    pub result: Option<RecordingAnalysis>,
}

/// A recording together with what is known about its analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryEntry {
    pub recording: crate::Recording,
    /// Known once the recording has been scanned, whatever became of its analysis.
    pub length: Option<crate::Millis>,
    pub analysis: AnalysisInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_class_wraps_in_both_directions() {
        assert_eq!(PitchClass::from_semitones(14).index(), 2);
        assert_eq!(PitchClass::from_semitones(-1).index(), 11);
        assert_eq!(PitchClass::from_semitones(0).index(), 0);
    }

    #[test]
    fn confidence_is_at_least_one_and_finite() {
        assert_eq!(Confidence::new(0.2).ratio(), 1.0);
        assert_eq!(Confidence::new(f64::NAN).ratio(), 1.0);
        assert_eq!(Confidence::new(1.8).ratio(), 1.8);
    }
}
