//! Short-time spectrum of a mono signal: tonal peaks per frame and the onset strength.

use rustfft::{FftPlanner, num_complex::Complex};

pub const FFT_SIZE: usize = 4096;
pub const HOP: usize = 1024;
const PEAK_RANGE_HZ: (f64, f64) = (80.0, 4_000.0);
const PEAKS_PER_FRAME: usize = 15;
/// A frame is tonal when its strongest bin is this many times the mean magnitude.
const TONAL_CREST: f32 = 6.0;
const ONSET_SMOOTHING: usize = 5;

/// A spectral peak: frequency in hertz and a weight that tames outliers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Peak {
    pub hertz: f64,
    pub weight: f64,
}

/// What the analyses need from the spectrum of a whole recording.
pub struct Frames {
    /// Tonal peaks of each frame; frames without a clear tone have none.
    pub peaks: Vec<Vec<Peak>>,
    /// Onset strength per frame (smoothed spectral flux, z-scored, rectified).
    pub onset_strength: Vec<f64>,
    /// Time between two frames.
    pub hop_seconds: f64,
}

pub fn frames(mono: &[f32], sample_rate: u32) -> Frames {
    let window = hann_window();
    let fft = FftPlanner::<f32>::new().plan_fft_forward(FFT_SIZE);
    let mut buffer = vec![Complex::new(0.0_f32, 0.0); FFT_SIZE];
    let mut magnitudes = vec![0.0_f32; FFT_SIZE / 2 + 1];
    let mut previous_log = vec![0.0_f32; FFT_SIZE / 2 + 1];
    let mut peaks = Vec::new();
    let mut flux = Vec::new();

    for (index, start) in (0..=mono.len().saturating_sub(FFT_SIZE))
        .step_by(HOP)
        .enumerate()
    {
        let Some(chunk) = mono.get(start..start + FFT_SIZE) else {
            break;
        };
        for ((slot, sample), weight) in buffer.iter_mut().zip(chunk).zip(&window) {
            *slot = Complex::new(sample * weight, 0.0);
        }
        fft.process(&mut buffer);
        for (magnitude, bin) in magnitudes.iter_mut().zip(&buffer) {
            *magnitude = bin.norm();
        }
        peaks.push(tonal_peaks(&magnitudes, sample_rate));
        flux.push(if index == 0 {
            0.0
        } else {
            flux_against(&magnitudes, &previous_log)
        });
        for (log, magnitude) in previous_log.iter_mut().zip(&magnitudes) {
            *log = magnitude.ln_1p();
        }
    }

    Frames {
        peaks,
        onset_strength: normalise_onsets(&flux),
        hop_seconds: HOP as f64 / f64::from(sample_rate),
    }
}

fn hann_window() -> Vec<f32> {
    (0..FFT_SIZE)
        .map(|index| {
            0.5 - 0.5 * (2.0 * std::f32::consts::PI * index as f32 / (FFT_SIZE - 1) as f32).cos()
        })
        .collect()
}

fn tonal_peaks(magnitudes: &[f32], sample_rate: u32) -> Vec<Peak> {
    let mean = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32 + 1e-12;
    let max = magnitudes.iter().copied().fold(0.0, f32::max);
    if max / mean < TONAL_CREST {
        return Vec::new();
    }
    let bin_hertz = f64::from(sample_rate) / FFT_SIZE as f64;
    let first = ((PEAK_RANGE_HZ.0 / bin_hertz).ceil() as usize).max(2);
    let last = ((PEAK_RANGE_HZ.1 / bin_hertz).floor() as usize).min(magnitudes.len() - 3);
    let mut peaks: Vec<Peak> = (first..=last.max(first))
        .filter_map(|bin| peak_at(magnitudes, bin, bin_hertz))
        .collect();
    peaks.sort_by(|left, right| right.weight.total_cmp(&left.weight));
    peaks.truncate(PEAKS_PER_FRAME);
    peaks
}

/// A local maximum at `bin`, placed between bins by a parabola through its neighbours.
fn peak_at(magnitudes: &[f32], bin: usize, bin_hertz: f64) -> Option<Peak> {
    let (before, centre, after) = (
        f64::from(*magnitudes.get(bin - 1)?),
        f64::from(*magnitudes.get(bin)?),
        f64::from(*magnitudes.get(bin + 1)?),
    );
    if centre <= before || centre <= after {
        return None;
    }
    let curvature = before - 2.0 * centre + after;
    let offset = if curvature.abs() < 1e-12 {
        0.0
    } else {
        0.5 * (before - after) / curvature
    };
    Some(Peak {
        hertz: (bin as f64 + offset) * bin_hertz,
        weight: centre.ln_1p(),
    })
}

/// Sum of the log-magnitude increases since the previous frame (without the DC bin).
fn flux_against(magnitudes: &[f32], previous_log: &[f32]) -> f64 {
    magnitudes
        .iter()
        .zip(previous_log)
        .skip(1)
        .map(|(magnitude, previous)| f64::from((magnitude.ln_1p() - previous).max(0.0)))
        .sum()
}

fn normalise_onsets(flux: &[f64]) -> Vec<f64> {
    let smoothed = moving_average(flux, ONSET_SMOOTHING);
    let count = smoothed.len().max(1) as f64;
    let mean = smoothed.iter().sum::<f64>() / count;
    let deviation = (smoothed.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count).sqrt();
    if deviation < 1e-12 {
        return vec![0.0; smoothed.len()];
    }
    smoothed
        .iter()
        .map(|value| ((value - mean) / deviation).max(0.0))
        .collect()
}

fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    let half = window / 2;
    (0..values.len())
        .map(|index| {
            let slice = &values[index.saturating_sub(half)..(index + half + 1).min(values.len())];
            slice.iter().sum::<f64>() / slice.len() as f64
        })
        .collect()
}
