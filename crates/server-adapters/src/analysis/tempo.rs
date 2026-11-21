//! Tempo from the onset strength: comb of autocorrelation values over candidate beat periods.

use veltrunox_core::{Bpm, Confidence};

const SEARCH_BPM: (f64, f64) = (60.0, 200.0);
/// Estimates outside this range are doubled or halved into it.
const FOLD_BPM: (f64, f64) = (80.0, 180.0);
const STEP_BPM: f64 = 0.1;
/// Period multiples that must carry a beat for a candidate to score.
const COMB_TEETH: usize = 4;
/// A rival must be this far (relative) from the best tempo to count for the confidence.
const RIVAL_DISTANCE: f64 = 0.05;
/// Peaks scoring at least this share of the best one compete on tempo, not on score.
const NEAR_BEST: f64 = 0.95;
/// A peak is the highest score within this relative distance.
const PEAK_WINDOW: f64 = 0.03;
/// Shorter material has too few beats for a stable estimate.
const MIN_SECONDS: f64 = 6.0;

pub struct TempoEstimate {
    pub tempo: Bpm,
    pub confidence: Confidence,
}

/// `None` when the material is too short or has no rhythmic structure.
pub fn estimate(onset_strength: &[f64], hop_seconds: f64) -> Option<TempoEstimate> {
    if (onset_strength.len() as f64) * hop_seconds < MIN_SECONDS {
        return None;
    }
    let longest_lag = ((60.0 / SEARCH_BPM.0) * COMB_TEETH as f64 / hop_seconds).ceil() as usize + 1;
    let correlation = autocorrelation(onset_strength, longest_lag);
    if correlation.first().is_none_or(|zero| *zero < 1e-12) {
        return None;
    }

    let candidates: Vec<(f64, f64)> = (0..)
        .map(|step| SEARCH_BPM.0 + f64::from(step) * STEP_BPM)
        .take_while(|bpm| *bpm <= SEARCH_BPM.1)
        .map(|bpm| (bpm, comb_score(&correlation, 60.0 / bpm / hop_seconds)))
        .collect();
    let (best_bpm, best_score) = fastest_strong_peak(&candidates)?;
    let rival = candidates
        .iter()
        .filter(|(bpm, _)| (bpm / best_bpm - 1.0).abs() >= RIVAL_DISTANCE)
        .map(|(_, score)| *score)
        .fold(0.0, f64::max);

    Some(TempoEstimate {
        tempo: Bpm::new(fold(best_bpm)).ok()?,
        confidence: Confidence::new(if rival > 0.0 { best_score / rival } else { 1.0 }),
    })
}

/// A regular pulse also scores at every slower tempo whose beats it contains (a third of
/// the tempo hits every third beat), so near-equal peaks are common. The shortest period
/// that scores almost as well as the best one is the real one.
fn fastest_strong_peak(candidates: &[(f64, f64)]) -> Option<(f64, f64)> {
    let best = candidates
        .iter()
        .map(|candidate| candidate.1)
        .fold(0.0, f64::max);
    if best <= 0.0 {
        return None;
    }
    candidates
        .iter()
        .copied()
        .filter(|(bpm, score)| {
            *score >= NEAR_BEST * best
                && candidates
                    .iter()
                    .filter(|(other, _)| (other / bpm - 1.0).abs() <= PEAK_WINDOW)
                    .all(|(_, other_score)| other_score <= score)
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
}

/// Mean product of the signal with itself shifted by each whole lag in `0..=longest`.
fn autocorrelation(values: &[f64], longest: usize) -> Vec<f64> {
    (0..=longest.min(values.len().saturating_sub(1)))
        .map(|lag| {
            let overlap = values.len() - lag;
            values[lag..]
                .iter()
                .zip(values)
                .map(|(shifted, original)| shifted * original)
                .sum::<f64>()
                / overlap as f64
        })
        .collect()
}

/// Sum of the correlation at the first `COMB_TEETH` multiples of a period given in frames.
fn comb_score(correlation: &[f64], period_frames: f64) -> f64 {
    (1..=COMB_TEETH)
        .map(|tooth| interpolated(correlation, period_frames * tooth as f64))
        .sum()
}

fn interpolated(values: &[f64], position: f64) -> f64 {
    let lower = position.floor() as usize;
    let fraction = position - position.floor();
    match (values.get(lower), values.get(lower + 1)) {
        (Some(low), Some(high)) => low * (1.0 - fraction) + high * fraction,
        _ => 0.0,
    }
}

fn fold(mut bpm: f64) -> f64 {
    while bpm < FOLD_BPM.0 {
        bpm *= 2.0;
    }
    while bpm > FOLD_BPM.1 {
        bpm /= 2.0;
    }
    bpm
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOP_SECONDS: f64 = 1024.0 / 44_100.0;

    /// Onset strength with a short decaying bump at every beat.
    fn pulses(bpm: f64, seconds: f64) -> Vec<f64> {
        let frames = (seconds / HOP_SECONDS) as usize;
        (0..frames)
            .map(|frame| {
                let beats = frame as f64 * HOP_SECONDS * bpm / 60.0;
                let distance_seconds = (beats - beats.round()).abs() * 60.0 / bpm;
                (-distance_seconds / 0.03).exp()
            })
            .collect()
    }

    #[test]
    fn finds_the_tempo_within_a_third_of_a_bpm() {
        for bpm in [97.3, 120.0, 143.7, 88.0] {
            let found = estimate(&pulses(bpm, 40.0), HOP_SECONDS)
                .unwrap()
                .tempo
                .value();
            assert!((found - bpm).abs() < 0.35, "{bpm}: found {found}");
        }
    }

    #[test]
    fn slow_and_fast_tempi_fold_into_the_usual_range() {
        let found = estimate(&pulses(65.0, 40.0), HOP_SECONDS)
            .unwrap()
            .tempo
            .value();
        assert!((found - 130.0).abs() < 0.5, "found {found}");
        let found = estimate(&pulses(190.0, 40.0), HOP_SECONDS)
            .unwrap()
            .tempo
            .value();
        // Fast tempi are resolved less finely: a period is only about nine onset frames.
        assert!((found - 95.0).abs() < 1.0, "found {found}");
    }

    #[test]
    fn short_or_flat_material_gives_no_tempo() {
        assert!(estimate(&pulses(120.0, 3.0), HOP_SECONDS).is_none());
        assert!(estimate(&vec![0.0; 2_000], HOP_SECONDS).is_none());
    }
}
