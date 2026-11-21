//! Translation between wire messages and domain types.

use crate::{BookmarksDto, ClientMessageDto, LabelDto, PlaybackStateDto, RangeDto, RegionDto};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Intent, InvalidValue, KeyShift, Label, Millis,
    PlaybackSettings, PlaybackState, PositionReference, RecordingId, Region, Speed, TimeRange,
    TuningHz, Version,
};

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
        loop_range: state.loop_range.map(range_to_dto),
        scrubbing: state.scrubbing,
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
        loop_range: dto.loop_range.map(range_from_dto).transpose()?,
        scrubbing: dto.scrubbing,
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
        Intent::SetLoop(range) => ClientMessageDto::SetLoop {
            loop_range: range.map(range_to_dto),
        },
    }
}

/// The intent a message carries after validation; `None` for messages that are not intents.
pub fn intent_from_message(message: &ClientMessageDto) -> Result<Option<Intent>, InvalidValue> {
    Ok(Some(match message {
        ClientMessageDto::Ping { .. } => return Ok(None),
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
        ClientMessageDto::SetLoop { loop_range } => {
            Intent::SetLoop(loop_range.map(range_from_dto).transpose()?)
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

pub fn bookmarks_from_dto(dto: &BookmarksDto) -> Result<Bookmarks, InvalidValue> {
    let labels = dto
        .labels
        .iter()
        .map(|label| {
            Ok(Label {
                id: BookmarkId::new(label.id),
                name: BookmarkName::new(&label.name)?,
                position: Millis::position(label.position_ms)?,
                version: Version::new(label.version),
            })
        })
        .collect::<Result<_, InvalidValue>>()?;
    let regions = dto
        .regions
        .iter()
        .map(|region| {
            Ok(Region {
                id: BookmarkId::new(region.id),
                name: BookmarkName::new(&region.name)?,
                range: range_from_dto(region.range)?,
                version: Version::new(region.version),
            })
        })
        .collect::<Result<_, InvalidValue>>()?;
    Ok(Bookmarks { labels, regions })
}

#[cfg(test)]
mod tests {
    use super::*;

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
            loop_range: Some(TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap()),
            scrubbing: true,
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
            Intent::SetLoop(None),
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
        let backwards = ClientMessageDto::SetLoop {
            loop_range: Some(RangeDto {
                start_ms: 5,
                end_ms: 2,
            }),
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
    fn a_ping_is_not_an_intent() {
        let ping = ClientMessageDto::Ping { client_time_ms: 1 };
        assert_eq!(intent_from_message(&ping), Ok(None));
    }
}
