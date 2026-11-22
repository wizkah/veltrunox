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

    fn rename(&self, id: RecordingId, title: RecordingTitle) -> Result<bool, StorageError> {
        let mut items = self.items.lock().unwrap();
        Ok(items
            .iter_mut()
            .find(|(r, _)| r.id == id)
            .map(|entry| entry.0.title = title)
            .is_some())
    }

    fn delete(&self, id: RecordingId) -> Result<bool, StorageError> {
        let mut items = self.items.lock().unwrap();
        let before = items.len();
        items.retain(|(r, _)| r.id != id);
        Ok(items.len() != before)
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

    fn recording_by_hash(&self, _content_hash: &str) -> Result<Option<Recording>, StorageError> {
        Ok(None)
    }

    fn media_paths(&self) -> Result<Vec<PathBuf>, StorageError> {
        Ok(self.0.iter().cloned().collect())
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

pub use account_fakes::*;

mod account_fakes {
    use crate::ports::{
        HashFailure, PasswordHasher, SessionRepository, SessionTokens, StorageError, UserRecord,
        UserRepository, UserStoreError, WallClock,
    };
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use veltrunox_core::{Role, UserId, UserName};

    #[derive(Default)]
    pub struct FakeUsers {
        users: Mutex<Vec<UserRecord>>,
        guest: Mutex<bool>,
        policy: Mutex<veltrunox_core::PasswordPolicy>,
        sessions: Mutex<Vec<(String, UserId, u64)>>,
    }

    impl UserRepository for FakeUsers {
        fn count(&self) -> Result<u64, StorageError> {
            Ok(self.users.lock().unwrap().len() as u64)
        }

        fn create_first_admin(
            &self,
            name: &UserName,
            password_hash: &str,
        ) -> Result<Option<UserRecord>, UserStoreError> {
            if !self.users.lock().unwrap().is_empty() {
                return Ok(None);
            }
            self.create(name, Role::Admin, password_hash).map(Some)
        }

        fn create(
            &self,
            name: &UserName,
            role: Role,
            password_hash: &str,
        ) -> Result<UserRecord, UserStoreError> {
            let mut users = self.users.lock().unwrap();
            if users.iter().any(|user| user.name.key() == name.key()) {
                return Err(UserStoreError::NameTaken);
            }
            let record = UserRecord {
                id: UserId::new(users.len() as u64 + 1),
                name: name.clone(),
                role,
                password_hash: password_hash.to_owned(),
                active: true,
            };
            users.push(record.clone());
            Ok(record)
        }

        fn find_by_name(&self, key: &str) -> Result<Option<UserRecord>, StorageError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|user| user.name.key() == key).cloned())
        }

        fn find(&self, id: UserId) -> Result<Option<UserRecord>, StorageError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|user| user.id == id).cloned())
        }

        fn list(&self) -> Result<Vec<UserRecord>, StorageError> {
            Ok(self.users.lock().unwrap().clone())
        }

        fn set_role(&self, id: UserId, role: Role) -> Result<(), UserStoreError> {
            self.change(id, |user| user.role = role)
        }

        fn set_active(&self, id: UserId, active: bool) -> Result<(), UserStoreError> {
            self.change(id, |user| user.active = active)
        }

        fn set_password_hash(&self, id: UserId, hash: &str) -> Result<(), UserStoreError> {
            self.change(id, |user| user.password_hash = hash.to_owned())
        }

        fn delete(&self, id: UserId) -> Result<(), UserStoreError> {
            let mut users = self.users.lock().unwrap();
            let before = users.len();
            users.retain(|user| user.id != id);
            if users.len() == before {
                return Err(UserStoreError::NotFound);
            }
            self.sessions
                .lock()
                .unwrap()
                .retain(|(_, user, _)| *user != id);
            Ok(())
        }

        fn guest_enabled(&self) -> Result<bool, StorageError> {
            Ok(*self.guest.lock().unwrap())
        }

        fn set_guest_enabled(&self, enabled: bool) -> Result<(), StorageError> {
            *self.guest.lock().unwrap() = enabled;
            Ok(())
        }

        fn password_policy(&self) -> Result<veltrunox_core::PasswordPolicy, StorageError> {
            Ok(*self.policy.lock().unwrap())
        }

        fn set_password_policy(
            &self,
            policy: veltrunox_core::PasswordPolicy,
        ) -> Result<(), StorageError> {
            *self.policy.lock().unwrap() = policy;
            Ok(())
        }
    }

    impl FakeUsers {
        fn change(
            &self,
            id: UserId,
            apply: impl FnOnce(&mut UserRecord),
        ) -> Result<(), UserStoreError> {
            let mut users = self.users.lock().unwrap();
            let user = users
                .iter_mut()
                .find(|user| user.id == id)
                .ok_or(UserStoreError::NotFound)?;
            apply(user);
            Ok(())
        }
    }

    impl SessionRepository for FakeUsers {
        fn save(&self, digest: &str, user: UserId, now: u64) -> Result<(), StorageError> {
            self.sessions
                .lock()
                .unwrap()
                .push((digest.to_owned(), user, now));
            Ok(())
        }

        fn user_of(
            &self,
            digest: &str,
            now: u64,
            max_idle: u64,
        ) -> Result<Option<UserRecord>, StorageError> {
            let mut sessions = self.sessions.lock().unwrap();
            let Some(entry) = sessions.iter_mut().find(|(known, _, _)| known == digest) else {
                return Ok(None);
            };
            if now.saturating_sub(entry.2) > max_idle {
                return Ok(None);
            }
            entry.2 = now;
            let user = entry.1;
            drop(sessions);
            Ok(self.find(user)?.filter(|user| user.active))
        }

        fn end(&self, digest: &str) -> Result<(), StorageError> {
            self.sessions
                .lock()
                .unwrap()
                .retain(|(known, _, _)| known != digest);
            Ok(())
        }

        fn delete_for_user(&self, user: UserId) -> Result<(), StorageError> {
            self.sessions
                .lock()
                .unwrap()
                .retain(|(_, owner, _)| *owner != user);
            Ok(())
        }
    }

    /// "Hashes" by prefixing, so tests need no slow algorithm.
    pub struct PrefixHasher;

    impl PasswordHasher for PrefixHasher {
        fn hash(&self, password: &str) -> Result<String, HashFailure> {
            Ok(format!("hash:{password}"))
        }

        fn verify(&self, password: &str, hash: &str) -> bool {
            hash == format!("hash:{password}")
        }
    }

    #[derive(Default)]
    pub struct CountingTokens(AtomicU64);

    impl SessionTokens for CountingTokens {
        fn issue(&self) -> String {
            format!("token-{}", self.0.fetch_add(1, Ordering::SeqCst))
        }

        fn digest(&self, token: &str) -> String {
            format!("digest-{token}")
        }
    }

    #[derive(Default)]
    pub struct FakeWallClock(pub AtomicU64);

    impl FakeWallClock {
        pub fn advance(&self, seconds: u64) {
            self.0.fetch_add(seconds, Ordering::SeqCst);
        }
    }

    impl WallClock for FakeWallClock {
        fn unix_seconds(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }
}
