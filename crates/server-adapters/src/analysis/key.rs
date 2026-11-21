//! Musical key from the tonal peaks: chroma profile matched against Krumhansl–Kessler templates.

use super::spectrum::Peak;
use veltrunox_core::{Confidence, KeyMode, MusicalKey, PitchClass, TuningHz};

const PITCH_CLASSES: usize = PitchClass::COUNT as usize;

/// Krumhansl–Kessler key profiles, tonic first.
const MAJOR: [f64; PITCH_CLASSES] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const MINOR: [f64; PITCH_CLASSES] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

pub struct KeyEstimate {
    pub key: MusicalKey,
    pub confidence: Confidence,
}

/// `None` when no frame has a tonal peak (silence, noise).
pub fn estimate(frames: &[Vec<Peak>], tuning: TuningHz) -> Option<KeyEstimate> {
    let mut global = [0.0; PITCH_CLASSES];
    for peaks in frames {
        let (chroma, strength) = frame_chroma(peaks, tuning);
        for (total, value) in global.iter_mut().zip(chroma) {
            *total += strength * value;
        }
    }
    let norm = global.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm < 1e-12 {
        return None;
    }
    let profile = global.map(|value| value / norm);

    let mut scores: Vec<(f64, MusicalKey)> = (0..PITCH_CLASSES)
        .flat_map(|tonic| {
            [(MAJOR, KeyMode::Major), (MINOR, KeyMode::Minor)].map(|(template, mode)| {
                (
                    cosine(&profile, &rotated(&template, tonic)),
                    MusicalKey {
                        tonic: PitchClass::from_semitones(tonic as i32),
                        mode,
                    },
                )
            })
        })
        .collect();
    scores.sort_by(|left, right| right.0.total_cmp(&left.0));
    let (best, second) = (scores[0], scores[1]);
    Some(KeyEstimate {
        key: best.1,
        confidence: Confidence::new(if second.0 > 1e-12 {
            best.0 / second.0
        } else {
            1.0
        }),
    })
}

/// Pitch-class distribution of one frame (sums to 1) and the frame's weight in the whole.
fn frame_chroma(peaks: &[Peak], tuning: TuningHz) -> ([f64; PITCH_CLASSES], f64) {
    let mut chroma = [0.0; PITCH_CLASSES];
    let mut strength = 0.0;
    for peak in peaks.iter().filter(|peak| peak.hertz > 0.0) {
        // Pitch class counted from C: A4 is MIDI 69, which is pitch class 9.
        let semitones = 69.0 + 12.0 * (peak.hertz / tuning.hertz()).log2();
        let position = semitones.rem_euclid(PITCH_CLASSES as f64);
        let lower = position.floor();
        let fraction = position - lower;
        let lower = lower as usize % PITCH_CLASSES;
        chroma[lower] += peak.weight * (1.0 - fraction);
        chroma[(lower + 1) % PITCH_CLASSES] += peak.weight * fraction;
        strength += peak.weight;
    }
    if strength > 0.0 {
        for value in &mut chroma {
            *value /= strength;
        }
    }
    (chroma, strength)
}

/// The template moved so that its tonic sits on `tonic`.
fn rotated(template: &[f64; PITCH_CLASSES], tonic: usize) -> [f64; PITCH_CLASSES] {
    std::array::from_fn(|index| template[(index + PITCH_CLASSES - tonic) % PITCH_CLASSES])
}

fn cosine(left: &[f64; PITCH_CLASSES], right: &[f64; PITCH_CLASSES]) -> f64 {
    let dot: f64 = left.iter().zip(right).map(|(a, b)| a * b).sum();
    let norms = left.iter().map(|a| a * a).sum::<f64>().sqrt()
        * right.iter().map(|b| b * b).sum::<f64>().sqrt();
    if norms < 1e-12 { 0.0 } else { dot / norms }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Peaks of the given semitone offsets from C4 (261.63 Hz at A4 = 440).
    fn frame(semitones: &[i32], tuning: f64) -> Vec<Peak> {
        semitones
            .iter()
            .map(|offset| Peak {
                hertz: tuning * 2.0_f64.powf((f64::from(*offset) - 9.0) / 12.0),
                weight: 1.0,
            })
            .collect()
    }

    fn estimate_of(chords: &[&[i32]], tuning: f64) -> MusicalKey {
        let frames: Vec<_> = chords.iter().map(|chord| frame(chord, tuning)).collect();
        estimate(&frames, TuningHz::new(tuning).unwrap())
            .unwrap()
            .key
    }

    #[test]
    fn recognises_c_major_from_its_chords() {
        let key = estimate_of(
            &[
                &[0, 4, 7],
                &[5, 9, 12],
                &[7, 11, 14],
                &[0, 4, 7],
                &[0, 4, 7],
            ],
            440.0,
        );
        assert_eq!((key.tonic.index(), key.mode), (0, KeyMode::Major));
    }

    #[test]
    fn recognises_d_minor_from_its_chords() {
        let key = estimate_of(
            &[
                &[2, 5, 9],
                &[7, 10, 14],
                &[9, 13, 16],
                &[2, 5, 9],
                &[2, 5, 9],
            ],
            440.0,
        );
        assert_eq!((key.tonic.index(), key.mode), (2, KeyMode::Minor));
    }

    #[test]
    fn follows_the_given_tuning() {
        let key = estimate_of(
            &[&[7, 11, 14], &[7, 11, 14], &[0, 4, 7], &[7, 11, 14]],
            432.0,
        );
        assert_eq!(key.tonic.index(), 7);
    }

    #[test]
    fn no_tonal_content_gives_no_key() {
        assert!(estimate(&[Vec::new(), Vec::new()], TuningHz::CONCERT_PITCH).is_none());
    }
}
