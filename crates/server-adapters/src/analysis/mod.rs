//! Estimates reference tuning, key and tempo of a recording (ADR-007).

mod key;
mod spectrum;
mod tempo;
mod tuning;

use crate::audio::Pcm;
use veltrunox_core::{Confidence, RecordingAnalysis, TuningHz};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AnalysisError {
    #[error("the recording is too short to analyse")]
    TooShort,
    #[error("the recording has no tonal content to find a key in")]
    NoTonalContent,
    #[error("the recording has no rhythmic structure to find a tempo in")]
    NoRhythm,
}

impl veltrunox_core::Refusal for AnalysisError {
    fn reason(&self) -> veltrunox_core::Reason {
        veltrunox_core::Reason::new(
            self,
            match self {
                Self::TooShort => "analysis_too_short",
                Self::NoTonalContent => "analysis_no_tonal_content",
                Self::NoRhythm => "analysis_no_rhythm",
            },
        )
    }
}

/// Analyses a decoded recording. CPU-bound: run it off the real-time paths.
pub fn analyse(pcm: &Pcm) -> Result<RecordingAnalysis, AnalysisError> {
    analyse_reporting(pcm, &|_| {})
}

/// Like `analyse`; `report` is told how far the analysis has come in percent (0 to 100).
pub fn analyse_reporting(
    pcm: &Pcm,
    report: &dyn Fn(u8),
) -> Result<RecordingAnalysis, AnalysisError> {
    let mono = pcm.mono();
    if mono.len() < spectrum::FFT_SIZE {
        return Err(AnalysisError::TooShort);
    }
    let frames = spectrum::frames(&mono, pcm.sample_rate());
    drop(mono);
    report(30);

    let all_peaks: Vec<_> = frames.peaks.iter().flatten().copied().collect();
    let (tuning, tuning_confidence) = match tuning::estimate(&all_peaks) {
        Some(found) => (found.tuning, found.confidence),
        None => (TuningHz::CONCERT_PITCH, Confidence::new(1.0)),
    };
    report(60);
    let key = key::estimate(&frames.peaks, tuning).ok_or(AnalysisError::NoTonalContent)?;
    report(80);
    let tempo = tempo::estimate(&frames.onset_strength, frames.hop_seconds)
        .ok_or(AnalysisError::NoRhythm)?;

    Ok(RecordingAnalysis {
        tuning,
        tuning_confidence,
        key: key.key,
        key_confidence: key.confidence,
        tempo: tempo.tempo,
        tempo_confidence: tempo.confidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::KeyMode;

    const RATE: u32 = 44_100;

    /// Chords struck on every beat, C major (C, F, G, C), tuned to `a4` hertz.
    fn rehearsal(a4: f64, bpm: f64, seconds: f64) -> Pcm {
        let chords: [[f64; 3]; 4] = [
            [0.0, 4.0, 7.0],
            [5.0, 9.0, 12.0],
            [7.0, 11.0, 14.0],
            [0.0, 4.0, 7.0],
        ];
        let beat_seconds = 60.0 / bpm;
        let frames = (seconds * f64::from(RATE)) as usize;
        let mut samples = Vec::with_capacity(frames * 2);
        for frame in 0..frames {
            let time = frame as f64 / f64::from(RATE);
            let beat = (time / beat_seconds) as usize;
            let since_strike = time - beat as f64 * beat_seconds;
            let envelope = (-since_strike / 0.2).exp();
            let chord = chords[beat / 4 % chords.len()];
            let value: f64 = chord
                .iter()
                .flat_map(|semitones| {
                    let base = a4 * 2.0_f64.powf((semitones - 9.0) / 12.0);
                    (1..=3).map(move |harmonic| {
                        (base * f64::from(harmonic), 1.0 / f64::from(harmonic))
                    })
                })
                .map(|(hertz, amplitude)| {
                    amplitude * (2.0 * std::f64::consts::PI * hertz * time).sin()
                })
                .sum();
            let sample = (0.1 * envelope * value) as f32;
            samples.extend([sample, sample]);
        }
        Pcm::new(samples, RATE)
    }

    #[test]
    fn finds_tuning_key_and_tempo_of_synthesised_music() {
        let found = analyse(&rehearsal(435.0, 100.0, 40.0)).unwrap();
        let cents = 1200.0 * (found.tuning.hertz() / 435.0).log2();
        assert!(cents.abs() < 3.0, "tuning {} Hz", found.tuning.hertz());
        assert_eq!(
            (found.key.tonic.index(), found.key.mode),
            (0, KeyMode::Major)
        );
        assert!(
            (found.tempo.value() - 100.0).abs() < 1.0,
            "tempo {}",
            found.tempo.value()
        );
    }

    #[test]
    fn silence_has_no_key() {
        let silence = Pcm::new(vec![0.0; RATE as usize * 20], RATE);
        assert_eq!(analyse(&silence), Err(AnalysisError::NoTonalContent));
    }

    #[test]
    fn a_fraction_of_a_second_is_too_short() {
        let short = Pcm::new(vec![0.0; 2_000], RATE);
        assert_eq!(analyse(&short), Err(AnalysisError::TooShort));
    }
}
