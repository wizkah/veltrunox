//! Translation between wire messages and domain types.

use crate::{
    AnalysisProgressDto, AnalysisResultDto, BookmarksDto, ClientMessageDto, ImportBookmarksDto,
    LabelDto, PlayModeDto, PlaybackStateDto, RangeDto, RecordingSummaryDto, RegionDto,
    ReplaceBookmarksDto, WaveformInfoDto, WaveformLevelInfoDto,
};
use veltrunox_core::{
    AnalysisInfo, AnalysisProgress, BookmarkId, BookmarkName, Bookmarks, Bpm, Confidence, EntryRef,
    ImportedBookmarks, ImportedLabel, ImportedRegion, Intent, InvalidRecording, InvalidValue,
    KeyMode, KeyShift, Label, LibraryEntry, Millis, MusicalKey, NewBookmarks, NewLabel, NewRegion,
    Peak, PitchClass, PlayMode, PlaybackSettings, PlaybackState, PositionReference, Recording,
    RecordingAnalysis, RecordingId, RecordingTitle, Region, Speed, TimeRange, TuningHz, Version,
    Volume, Waveform,
};

/// Describes the levels of a waveform without their peaks.
pub fn waveform_info_to_dto(length: Millis, waveform: &Waveform) -> WaveformInfoDto {
    WaveformInfoDto {
        length_ms: length.value(),
        sample_rate: waveform.sample_rate,
        levels: waveform
            .levels
            .iter()
            .map(|level| WaveformLevelInfoDto {
                frames_per_block: level.frames_per_block,
                blocks: u32::try_from(level.peaks.len()).unwrap_or(u32::MAX),
            })
            .collect(),
    }
}

/// Wire form of the peaks of one level: per block the minimum, then the maximum, as signed bytes.
pub fn peaks_to_bytes(peaks: &[Peak]) -> Vec<u8> {
    peaks
        .iter()
        .flat_map(|peak| [peak.min.to_le_bytes()[0], peak.max.to_le_bytes()[0]])
        .collect()
}

/// `None` if the bytes do not hold whole blocks.
pub fn peaks_from_bytes(bytes: &[u8]) -> Option<Vec<Peak>> {
    let (pairs, rest) = bytes.as_chunks::<2>();
    rest.is_empty().then(|| {
        pairs
            .iter()
            .map(|[min, max]| Peak {
                min: i8::from_le_bytes([*min]),
                max: i8::from_le_bytes([*max]),
            })
            .collect()
    })
}

pub fn analysis_to_dto(analysis: &RecordingAnalysis) -> AnalysisResultDto {
    AnalysisResultDto {
        tuning_hz: analysis.tuning.hertz(),
        tuning_confidence: analysis.tuning_confidence.ratio(),
        key_tonic: analysis.key.tonic.index(),
        key_minor: analysis.key.mode == KeyMode::Minor,
        key_confidence: analysis.key_confidence.ratio(),
        tempo_bpm: analysis.tempo.value(),
        tempo_confidence: analysis.tempo_confidence.ratio(),
    }
}

/// Validates the values of an analysis received from outside.
pub fn analysis_from_dto(dto: &AnalysisResultDto) -> Result<RecordingAnalysis, InvalidValue> {
    if dto.key_tonic >= PitchClass::COUNT {
        return Err(InvalidValue::KeyTonicOutOfRange);
    }
    Ok(RecordingAnalysis {
        tuning: TuningHz::new(dto.tuning_hz)?,
        tuning_confidence: Confidence::new(dto.tuning_confidence),
        key: MusicalKey {
            tonic: PitchClass::from_semitones(i32::from(dto.key_tonic)),
            mode: if dto.key_minor {
                KeyMode::Minor
            } else {
                KeyMode::Major
            },
        },
        key_confidence: Confidence::new(dto.key_confidence),
        tempo: Bpm::new(dto.tempo_bpm)?,
        tempo_confidence: Confidence::new(dto.tempo_confidence),
    })
}

pub fn library_entry_to_dto(entry: &LibraryEntry) -> RecordingSummaryDto {
    RecordingSummaryDto {
        id: entry.recording.id.value(),
        title: entry.recording.title.as_str().to_owned(),
        length_ms: entry.length.map(Millis::value),
        analysis_progress: match &entry.analysis.progress {
            AnalysisProgress::None => AnalysisProgressDto::None,
            AnalysisProgress::Queued => AnalysisProgressDto::Queued,
            AnalysisProgress::Running(percent) => {
                AnalysisProgressDto::Running { percent: *percent }
            }
            AnalysisProgress::Failed(code) => AnalysisProgressDto::Failed { code: code.clone() },
            AnalysisProgress::Done => AnalysisProgressDto::Done,
        },
        analysis: entry.analysis.result.as_ref().map(analysis_to_dto),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidEntry {
    #[error(transparent)]
    Title(#[from] InvalidRecording),
    #[error(transparent)]
    Value(#[from] InvalidValue),
}

pub fn library_entry_from_dto(dto: &RecordingSummaryDto) -> Result<LibraryEntry, InvalidEntry> {
    Ok(LibraryEntry {
        recording: Recording {
            id: RecordingId::new(dto.id),
            title: RecordingTitle::new(&dto.title)?,
        },
        length: dto.length_ms.map(Millis::position).transpose()?,
        analysis: AnalysisInfo {
            progress: match &dto.analysis_progress {
                AnalysisProgressDto::None => AnalysisProgress::None,
                AnalysisProgressDto::Queued => AnalysisProgress::Queued,
                AnalysisProgressDto::Running { percent } => {
                    AnalysisProgress::Running((*percent).min(100))
                }
                AnalysisProgressDto::Failed { code } => AnalysisProgress::Failed(code.clone()),
                AnalysisProgressDto::Done => AnalysisProgress::Done,
            },
            result: dto.analysis.as_ref().map(analysis_from_dto).transpose()?,
        },
    })
}

pub fn range_to_dto(range: TimeRange) -> RangeDto {
    RangeDto {
        start_ms: range.start().value(),
        end_ms: range.end().value(),
    }
}

/// Validates positions and ordering of a range received from outside.
pub fn range_from_dto(dto: RangeDto) -> Result<TimeRange, InvalidValue> {
    TimeRange::new(
        Millis::position(dto.start_ms)?,
        Millis::position(dto.end_ms)?,
    )
}

pub fn play_mode_to_dto(mode: PlayMode) -> PlayModeDto {
    match mode {
        PlayMode::ToEnd => PlayModeDto::ToEnd,
        PlayMode::ToRegionEnd => PlayModeDto::ToRegionEnd,
        PlayMode::Loop => PlayModeDto::Loop,
    }
}

pub fn play_mode_from_dto(dto: PlayModeDto) -> PlayMode {
    match dto {
        PlayModeDto::ToEnd => PlayMode::ToEnd,
        PlayModeDto::ToRegionEnd => PlayMode::ToRegionEnd,
        PlayModeDto::Loop => PlayMode::Loop,
    }
}

pub fn state_to_dto(state: &PlaybackState) -> PlaybackStateDto {
    PlaybackStateDto {
        revision: state.revision,
        recording_id: state.recording.map(RecordingId::value),
        position_ms: state.reference.position.value(),
        position_server_time_ms: state.reference.server_time.value(),
        running: state.running,
        speed: state.settings.speed.factor(),
        tuning_hz: state.settings.tuning.hertz(),
        key_shift_semitones: state.settings.key.semitones(),
        play_range: state.play_range.map(range_to_dto),
        play_mode: play_mode_to_dto(state.play_mode),
        scrubbing: state.scrubbing,
        volume_percent: state.volume.percent(),
    }
}

pub fn state_from_dto(dto: &PlaybackStateDto) -> Result<PlaybackState, InvalidValue> {
    Ok(PlaybackState {
        revision: dto.revision,
        recording: dto.recording_id.map(RecordingId::new),
        reference: PositionReference {
            position: Millis::position(dto.position_ms)?,
            server_time: Millis::new(dto.position_server_time_ms),
        },
        running: dto.running,
        settings: PlaybackSettings {
            speed: Speed::new(dto.speed)?,
            tuning: TuningHz::new(dto.tuning_hz)?,
            key: KeyShift::new(dto.key_shift_semitones)?,
        },
        play_range: dto.play_range.map(range_from_dto).transpose()?,
        play_mode: play_mode_from_dto(dto.play_mode),
        scrubbing: dto.scrubbing,
        volume: Volume::new(dto.volume_percent)?,
    })
}

pub fn intent_to_message(intent: Intent) -> ClientMessageDto {
    match intent {
        Intent::SelectRecording(id) => ClientMessageDto::SelectRecording {
            recording_id: id.value(),
        },
        Intent::Play => ClientMessageDto::Play,
        Intent::Pause => ClientMessageDto::Pause,
        Intent::Seek {
            position,
            scrubbing,
        } => ClientMessageDto::Seek {
            position_ms: position.value(),
            scrubbing,
        },
        Intent::SetSpeed(speed) => ClientMessageDto::SetSpeed {
            speed: speed.factor(),
        },
        Intent::SetTuning(tuning) => ClientMessageDto::SetTuning {
            tuning_hz: tuning.hertz(),
        },
        Intent::SetKey(key) => ClientMessageDto::SetKey {
            key_shift_semitones: key.semitones(),
        },
        Intent::PlayRegion(range) => ClientMessageDto::PlayRegion {
            range: range_to_dto(range),
        },
        Intent::SetPlayMode(mode) => ClientMessageDto::SetPlayMode {
            play_mode: play_mode_to_dto(mode),
        },
        Intent::SetVolume(volume) => ClientMessageDto::SetVolume {
            volume_percent: volume.percent(),
        },
    }
}

/// The intent a message carries after validation; `None` for messages that are not intents.
pub fn intent_from_message(message: &ClientMessageDto) -> Result<Option<Intent>, InvalidValue> {
    Ok(Some(match message {
        ClientMessageDto::Ping { .. } | ClientMessageDto::Auth { .. } => return Ok(None),
        ClientMessageDto::SelectRecording { recording_id } => {
            Intent::SelectRecording(RecordingId::new(*recording_id))
        }
        ClientMessageDto::Play => Intent::Play,
        ClientMessageDto::Pause => Intent::Pause,
        ClientMessageDto::Seek {
            position_ms,
            scrubbing,
        } => Intent::Seek {
            position: Millis::position(*position_ms)?,
            scrubbing: *scrubbing,
        },
        ClientMessageDto::SetSpeed { speed } => Intent::SetSpeed(Speed::new(*speed)?),
        ClientMessageDto::SetTuning { tuning_hz } => Intent::SetTuning(TuningHz::new(*tuning_hz)?),
        ClientMessageDto::SetKey {
            key_shift_semitones,
        } => Intent::SetKey(KeyShift::new(*key_shift_semitones)?),
        ClientMessageDto::PlayRegion { range } => Intent::PlayRegion(range_from_dto(*range)?),
        ClientMessageDto::SetPlayMode { play_mode } => {
            Intent::SetPlayMode(play_mode_from_dto(*play_mode))
        }
        ClientMessageDto::SetVolume { volume_percent } => {
            Intent::SetVolume(Volume::new(*volume_percent)?)
        }
    }))
}

pub fn label_to_dto(label: &Label) -> LabelDto {
    LabelDto {
        id: label.id.value(),
        version: label.version.value(),
        name: label.name.as_str().to_owned(),
        position_ms: label.position.value(),
    }
}

pub fn region_to_dto(region: &Region) -> RegionDto {
    RegionDto {
        id: region.id.value(),
        version: region.version.value(),
        name: region.name.as_str().to_owned(),
        range: range_to_dto(region.range),
    }
}

pub fn bookmarks_to_dto(bookmarks: &Bookmarks) -> BookmarksDto {
    BookmarksDto {
        labels: bookmarks.labels.iter().map(label_to_dto).collect(),
        regions: bookmarks.regions.iter().map(region_to_dto).collect(),
    }
}

pub fn label_from_dto(dto: &LabelDto) -> Result<Label, InvalidValue> {
    Ok(Label {
        id: BookmarkId::new(dto.id),
        name: BookmarkName::new(&dto.name)?,
        position: Millis::position(dto.position_ms)?,
        version: Version::new(dto.version),
    })
}

pub fn region_from_dto(dto: &RegionDto) -> Result<Region, InvalidValue> {
    Ok(Region {
        id: BookmarkId::new(dto.id),
        name: BookmarkName::new(&dto.name)?,
        range: range_from_dto(dto.range)?,
        version: Version::new(dto.version),
    })
}

pub fn bookmarks_from_dto(dto: &BookmarksDto) -> Result<Bookmarks, InvalidValue> {
    Ok(Bookmarks {
        labels: dto
            .labels
            .iter()
            .map(label_from_dto)
            .collect::<Result<_, _>>()?,
        regions: dto
            .regions
            .iter()
            .map(region_from_dto)
            .collect::<Result<_, _>>()?,
    })
}

/// An entry of an import that is not valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryIssue {
    pub entry: EntryRef,
    pub error: InvalidValue,
}

/// Names an entry the way an import answer does: `labels[2]`, `regions[0]`.
pub fn entry_path(entry: EntryRef) -> String {
    match entry {
        EntryRef::Label(index) => format!("labels[{index}]"),
        EntryRef::Region(index) => format!("regions[{index}]"),
    }
}

/// An entry that names a stored entry by id and version; one with only half of that is new.
fn known_entry(id: Option<u64>, version: Option<u64>) -> Option<(BookmarkId, Version)> {
    Some((BookmarkId::new(id?), Version::new(version?)))
}

/// Validates every entry of an import and reports all that are invalid, not only the first.
pub fn imported_bookmarks_from_dto(
    dto: &ImportBookmarksDto,
) -> Result<ImportedBookmarks, Vec<EntryIssue>> {
    let mut issues = Vec::new();
    let mut imported = ImportedBookmarks::default();
    for (index, label) in dto.labels.iter().enumerate() {
        let converted = BookmarkName::new(&label.name)
            .and_then(|name| Ok((name, Millis::position(label.position_ms)?)));
        match converted {
            Ok((name, position)) => imported.labels.push(ImportedLabel {
                known: known_entry(label.id, label.version),
                label: NewLabel { name, position },
            }),
            Err(error) => issues.push(EntryIssue {
                entry: EntryRef::Label(index),
                error,
            }),
        }
    }
    for (index, region) in dto.regions.iter().enumerate() {
        let converted = BookmarkName::new(&region.name)
            .and_then(|name| Ok((name, range_from_dto(region.range)?)));
        match converted {
            Ok((name, range)) => imported.regions.push(ImportedRegion {
                known: known_entry(region.id, region.version),
                region: NewRegion { name, range },
            }),
            Err(error) => issues.push(EntryIssue {
                entry: EntryRef::Region(index),
                error,
            }),
        }
    }
    if issues.is_empty() {
        Ok(imported)
    } else {
        Err(issues)
    }
}

/// Validates the names, positions and ranges of a replacement set received from outside.
pub fn new_bookmarks_from_dto(dto: &ReplaceBookmarksDto) -> Result<NewBookmarks, InvalidValue> {
    Ok(NewBookmarks {
        labels: dto
            .labels
            .iter()
            .map(|label| {
                Ok(NewLabel {
                    name: BookmarkName::new(&label.name)?,
                    position: Millis::position(label.position_ms)?,
                })
            })
            .collect::<Result<_, InvalidValue>>()?,
        regions: dto
            .regions
            .iter()
            .map(|region| {
                Ok(NewRegion {
                    name: BookmarkName::new(&region.name)?,
                    range: range_from_dto(region.range)?,
                })
            })
            .collect::<Result<_, InvalidValue>>()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peaks_round_trip_through_bytes_and_odd_lengths_are_refused() {
        let peaks = [Peak { min: -127, max: 5 }, Peak { min: 0, max: 127 }];
        let bytes = peaks_to_bytes(&peaks);
        assert_eq!(bytes, [129, 5, 0, 127]);
        assert_eq!(peaks_from_bytes(&bytes), Some(peaks.to_vec()));
        assert_eq!(peaks_from_bytes(&[1, 2, 3]), None);
    }

    #[test]
    fn the_waveform_info_lists_the_levels_with_their_block_counts() {
        let waveform = Waveform::from_mono(&vec![0.1; 1024 * 300], 44_100);
        let info = waveform_info_to_dto(Millis::new(6_966), &waveform);
        assert_eq!(info.length_ms, 6_966);
        assert_eq!(info.sample_rate, 44_100);
        let shape: Vec<_> = info
            .levels
            .iter()
            .map(|level| (level.frames_per_block, level.blocks))
            .collect();
        assert_eq!(shape, [(2048, 150), (1024, 300)]);
    }

    #[test]
    fn a_library_entry_round_trips_through_its_wire_form() {
        let entry = LibraryEntry {
            recording: Recording {
                id: RecordingId::new(7),
                title: RecordingTitle::new("Intro").unwrap(),
            },
            length: Some(Millis::new(61_000)),
            analysis: AnalysisInfo {
                progress: AnalysisProgress::Done,
                result: Some(RecordingAnalysis {
                    tuning: TuningHz::new(441.5).unwrap(),
                    tuning_confidence: Confidence::new(1.4),
                    key: MusicalKey {
                        tonic: PitchClass::from_semitones(9),
                        mode: KeyMode::Minor,
                    },
                    key_confidence: Confidence::new(1.02),
                    tempo: Bpm::new(96.5).unwrap(),
                    tempo_confidence: Confidence::new(1.2),
                }),
            },
        };
        let dto = library_entry_to_dto(&entry);
        assert_eq!(library_entry_from_dto(&dto).unwrap(), entry);
    }

    #[test]
    fn invalid_analysis_values_are_refused() {
        let mut dto = library_entry_to_dto(&LibraryEntry {
            recording: Recording {
                id: RecordingId::new(1),
                title: RecordingTitle::new("x").unwrap(),
            },
            length: None,
            analysis: AnalysisInfo::default(),
        });
        dto.analysis = Some(AnalysisResultDto {
            tuning_hz: 440.0,
            tuning_confidence: 1.0,
            key_tonic: 12,
            key_minor: false,
            key_confidence: 1.0,
            tempo_bpm: 100.0,
            tempo_confidence: 1.0,
        });
        assert!(library_entry_from_dto(&dto).is_err());
        dto.title = "  ".into();
        dto.analysis = None;
        assert!(library_entry_from_dto(&dto).is_err());
    }

    #[test]
    fn state_survives_the_round_trip() {
        let state = PlaybackState {
            revision: 9,
            recording: Some(RecordingId::new(4)),
            reference: PositionReference {
                position: Millis::new(1_200),
                server_time: Millis::new(77_000),
            },
            running: true,
            settings: PlaybackSettings {
                speed: Speed::new(0.75).unwrap(),
                tuning: TuningHz::new(432.0).unwrap(),
                key: KeyShift::new(2).unwrap(),
            },
            play_range: Some(TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap()),
            play_mode: PlayMode::Loop,
            scrubbing: true,
            volume: Volume::new(35).unwrap(),
        };
        assert_eq!(state_from_dto(&state_to_dto(&state)), Ok(state));
    }

    #[test]
    fn intents_survive_the_round_trip() {
        let intents = [
            Intent::SelectRecording(RecordingId::new(3)),
            Intent::Play,
            Intent::Pause,
            Intent::Seek {
                position: Millis::new(900),
                scrubbing: true,
            },
            Intent::SetSpeed(Speed::new(1.5).unwrap()),
            Intent::SetTuning(TuningHz::new(415.0).unwrap()),
            Intent::SetKey(KeyShift::new(-5).unwrap()),
            Intent::PlayRegion(TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap()),
            Intent::SetPlayMode(PlayMode::ToRegionEnd),
            Intent::SetVolume(Volume::new(80).unwrap()),
        ];
        for intent in intents {
            assert_eq!(
                intent_from_message(&intent_to_message(intent)),
                Ok(Some(intent))
            );
        }
    }

    #[test]
    fn invalid_values_are_rejected_at_the_boundary() {
        let too_fast = ClientMessageDto::SetSpeed { speed: 5.0 };
        assert_eq!(
            intent_from_message(&too_fast),
            Err(InvalidValue::SpeedOutOfRange)
        );
        let far_away = ClientMessageDto::Seek {
            position_ms: u64::MAX,
            scrubbing: false,
        };
        assert_eq!(
            intent_from_message(&far_away),
            Err(InvalidValue::PositionOutOfRange)
        );
        let too_loud = ClientMessageDto::SetVolume {
            volume_percent: 101,
        };
        assert_eq!(
            intent_from_message(&too_loud),
            Err(InvalidValue::VolumeOutOfRange)
        );
        let backwards = ClientMessageDto::PlayRegion {
            range: RangeDto {
                start_ms: 5,
                end_ms: 2,
            },
        };
        assert_eq!(
            intent_from_message(&backwards),
            Err(InvalidValue::EmptyRange)
        );
    }

    #[test]
    fn bookmarks_survive_the_round_trip() {
        let bookmarks = Bookmarks {
            labels: vec![Label {
                id: BookmarkId::new(1),
                name: BookmarkName::new("Verse").unwrap(),
                position: Millis::new(500),
                version: Version::new(2),
            }],
            regions: vec![Region {
                id: BookmarkId::new(3),
                name: BookmarkName::new("Chorus").unwrap(),
                range: TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap(),
                version: Version::FIRST,
            }],
        };
        assert_eq!(
            bookmarks_from_dto(&bookmarks_to_dto(&bookmarks)),
            Ok(bookmarks)
        );
    }

    #[test]
    fn a_replacement_set_is_validated_as_a_whole() {
        let valid = ReplaceBookmarksDto {
            labels: vec![crate::NewLabelDto {
                name: " Verse ".into(),
                position_ms: 5,
            }],
            regions: vec![crate::NewRegionDto {
                name: "Chorus".into(),
                range: RangeDto {
                    start_ms: 10,
                    end_ms: 20,
                },
            }],
        };
        let converted = new_bookmarks_from_dto(&valid).unwrap();
        assert_eq!(converted.labels[0].name.as_str(), "Verse");
        assert_eq!(converted.regions[0].range.end(), Millis::new(20));

        let mut backwards = valid.clone();
        backwards.regions[0].range.end_ms = 10;
        assert_eq!(
            new_bookmarks_from_dto(&backwards),
            Err(InvalidValue::EmptyRange)
        );
        let mut unnamed = valid;
        unnamed.labels[0].name = " ".into();
        assert_eq!(
            new_bookmarks_from_dto(&unnamed),
            Err(InvalidValue::EmptyName)
        );
    }

    #[test]
    fn an_import_reports_every_invalid_entry_with_its_place() {
        use crate::{ImportLabelDto, ImportRegionDto};
        let dto = ImportBookmarksDto {
            labels: vec![
                ImportLabelDto {
                    id: Some(4),
                    version: Some(2),
                    name: "Verse".into(),
                    position_ms: 5,
                },
                ImportLabelDto {
                    id: None,
                    version: None,
                    name: " ".into(),
                    position_ms: 5,
                },
            ],
            regions: vec![ImportRegionDto {
                id: Some(1),
                version: None,
                name: "Chorus".into(),
                range: RangeDto {
                    start_ms: 9,
                    end_ms: 3,
                },
            }],
        };
        let issues = imported_bookmarks_from_dto(&dto).unwrap_err();
        assert_eq!(
            issues,
            vec![
                EntryIssue {
                    entry: EntryRef::Label(1),
                    error: InvalidValue::EmptyName
                },
                EntryIssue {
                    entry: EntryRef::Region(0),
                    error: InvalidValue::EmptyRange
                },
            ]
        );
        assert_eq!(entry_path(issues[0].entry), "labels[1]");

        let valid = ImportBookmarksDto {
            labels: vec![dto.labels[0].clone()],
            regions: vec![ImportRegionDto {
                range: RangeDto {
                    start_ms: 3,
                    end_ms: 9,
                },
                ..dto.regions[0].clone()
            }],
        };
        let imported = imported_bookmarks_from_dto(&valid).unwrap();
        assert_eq!(
            imported.labels[0].known,
            Some((BookmarkId::new(4), Version::new(2)))
        );
        // An id without a version does not tie the entry to a stored one.
        assert_eq!(imported.regions[0].known, None);
    }

    #[test]
    fn a_ping_is_not_an_intent() {
        let ping = ClientMessageDto::Ping { client_time_ms: 1 };
        assert_eq!(intent_from_message(&ping), Ok(None));
    }
}
