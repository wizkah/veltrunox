//! The render engine: turns the shared playback settings into audio buffers.
//! No device, no clock and no allocation while rendering (apart from a first
//! growth of the scratch buffer), so it is tested without hardware.

use signalsmith_stretch::Stretch;
use std::ops::Range;
use std::sync::Arc;
use veltrunox_core::{MAX_STEMS, StemMix};

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
/// Fade-out before the stop position, in milliseconds of the recording, so the cut does not click.
const STOP_FADE_MS: u64 = 5;
/// Pieces sounding at once: the newest and the one it fades out.
const MAX_GRAINS: usize = 3;

/// A change of the mix reaches its new gains within this time, so it does not click.
const MIX_RAMP_MS: u64 = 15;
/// The sources a frame is summed from: the stems, then the original recording.
const SOURCES: usize = MAX_STEMS + 1;
const ORIGINAL: usize = MAX_STEMS;

/// Plays one recording: speed and pitch independently, with a loop that wraps
/// and a stop position that ends playback, both at sample level.
pub struct Engine {
    pcm: Arc<Pcm>,
    /// Tracks of the recording at its sample rate; they replace it while the mix says so.
    stems: Vec<Arc<Pcm>>,
    /// Gain of each source now and where it is moving to.
    gains: [f32; SOURCES],
    targets: [f32; SOURCES],
    stretch: Stretch,
    /// Device sample rate; a difference to the recording's rate is absorbed here.
    output_rate: u32,
    running: bool,
    speed: f64,
    semitones: f64,
    loop_frames: Option<Range<usize>>,
    /// The source frame at which playback ends, if the engine reaches it.
    stop_frame: Option<usize>,
    /// Set once playback has ended at the stop position.
    ended: Option<Ended>,
    /// Next source frame to feed to the stretcher.
    read: usize,
    /// Fraction of an input frame not yet consumed.
    carry: f64,
    input: Vec<f32>,
    /// The pieces of a dragged position handle that sound now.
    grains: Vec<Grain>,
}

/// Playback that ended at the stop position; what the stretcher still holds is played out.
struct Ended {
    frame: usize,
    /// Output frames until the stretcher has given back everything before the stop.
    tail_left: usize,
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
            stems: Vec::new(),
            gains: original_only(),
            targets: original_only(),
            stretch,
            output_rate,
            running: false,
            speed: 1.0,
            semitones: 0.0,
            loop_frames: None,
            stop_frame: None,
            ended: None,
            read: 0,
            carry: 0.0,
            input: Vec::with_capacity(SCRATCH_FRAMES * CHANNELS),
            grains: Vec::with_capacity(MAX_GRAINS),
        };
        engine.apply_pitch();
        engine
    }

    /// Gives the engine the stems of the recording. Those beyond `MAX_STEMS` are ignored;
    /// stems of another sample rate would play at the wrong speed and are dropped.
    pub fn with_stems(mut self, mut stems: Vec<Arc<Pcm>>) -> Self {
        let rate = self.pcm.sample_rate();
        if stems.iter().any(|stem| stem.sample_rate() != rate) {
            stems.clear();
        }
        stems.truncate(MAX_STEMS);
        self.stems = stems;
        self
    }

    /// Which sources sound, and how loud. Without stems the original plays whatever the
    /// mix says. While silent the change applies at once, otherwise it moves there.
    pub fn set_mix(&mut self, mix: &StemMix) {
        self.targets = if mix.enabled && !self.stems.is_empty() {
            let mut targets = [0.0; SOURCES];
            for (index, target) in targets.iter_mut().enumerate().take(self.stems.len()) {
                *target = mix.gain(index);
            }
            targets
        } else {
            original_only()
        };
        if !self.running && self.grains.is_empty() {
            self.gains = self.targets;
        }
    }

    /// The mixed frame at `index` with the gains as they are now.
    fn frame(&self, index: usize) -> [f32; CHANNELS] {
        let mut mixed = [0.0; CHANNELS];
        let sources = self
            .stems
            .iter()
            .enumerate()
            .chain(std::iter::once((ORIGINAL, &self.pcm)));
        for (source, pcm) in sources {
            let gain = self.gains[source];
            if gain == 0.0 {
                continue;
            }
            let at = index * CHANNELS;
            if let Some(samples) = pcm.samples.get(at..at + CHANNELS) {
                for (sum, sample) in mixed.iter_mut().zip(samples) {
                    *sum += sample * gain;
                }
            }
        }
        mixed
    }

    /// One step of the gains towards their targets.
    fn ramp_mix(&mut self, step: f32) {
        for (gain, target) in self.gains.iter_mut().zip(self.targets) {
            *gain += (target - *gain).clamp(-step, step);
        }
    }

    /// Starts or stops playing; ends the pieces of scrubbing.
    pub fn set_running(&mut self, running: bool) {
        self.grains.clear();
        self.running = running;
        // Playing on after the stop continues from the stop position.
        if running && let Some(ended) = self.ended.take() {
            self.seek_frame(ended.frame);
        }
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

    /// The position in milliseconds at which playback ends; `None` plays on. A stop that the
    /// playing position has already passed is not reached. The sound before it plays out, the
    /// last few milliseconds fading, and nothing after it is heard.
    pub fn set_stop_ms(&mut self, position: Option<u64>) {
        self.stop_frame = position.map(|position| self.frame_at(position));
    }

    /// Jumps to a position in milliseconds; the stretcher is primed with the
    /// audio before it so the first output is not a fade-in.
    pub fn seek_ms(&mut self, position: u64) {
        let target = self.frame_at(position).min(self.pcm.frames());
        // Pausing on the stop position must not cut off the sound still being played out.
        if self
            .ended
            .as_ref()
            .is_some_and(|ended| ended.frame == target)
        {
            return;
        }
        self.ended = None;
        self.seek_frame(target);
    }

    fn seek_frame(&mut self, target: usize) {
        self.read = target;
        self.carry = 0.0;
        self.stretch.reset();
        let preroll = self.stretch.input_latency().min(target);
        let from = (target - preroll) * CHANNELS;
        self.gains = self.targets;
        self.input.clear();
        for index in from / CHANNELS..target {
            let frame = self.frame(index);
            self.input.extend(frame);
        }
        self.stretch.seek(&self.input, self.speed);
    }

    /// The source position in milliseconds: the frames handed to the stretcher.
    /// The sound leaves the stretcher up to its input latency later.
    pub fn position_ms(&self) -> u64 {
        self.read as u64 * 1_000 / u64::from(self.pcm.sample_rate())
    }

    /// True when a recording without loop has been played to its end.
    pub fn finished(&self) -> bool {
        self.ended.is_none() && self.loop_frames.is_none() && self.read >= self.pcm.frames()
    }

    /// True while the sound before the stop position is still being played out.
    fn playing_out(&self) -> bool {
        self.ended.as_ref().is_some_and(|ended| ended.tail_left > 0)
    }

    /// True once playback has ended at the stop position.
    pub fn stopped(&self) -> bool {
        self.ended.is_some()
    }

    /// Fills `output` (interleaved stereo at the output rate).
    pub fn render(&mut self, output: &mut [f32]) {
        if !self.grains.is_empty() {
            self.render_grains(output);
            return;
        }
        if !(self.running || self.playing_out()) || self.finished() {
            output.fill(0.0);
            return;
        }
        let rate_ratio = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let wanted = (output.len() / CHANNELS) as f64 * self.speed * rate_ratio + self.carry;
        let frames = wanted.floor() as usize;
        self.carry = wanted - frames as f64;
        self.fill_input(frames);
        self.stretch.process(&self.input, &mut *output);
        if let Some(ended) = &mut self.ended {
            ended.tail_left = ended.tail_left.saturating_sub(output.len() / CHANNELS);
        }
    }

    /// Mixes the sounding pieces of scrubbing into `output`.
    fn render_grains(&mut self, output: &mut [f32]) {
        output.fill(0.0);
        let step = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let fade_in = self.frames_out(GRAIN_FADE_IN_MS).max(1) as f32;
        let fade_out = self.frames_out(GRAIN_FADE_OUT_MS).max(1) as f32;
        let total = self.pcm.frames();
        // A scrubbed piece is short; it takes the mix as it is.
        self.gains = self.targets;
        let mut grains = std::mem::take(&mut self.grains);
        for grain in &mut grains {
            for out in output.as_chunks_mut::<CHANNELS>().0 {
                if grain.left == 0 {
                    break;
                }
                let source = grain.frame as usize;
                if source < total {
                    let gain = ((grain.played + 1) as f32 / fade_in).min(1.0)
                        * (grain.left as f32 / fade_out).min(1.0);
                    for (sample, mixed) in out.iter_mut().zip(self.frame(source)) {
                        *sample += mixed * gain;
                    }
                }
                grain.frame += step;
                grain.played += 1;
                grain.left -= 1;
            }
        }
        grains.retain(|grain| grain.left > 0);
        self.grains = grains;
    }

    /// Source frames for the next `frames` input frames; a running loop wraps at its end and
    /// the stop position ends the input, after a fade, in silence.
    fn fill_input(&mut self, frames: usize) {
        self.input.clear();
        let total = self.pcm.frames();
        let fade = self.frame_at(STOP_FADE_MS).max(1);
        let mix_step = 1.0 / self.frame_at(MIX_RAMP_MS).max(1) as f32;
        for _ in 0..frames {
            if self.ended.is_some() {
                self.input.extend([0.0; CHANNELS]);
                continue;
            }
            // A position that starts beyond the loop plays on; reaching its end wraps.
            if let Some(range) = &self.loop_frames
                && self.read == range.end
            {
                self.read = range.start;
            }
            if self.stop_frame == Some(self.read) {
                self.end_here();
                self.input.extend([0.0; CHANNELS]);
                continue;
            }
            self.ramp_mix(mix_step);
            if self.read < total {
                let gain = match self.stop_frame {
                    Some(stop) if self.read < stop && stop - self.read <= fade => {
                        (stop - self.read) as f32 / (fade + 1) as f32
                    }
                    _ => 1.0,
                };
                let frame = self.frame(self.read);
                self.input.extend(frame.map(|sample| sample * gain));
            } else {
                self.input.extend([0.0; CHANNELS]);
            }
            self.read += 1;
        }
    }

    /// Playback has reached the stop position: no more input from here on.
    fn end_here(&mut self) {
        let rate_ratio = f64::from(self.pcm.sample_rate()) / f64::from(self.output_rate);
        let before = self.stretch.input_latency() as f64 / (self.speed * rate_ratio);
        let tail = before.ceil() as usize + self.stretch.output_latency() + self.frames_out(10);
        self.ended = Some(Ended {
            frame: self.read,
            tail_left: tail,
        });
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

/// The gains that play the original recording and nothing else.
fn original_only() -> [f32; SOURCES] {
    let mut gains = [0.0; SOURCES];
    gains[ORIGINAL] = 1.0;
    gains
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

    fn stems_of(engine: Engine) -> Engine {
        engine.with_stems(vec![sine(220.0, 4.0), sine(880.0, 4.0)])
    }

    fn mix(change: impl FnOnce(&mut StemMix)) -> StemMix {
        let mut mix = StemMix {
            enabled: true,
            ..StemMix::default()
        };
        change(&mut mix);
        mix
    }

    #[test]
    fn the_mix_chooses_what_is_heard() {
        let heard = |mix: StemMix| {
            let mut engine = stems_of(Engine::new(sine(440.0, 4.0), RATE));
            engine.set_mix(&mix);
            engine.set_running(true);
            let rendered = render_seconds(&mut engine, 1.0);
            let settled = &rendered[RATE as usize / 2..];
            (measured_hertz(settled, RATE), rms(settled))
        };
        let (hertz, _) = heard(StemMix::default());
        assert!((hertz - 440.0).abs() < 8.0, "original: {hertz}");
        let (hertz, _) = heard(mix(|mix| mix.channels[0].muted = true));
        assert!((hertz - 880.0).abs() < 12.0, "second stem: {hertz}");
        let (hertz, _) = heard(mix(|mix| mix.channels[0].solo = true));
        assert!((hertz - 220.0).abs() < 8.0, "first stem: {hertz}");
        let (_, level) = heard(mix(|mix| {
            mix.channels[0].muted = true;
            mix.channels[1].muted = true;
        }));
        assert!(level < 0.001, "silence: {level}");
    }

    #[test]
    fn a_mix_changed_while_playing_moves_there_without_a_jump() {
        let mut engine = stems_of(Engine::new(sine(440.0, 4.0), RATE));
        engine.set_running(true);
        render_seconds(&mut engine, 0.5);
        engine.set_mix(&mix(|mix| {
            mix.channels[0].muted = true;
            mix.channels[1].muted = true;
        }));
        let after = render_seconds(&mut engine, 1.0);
        let step = after
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0, f32::max);
        assert!(step < 0.2, "{step}");
        assert!(rms(&after[RATE as usize / 2..]) < 0.001);
    }

    #[test]
    fn stems_of_another_rate_are_not_used() {
        let other = Arc::new(Pcm::new(vec![0.5; 200], 8_000));
        let mut engine = Engine::new(sine(440.0, 1.0), RATE).with_stems(vec![other]);
        engine.set_mix(&mix(|_| {}));
        engine.set_running(true);
        assert!(rms(&render_seconds(&mut engine, 0.5)) > 0.3);
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

    /// Renders until the output has been silent for 20 ms; returns the left channel and the
    /// number of frames up to the last audible one.
    fn render_until_silent(engine: &mut Engine, limit_seconds: f64) -> (Vec<f32>, usize) {
        let rendered = render_seconds(engine, limit_seconds);
        let last = rendered
            .iter()
            .rposition(|sample| sample.abs() > 1e-4)
            .unwrap_or(0);
        (rendered, last)
    }

    #[test]
    fn playback_ends_at_the_stop_position() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_stop_ms(Some(1_500));
        engine.seek_ms(1_000);
        engine.set_running(true);
        let (rendered, last) = render_until_silent(&mut engine, 2.0);
        assert!(engine.stopped());
        assert_eq!(engine.position_ms(), 1_500);
        // 500 ms of material; the stretcher delays it by its latency, nothing is added after.
        let latency = engine.stretch.input_latency() + engine.stretch.output_latency();
        let audible = last as i64 - latency as i64;
        let expected = i64::from(RATE) / 2;
        assert!(
            (audible - expected).abs() < i64::from(RATE) / 100,
            "audible until {last}, expected about {}",
            expected + latency as i64
        );
        // Silent for good afterwards.
        assert!(
            rendered[last + 1..]
                .iter()
                .all(|sample| sample.abs() <= 1e-4)
        );
        assert_eq!(rendered[rendered.len() - 1], 0.0);
    }

    #[test]
    fn the_stop_fades_out_without_a_click() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_stop_ms(Some(1_500));
        engine.seek_ms(1_000);
        engine.set_running(true);
        let (rendered, last) = render_until_silent(&mut engine, 2.0);
        // The level before the end falls smoothly: no step larger than the sine's own slope.
        let steps = rendered[..=last]
            .windows(2)
            .skip(RATE as usize / 4)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0_f32, f32::max);
        assert!(steps < 0.15, "largest step {steps}");
        let tail = &rendered[last.saturating_sub(40)..=last];
        assert!(tail.iter().all(|sample| sample.abs() < 0.2), "ends loud");
    }

    #[test]
    fn the_stop_position_follows_the_speed() {
        for speed in [0.5, 2.0] {
            let mut engine = Engine::new(sine(440.0, 8.0), RATE);
            engine.set_speed(speed);
            engine.set_stop_ms(Some(2_000));
            engine.seek_ms(1_000);
            engine.set_running(true);
            let (_, last) = render_until_silent(&mut engine, 4.0);
            assert_eq!(engine.position_ms(), 2_000, "{speed}x");
            let latency = engine.stretch.input_latency() as f64 / speed
                + engine.stretch.output_latency() as f64;
            let seconds = (last as f64 - latency) / f64::from(RATE);
            let expected = 1.0 / speed;
            assert!(
                (seconds - expected).abs() < 0.15,
                "{speed}x ended after {seconds} s, expected about {expected} s"
            );
        }
    }

    #[test]
    fn a_stop_behind_the_position_is_not_reached() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_stop_ms(Some(500));
        engine.seek_ms(1_000);
        engine.set_running(true);
        let rendered = render_seconds(&mut engine, 1.0);
        assert!(!engine.stopped());
        assert!(rms(&rendered[RATE as usize / 2..]) > 0.5);
    }

    #[test]
    fn pausing_on_the_stop_keeps_the_sound_playing_out_and_play_continues() {
        let mut engine = Engine::new(sine(440.0, 8.0), RATE);
        engine.set_stop_ms(Some(1_200));
        engine.seek_ms(1_000);
        engine.set_running(true);
        render_seconds(&mut engine, 0.3);
        // What the player sends when the server pauses at the region end.
        engine.set_running(false);
        engine.set_stop_ms(None);
        engine.seek_ms(1_200);
        assert!(engine.stopped());
        let (rendered, last) = render_until_silent(&mut engine, 0.3);
        assert!(last > 0, "the tail was cut off");
        assert!(
            rendered[last + 1..]
                .iter()
                .all(|sample| sample.abs() <= 1e-4)
        );
        // Playing on runs past the old stop.
        engine.set_running(true);
        let after = render_seconds(&mut engine, 0.5);
        assert!(!engine.stopped());
        assert!(rms(&after[RATE as usize / 4..]) > 0.5);
        assert!(engine.position_ms() > 1_500);
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
