//! The stem sets in SQLite (ADR-015).

use crate::sqlite::{SqliteStore, decode_peaks, encode_peaks, sql_int, storage, unsigned};
use rusqlite::{OptionalExtension, params};
use std::path::PathBuf;
use veltrunox_core::{Millis, RecordingId, StemName, Waveform, WaveformLevel};
use veltrunox_server_app::{Stem, StemRepository, StemSet, StorageError};

impl StemRepository for SqliteStore {
    fn stem_set(&self, id: RecordingId) -> Result<StemSet, StorageError> {
        let connection = self.connection();
        let recording = sql_int(id.value())?;
        let analysis = connection
            .query_row(
                "SELECT analysis FROM stem_sets WHERE recording_id = ?1",
                params![recording],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(storage)?
            .flatten();
        let mut statement = connection
            .prepare(
                "SELECT name, path, length_ms, sample_rate FROM stems \
                 WHERE recording_id = ?1 ORDER BY position",
            )
            .map_err(storage)?;
        let rows = statement
            .query_map(params![recording], |row| {
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
                 WHERE recording_id = ?1 AND position = ?2 ORDER BY level",
            )
            .map_err(storage)?;
        let mut stems = Vec::new();
        for (position, row) in rows.enumerate() {
            let (name, path, length, sample_rate) = row.map_err(storage)?;
            let levels = levels_of
                .query_map(params![recording, position as i64], |row| {
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
        Ok(StemSet { stems, analysis })
    }

    fn replace_stem_set(&self, id: RecordingId, set: &StemSet) -> Result<(), StorageError> {
        let mut connection = self.connection();
        let transaction = connection.transaction().map_err(storage)?;
        let recording = sql_int(id.value())?;
        transaction
            .execute(
                "DELETE FROM stem_sets WHERE recording_id = ?1",
                params![recording],
            )
            .map_err(storage)?;
        // The mix belonged to the old stems.
        transaction
            .execute(
                "UPDATE recordings SET stem_mix = NULL WHERE id = ?1",
                params![recording],
            )
            .map_err(storage)?;
        if !set.stems.is_empty() {
            transaction
                .execute(
                    "INSERT INTO stem_sets (recording_id, analysis) VALUES (?1, ?2)",
                    params![recording, set.analysis],
                )
                .map_err(storage)?;
        }
        for (position, stem) in set.stems.iter().enumerate() {
            let path = stem
                .path
                .to_str()
                .ok_or_else(|| StorageError("file path is not valid text".into()))?;
            transaction
                .execute(
                    "INSERT INTO stems (recording_id, position, name, path, length_ms, \
                     sample_rate) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        recording,
                        position as i64,
                        stem.name.as_str(),
                        path,
                        sql_int(stem.length.value())?,
                        stem.waveform.sample_rate
                    ],
                )
                .map_err(storage)?;
            for (level, data) in stem.waveform.levels.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO stem_waveform_levels (recording_id, position, level, \
                         frames_per_block, peaks) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            recording,
                            position as i64,
                            level as i64,
                            data.frames_per_block,
                            encode_peaks(&data.peaks)
                        ],
                    )
                    .map_err(storage)?;
            }
        }
        transaction.commit().map_err(storage)
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
