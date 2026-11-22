//! Runs the analyses one after the other on a background thread: they are CPU
//! and memory heavy, and the server may be a Raspberry Pi.

use crate::analysis::analyse;
use crate::audio::decode_file;
use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use veltrunox_core::{Millis, RecordingId, Waveform};
use veltrunox_server_app::{AnalysisFailure, AnalysisService, RecordingAnalyzer, RecordingScan};

/// Decodes a file once and derives length, waveform and analysis from it.
pub struct FileAnalyzer;

impl RecordingAnalyzer for FileAnalyzer {
    fn scan(&self, path: &Path) -> Result<RecordingScan, AnalysisFailure> {
        let pcm = decode_file(path).map_err(|error| AnalysisFailure(error.to_string()))?;
        let waveform = Waveform::from_mono(&pcm.mono(), pcm.sample_rate());
        Ok(RecordingScan {
            length: Millis::new(pcm.duration_ms()),
            waveform,
            analysis: analyse(&pcm).map_err(|error| AnalysisFailure(error.to_string())),
        })
    }
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
