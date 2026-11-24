//! Runs the analyses one after the other on a background thread: they are CPU
//! and memory heavy, and the server may be a Raspberry Pi.

use crate::analysis::{FrameBuilder, analyse_frames, analyse_spectrum};
use crate::audio::{append_mono, decode_blocks};
use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use veltrunox_core::{Millis, RecordingId, Refusal, WaveformBuilder};
use veltrunox_server_app::{AnalysisFailure, AnalysisService, RecordingAnalyzer, RecordingScan};

/// Decodes a file once and derives length, waveform and analysis from it, piece by piece:
/// the recording is never in memory as a whole, which a small Raspberry Pi could not bear.
pub struct FileAnalyzer;

/// The decoding pass, which also builds the waveform and the spectrum, counts for this
/// many percent of the whole; the analyses of the spectrum take the rest.
const PASS_PERCENT: u8 = 60;

impl RecordingAnalyzer for FileAnalyzer {
    fn scan(&self, path: &Path, report: &dyn Fn(u8)) -> Result<RecordingScan, AnalysisFailure> {
        let mut waveform = None;
        let mut frames = None;
        let mut block = Vec::new();
        let mut total_samples = 0u64;
        let sample_rate = decode_blocks(
            path,
            &|percent| report(scaled(percent, 0, PASS_PERCENT)),
            &mut |rate, interleaved, channels| {
                block.clear();
                append_mono(&mut block, interleaved, channels);
                total_samples += block.len() as u64;
                waveform
                    .get_or_insert_with(|| WaveformBuilder::new(rate))
                    .push(&block);
                frames
                    .get_or_insert_with(|| FrameBuilder::new(rate))
                    .push(&block);
            },
        )
        .map_err(|error| AnalysisFailure(error.reason().code))?;
        report(PASS_PERCENT);
        let (Some(waveform), Some(frames)) = (waveform, frames) else {
            return Err(AnalysisFailure("audio_unsupported".to_owned()));
        };
        let analysis = analyse_spectrum(frames).and_then(|frames| {
            analyse_frames(frames, &|percent| {
                report(scaled(percent, PASS_PERCENT, 100 - PASS_PERCENT));
            })
        });
        Ok(RecordingScan {
            length: Millis::new(total_samples * 1_000 / u64::from(sample_rate)),
            waveform: waveform.finish(),
            analysis: analysis.map_err(|error| AnalysisFailure(error.reason().code)),
        })
    }
}

/// `percent` of a step that starts at `from` percent of the whole and spans `span` percent.
fn scaled(percent: u8, from: u8, span: u8) -> u8 {
    let step = u16::from(percent.min(100)) * u16::from(span) / 100;
    (u16::from(from) + step).min(100) as u8
}

/// Queue in front of the analysis thread.
pub struct AnalysisQueue {
    service: Arc<AnalysisService>,
    jobs: Mutex<Sender<RecordingId>>,
}

impl AnalysisQueue {
    pub fn start(service: Arc<AnalysisService>) -> Self {
        let (jobs, incoming) = channel::<RecordingId>();
        let worker = service.clone();
        thread::spawn(move || {
            while let Ok(id) = incoming.recv() {
                worker.run(id);
            }
        });
        Self {
            service,
            jobs: Mutex::new(jobs),
        }
    }

    /// Queues the recording unless it is already queued or running.
    pub fn enqueue(&self, id: RecordingId) {
        if self.service.queue(id) {
            let jobs = self.jobs.lock().unwrap_or_else(PoisonError::into_inner);
            // A stopped thread means the server is shutting down.
            let _ = jobs.send(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scaled;

    #[test]
    fn a_step_maps_onto_its_share_of_the_whole() {
        assert_eq!(scaled(0, 45, 55), 45);
        assert_eq!(scaled(60, 45, 55), 78);
        assert_eq!(scaled(100, 45, 55), 100);
        assert_eq!(scaled(100, 0, 40), 40);
        assert_eq!(scaled(255, 45, 55), 100);
    }
}
