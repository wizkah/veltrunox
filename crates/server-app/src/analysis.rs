//! Use case: scan recordings in the background (waveform, length, tuning, key,
//! tempo) and keep the results (ADR-007).

use crate::ports::{
    AnalysisRepository, MediaCatalog, PlaybackBroadcast, RecordingAnalyzer, StorageError,
    WaveformRepository,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{AnalysisInfo, AnalysisProgress, LibraryEntry, Recording, RecordingId};

/// Smallest rise of the percentage that is told to the clients.
const REPORT_STEP: u8 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Progress {
    Queued,
    /// Percent of the scan done.
    Running(u8),
    Failed(String),
}

pub struct AnalysisService {
    media: Arc<dyn MediaCatalog>,
    results: Arc<dyn AnalysisRepository>,
    waveforms: Arc<dyn WaveformRepository>,
    analyzer: Arc<dyn RecordingAnalyzer>,
    broadcast: Arc<dyn PlaybackBroadcast>,
    progress: Mutex<HashMap<RecordingId, Progress>>,
}

impl AnalysisService {
    pub fn new(
        media: Arc<dyn MediaCatalog>,
        results: Arc<dyn AnalysisRepository>,
        waveforms: Arc<dyn WaveformRepository>,
        analyzer: Arc<dyn RecordingAnalyzer>,
        broadcast: Arc<dyn PlaybackBroadcast>,
    ) -> Self {
        Self {
            media,
            results,
            waveforms,
            analyzer,
            broadcast,
            progress: Mutex::default(),
        }
    }

    /// Notes that the recording waits for its analysis. `false` if one is already
    /// queued or running, so the caller does not queue it twice.
    pub fn queue(&self, id: RecordingId) -> bool {
        let queued = {
            let mut progress = self.progress();
            match progress.get(&id) {
                Some(Progress::Queued | Progress::Running(_)) => false,
                _ => {
                    progress.insert(id, Progress::Queued);
                    true
                }
            }
        };
        if queued {
            self.broadcast.analysis_changed(id);
        }
        queued
    }

    /// Analyses a recording and stores the result, replacing an earlier one.
    /// Blocks for the duration of the analysis.
    pub fn run(&self, id: RecordingId) {
        self.set(id, Some(Progress::Running(0)));
        let outcome = self.analyse_and_store(id);
        self.set(id, outcome.err().map(Progress::Failed));
    }

    fn analyse_and_store(&self, id: RecordingId) -> Result<(), String> {
        let path = self
            .media
            .media_path(id)
            .map_err(|_| "storage_failed".to_owned())?
            .ok_or("audio_file_missing".to_owned())?;
        let scan = self
            .analyzer
            .scan(&path, &|percent| self.report(id, percent))
            .map_err(|error| error.to_string())?;
        self.waveforms
            .save_waveform(id, scan.length, &scan.waveform)
            .map_err(|_| "storage_failed".to_owned())?;
        let found = scan.analysis.map_err(|error| error.to_string())?;
        self.results
            .save_analysis(id, found)
            .map_err(|_| "storage_failed".to_owned())
    }

    pub fn status(&self, id: RecordingId) -> Result<AnalysisProgress, StorageError> {
        if let Some(progress) = self.progress().get(&id) {
            return Ok(match progress {
                Progress::Queued => AnalysisProgress::Queued,
                Progress::Running(percent) => AnalysisProgress::Running(*percent),
                Progress::Failed(reason) => AnalysisProgress::Failed(reason.clone()),
            });
        }
        Ok(if self.results.analysis(id)?.is_some() {
            AnalysisProgress::Done
        } else {
            AnalysisProgress::None
        })
    }

    /// The recordings with their analysis status and result.
    pub fn entries(&self, recordings: Vec<Recording>) -> Result<Vec<LibraryEntry>, StorageError> {
        let results: HashMap<_, _> = self.results.analyses()?.into_iter().collect();
        let mut lengths = HashMap::new();
        for recording in &recordings {
            if let Some(length) = self.waveforms.stored_length(recording.id)? {
                lengths.insert(recording.id, length);
            }
        }
        let progress = self.progress();
        Ok(recordings
            .into_iter()
            .map(|recording| {
                let result = results.get(&recording.id).copied();
                let state = match progress.get(&recording.id) {
                    Some(Progress::Queued) => AnalysisProgress::Queued,
                    Some(Progress::Running(percent)) => AnalysisProgress::Running(*percent),
                    Some(Progress::Failed(reason)) => AnalysisProgress::Failed(reason.clone()),
                    None if result.is_some() => AnalysisProgress::Done,
                    None => AnalysisProgress::None,
                };
                LibraryEntry {
                    // Stems are not the analysis' business; the list adds them.
                    has_stems: false,
                    length: lengths.get(&recording.id).copied(),
                    recording,
                    analysis: AnalysisInfo {
                        progress: state,
                        result,
                    },
                }
            })
            .collect())
    }

    /// Recordings that have audio but no stored result yet.
    pub fn unanalysed(&self) -> Result<Vec<RecordingId>, StorageError> {
        self.results.unanalysed()
    }

    /// Notes how far the running scan has come. Clients are told in steps of
    /// `REPORT_STEP` percent, not for every packet.
    fn report(&self, id: RecordingId, percent: u8) {
        let percent = percent.min(100);
        let changed = {
            let mut map = self.progress();
            match map.get_mut(&id) {
                Some(Progress::Running(shown)) if percent >= shown.saturating_add(REPORT_STEP) => {
                    *shown = percent;
                    true
                }
                _ => false,
            }
        };
        if changed {
            self.broadcast.analysis_changed(id);
        }
    }

    fn set(&self, id: RecordingId, progress: Option<Progress>) {
        {
            let mut map = self.progress();
            match progress {
                Some(progress) => map.insert(id, progress),
                None => map.remove(&id),
            };
        }
        self.broadcast.analysis_changed(id);
    }

    fn progress(&self) -> std::sync::MutexGuard<'_, HashMap<RecordingId, Progress>> {
        self.progress.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{AnalysisFailure, RecordingScan};
    use crate::testing::{FakeAnalyses, FakeBroadcast, FakeMedia, FakeWaveforms};
    use std::path::Path;
    use veltrunox_core::{
        Bpm, Confidence, KeyMode, Millis, MusicalKey, PitchClass, RecordingAnalysis, TuningHz,
        Waveform,
    };

    fn found() -> RecordingAnalysis {
        RecordingAnalysis {
            tuning: TuningHz::new(442.0).unwrap(),
            tuning_confidence: Confidence::new(1.5),
            key: MusicalKey {
                tonic: PitchClass::from_semitones(9),
                mode: KeyMode::Minor,
            },
            key_confidence: Confidence::new(1.1),
            tempo: Bpm::new(120.0).unwrap(),
            tempo_confidence: Confidence::new(1.2),
        }
    }

    struct Analyzer(Result<RecordingAnalysis, String>);

    impl RecordingAnalyzer for Analyzer {
        fn scan(
            &self,
            _path: &Path,
            report: &dyn Fn(u8),
        ) -> Result<RecordingScan, AnalysisFailure> {
            report(50);
            Ok(RecordingScan {
                length: Millis::new(90_000),
                waveform: Waveform::from_mono(&[0.5, -0.5], 44_100),
                analysis: self.0.clone().map_err(AnalysisFailure),
            })
        }
    }

    const ID: RecordingId = RecordingId::new(1);

    fn service(
        media: FakeMedia,
        analyzer: Result<RecordingAnalysis, String>,
    ) -> (
        AnalysisService,
        Arc<FakeAnalyses>,
        Arc<FakeBroadcast>,
        Arc<FakeWaveforms>,
    ) {
        let results = Arc::new(FakeAnalyses::default());
        let waveforms = Arc::new(FakeWaveforms::default());
        let broadcast = Arc::new(FakeBroadcast::default());
        let service = AnalysisService::new(
            Arc::new(media),
            results.clone(),
            waveforms.clone(),
            Arc::new(Analyzer(analyzer)),
            broadcast.clone(),
        );
        (service, results, broadcast, waveforms)
    }

    #[test]
    fn a_queued_recording_is_queued_once() {
        let (service, _, broadcast, _) = service(FakeMedia::with_file(), Ok(found()));
        assert!(service.queue(ID));
        assert!(!service.queue(ID));
        assert_eq!(service.status(ID).unwrap(), AnalysisProgress::Queued);
        assert_eq!(broadcast.analysis_notices(), [ID]);
    }

    #[test]
    fn running_stores_the_result_and_tells_the_clients() {
        let (service, results, broadcast, _) = service(FakeMedia::with_file(), Ok(found()));
        service.queue(ID);
        service.run(ID);
        assert_eq!(results.analysis(ID).unwrap(), Some(found()));
        assert_eq!(service.status(ID).unwrap(), AnalysisProgress::Done);
        // queued, running, half way (reported by the analyzer), finished
        assert_eq!(broadcast.analysis_notices().len(), 4);
    }

    #[test]
    fn the_percentage_is_told_in_steps_and_only_while_running() {
        let (service, _, broadcast, _) = service(FakeMedia::with_file(), Ok(found()));
        service.report(ID, 40);
        assert!(broadcast.analysis_notices().is_empty(), "nothing runs yet");

        service.set(ID, Some(Progress::Running(0)));
        service.report(ID, 2);
        assert_eq!(service.status(ID).unwrap(), AnalysisProgress::Running(0));
        service.report(ID, 40);
        service.report(ID, 42);
        assert_eq!(service.status(ID).unwrap(), AnalysisProgress::Running(40));
        // the start and the 40 percent
        assert_eq!(broadcast.analysis_notices().len(), 2);
    }

    #[test]
    fn running_stores_waveform_and_length() {
        let (service, _, _, waveforms) = service(FakeMedia::with_file(), Ok(found()));
        service.run(ID);
        assert_eq!(
            waveforms.stored_length(ID).unwrap(),
            Some(Millis::new(90_000))
        );
        assert!(waveforms.waveform(ID).unwrap().is_some());
    }

    #[test]
    fn a_failed_analysis_still_keeps_waveform_and_length() {
        let (service, _, _, waveforms) = service(FakeMedia::with_file(), Err("silence".into()));
        service.run(ID);
        assert_eq!(
            service.status(ID).unwrap(),
            AnalysisProgress::Failed("silence".into())
        );
        assert_eq!(
            waveforms.stored_length(ID).unwrap(),
            Some(Millis::new(90_000))
        );
    }

    #[test]
    fn a_failure_is_kept_with_its_reason_and_can_be_retried() {
        let (service, results, _, _) = service(FakeMedia::with_file(), Err("too short".into()));
        service.queue(ID);
        service.run(ID);
        assert_eq!(
            service.status(ID).unwrap(),
            AnalysisProgress::Failed("too short".into())
        );
        assert_eq!(results.analysis(ID).unwrap(), None);
        assert!(service.queue(ID));
    }

    #[test]
    fn a_recording_without_a_file_fails_and_stores_nothing() {
        let (service, results, _, _) = service(FakeMedia::without_file(), Ok(found()));
        service.run(ID);
        assert!(matches!(
            service.status(ID).unwrap(),
            AnalysisProgress::Failed(_)
        ));
        assert_eq!(results.analysis(ID).unwrap(), None);
    }

    #[test]
    fn entries_combine_recordings_progress_and_results() {
        let (service, results, _, _) = service(FakeMedia::with_file(), Ok(found()));
        let recording = |id: u64, title: &str| Recording {
            id: RecordingId::new(id),
            title: veltrunox_core::RecordingTitle::new(title).unwrap(),
        };
        results.save_analysis(RecordingId::new(1), found()).unwrap();
        service.queue(RecordingId::new(2));
        let entries = service
            .entries(vec![
                recording(1, "a"),
                recording(2, "b"),
                recording(3, "c"),
            ])
            .unwrap();
        let progress: Vec<_> = entries
            .iter()
            .map(|e| e.analysis.progress.clone())
            .collect();
        assert_eq!(
            progress,
            [
                AnalysisProgress::Done,
                AnalysisProgress::Queued,
                AnalysisProgress::None
            ]
        );
        assert_eq!(entries[0].analysis.result, Some(found()));
        assert_eq!(entries[1].analysis.result, None);
    }

    #[test]
    fn a_new_run_replaces_the_stored_result() {
        let (service, results, _, _) = service(FakeMedia::with_file(), Ok(found()));
        results.save_analysis(ID, found()).unwrap();
        service.run(ID);
        assert_eq!(service.status(ID).unwrap(), AnalysisProgress::Done);
    }
}
