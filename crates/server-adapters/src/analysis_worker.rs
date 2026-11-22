//! Runs the analyses one after the other on a background thread: they are CPU
//! and memory heavy, and the server may be a Raspberry Pi.

use crate::analysis::analyse_reporting;
use crate::audio::decode_file_reporting;
use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use veltrunox_core::{Millis, RecordingId, Waveform};
use veltrunox_server_app::{AnalysisFailure, AnalysisService, RecordingAnalyzer, RecordingScan};

/// Decodes a file once and derives length, waveform and analysis from it.
pub struct FileAnalyzer;

impl RecordingAnalyzer for FileAnalyzer {
    /// Decoding counts for the first 40 percent, the waveform up to 45, the analysis the rest.
    fn scan(&self, path: &Path, report: &dyn Fn(u8)) -> Result<RecordingScan, AnalysisFailure> {
        let pcm = decode_file_reporting(path, &|percent| report(scaled(percent, 0, 40)))
            .map_err(|error| AnalysisFailure(error.to_string()))?;
        report(40);
        let waveform = Waveform::from_mono(&pcm.mono(), pcm.sample_rate());
        report(45);
        Ok(RecordingScan {
            length: Millis::new(pcm.duration_ms()),
            waveform,
            analysis: analyse_reporting(&pcm, &|percent| report(scaled(percent, 45, 55)))
                .map_err(|error| AnalysisFailure(error.to_string())),
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
