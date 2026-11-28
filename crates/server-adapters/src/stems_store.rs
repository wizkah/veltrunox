//! The stem sets in SQLite (ADR-015).

use crate::sqlite::{SqliteStore, decode_peaks, encode_peaks, sql_int, storage, unsigned};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::PathBuf;
use veltrunox_core::{Millis, RecordingId, StemName, StemSource, Waveform, WaveformLevel};
use veltrunox_server_app::{Stem, StemRepository, StemSet, StemSources, StorageError};

/// The mix belongs to the stems that were selected.
fn reset_mix(connection: &Connection, recording: i64) -> Result<(), StorageError> {
    connection
        .execute(
            "UPDATE recordings SET stem_mix = NULL WHERE id = ?1",
            params![recording],
        )
        .map_err(storage)?;
    Ok(())
}

fn source_of(stored: &str) -> Result<StemSource, StorageError> {
    StemSource::new(stored).map_err(|error| StorageError(format!("stored stem source: {error}")))
}

impl StemRepository for SqliteStore {
    fn stem_set(&self, id: RecordingId) -> Result<StemSet, StorageError> {
        let connection = self.connection();
        let recording = sql_int(id.value())?;
        let selected = connection
            .query_row(
                "SELECT source, analysis FROM stem_sets WHERE recording_id = ?1 AND selected",
                params![recording],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(storage)?;
        let Some((source, analysis)) = selected else {
            return Ok(StemSet::default());
        };
        let mut statement = connection
            .prepare(
                "SELECT name, path, length_ms, sample_rate FROM stems \
                 WHERE recording_id = ?1 AND source = ?2 ORDER BY position",
            )
            .map_err(storage)?;
        let rows = statement
            .query_map(params![recording, source], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, u32>(3)?,
                ))
            })
            .map_err(storage)?;
        let mut levels_of = connection
            .prepare(
                "SELECT frames_per_block, peaks FROM stem_waveform_levels \
                 WHERE recording_id = ?1 AND source = ?2 AND position = ?3 ORDER BY level",
            )
            .map_err(storage)?;
        let mut stems = Vec::new();
        for (position, row) in rows.enumerate() {
            let (name, path, length, sample_rate) = row.map_err(storage)?;
            let levels = levels_of
                .query_map(params![recording, source, position as i64], |row| {
                    Ok((row.get::<_, u32>(0)?, row.get::<_, Vec<u8>>(1)?))
                })
                .map_err(storage)?
                .map(|level| {
                    let (frames_per_block, bytes) = level.map_err(storage)?;
                    Ok(WaveformLevel {
                        frames_per_block,
                        peaks: decode_peaks(&bytes)?,
                    })
                })
                .collect::<Result<Vec<_>, StorageError>>()?;
            stems.push(Stem {
                name: StemName::new(&name)
                    .map_err(|error| StorageError(format!("stored stem name: {error}")))?,
                path: PathBuf::from(path),
                length: Millis::new(unsigned(length)?),
                waveform: Waveform {
                    sample_rate,
                    levels,
                },
            });
        }
        Ok(StemSet {
            source: source_of(&source)?,
            stems,
            analysis,
        })
    }

    fn stem_sources(&self, id: RecordingId) -> Result<StemSources, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare(
                "SELECT source, selected FROM stem_sets WHERE recording_id = ?1 ORDER BY source",
            )
            .map_err(storage)?;
        let rows = statement
            .query_map(params![sql_int(id.value())?], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
            })
            .map_err(storage)?;
        let mut sources = StemSources::default();
        for row in rows {
            let (name, selected) = row.map_err(storage)?;
            let source = source_of(&name)?;
            if selected {
                sources.selected = Some(source.clone());
            }
            sources.all.push(source);
        }
        Ok(sources)
    }

    fn put_stem_set(&self, id: RecordingId, set: &StemSet) -> Result<(), StorageError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        let recording = sql_int(id.value())?;
        let source = set.source.as_str();
        transaction
            .execute(
                "DELETE FROM stem_sets WHERE recording_id = ?1 AND source = ?2",
                params![recording, source],
            )
            .map_err(storage)?;
        transaction
            .execute(
                "UPDATE stem_sets SET selected = 0 WHERE recording_id = ?1",
                params![recording],
            )
            .map_err(storage)?;
        transaction
            .execute(
                "INSERT INTO stem_sets (recording_id, source, selected, analysis) \
                 VALUES (?1, ?2, 1, ?3)",
                params![recording, source, set.analysis],
            )
            .map_err(storage)?;
        reset_mix(&transaction, recording)?;
        for (position, stem) in set.stems.iter().enumerate() {
            let path = stem
                .path
                .to_str()
                .ok_or_else(|| StorageError("file path is not valid text".into()))?;
            transaction
                .execute(
                    "INSERT INTO stems (recording_id, position, name, path, length_ms, \
                     sample_rate, source) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        recording,
                        position as i64,
                        stem.name.as_str(),
                        path,
                        sql_int(stem.length.value())?,
                        stem.waveform.sample_rate,
                        source
                    ],
                )
                .map_err(storage)?;
            for (level, data) in stem.waveform.levels.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO stem_waveform_levels (recording_id, position, level, \
                         frames_per_block, peaks, source) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            recording,
                            position as i64,
                            level as i64,
                            data.frames_per_block,
                            encode_peaks(&data.peaks),
                            source
                        ],
                    )
                    .map_err(storage)?;
            }
        }
        transaction.commit().map_err(storage)
    }

    fn select_stem_set(&self, id: RecordingId, source: &StemSource) -> Result<bool, StorageError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        let recording = sql_int(id.value())?;
        let known = transaction
            .query_row(
                "SELECT selected FROM stem_sets WHERE recording_id = ?1 AND source = ?2",
                params![recording, source.as_str()],
                |row| row.get::<_, bool>(0),
            )
            .optional()
            .map_err(storage)?;
        // Selecting what is selected keeps its mix.
        if known == Some(false) {
            transaction
                .execute(
                    "UPDATE stem_sets SET selected = (source = ?2) WHERE recording_id = ?1",
                    params![recording, source.as_str()],
                )
                .map_err(storage)?;
            reset_mix(&transaction, recording)?;
        }
        transaction.commit().map_err(storage)?;
        Ok(known.is_some())
    }

    fn remove_stem_sets(
        &self,
        id: RecordingId,
        source: Option<&StemSource>,
    ) -> Result<(), StorageError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        let recording = sql_int(id.value())?;
        let name = source.map(StemSource::as_str);
        let removed_selected = transaction
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM stem_sets WHERE recording_id = ?1 AND selected \
                 AND (?2 IS NULL OR source = ?2))",
                params![recording, name],
                |row| row.get::<_, bool>(0),
            )
            .map_err(storage)?;
        transaction
            .execute(
                "DELETE FROM stem_sets WHERE recording_id = ?1 AND (?2 IS NULL OR source = ?2)",
                params![recording, name],
            )
            .map_err(storage)?;
        if removed_selected {
            transaction
                .execute(
                    "UPDATE stem_sets SET selected = 1 WHERE recording_id = ?1 AND source = \
                     (SELECT min(source) FROM stem_sets WHERE recording_id = ?1)",
                    params![recording],
                )
                .map_err(storage)?;
            reset_mix(&transaction, recording)?;
        }
        transaction.commit().map_err(storage)
    }

    fn set_stem_analysis(
        &self,
        id: RecordingId,
        source: &StemSource,
        analysis: &str,
    ) -> Result<(), StorageError> {
        self.connection()
            .execute(
                "UPDATE stem_sets SET analysis = ?3 WHERE recording_id = ?1 AND source = ?2",
                params![sql_int(id.value())?, source.as_str(), analysis],
            )
            .map_err(storage)?;
        Ok(())
    }

    fn recordings_with_stems(&self) -> Result<Vec<RecordingId>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare("SELECT DISTINCT recording_id FROM stem_sets")
            .map_err(storage)?;
        let rows = statement
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(storage)?;
        rows.map(|row| Ok(RecordingId::new(unsigned(row.map_err(storage)?)?)))
            .collect()
    }

    fn stem_paths(&self) -> Result<Vec<PathBuf>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare("SELECT path FROM stems")
            .map_err(storage)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(storage)?;
        rows.map(|row| row.map(PathBuf::from).map_err(storage))
            .collect()
    }
}
