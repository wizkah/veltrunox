//! Reference tuning: the offset from concert pitch that makes the peaks fall on semitones.

use super::spectrum::Peak;
use veltrunox_core::{Confidence, TuningHz};

const CONCERT_PITCH_HZ: f64 = 440.0;
/// Too few peaks give no reliable histogram; concert pitch is assumed then.
const MIN_PEAKS: usize = 300;
/// The search covers one semitone around concert pitch: a whole-semitone offset
/// is a transposition of the recording, not a tuning (it shows up in the key).
const CENTS_PER_SEMITONE: usize = 100;
/// A rival candidate must be this far from the best to count for the confidence.
const RIVAL_DISTANCE_CENTS: usize = 10;
/// How strongly a residual of 0, 1, 2 and 3 or more cents counts as "on the semitone".
const KERNEL: [f64; 4] = [1.0, 0.8, 0.5, 0.25];
const FINE_STEPS_PER_CENT: usize = 10;

pub struct TuningEstimate {
    pub tuning: TuningHz,
    pub confidence: Confidence,
}

/// `None` when there are too few peaks to decide.
pub fn estimate(peaks: &[Peak]) -> Option<TuningEstimate> {
    if peaks.len() < MIN_PEAKS {
        return None;
    }
    // Semitone position of each peak relative to concert pitch, computed once.
    let positions: Vec<(f64, f64)> = peaks
        .iter()
        .map(|peak| (12.0 * (peak.hertz / CONCERT_PITCH_HZ).log2(), peak.weight))
        .collect();
    let coarse: Vec<f64> = (0..CENTS_PER_SEMITONE)
        .map(|cents| score(&positions, cents_from_index(cents as f64)))
        .collect();
    let best = coarse
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, _)| index)?;
    let rival = (0..CENTS_PER_SEMITONE)
        .filter(|index| circular_distance(*index, best) >= RIVAL_DISTANCE_CENTS)
        .map(|index| coarse[index])
        .fold(0.0, f64::max);

    let centre = cents_from_index(best as f64);
    let fine = (0..=2 * FINE_STEPS_PER_CENT)
        .map(|step| centre - 1.0 + step as f64 / FINE_STEPS_PER_CENT as f64)
        .max_by(|left, right| score(&positions, *left).total_cmp(&score(&positions, *right)))?;

    Some(TuningEstimate {
        tuning: TuningHz::new(CONCERT_PITCH_HZ * 2.0_f64.powf(fine / 1200.0)).ok()?,
        confidence: Confidence::new(if rival > 0.0 {
            coarse[best] / rival
        } else {
            1.0
        }),
    })
}

/// Index 0..100 to a cent offset in [-50, 50).
fn cents_from_index(index: f64) -> f64 {
    index - 50.0
}

fn circular_distance(left: usize, right: usize) -> usize {
    let distance = left.abs_diff(right);
    distance.min(CENTS_PER_SEMITONE - distance)
}

/// Weight of the peaks that lie within a few cents of a semitone when A4 is
/// `offset_cents` away from concert pitch.
fn score(positions: &[(f64, f64)], offset_cents: f64) -> f64 {
    let offset = offset_cents / 100.0;
    positions
        .iter()
        .map(|(semitones, weight)| {
            let shifted = semitones - offset;
            let residual_cents = ((shifted - shifted.round()) * 100.0).abs();
            let bin = residual_cents.round() as usize;
            KERNEL.get(bin).map_or(0.0, |kernel| kernel * weight)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peaks_at(tuning: f64, count: usize) -> Vec<Peak> {
        // Notes of a major scale over three octaves.
        let steps = [0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21, 23];
        (0..count)
            .map(|index| Peak {
                hertz: tuning * 2.0_f64.powf((f64::from(steps[index % steps.len()]) - 12.0) / 12.0),
                weight: 1.0,
            })
            .collect()
    }

    #[test]
    fn too_few_peaks_decide_nothing() {
        assert!(estimate(&peaks_at(440.0, 100)).is_none());
    }

    #[test]
    fn finds_concert_pitch_and_offsets_within_half_a_cent() {
        for tuning in [440.0, 432.0, 447.5, 428.0] {
            let found = estimate(&peaks_at(tuning, 500)).unwrap().tuning.hertz();
            let cents = 1200.0 * (found / tuning).log2();
            // A whole-semitone ambiguity folds into the same residual.
            let folded = (cents + 50.0).rem_euclid(100.0) - 50.0;
            assert!(folded.abs() < 0.5, "tuning {tuning}: found {found}");
        }
    }
}
