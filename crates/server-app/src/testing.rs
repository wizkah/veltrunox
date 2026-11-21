//! Fakes for the ports, shared by the unit tests of this crate.

use crate::ports::{Clock, PlaybackBroadcast, RecordingRepository, StorageError};
use std::sync::Mutex;
use veltrunox_core::{
    Millis, PlaybackSettings, PlaybackState, Recording, RecordingId, RecordingTitle,
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
}

impl FakeBroadcast {
    pub fn states(&self) -> Vec<PlaybackState> {
        self.states.lock().unwrap().clone()
    }
}

impl PlaybackBroadcast for FakeBroadcast {
    fn state_changed(&self, state: &PlaybackState) {
        self.states.lock().unwrap().push(*state);
    }

    fn bookmarks_changed(&self, _recording: RecordingId) {}
}
