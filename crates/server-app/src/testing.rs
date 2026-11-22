//! Fakes for the ports, shared by the unit tests of this crate.

use crate::ports::{
    AnalysisRepository, Clock, MediaCatalog, PlaybackBroadcast, RecordingLengths,
    RecordingRepository, StorageError, WaveformRepository,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use veltrunox_core::{
    Millis, PlaybackSettings, PlaybackState, Recording, RecordingAnalysis, RecordingId,
    RecordingTitle, Waveform,
};

pub struct FakeRecordings {
    items: Mutex<Vec<(Recording, PlaybackSettings)>>,
}

impl FakeRecordings {
    pub fn with_one() -> Self {
        let recording = Recording {
            id: RecordingId::new(1),
            title: RecordingTitle::new("Intro").unwrap(),
        };
        Self {
            items: Mutex::new(vec![(recording, PlaybackSettings::default())]),
        }
    }

    pub fn stored(&self, id: RecordingId) -> PlaybackSettings {
        let items = self.items.lock().unwrap();
        items.iter().find(|(r, _)| r.id == id).unwrap().1
    }
}

impl RecordingRepository for FakeRecordings {
    fn list(&self) -> Result<Vec<Recording>, StorageError> {
        Ok(self
            .items
            .lock()
            .unwrap()
            .iter()
            .map(|(r, _)| r.clone())
            .collect())
    }

    fn add(&self, title: RecordingTitle) -> Result<Recording, StorageError> {
        let mut items = self.items.lock().unwrap();
        let recording = Recording {
            id: RecordingId::new(items.len() as u64 + 1),
            title,
        };
        items.push((recording.clone(), PlaybackSettings::default()));
        Ok(recording)
    }

    fn settings(&self, id: RecordingId) -> Result<Option<PlaybackSettings>, StorageError> {
        let items = self.items.lock().unwrap();
        Ok(items.iter().find(|(r, _)| r.id == id).map(|(_, s)| *s))
    }

    fn save_settings(
        &self,
        id: RecordingId,
        settings: PlaybackSettings,
    ) -> Result<(), StorageError> {
        let mut items = self.items.lock().unwrap();
        if let Some(entry) = items.iter_mut().find(|(r, _)| r.id == id) {
            entry.1 = settings;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeLengths(Mutex<Option<Millis>>);

impl FakeLengths {
    pub fn set(&self, length: Millis) {
        *self.0.lock().unwrap() = Some(length);
    }
}

impl RecordingLengths for FakeLengths {
    fn length(&self, _id: RecordingId) -> Option<Millis> {
        *self.0.lock().unwrap()
    }
}

#[derive(Default)]
pub struct FakeClock(Mutex<u64>);

impl FakeClock {
    pub fn set(&self, now: Millis) {
        *self.0.lock().unwrap() = now.value();
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Millis {
        Millis::new(*self.0.lock().unwrap())
    }
}

#[derive(Default)]
pub struct FakeBroadcast {
    states: Mutex<Vec<PlaybackState>>,
    analysis: Mutex<Vec<RecordingId>>,
}

impl FakeBroadcast {
    pub fn analysis_notices(&self) -> Vec<RecordingId> {
        self.analysis.lock().unwrap().clone()
    }

    pub fn states(&self) -> Vec<PlaybackState> {
        self.states.lock().unwrap().clone()
    }
}

impl PlaybackBroadcast for FakeBroadcast {
    fn state_changed(&self, state: &PlaybackState) {
        self.states.lock().unwrap().push(*state);
    }

    fn bookmarks_changed(&self, _recording: RecordingId) {}

    fn analysis_changed(&self, recording: RecordingId) {
        self.analysis.lock().unwrap().push(recording);
    }
}

pub struct FakeMedia(Option<PathBuf>);

impl FakeMedia {
    pub fn with_file() -> Self {
        Self(Some(PathBuf::from("song.wav")))
    }

    pub fn without_file() -> Self {
        Self(None)
    }
}

impl MediaCatalog for FakeMedia {
    fn media_path(&self, _id: RecordingId) -> Result<Option<PathBuf>, StorageError> {
        Ok(self.0.clone())
    }

    fn add_media(
        &self,
        _title: RecordingTitle,
        _path: &Path,
        _content_hash: &str,
    ) -> Result<Option<Recording>, StorageError> {
        Ok(None)
    }
}

#[derive(Default)]
pub struct FakeAnalyses(Mutex<Vec<(RecordingId, RecordingAnalysis)>>);

impl AnalysisRepository for FakeAnalyses {
    fn analysis(&self, id: RecordingId) -> Result<Option<RecordingAnalysis>, StorageError> {
        let items = self.0.lock().unwrap();
        Ok(items.iter().find(|(r, _)| *r == id).map(|(_, a)| *a))
    }

    fn analyses(&self) -> Result<Vec<(RecordingId, RecordingAnalysis)>, StorageError> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn save_analysis(
        &self,
        id: RecordingId,
        analysis: RecordingAnalysis,
    ) -> Result<(), StorageError> {
        let mut items = self.0.lock().unwrap();
        items.retain(|(r, _)| *r != id);
        items.push((id, analysis));
        Ok(())
    }

    fn unanalysed(&self) -> Result<Vec<RecordingId>, StorageError> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
pub struct FakeWaveforms(Mutex<Vec<(RecordingId, Millis, Waveform)>>);

impl WaveformRepository for FakeWaveforms {
    fn save_waveform(
        &self,
        id: RecordingId,
        length: Millis,
        waveform: &Waveform,
    ) -> Result<(), StorageError> {
        let mut items = self.0.lock().unwrap();
        items.retain(|(r, _, _)| *r != id);
        items.push((id, length, waveform.clone()));
        Ok(())
    }

    fn waveform(&self, id: RecordingId) -> Result<Option<Waveform>, StorageError> {
        let items = self.0.lock().unwrap();
        Ok(items
            .iter()
            .find(|(r, _, _)| *r == id)
            .map(|(_, _, w)| w.clone()))
    }

    fn stored_length(&self, id: RecordingId) -> Result<Option<Millis>, StorageError> {
        let items = self.0.lock().unwrap();
        Ok(items.iter().find(|(r, _, _)| *r == id).map(|(_, l, _)| *l))
    }
}
