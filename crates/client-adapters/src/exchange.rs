//! Wire types of the bookmark exchange and of API tokens, turned into the client core's types.

use veltrunox_client_app::{
    ApiToken, CreatedApiToken, ExchangeFormat, ImportCounts, ImportIssue, ImportReport,
    ImportStatus, RecordingImport,
};
use veltrunox_core::{ApiTokenId, RecordingId};
use veltrunox_protocol::{
    ApiTokenDto, CreatedApiTokenDto, ImportOutcomeDto, ImportResultDto, ImportStatusDto,
    entry_from_path, token_scope_from_dto,
};

/// The query value and the content type of a format.
pub fn format_name(format: ExchangeFormat) -> &'static str {
    match format {
        ExchangeFormat::Json => "json",
        ExchangeFormat::Csv => "csv",
    }
}

pub fn content_type(format: ExchangeFormat) -> &'static str {
    match format {
        ExchangeFormat::Json => "application/json",
        ExchangeFormat::Csv => "text/csv; charset=utf-8",
    }
}

fn status_from_dto(status: ImportStatusDto) -> ImportStatus {
    match status {
        ImportStatusDto::Ok => ImportStatus::Ready,
        ImportStatusDto::Conflict => ImportStatus::Conflict,
        ImportStatusDto::Invalid => ImportStatus::Invalid,
        ImportStatusDto::UnknownRecording => ImportStatus::UnknownRecording,
    }
}

fn recording_import_from_dto(dto: &ImportOutcomeDto) -> Result<RecordingImport, String> {
    let issues = dto
        .issues
        .iter()
        .map(|issue| {
            let entry = entry_from_path(&issue.entry)
                .ok_or_else(|| format!("unknown place of an import issue: {}", issue.entry))?;
            Ok(ImportIssue {
                entry,
                reason: issue.reason.clone().into(),
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(RecordingImport {
        recording: RecordingId::new(dto.recording_id),
        status: status_from_dto(dto.status),
        counts: ImportCounts {
            added: dto.added,
            changed: dto.changed,
            unchanged: dto.unchanged,
            removed: dto.removed,
        },
        issues,
    })
}

/// The answer of an import; an error names what in it was not understood.
pub fn import_report_from_dto(dto: &ImportResultDto) -> Result<ImportReport, String> {
    Ok(ImportReport {
        dry_run: dto.dry_run,
        recordings: dto
            .recordings
            .iter()
            .map(recording_import_from_dto)
            .collect::<Result<_, _>>()?,
    })
}

pub fn api_token_from_dto(dto: &ApiTokenDto) -> ApiToken {
    ApiToken {
        id: ApiTokenId::new(dto.id),
        name: dto.name.clone(),
        scope: token_scope_from_dto(dto.scope),
        created_at: dto.created_at,
        last_used: dto.last_used,
    }
}

pub fn created_api_token_from_dto(dto: &CreatedApiTokenDto) -> CreatedApiToken {
    CreatedApiToken {
        token: dto.token.clone(),
        api_token: api_token_from_dto(&dto.api_token),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{EntryRef, TokenScope};
    use veltrunox_protocol::{ImportIssueDto, ReasonDto, TokenScopeDto};

    #[test]
    fn an_import_answer_becomes_a_report() {
        let report = import_report_from_dto(&ImportResultDto {
            dry_run: true,
            recordings: vec![ImportOutcomeDto {
                recording_id: 4,
                status: ImportStatusDto::Conflict,
                added: 0,
                changed: 0,
                unchanged: 0,
                removed: 0,
                issues: vec![ImportIssueDto {
                    entry: "labels[0]".into(),
                    reason: ReasonDto {
                        code: "bookmark_conflict".into(),
                        args: vec![3],
                        message: "changed".into(),
                    },
                }],
            }],
        })
        .unwrap();
        assert!(report.dry_run);
        let found = &report.recordings[0];
        assert_eq!(found.recording, RecordingId::new(4));
        assert_eq!(found.status, ImportStatus::Conflict);
        assert_eq!(found.issues[0].entry, EntryRef::Label(0));
        assert_eq!(found.issues[0].reason.code, "bookmark_conflict");
        assert_eq!(found.issues[0].reason.args, vec![3]);
    }

    #[test]
    fn a_token_keeps_its_reach() {
        let token = api_token_from_dto(&ApiTokenDto {
            id: 2,
            name: "llm".into(),
            scope: TokenScopeDto::EditBookmarks,
            created_at: 10,
            last_used: None,
        });
        assert_eq!(token.scope, TokenScope::EditBookmarks);
        assert_eq!(token.last_used, None);
    }
}
