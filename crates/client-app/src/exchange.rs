//! Moving bookmarks in and out as files, and the API tokens for scripts, as the client core
//! sees them.

use veltrunox_core::{ApiTokenId, EntryRef, Reason, RecordingId, TokenScope};

/// The file formats of an export and an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeFormat {
    Json,
    Csv,
}

/// How an import is carried out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportOptions {
    /// Only report what would change.
    pub dry_run: bool,
    /// Also take entries whose stored original changed or is gone.
    pub overwrite: bool,
}

/// What became of one recording of an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStatus {
    /// Applied, or, for a dry run, would be applied.
    Ready,
    /// An entry's stored original changed or is gone.
    Conflict,
    /// Entries are invalid.
    Invalid,
    UnknownRecording,
}

/// How many bookmarks an import adds, changes, keeps and removes in one recording.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportCounts {
    pub added: u32,
    pub changed: u32,
    pub unchanged: u32,
    pub removed: u32,
}

/// A problem with one entry of an import: which label or region of the recording's set it is,
/// counted from 0 in the order of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportIssue {
    pub entry: EntryRef,
    pub reason: Reason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingImport {
    pub recording: RecordingId,
    pub status: ImportStatus,
    /// Zero unless the status is `Ready`.
    pub counts: ImportCounts,
    pub issues: Vec<ImportIssue>,
}

/// The server's answer to an import, one entry per recording of the file in its order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub dry_run: bool,
    pub recordings: Vec<RecordingImport>,
}

impl ImportReport {
    /// How many recordings are (or would be) applied.
    pub fn ready(&self) -> usize {
        self.recordings
            .iter()
            .filter(|found| found.status == ImportStatus::Ready)
            .count()
    }

    pub fn has_conflicts(&self) -> bool {
        self.recordings
            .iter()
            .any(|found| found.status == ImportStatus::Conflict)
    }

    /// The bookmarks the ready recordings lose.
    pub fn removed(&self) -> u32 {
        self.recordings
            .iter()
            .filter(|found| found.status == ImportStatus::Ready)
            .map(|found| found.counts.removed)
            .sum()
    }
}

/// An API token as listed; its text is known only when it is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiToken {
    pub id: ApiTokenId,
    pub name: String,
    pub scope: TokenScope,
    /// Unix seconds.
    pub created_at: u64,
    /// Unix seconds; `None` until the token was used.
    pub last_used: Option<u64>,
}

/// A new token with its text, which the server shows only this once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedApiToken {
    pub token: String,
    pub api_token: ApiToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recording(id: u64, status: ImportStatus, removed: u32) -> RecordingImport {
        RecordingImport {
            recording: RecordingId::new(id),
            status,
            counts: ImportCounts {
                removed,
                ..ImportCounts::default()
            },
            issues: vec![],
        }
    }

    #[test]
    fn the_report_counts_what_would_be_applied() {
        let report = ImportReport {
            dry_run: true,
            recordings: vec![
                recording(1, ImportStatus::Ready, 2),
                recording(2, ImportStatus::Conflict, 0),
                recording(3, ImportStatus::Ready, 3),
                recording(4, ImportStatus::Invalid, 0),
            ],
        };
        assert_eq!(report.ready(), 2);
        assert!(report.has_conflicts());
        assert_eq!(report.removed(), 5);
    }
}
