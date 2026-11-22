//! The render engine: turns the shared playback settings into audio buffers.
//! No device, no clock and no allocation while rendering (apart from a first
//! growth of the scratch buffer), so it is tested without hardware.

use signalsmith_stretch::Stretch;
use std::ops::Range;
use std::sync::Arc;

const CHANNELS: usize = 2;
/// Frames of input the scratch buffer can take without growing.
const SCRATCH_FRAMES: usize = 1 << 15;

/// A decoded recording: interleaved stereo `f32`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcm {
    samples: Vec<f32>,
    sample_rate: u32,
}

impl Pcm {
    /// `samples` is interleaved stereo; a trailing odd sample is dropped.
    pub fn new(mut samples: Vec<f32>, sample_rate: u32) -> Self {
        samples.truncate(samples.len() / CHANNELS * CHANNELS);
        Self {
            samples,
            sample_rate,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / CHANNELS
    }

    /// Both channels averaged into one.
    pub fn mono(&self) -> Vec<f32> {
        self.samples
            .as_chunks::<CHANNELS>()
            .0
            .iter()
            .map(|frame| frame.iter().sum::<f32>() / CHANNELS as f32)
            .collect()
    }

    /// Length in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.frames() as u64 * 1_000 / u64::from(self.sample_rate)
    }
}

/// Length of the piece played at a scrub position, in milliseconds.
const GRAIN_MS: u64 = 120;
/// Fades at the edges of a piece, in milliseconds, so the cuts do not click.
const GRAIN_FADE_IN_MS: u64 = 3;
const GRAIN_FADE_OUT_MS: u64 = 8;
/// Pieces sounding at once: the newest and the one it fades out.
const MAX_GRAINS: usize = 3;

/// Plays one recording: speed and pitch independently, with a loop that wraps
/// at sample level.
pub struct Engine {
    pcm: Arc<Pcm>,
    stretch: Stretch,
    /// Device sample rate; a difference to the recording's rate is absorbed here.
    output_rate: u32,
    running: bool,
    speed: f64,
    semitones: f64,
    loop_frames: Option<Range<usize>>,
    /// Next source frame to feed to the stretcher.
    read: usize,
    /// Fraction of an input frame not yet consumed.
    carry: f64,
    input: Vec<f32>,
    /// The pieces of a dragged position handle that sound now.
    grains: Vec<Grain>,
}

/// A short piece of the recording played as it is, for scrubbing.
struct Grain {
    /// Next source frame to play.
    frame: f64,
    /// Output frames played so far and left.
    played: usize,
    left: usize,
}

impl Engine {
    pub fn new(pcm: Arc<Pcm>, output_rate: u32) -> Self {
        let stretch = Stretch::preset_default(CHANNELS as u32, pcm.sample_rate());
        let mut engine = Self {
            pcm,
            stretch,
            output_rate,
            running: false,
            speed: 1.0,
            semitones: 0.0,
            loop_frames: None,
            read: 0,
            carry: 0.0,
            input: Vec::with_capacity(SCRATCH_FRAMES * CHANNELS),
            grains: Vec::with_capacity(MAX_GRAINS),
        };
        engine.apply_pitch();
        engine
    }

    /// Starts or stops playing; ends the pieces of scrubbing.
    pub fn set_running(&mut self, running: bool) {
        self.grains.clear();
        self.running = running;
    }

    /// Plays a short piece from `position` (milliseconds) as it is recorded, so
    /// dragging through the recording is audible at once. The piece before it
    /// fades out; settings of speed and pitch do not apply to it.
    pub fn scrub_ms(&mut self, position: u64) {
        let fade_out = self.frames_out(GRAIN_FADE_OUT_MS);
        for grain in &mut self.grains {
            grain.left = grain.left.min(fade_out);
        }
        if self.grains.len() == MAX_GRAINS {
            self.grains.remove(0);
        }
        self.grains.push(Grain {
            frame: self.frame_at(position) as f64,
            played: 0,
            left: self.frames_out(GRAIN_MS),
        });
    }

    fn frames_out(&self, millis: u64) -> usize {
        (u64::from(self.output_rate) * millis / 1_000) as usize
    }

    pub fn set_speed(&mut self, factor: f64) {
        self.speed = factor;
    }

    /// Pitch shift in semitones, independent of speed.
    pub fn set_pitch_semitones(&mut self, semitones: f64) {
        self.semitones = semitones;
        self.apply_pitch();
    }

    /// The loop in milliseconds of the recording; `None` plays on to the end.
    pub fn set_loop_ms(&mut self, range: Option<(u64, u64)>) {
        self.loop_frames = range.and_then(|(start, end)| {
            let (start, end) = (
                self.frame_at(start),
                self.frame_at(end).min(self.pcm.frames()),
            );
            (end > start).then_some(start..end)
        });
    }

    /// Jumps to a position in milliseconds; the stretcher is primed with the
    /// audio before it so the first output is not a fade-in.
    pub fn seek_ms(&mut self, position: u64) {
        let target = self.frame_at(position).min(self.pcm.frames());
        self.read = target;
        self.carry = 0.0;
        self.stretch.reset();
        let preroll = self.stretch.input_latency().min(target);
        let from = (target - preroll) * CHANNELS;
        self.stretch
            .seek(&self.pcm.samples[from..target * CHANNELS], self.speed);
    }

    /// The source position in milliseconds: the frames handed to the stretcher.
    /// The sound leaves the stretcher up to its input latency later.
    pub fn position_ms(&self) -> u64 {
        self.read as u64 * 1_000 / u64::from(self.pcm.sample_rate())
    }

    /// True when a recording without loop has been played to its end.
    pub fn finished(&self) -> bool {
        self.loop_frames.is_none() && self.read >= self.pcm.frames()
    }

    /// Fills `output` (interleaved stereo at the output rate).
    pub fn render(&mut self, output: &mut [f32]) {
        if !self.grains.is_empty() {
            self.render_grains(output);
            return;
        }
        if !self.running || self.finished() {
            output.fill(0.0);
            return;
        }
        let rate_ratio = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let wanted = (output.len() / CHANNELS) as f64 * self.speed * rate_ratio + self.carry;
        let frames = wanted.floor() as usize;
        self.carry = wanted - frames as f64;
        self.fill_input(frames);
        self.stretch.process(&self.input, &mut *output);
    }

    /// Mixes the sounding pieces of scrubbing into `output`.
    fn render_grains(&mut self, output: &mut [f32]) {
        output.fill(0.0);
        let step = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let fade_in = self.frames_out(GRAIN_FADE_IN_MS).max(1) as f32;
        let fade_out = self.frames_out(GRAIN_FADE_OUT_MS).max(1) as f32;
        let total = self.pcm.frames();
        for grain in &mut self.grains {
            for out in output.as_chunks_mut::<CHANNELS>().0 {
                if grain.left == 0 {
                    break;
                }
                let source = grain.frame as usize;
                if source < total {
                    let gain = ((grain.played + 1) as f32 / fade_in).min(1.0)
                        * (grain.left as f32 / fade_out).min(1.0);
                    for (channel, sample) in out.iter_mut().enumerate() {
                        *sample += self.pcm.samples[source * CHANNELS + channel] * gain;
                    }
                }
                grain.frame += step;
                grain.played += 1;
                grain.left -= 1;
            }
        }
        self.grains.retain(|grain| grain.left > 0);
    }

    /// Source frames for the next `frames` input frames; a running loop wraps at its end.
    fn fill_input(&mut self, frames: usize) {
        self.input.clear();
        let total = self.pcm.frames();
        for _ in 0..frames {
            // A position that starts beyond the loop plays on; reaching its end wraps.
            if let Some(range) = &self.loop_frames
                && self.read == range.end
            {
                self.read = range.start;
            }
            if self.read < total {
                let at = self.read * CHANNELS;
                self.input
                    .extend_from_slice(&self.pcm.samples[at..at + CHANNELS]);
            } else {
                self.input.extend([0.0; CHANNELS]);
            }
            self.read += 1;
        }
    }

    fn frame_at(&self, millis: u64) -> usize {
        (millis * u64::from(self.pcm.sample_rate()) / 1_000) as usize
    }

    fn apply_pitch(&mut self) {
        // Playing samples of one rate at another changes the pitch; undo that here.
        let rate_ratio = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let factor = 2.0_f64.powf(self.semitones / 12.0) * rate_ratio;
        self.stretch.set_transpose_factor(factor as f32, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 44_100;

    /// Stereo sine of `hertz` for `seconds`.
    fn sine(hertz: f64, seconds: f64) -> Arc<Pcm> {
        let frames = (f64::from(RATE) * seconds) as usize;
        let mut samples = Vec::with_capacity(frames * CHANNELS);
        for frame in 0..frames {
            let value = (std::f64::consts::TAU * hertz * frame as f64 / f64::from(RATE)).sin();
            samples.extend([value as f32; CHANNELS]);
        }
        Arc::new(Pcm::new(samples, RATE))
    }

    /// Renders `seconds` of output in device-sized blocks; left channel only.
    fn render_seconds(engine: &mut Engine, seconds: f64) -> Vec<f32> {
        let mut rendered = Vec::new();
        let mut block = vec![0.0; 512 * CHANNELS];
        for _ in 0..(seconds * f64::from(engine.output_rate) / 512.0) as usize {
            engine.render(&mut block);
            rendered.extend(block.iter().step_by(CHANNELS));
        }
        rendered
    }

    /// Frequency estimated from rising zero crossings.
    fn measured_hertz(samples: &[f32], rate: u32) -> f64 {
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
            .count();
        crossings as f64 * f64::from(rate) / samples.len() as f64
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    }

    #[test]
    fn paused_engine_is_silent_and_stays_put() {
        let mut engine = Engine::new(sine(440.0, 1.0), RATE);
        let rendered = render_seconds(&mut engine, 0.2);
        assert!(rendered.iter().all(|sample| *sample == 0.0));
        assert_eq!(engine.position_ms(), 0);
    }

    #[test]
    fn speed_changes_the_progress_but_not_the_pitch() {
        for speed in [0.5, 1.0, 2.0] {
            let mut engine = Engine::new(sine(440.0, 8.0), RATE);
            engine.set_speed(speed);
            engine.set_running(true);
            let rendered = render_seconds(&mut engine, 2.0);
            let settled = &rendered[RATE as usize / 2..];
            let hertz = measured_hertz(settled, RATE);
            assert!((hertz - 440.0).abs() < 8.0, "{speed}x gave {hertz} Hz");
            let expected = (2_000.0 * speed) as i64;
            assert!(
                (engine.position_ms() as i64 - expected).abs() < 30,
                "{speed}x reached {} ms",
                engine.position_ms()
            );
        }
    }

    #[test]
    fn pitch_shift_moves_the_frequency_but_not_the_progress() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_pitch_semitones(12.0);
        engine.set_running(true);
        let rendered = render_seconds(&mut engine, 2.0);
        let hertz = measured_hertz(&rendered[RATE as usize / 2..], RATE);
        assert!((hertz - 880.0).abs() < 16.0, "got {hertz} Hz");
        assert!((engine.position_ms() as i64 - 2_000).abs() < 30);
    }

    #[test]
    fn a_different_device_rate_keeps_pitch_and_duration() {
        let mut engine = Engine::new(sine(440.0, 8.0), 48_000);
        engine.set_running(true);
        let rendered = render_seconds(&mut engine, 2.0);
        let hertz = measured_hertz(&rendered[24_000..], 48_000);
        assert!((hertz - 440.0).abs() < 8.0, "got {hertz} Hz");
        assert!((engine.position_ms() as i64 - 2_000).abs() < 30);
    }

    #[test]
    fn a_loop_wraps_without_a_gap() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_loop_ms(Some((1_000, 1_500)));
        engine.seek_ms(1_000);
        engine.set_running(true);
        // Several wraps; every 10 ms window keeps its level.
        let rendered = render_seconds(&mut engine, 3.0);
        for window in rendered[RATE as usize / 2..].chunks(RATE as usize / 100) {
            assert!(rms(window) > 0.4, "level dropped to {}", rms(window));
        }
        let position = engine.position_ms();
        assert!((1_000..=1_500).contains(&position), "position {position}");
    }

    #[test]
    fn playing_past_the_end_finishes_and_goes_silent() {
        let mut engine = Engine::new(sine(440.0, 0.5), RATE);
        engine.set_running(true);
        render_seconds(&mut engine, 1.0);
        assert!(engine.finished());
        let after = render_seconds(&mut engine, 0.1);
        assert!(after.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn a_scrub_piece_is_audible_at_once_and_then_silent() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.scrub_ms(4_000);
        let rendered = render_seconds(&mut engine, 0.5);
        let piece = (u64::from(RATE) * GRAIN_MS / 1_000) as usize;
        assert!(rms(&rendered[200..2_000]) > 0.4);
        assert!(rendered[piece + 1..].iter().all(|sample| *sample == 0.0));
        // It starts and ends softly, and plays as recorded.
        assert!(rendered[0].abs() < 0.1 && rendered[piece - 1].abs() < 0.1);
        let hertz = measured_hertz(&rendered[200..piece - 400], RATE);
        assert!((hertz - 440.0).abs() < 8.0, "got {hertz} Hz");
    }

    #[test]
    fn a_new_scrub_position_overlaps_the_old_piece_without_a_gap() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.scrub_ms(1_000);
        render_seconds(&mut engine, 0.05);
        engine.scrub_ms(5_000);
        let rendered = render_seconds(&mut engine, 0.1);
        assert!(rms(&rendered) > 0.4);
    }

    #[test]
    fn stopping_silences_a_scrub_piece_and_playing_goes_on_normally() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.scrub_ms(1_000);
        engine.set_running(false);
        assert!(render_seconds(&mut engine, 0.05).iter().all(|s| *s == 0.0));
        engine.seek_ms(2_000);
        engine.set_running(true);
        assert!(rms(&render_seconds(&mut engine, 0.5)[RATE as usize / 4..]) > 0.4);
    }

    #[test]
    fn seeking_starts_at_the_new_position() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.seek_ms(4_000);
        assert_eq!(engine.position_ms(), 4_000);
        engine.set_running(true);
        let rendered = render_seconds(&mut engine, 0.5);
        assert!(rms(&rendered[RATE as usize / 4..]) > 0.5);
    }
}
