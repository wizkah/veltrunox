//! Persistence in one SQLite file (ADR-004): recordings, their playback settings
//! and bookmarks.

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Bpm, Confidence, KeyMode, KeyShift, Label, Millis,
    MusicalKey, PitchClass, PlaybackSettings, Recording, RecordingAnalysis, RecordingId,
    RecordingTitle, Region, Speed, TimeRange, TuningHz, Version, check_version,
};
use veltrunox_server_app::{
    AnalysisRepository, BookmarkError, BookmarkRepository, MediaCatalog, Owned,
    RecordingRepository, StorageError,
};

/// Versioned schema changes; entry `n` upgrades from `user_version` `n` to `n + 1`.
const MIGRATIONS: &[&str] = &[
    "
    CREATE TABLE recordings (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL,
        speed REAL NOT NULL DEFAULT 1.0,
        tuning_hz REAL NOT NULL DEFAULT 440.0,
        key_shift INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE labels (
        id INTEGER PRIMARY KEY,
        recording_id INTEGER NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        position_ms INTEGER NOT NULL,
        version INTEGER NOT NULL
    );
    CREATE TABLE regions (
        id INTEGER PRIMARY KEY,
        recording_id INTEGER NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        start_ms INTEGER NOT NULL,
        end_ms INTEGER NOT NULL,
        version INTEGER NOT NULL
    );
    CREATE INDEX labels_by_recording ON labels(recording_id);
    CREATE INDEX regions_by_recording ON regions(recording_id);
",
    "
    ALTER TABLE recordings ADD COLUMN media_path TEXT;
    ALTER TABLE recordings ADD COLUMN content_hash TEXT;
    CREATE UNIQUE INDEX recordings_by_hash ON recordings(content_hash);
",
    "
    ALTER TABLE recordings ADD COLUMN analysis_tuning_hz REAL;
    ALTER TABLE recordings ADD COLUMN analysis_tuning_confidence REAL;
    ALTER TABLE recordings ADD COLUMN analysis_key_tonic INTEGER;
    ALTER TABLE recordings ADD COLUMN analysis_key_minor INTEGER;
    ALTER TABLE recordings ADD COLUMN analysis_key_confidence REAL;
    ALTER TABLE recordings ADD COLUMN analysis_tempo_bpm REAL;
    ALTER TABLE recordings ADD COLUMN analysis_tempo_confidence REAL;
",
];

/// Blocking store; callers on an async runtime run it off the runtime threads.
pub struct SqliteStore {
    connection: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        Self::prepare(Connection::open(path).map_err(storage)?)
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        Self::prepare(Connection::open_in_memory().map_err(storage)?)
    }

    fn prepare(mut connection: Connection) -> Result<Self, StorageError> {
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(storage)?;
        // WAL is not available for in-memory databases; the answer is not needed.
        connection
            .query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
            .map_err(storage)?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

fn storage(error: rusqlite::Error) -> StorageError {
    StorageError(error.to_string())
}

fn corrupt(what: &str, detail: impl std::fmt::Display) -> StorageError {
    StorageError(format!("stored {what} is invalid: {detail}"))
}

fn migrate(connection: &mut Connection) -> Result<(), StorageError> {
    let applied: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(storage)?;
    let applied = usize::try_from(applied).map_err(|_| corrupt("schema version", "negative"))?;
    for (index, script) in MIGRATIONS.iter().enumerate().skip(applied) {
        let transaction = connection.transaction().map_err(storage)?;
        transaction.execute_batch(script).map_err(storage)?;
        transaction
            .pragma_update(None, "user_version", index as i64 + 1)
            .map_err(storage)?;
        transaction.commit().map_err(storage)?;
    }
    Ok(())
}

fn sql_int(value: u64) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|_| StorageError("value exceeds the storable range".into()))
}

fn unsigned(value: i64) -> Result<u64, StorageError> {
    u64::try_from(value).map_err(|_| corrupt("number", "negative"))
}

impl MediaCatalog for SqliteStore {
    fn media_path(&self, id: RecordingId) -> Result<Option<PathBuf>, StorageError> {
        let path = self
            .connection()
            .query_row(
                "SELECT media_path FROM recordings WHERE id = ?1",
                params![sql_int(id.value())?],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(storage)?;
        Ok(path.flatten().map(PathBuf::from))
    }

    fn add_media(
        &self,
        title: RecordingTitle,
        path: &Path,
        content_hash: &str,
    ) -> Result<Option<Recording>, StorageError> {
        let connection = self.connection();
        let path_text = path
            .to_str()
            .ok_or_else(|| StorageError("file path is not valid text".into()))?;
        let added = connection
            .execute(
                "INSERT OR IGNORE INTO recordings (title, media_path, content_hash) VALUES (?1, ?2, ?3)",
                params![title.as_str(), path_text, content_hash],
            )
            .map_err(storage)?;
        if added == 0 {
            return Ok(None);
        }
        Ok(Some(Recording {
            id: RecordingId::new(unsigned(connection.last_insert_rowid())?),
            title,
        }))
    }
}

impl RecordingRepository for SqliteStore {
    fn list(&self) -> Result<Vec<Recording>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare("SELECT id, title FROM recordings ORDER BY id")
            .map_err(storage)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, title) = row.map_err(storage)?;
            Ok(Recording {
                id: RecordingId::new(unsigned(id)?),
                title: RecordingTitle::new(&title).map_err(|error| corrupt("title", error))?,
            })
        })
        .collect()
    }

    fn add(&self, title: RecordingTitle) -> Result<Recording, StorageError> {
        let connection = self.connection();
        connection
            .execute(
                "INSERT INTO recordings (title) VALUES (?1)",
                params![title.as_str()],
            )
            .map_err(storage)?;
        Ok(Recording {
            id: RecordingId::new(unsigned(connection.last_insert_rowid())?),
            title,
        })
    }

    fn settings(&self, id: RecordingId) -> Result<Option<PlaybackSettings>, StorageError> {
        let row = self
            .connection()
            .query_row(
                "SELECT speed, tuning_hz, key_shift FROM recordings WHERE id = ?1",
                params![sql_int(id.value())?],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, i8>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?;
        row.map(|(speed, tuning, key)| {
            Ok(PlaybackSettings {
                speed: Speed::new(speed).map_err(|error| corrupt("speed", error))?,
                tuning: TuningHz::new(tuning).map_err(|error| corrupt("tuning", error))?,
                key: KeyShift::new(key).map_err(|error| corrupt("key", error))?,
            })
        })
        .transpose()
    }

    fn save_settings(
        &self,
        id: RecordingId,
        settings: PlaybackSettings,
    ) -> Result<(), StorageError> {
        self.connection()
            .execute(
                "UPDATE recordings SET speed = ?2, tuning_hz = ?3, key_shift = ?4 WHERE id = ?1",
                params![
                    sql_int(id.value())?,
                    settings.speed.factor(),
                    settings.tuning.hertz(),
                    settings.key.semitones()
                ],
            )
            .map_err(storage)?;
        Ok(())
    }
}

const ANALYSIS_COLUMNS: &str = "id, analysis_tuning_hz, analysis_tuning_confidence, \
    analysis_key_tonic, analysis_key_minor, analysis_key_confidence, \
    analysis_tempo_bpm, analysis_tempo_confidence";

/// One row of `ANALYSIS_COLUMNS`; `None` while the recording has no result.
fn read_analysis(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<Option<(RecordingId, RecordingAnalysis)>, StorageError>> {
    let id: i64 = row.get(0)?;
    let Some(tuning) = row.get::<_, Option<f64>>(1)? else {
        return Ok(Ok(None));
    };
    let (tuning_confidence, tonic, minor, key_confidence, tempo, tempo_confidence): (
        f64,
        i32,
        bool,
        f64,
        f64,
        f64,
    ) = (
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
    );
    Ok((|| {
        Ok(Some((
            RecordingId::new(unsigned(id)?),
            RecordingAnalysis {
                tuning: TuningHz::new(tuning).map_err(|error| corrupt("analysed tuning", error))?,
                tuning_confidence: Confidence::new(tuning_confidence),
                key: MusicalKey {
                    tonic: PitchClass::from_semitones(tonic),
                    mode: if minor {
                        KeyMode::Minor
                    } else {
                        KeyMode::Major
                    },
                },
                key_confidence: Confidence::new(key_confidence),
                tempo: Bpm::new(tempo).map_err(|error| corrupt("analysed tempo", error))?,
                tempo_confidence: Confidence::new(tempo_confidence),
            },
        )))
    })())
}

impl AnalysisRepository for SqliteStore {
    fn analysis(&self, id: RecordingId) -> Result<Option<RecordingAnalysis>, StorageError> {
        let row = self
            .connection()
            .query_row(
                &format!("SELECT {ANALYSIS_COLUMNS} FROM recordings WHERE id = ?1"),
                params![sql_int(id.value())?],
                read_analysis,
            )
            .optional()
            .map_err(storage)?;
        Ok(row.transpose()?.flatten().map(|(_, analysis)| analysis))
    }

    fn analyses(&self) -> Result<Vec<(RecordingId, RecordingAnalysis)>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare(&format!(
                "SELECT {ANALYSIS_COLUMNS} FROM recordings ORDER BY id"
            ))
            .map_err(storage)?;
        let rows = statement.query_map([], read_analysis).map_err(storage)?;
        let mut found = Vec::new();
        for row in rows {
            if let Some(entry) = row.map_err(storage)?? {
                found.push(entry);
            }
        }
        Ok(found)
    }

    fn save_analysis(
        &self,
        id: RecordingId,
        analysis: RecordingAnalysis,
    ) -> Result<(), StorageError> {
        let changed = self
            .connection()
            .execute(
                "UPDATE recordings SET analysis_tuning_hz = ?2, analysis_tuning_confidence = ?3, \
                 analysis_key_tonic = ?4, analysis_key_minor = ?5, analysis_key_confidence = ?6, \
                 analysis_tempo_bpm = ?7, analysis_tempo_confidence = ?8 WHERE id = ?1",
                params![
                    sql_int(id.value())?,
                    analysis.tuning.hertz(),
                    analysis.tuning_confidence.ratio(),
                    analysis.key.tonic.index(),
                    analysis.key.mode == KeyMode::Minor,
                    analysis.key_confidence.ratio(),
                    analysis.tempo.value(),
                    analysis.tempo_confidence.ratio(),
                ],
            )
            .map_err(storage)?;
        if changed == 0 {
            return Err(StorageError("recording does not exist".into()));
        }
        Ok(())
    }

    fn unanalysed(&self) -> Result<Vec<RecordingId>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare(
                "SELECT id FROM recordings \
                 WHERE media_path IS NOT NULL AND analysis_tuning_hz IS NULL ORDER BY id",
            )
            .map_err(storage)?;
        let ids = statement
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(storage)?;
        ids.map(|id| Ok(RecordingId::new(unsigned(id.map_err(storage)?)?)))
            .collect()
    }
}

/// The two kinds of bookmark differ only in the table and in what they locate.
#[derive(Clone, Copy)]
enum Table {
    Labels,
    Regions,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Self::Labels => "labels",
            Self::Regions => "regions",
        }
    }
}

/// Reads the owner and version of a record, or `NotFound`.
fn stored_version(
    transaction: &Transaction<'_>,
    table: Table,
    id: BookmarkId,
) -> Result<(RecordingId, Version), BookmarkError> {
    let sql = format!(
        "SELECT recording_id, version FROM {} WHERE id = ?1",
        table.name()
    );
    let row = transaction
        .query_row(&sql, params![sql_int(id.value())?], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .optional()
        .map_err(storage)?
        .ok_or(BookmarkError::NotFound)?;
    Ok((
        RecordingId::new(unsigned(row.0)?),
        Version::new(unsigned(row.1)?),
    ))
}

impl SqliteStore {
    /// Runs `change` inside a transaction after the version check passed.
    fn write_versioned<T>(
        &self,
        table: Table,
        id: BookmarkId,
        expected: Version,
        change: impl FnOnce(&Transaction<'_>, RecordingId, Version) -> Result<T, BookmarkError>,
    ) -> Result<T, BookmarkError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        let (recording, current) = stored_version(&transaction, table, id)?;
        let next =
            check_version(current, expected).map_err(|c| BookmarkError::Conflict(c.current))?;
        let result = change(&transaction, recording, next)?;
        transaction.commit().map_err(storage)?;
        Ok(result)
    }

    fn require_recording(
        transaction: &Transaction<'_>,
        recording: RecordingId,
    ) -> Result<(), BookmarkError> {
        let found = transaction
            .query_row(
                "SELECT 1 FROM recordings WHERE id = ?1",
                params![sql_int(recording.value())?],
                |_| Ok(()),
            )
            .optional()
            .map_err(storage)?;
        found.ok_or(BookmarkError::NotFound)
    }
}

impl BookmarkRepository for SqliteStore {
    fn list(&self, recording: RecordingId) -> Result<Bookmarks, BookmarkError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        Self::require_recording(&transaction, recording)?;
        let recording_id = sql_int(recording.value())?;
        let labels = read_labels(&transaction, recording_id)?;
        let regions = read_regions(&transaction, recording_id)?;
        Ok(Bookmarks { labels, regions })
    }

    fn add_label(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Label, BookmarkError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        Self::require_recording(&transaction, recording)?;
        transaction
            .execute(
                "INSERT INTO labels (recording_id, name, position_ms, version) VALUES (?1, ?2, ?3, ?4)",
                params![
                    sql_int(recording.value())?,
                    name.as_str(),
                    sql_int(position.value())?,
                    sql_int(Version::FIRST.value())?
                ],
            )
            .map_err(storage)?;
        let id = BookmarkId::new(unsigned(transaction.last_insert_rowid())?);
        transaction.commit().map_err(storage)?;
        Ok(Label {
            id,
            name,
            position,
            version: Version::FIRST,
        })
    }

    fn update_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Owned<Label>, BookmarkError> {
        self.write_versioned(
            Table::Labels,
            id,
            expected,
            |transaction, recording, next| {
                transaction
                    .execute(
                        "UPDATE labels SET name = ?2, position_ms = ?3, version = ?4 WHERE id = ?1",
                        params![
                            sql_int(id.value())?,
                            name.as_str(),
                            sql_int(position.value())?,
                            sql_int(next.value())?
                        ],
                    )
                    .map_err(storage)?;
                Ok(Owned {
                    recording,
                    item: Label {
                        id,
                        name,
                        position,
                        version: next,
                    },
                })
            },
        )
    }

    fn delete_label(
        &self,
        id: BookmarkId,
        expected: Version,
    ) -> Result<RecordingId, BookmarkError> {
        self.write_versioned(Table::Labels, id, expected, |transaction, recording, _| {
            delete_row(transaction, Table::Labels, id)?;
            Ok(recording)
        })
    }

    fn add_region(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        Self::require_recording(&transaction, recording)?;
        transaction
            .execute(
                "INSERT INTO regions (recording_id, name, start_ms, end_ms, version) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sql_int(recording.value())?,
                    name.as_str(),
                    sql_int(range.start().value())?,
                    sql_int(range.end().value())?,
                    sql_int(Version::FIRST.value())?
                ],
            )
            .map_err(storage)?;
        let id = BookmarkId::new(unsigned(transaction.last_insert_rowid())?);
        transaction.commit().map_err(storage)?;
        Ok(Region {
            id,
            name,
            range,
            version: Version::FIRST,
        })
    }

    fn update_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Owned<Region>, BookmarkError> {
        self.write_versioned(Table::Regions, id, expected, |transaction, recording, next| {
            transaction
                .execute(
                    "UPDATE regions SET name = ?2, start_ms = ?3, end_ms = ?4, version = ?5 WHERE id = ?1",
                    params![
                        sql_int(id.value())?,
                        name.as_str(),
                        sql_int(range.start().value())?,
                        sql_int(range.end().value())?,
                        sql_int(next.value())?
                    ],
                )
                .map_err(storage)?;
            Ok(Owned {
                recording,
                item: Region {
                    id,
                    name,
                    range,
                    version: next,
                },
            })
        })
    }

    fn delete_region(
        &self,
        id: BookmarkId,
        expected: Version,
    ) -> Result<RecordingId, BookmarkError> {
        self.write_versioned(Table::Regions, id, expected, |transaction, recording, _| {
            delete_row(transaction, Table::Regions, id)?;
            Ok(recording)
        })
    }
}

fn delete_row(
    transaction: &Transaction<'_>,
    table: Table,
    id: BookmarkId,
) -> Result<(), BookmarkError> {
    transaction
        .execute(
            &format!("DELETE FROM {} WHERE id = ?1", table.name()),
            params![sql_int(id.value())?],
        )
        .map_err(storage)?;
    Ok(())
}

fn read_labels(transaction: &Transaction<'_>, recording: i64) -> Result<Vec<Label>, StorageError> {
    let mut statement = transaction
        .prepare(
            "SELECT id, name, position_ms, version FROM labels \
             WHERE recording_id = ?1 ORDER BY position_ms, id",
        )
        .map_err(storage)?;
    let rows = statement
        .query_map(params![recording], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(storage)?;
    rows.map(|row| {
        let (id, name, position, version) = row.map_err(storage)?;
        Ok(Label {
            id: BookmarkId::new(unsigned(id)?),
            name: BookmarkName::new(&name).map_err(|error| corrupt("label name", error))?,
            position: Millis::new(unsigned(position)?),
            version: Version::new(unsigned(version)?),
        })
    })
    .collect()
}

fn read_regions(
    transaction: &Transaction<'_>,
    recording: i64,
) -> Result<Vec<Region>, StorageError> {
    let mut statement = transaction
        .prepare(
            "SELECT id, name, start_ms, end_ms, version FROM regions \
             WHERE recording_id = ?1 ORDER BY start_ms, id",
        )
        .map_err(storage)?;
    let rows = statement
        .query_map(params![recording], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(storage)?;
    rows.map(|row| {
        let (id, name, start, end, version) = row.map_err(storage)?;
        Ok(Region {
            id: BookmarkId::new(unsigned(id)?),
            name: BookmarkName::new(&name).map_err(|error| corrupt("region name", error))?,
            range: TimeRange::new(Millis::new(unsigned(start)?), Millis::new(unsigned(end)?))
                .map_err(|error| corrupt("region range", error))?,
            version: Version::new(unsigned(version)?),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_recording() -> (SqliteStore, RecordingId) {
        let store = SqliteStore::in_memory().unwrap();
        let recording = store.add(RecordingTitle::new("Intro").unwrap()).unwrap();
        (store, recording.id)
    }

    fn name(text: &str) -> BookmarkName {
        BookmarkName::new(text).unwrap()
    }

    fn range(start: u64, end: u64) -> TimeRange {
        TimeRange::new(Millis::new(start), Millis::new(end)).unwrap()
    }

    #[test]
    fn new_recordings_start_with_default_settings() {
        let (store, id) = store_with_recording();
        assert_eq!(
            store.settings(id).unwrap(),
            Some(PlaybackSettings::default())
        );
        assert_eq!(store.settings(RecordingId::new(99)).unwrap(), None);
    }

    #[test]
    fn settings_survive_reopening_the_database_file() {
        let directory = std::env::temp_dir().join(format!("veltrunox-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("store.db");
        let settings = PlaybackSettings {
            speed: Speed::new(0.7).unwrap(),
            tuning: TuningHz::new(412.0).unwrap(),
            key: KeyShift::new(-3).unwrap(),
        };
        let id = {
            let store = SqliteStore::open(&path).unwrap();
            let id = store.add(RecordingTitle::new("Intro").unwrap()).unwrap().id;
            store.save_settings(id, settings).unwrap();
            BookmarkRepository::add_label(&store, id, name("Verse"), Millis::new(5)).unwrap();
            id
        };

        let reopened = SqliteStore::open(&path).unwrap();
        assert_eq!(reopened.settings(id).unwrap(), Some(settings));
        assert_eq!(
            BookmarkRepository::list(&reopened, id)
                .unwrap()
                .labels
                .len(),
            1
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn labels_are_listed_by_position() {
        let (store, id) = store_with_recording();
        store
            .add_label(id, name("Late"), Millis::new(9_000))
            .unwrap();
        store
            .add_label(id, name("Early"), Millis::new(1_000))
            .unwrap();
        let labels = BookmarkRepository::list(&store, id).unwrap().labels;
        let names: Vec<&str> = labels.iter().map(|label| label.name.as_str()).collect();
        assert_eq!(names, ["Early", "Late"]);
        assert_eq!(labels[0].version, Version::FIRST);
    }

    #[test]
    fn update_with_the_current_version_advances_it() {
        let (store, id) = store_with_recording();
        let label = store.add_label(id, name("A"), Millis::new(1)).unwrap();
        let updated = store
            .update_label(label.id, label.version, name("B"), Millis::new(2))
            .unwrap();
        assert_eq!(updated.recording, id);
        assert_eq!(updated.item.version, Version::new(2));
        assert_eq!(updated.item.name.as_str(), "B");
    }

    #[test]
    fn update_with_a_stale_version_is_a_conflict_and_changes_nothing() {
        let (store, id) = store_with_recording();
        let region = store
            .add_region(id, name("Chorus"), range(1_000, 2_000))
            .unwrap();
        store
            .update_region(
                region.id,
                region.version,
                name("Chorus 1"),
                range(1_000, 2_500),
            )
            .unwrap();

        let stale = store.update_region(region.id, region.version, name("Other"), range(0, 1));
        assert!(matches!(stale, Err(BookmarkError::Conflict(v)) if v == Version::new(2)));
        let regions = BookmarkRepository::list(&store, id).unwrap().regions;
        assert_eq!(regions[0].name.as_str(), "Chorus 1");
    }

    #[test]
    fn deleting_needs_the_current_version() {
        let (store, id) = store_with_recording();
        let label = store.add_label(id, name("A"), Millis::new(1)).unwrap();
        assert!(matches!(
            store.delete_label(label.id, Version::new(7)),
            Err(BookmarkError::Conflict(_))
        ));
        assert_eq!(store.delete_label(label.id, label.version).unwrap(), id);
        assert!(matches!(
            store.delete_label(label.id, label.version),
            Err(BookmarkError::NotFound)
        ));
    }

    #[test]
    fn bookmarks_need_an_existing_recording() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(matches!(
            store.add_label(RecordingId::new(1), name("A"), Millis::ZERO),
            Err(BookmarkError::NotFound)
        ));
    }

    #[test]
    fn regions_may_overlap() {
        let (store, id) = store_with_recording();
        store.add_region(id, name("A"), range(0, 5_000)).unwrap();
        store
            .add_region(id, name("B"), range(2_000, 6_000))
            .unwrap();
        assert_eq!(
            BookmarkRepository::list(&store, id).unwrap().regions.len(),
            2
        );
    }

    fn sample_analysis(tempo: f64) -> RecordingAnalysis {
        RecordingAnalysis {
            tuning: TuningHz::new(442.5).unwrap(),
            tuning_confidence: Confidence::new(1.4),
            key: MusicalKey {
                tonic: PitchClass::from_semitones(1),
                mode: KeyMode::Minor,
            },
            key_confidence: Confidence::new(1.02),
            tempo: Bpm::new(tempo).unwrap(),
            tempo_confidence: Confidence::new(1.3),
        }
    }

    #[test]
    fn analysis_results_are_stored_and_replaced() {
        let (store, id) = store_with_recording();
        assert_eq!(store.analysis(id).unwrap(), None);
        store.save_analysis(id, sample_analysis(148.0)).unwrap();
        assert_eq!(store.analysis(id).unwrap(), Some(sample_analysis(148.0)));
        store.save_analysis(id, sample_analysis(120.0)).unwrap();
        assert_eq!(store.analyses().unwrap(), [(id, sample_analysis(120.0))]);
    }

    #[test]
    fn analysis_of_a_missing_recording_is_an_error() {
        let (store, _) = store_with_recording();
        assert!(
            store
                .save_analysis(RecordingId::new(99), sample_analysis(100.0))
                .is_err()
        );
    }

    #[test]
    fn only_recordings_with_audio_and_no_result_are_unanalysed() {
        let store = SqliteStore::in_memory().unwrap();
        let title = |text: &str| RecordingTitle::new(text).unwrap();
        store.add(title("no audio")).unwrap();
        let first = store
            .add_media(title("one"), Path::new("/a.wav"), "h1")
            .unwrap()
            .unwrap();
        let second = store
            .add_media(title("two"), Path::new("/b.wav"), "h2")
            .unwrap()
            .unwrap();
        store
            .save_analysis(first.id, sample_analysis(90.0))
            .unwrap();
        assert_eq!(store.unanalysed().unwrap(), [second.id]);
    }

    #[test]
    fn analysis_survives_reopening_the_file() {
        let path =
            std::env::temp_dir().join(format!("veltrunox-analysis-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let id = {
            let store = SqliteStore::open(&path).unwrap();
            let id = store.add(RecordingTitle::new("x").unwrap()).unwrap().id;
            store.save_analysis(id, sample_analysis(97.5)).unwrap();
            id
        };
        let reopened = SqliteStore::open(&path).unwrap();
        assert_eq!(reopened.analysis(id).unwrap(), Some(sample_analysis(97.5)));
        std::fs::remove_file(&path).unwrap();
    }
}
