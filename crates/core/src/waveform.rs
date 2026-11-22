//! Waveform overview of a recording: minimum and maximum amplitude per block,
//! in levels from coarse to fine (ADR-007).

/// Source frames per block at the finest level.
pub const FINEST_BLOCK_FRAMES: u32 = 1024;
/// Merging stops once a level has at most this many blocks.
const COARSEST_MAX_BLOCKS: usize = 256;
const SCALE: f32 = 127.0;

/// Lowest and highest amplitude of one block, scaled from -1.0..=1.0 to -127..=127.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Peak {
    pub min: i8,
    pub max: i8,
}

impl Peak {
    fn merged(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// One resolution of the waveform; block `n` covers source frames
/// `n * frames_per_block ..` (the last block may be shorter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformLevel {
    pub frames_per_block: u32,
    pub peaks: Vec<Peak>,
}

/// All levels of one recording, coarsest first; each level has twice the
/// frames per block of the next finer one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waveform {
    pub sample_rate: u32,
    pub levels: Vec<WaveformLevel>,
}

impl Waveform {
    /// Builds all levels from the mono mix of a recording. No samples, no levels.
    pub fn from_mono(samples: &[f32], sample_rate: u32) -> Self {
        let mut levels = Vec::new();
        if !samples.is_empty() {
            let mut level = WaveformLevel {
                frames_per_block: FINEST_BLOCK_FRAMES,
                peaks: samples
                    .chunks(FINEST_BLOCK_FRAMES as usize)
                    .map(peak_of)
                    .collect(),
            };
            while level.peaks.len() > COARSEST_MAX_BLOCKS {
                let coarser = coarsen(&level);
                levels.push(std::mem::replace(&mut level, coarser));
            }
            levels.push(level);
            levels.reverse();
        }
        Self {
            sample_rate,
            levels,
        }
    }
}

fn peak_of(block: &[f32]) -> Peak {
    let (min, max) = block.iter().fold((f32::MAX, f32::MIN), |(min, max), &s| {
        (min.min(s), max.max(s))
    });
    Peak {
        min: quantise(min),
        max: quantise(max),
    }
}

fn quantise(sample: f32) -> i8 {
    // Clamped first, so the cast cannot overflow; NaN becomes 0.
    (sample.clamp(-1.0, 1.0) * SCALE).round() as i8
}

fn coarsen(level: &WaveformLevel) -> WaveformLevel {
    WaveformLevel {
        frames_per_block: level.frames_per_block * 2,
        peaks: level
            .peaks
            .chunks(2)
            .map(|pair| pair.iter().copied().reduce(Peak::merged).unwrap_or(pair[0]))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_samples_give_no_levels() {
        assert!(Waveform::from_mono(&[], 44_100).levels.is_empty());
    }

    #[test]
    fn a_short_recording_has_one_level_with_scaled_peaks() {
        let mut samples = vec![0.0; 1024];
        samples[10] = 0.5;
        samples[20] = -1.0;
        samples.push(0.25);
        let waveform = Waveform::from_mono(&samples, 8_000);
        assert_eq!(waveform.sample_rate, 8_000);
        assert_eq!(waveform.levels.len(), 1);
        assert_eq!(
            waveform.levels[0].peaks,
            [Peak { min: -127, max: 64 }, Peak { min: 32, max: 32 }]
        );
    }

    #[test]
    fn levels_run_from_coarse_to_fine_and_halve_the_blocks() {
        // 1000 finest blocks: 1000, 500, 250.
        let samples = vec![0.1; 1024 * 1000];
        let levels = Waveform::from_mono(&samples, 44_100).levels;
        let shape: Vec<_> = levels
            .iter()
            .map(|level| (level.frames_per_block, level.peaks.len()))
            .collect();
        assert_eq!(shape, [(4096, 250), (2048, 500), (1024, 1000)]);
    }

    #[test]
    fn a_coarse_block_holds_the_extremes_of_the_finer_blocks_it_covers() {
        let mut samples = vec![0.0; 1024 * 300];
        samples[0] = -0.5;
        samples[1024 * 299] = 1.0;
        let levels = Waveform::from_mono(&samples, 44_100).levels;
        let coarse = &levels[0];
        assert_eq!(coarse.frames_per_block, 2048);
        assert_eq!(coarse.peaks[0], Peak { min: -64, max: 0 });
        // 300 blocks pair up to 150; the last pair is blocks 298 and 299.
        assert_eq!(coarse.peaks[149], Peak { min: 0, max: 127 });
    }

    #[test]
    fn an_odd_block_count_keeps_the_unpaired_last_block() {
        let mut samples = vec![0.0; 1024 * 301];
        samples[1024 * 300] = 0.5;
        let coarse = &Waveform::from_mono(&samples, 44_100).levels[0];
        assert_eq!(coarse.peaks.len(), 151);
        assert_eq!(coarse.peaks[150], Peak { min: 0, max: 64 });
    }

    #[test]
    fn out_of_range_and_invalid_samples_stay_inside_the_scale() {
        let waveform = Waveform::from_mono(&[3.0, -3.0, f32::NAN], 8_000);
        assert_eq!(
            waveform.levels[0].peaks[0],
            Peak {
                min: -127,
                max: 127
            }
        );
    }
}
