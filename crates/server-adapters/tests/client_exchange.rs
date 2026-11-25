//! The client core exports and imports bookmarks and manages API tokens against a real server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use support::secured_server;
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::{
    Credentials, ExchangeFormat, ImportOptions, ImportStatus, LinkError, add_label, add_region,
    end_api_token, export_bookmarks, import_bookmarks, load_api_tokens, load_bookmarks,
    make_api_token, set_up_first_admin,
};
use veltrunox_core::{BookmarkName, EntryRef, Millis, RecordingId, TimeRange, TokenScope};

const PASSWORD: &str = "correct horse battery";
const RECORDING: RecordingId = RecordingId::new(1);

/// A secured server with an admin signed in and a label and a region on recording 1.
async fn admin_link() -> (String, HttpServerLink) {
    let url = secured_server().await;
    let anonymous = HttpServerLink::new(&url);
    let admin = set_up_first_admin(
        &anonymous,
        &Credentials {
            user_name: "root".into(),
            password: PASSWORD.into(),
        },
    )
    .await
    .unwrap();
    let link = HttpServerLink::with_token(&url, Some(admin.token));
    let name = |text: &str| BookmarkName::new(text).unwrap();
    add_label(&link, RECORDING, &name("Verse"), Millis::new(1_000))
        .await
        .unwrap();
    add_region(
        &link,
        RECORDING,
        &name("Chorus"),
        TimeRange::new(Millis::new(2_000), Millis::new(3_000)).unwrap(),
    )
    .await
    .unwrap();
    (url, link)
}

#[tokio::test]
async fn the_bookmarks_are_exported_as_json_and_csv() {
    let (_, link) = admin_link().await;
    let json = export_bookmarks(&link, Some(RECORDING), ExchangeFormat::Json)
        .await
        .unwrap();
    assert!(json.contains("\"Verse\"") && json.contains("\"Soundcheck\""));
    let csv = export_bookmarks(&link, None, ExchangeFormat::Csv)
        .await
        .unwrap();
    assert!(csv.starts_with("recording_id,type,name,start_ms,end_ms,id,version"));
    assert!(csv.contains("1,region,Chorus,2000,3000"));
    let missing = export_bookmarks(&link, Some(RecordingId::new(99)), ExchangeFormat::Json).await;
    assert!(missing.is_err());
}

#[tokio::test]
async fn an_exported_file_imports_again_in_both_formats_with_a_preview_first() {
    let (_, link) = admin_link().await;
    for format in [ExchangeFormat::Json, ExchangeFormat::Csv] {
        let file = export_bookmarks(&link, None, format).await.unwrap();
        let preview = import_bookmarks(
            &link,
            &file,
            format,
            ImportOptions {
                dry_run: true,
                overwrite: false,
            },
        )
        .await
        .unwrap();
        assert!(preview.dry_run);
        assert_eq!(preview.ready(), 1);
        let found = &preview.recordings[0];
        assert_eq!(found.status, ImportStatus::Ready);
        assert_eq!((found.counts.unchanged, found.counts.removed), (2, 0));
    }
}

#[tokio::test]
async fn an_edited_file_is_previewed_applied_and_conflicts_are_reported() {
    let (_, link) = admin_link().await;
    let before = load_bookmarks(&link, RECORDING).await.unwrap();
    let csv = "recording_id,type,name,start_ms,end_ms\n1,label,Intro,500,\n";
    let dry = ImportOptions {
        dry_run: true,
        overwrite: false,
    };
    let preview = import_bookmarks(&link, csv, ExchangeFormat::Csv, dry)
        .await
        .unwrap();
    assert_eq!(preview.removed(), 2);
    assert_eq!(preview.recordings[0].counts.added, 1);
    // The preview wrote nothing.
    assert_eq!(load_bookmarks(&link, RECORDING).await.unwrap(), before);

    import_bookmarks(&link, csv, ExchangeFormat::Csv, ImportOptions::default())
        .await
        .unwrap();
    let after = load_bookmarks(&link, RECORDING).await.unwrap();
    assert_eq!(after.labels.len(), 1);
    assert!(after.regions.is_empty());

    // A file made from the old state now conflicts, unless overwriting is asked for.
    let old = format!(
        "recording_id,type,name,start_ms,end_ms,id,version\n1,label,Verse,1000,,{},{}\n",
        before.labels[0].id.value(),
        before.labels[0].version.value()
    );
    let refused = import_bookmarks(&link, &old, ExchangeFormat::Csv, dry)
        .await
        .unwrap();
    assert!(refused.has_conflicts());
    assert_eq!(refused.recordings[0].issues[0].entry, EntryRef::Label(0));
    assert_eq!(
        refused.recordings[0].issues[0].reason.code,
        "bookmark_not_found"
    );
    let forced = import_bookmarks(
        &link,
        &old,
        ExchangeFormat::Csv,
        ImportOptions {
            dry_run: false,
            overwrite: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(forced.ready(), 1);
}

#[tokio::test]
async fn a_file_that_cannot_be_read_is_refused_with_a_reason() {
    let (_, link) = admin_link().await;
    let options = ImportOptions::default();
    let not_json = import_bookmarks(&link, "{oops", ExchangeFormat::Json, options).await;
    assert!(matches!(
        not_json,
        Err(LinkError::Refused(reason)) if reason.code == "import_malformed"
    ));
    let bad_table = "recording_id,type,name,start_ms\n1,label,A,soon\n";
    let not_csv = import_bookmarks(&link, bad_table, ExchangeFormat::Csv, options).await;
    assert!(matches!(
        not_csv,
        Err(LinkError::Refused(reason)) if reason.code == "csv_invalid" && reason.args == [2]
    ));
}

#[tokio::test]
async fn api_tokens_are_made_listed_used_and_revoked() {
    let (url, link) = admin_link().await;
    let made = make_api_token(&link, "llm", TokenScope::EditBookmarks)
        .await
        .unwrap();
    assert!(made.token.starts_with("vxt_"));
    assert_eq!(made.api_token.name, "llm");

    let listed = load_api_tokens(&link).await.unwrap();
    assert_eq!(listed, vec![made.api_token.clone()]);

    // The token itself works, but reaches only bookmarks.
    let scripted = HttpServerLink::with_token(&url, Some(made.token.clone()));
    assert!(
        export_bookmarks(&scripted, None, ExchangeFormat::Json)
            .await
            .is_ok()
    );
    assert!(matches!(
        make_api_token(&scripted, "more", TokenScope::ReadOnly).await,
        Err(LinkError::Forbidden)
    ));

    end_api_token(&link, made.api_token.id).await.unwrap();
    assert!(load_api_tokens(&link).await.unwrap().is_empty());
    assert!(matches!(
        export_bookmarks(&scripted, None, ExchangeFormat::Json).await,
        Err(LinkError::Unauthorized)
    ));
    assert!(matches!(
        end_api_token(&link, made.api_token.id).await,
        Err(LinkError::Refused(reason)) if reason.code == "token_not_found"
    ));
}
