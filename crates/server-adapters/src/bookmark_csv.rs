//! Bookmarks as a flat CSV table, the format for spreadsheets. One row per label or region:
//! `recording_id,type,name,start_ms,end_ms,id,version`. A label has no `end_ms`; `id` and
//! `version` tie a row to a stored bookmark and may be left out.

use std::collections::HashMap;
use veltrunox_protocol::{
    ExportDto, ImportBookmarksDto, ImportDto, ImportLabelDto, ImportRecordingDto, ImportRegionDto,
    RangeDto, RecordingRefDto,
};

const HEADER: [&str; 7] = [
    "recording_id",
    "type",
    "name",
    "start_ms",
    "end_ms",
    "id",
    "version",
];

/// A row that could not be read; `line` counts from 1 including the header.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {problem}")]
pub struct CsvError {
    pub line: u64,
    pub problem: String,
}

pub fn export_to_csv(export: &ExportDto) -> String {
    let mut writer = csv::Writer::from_writer(Vec::new());
    let mut write = |row: [String; 7]| {
        // Writing into memory cannot fail.
        let _ = writer.write_record(row);
    };
    write(HEADER.map(str::to_owned));
    for exported in &export.recordings {
        let recording = exported.recording.id.to_string();
        for label in &exported.bookmarks.labels {
            write([
                recording.clone(),
                "label".into(),
                label.name.clone(),
                label.position_ms.to_string(),
                String::new(),
                label.id.to_string(),
                label.version.to_string(),
            ]);
        }
        for region in &exported.bookmarks.regions {
            write([
                recording.clone(),
                "region".into(),
                region.name.clone(),
                region.range.start_ms.to_string(),
                region.range.end_ms.to_string(),
                region.id.to_string(),
                region.version.to_string(),
            ]);
        }
    }
    let bytes = writer.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

/// Groups the rows by recording, in the order the recordings first appear. A recording that
/// has no row is not part of the import and stays as it is.
pub fn import_from_csv(text: &str) -> Result<ImportDto, CsvError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let columns = Columns::read(reader.headers().map_err(|error| failure(1, &error))?)?;
    let mut order: Vec<u64> = Vec::new();
    let mut sets: HashMap<u64, ImportBookmarksDto> = HashMap::new();
    for record in reader.records() {
        let record = record.map_err(|error| failure(1, &error))?;
        let line = record.position().map_or(0, csv::Position::line);
        let row = columns.row(&record, line)?;
        let set = sets.entry(row.recording).or_insert_with(|| {
            order.push(row.recording);
            ImportBookmarksDto::default()
        });
        match row.kind {
            Kind::Label => set.labels.push(ImportLabelDto {
                id: row.id,
                version: row.version,
                name: row.name,
                position_ms: row.start,
            }),
            Kind::Region => set.regions.push(ImportRegionDto {
                id: row.id,
                version: row.version,
                name: row.name,
                range: RangeDto {
                    start_ms: row.start,
                    end_ms: row
                        .end
                        .ok_or_else(|| problem(line, "a region needs end_ms"))?,
                },
            }),
        }
    }
    Ok(ImportDto {
        format_version: veltrunox_protocol::EXPORT_FORMAT_VERSION,
        recordings: order
            .into_iter()
            .filter_map(|id| {
                sets.remove(&id).map(|bookmarks| ImportRecordingDto {
                    recording: RecordingRefDto { id },
                    bookmarks,
                })
            })
            .collect(),
    })
}

fn failure(line: u64, error: &dyn std::fmt::Display) -> CsvError {
    problem(line, &error.to_string())
}

fn problem(line: u64, text: &str) -> CsvError {
    CsvError {
        line,
        problem: text.to_owned(),
    }
}

enum Kind {
    Label,
    Region,
}

struct Row {
    recording: u64,
    kind: Kind,
    name: String,
    start: u64,
    end: Option<u64>,
    id: Option<u64>,
    version: Option<u64>,
}

/// Where the columns are, found by their names so their order does not matter.
struct Columns {
    recording: usize,
    kind: usize,
    name: usize,
    start: usize,
    end: Option<usize>,
    id: Option<usize>,
    version: Option<usize>,
}

impl Columns {
    fn read(header: &csv::StringRecord) -> Result<Self, CsvError> {
        let find = |name: &str| {
            header
                .iter()
                .position(|cell| cell.eq_ignore_ascii_case(name))
        };
        let need = |name: &str| {
            find(name).ok_or_else(|| problem(1, &format!("the column {name} is missing")))
        };
        Ok(Self {
            recording: need("recording_id")?,
            kind: need("type")?,
            name: need("name")?,
            start: need("start_ms")?,
            end: find("end_ms"),
            id: find("id"),
            version: find("version"),
        })
    }

    fn row(&self, record: &csv::StringRecord, line: u64) -> Result<Row, CsvError> {
        let cell = |index: usize| record.get(index).unwrap_or("");
        let number = |index: usize, name: &str| {
            cell(index)
                .parse::<u64>()
                .map_err(|_| problem(line, &format!("{name} is not a whole number")))
        };
        let optional = |index: Option<usize>, name: &str| match index.map(cell) {
            None | Some("") => Ok(None),
            Some(_) => number(index.unwrap_or_default(), name).map(Some),
        };
        let kind = match cell(self.kind).to_ascii_lowercase().as_str() {
            "label" => Kind::Label,
            "region" => Kind::Region,
            _ => return Err(problem(line, "type must be label or region")),
        };
        Ok(Row {
            recording: number(self.recording, "recording_id")?,
            kind,
            name: cell(self.name).to_owned(),
            start: number(self.start, "start_ms")?,
            end: optional(self.end, "end_ms")?,
            id: optional(self.id, "id")?,
            version: optional(self.version, "version")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_protocol::{
        AnalysisProgressDto, BookmarksDto, ExportedRecordingDto, LabelDto, RecordingSummaryDto,
        RegionDto,
    };

    fn exported() -> ExportDto {
        ExportDto {
            format_version: 1,
            recordings: vec![ExportedRecordingDto {
                recording: RecordingSummaryDto {
                    id: 3,
                    title: "Song".into(),
                    length_ms: None,
                    analysis_progress: AnalysisProgressDto::None,
                    analysis: None,
                },
                bookmarks: BookmarksDto {
                    labels: vec![LabelDto {
                        id: 5,
                        version: 2,
                        name: "Intro, \"slow\"".into(),
                        position_ms: 1_500,
                    }],
                    regions: vec![RegionDto {
                        id: 6,
                        version: 1,
                        name: "Chorus".into(),
                        range: RangeDto {
                            start_ms: 10,
                            end_ms: 20,
                        },
                    }],
                },
            }],
        }
    }

    #[test]
    fn an_export_survives_the_trip_through_csv() {
        let imported = import_from_csv(&export_to_csv(&exported())).unwrap();
        let [recording] = imported.recordings.as_slice() else {
            panic!("one recording expected");
        };
        assert_eq!(recording.recording.id, 3);
        assert_eq!(recording.bookmarks.labels[0].name, "Intro, \"slow\"");
        assert_eq!(recording.bookmarks.labels[0].id, Some(5));
        assert_eq!(recording.bookmarks.regions[0].range.end_ms, 20);
        assert_eq!(recording.bookmarks.regions[0].version, Some(1));
    }

    #[test]
    fn rows_are_grouped_by_recording_in_order_of_appearance() {
        let table = "recording_id,type,name,start_ms,end_ms\n\
                     9,label,A,1,\n2,region,B,5,8\n9,label,C,2,\n";
        let imported = import_from_csv(table).unwrap();
        let ids: Vec<_> = imported.recordings.iter().map(|r| r.recording.id).collect();
        assert_eq!(ids, [9, 2]);
        assert_eq!(imported.recordings[0].bookmarks.labels.len(), 2);
        assert_eq!(imported.recordings[0].bookmarks.labels[0].id, None);
    }

    #[test]
    fn a_bad_row_names_its_line() {
        let table = "recording_id,type,name,start_ms,end_ms\n1,label,A,1,\n1,label,B,x,\n";
        let error = import_from_csv(table).unwrap_err();
        assert_eq!(error.line, 3);
        let table = "recording_id,type,name,start_ms,end_ms\n1,region,A,1,\n";
        assert_eq!(import_from_csv(table).unwrap_err().line, 2);
        let table = "recording_id,type,name,start_ms,end_ms\n1,note,A,1,\n";
        assert_eq!(import_from_csv(table).unwrap_err().line, 2);
    }

    #[test]
    fn a_missing_column_is_reported() {
        let error = import_from_csv("recording_id,type,name\n1,label,A\n").unwrap_err();
        assert_eq!(error.line, 1);
        assert!(error.problem.contains("start_ms"));
    }
}
