//! Bookmarks over HTTP with versions, change notices, and persistence across restarts (AC-009).

mod support;

use std::sync::Arc;
use support::{TestClient, start_server, store_with_recording};
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::{
    ChannelEvent, LinkError, ServerEvent, add_label, add_region, convert_label_to_region,
    convert_region_to_label, delete_label, delete_region, edit_label, edit_region, load_bookmarks,
    load_library, merge_regions,
};
use veltrunox_core::{
    BookmarkName, Intent, KeyShift, Millis, RecordingId, Speed, TimeRange, Version,
};
use veltrunox_protocol::{
    BookmarksDto, ConflictDto, LabelDto, NewLabelDto, NewRegionDto, RangeDto, UpdateLabelDto,
};
use veltrunox_server_adapters::SqliteStore;

const RECORDING: RecordingId = RecordingId::new(1);

fn labels_url(base: &str) -> String {
    format!("{base}/api/v1/recordings/1/labels")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_new_label_is_announced_and_can_be_loaded_by_another_client() {
    let url = start_server(store_with_recording()).await;
    let mut watcher = TestClient::connect(&url);

    let response = reqwest::Client::new()
        .post(labels_url(&url))
        .json(&NewLabelDto {
            name: "Verse".into(),
            position_ms: 12_000,
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);

    let announced = watcher.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::BookmarksChanged(id)) => Some(*id),
        _ => None,
    });
    assert_eq!(announced, RECORDING);

    let loaded = load_bookmarks(&HttpServerLink::new(&url), announced)
        .await
        .unwrap();
    assert_eq!(loaded.labels.len(), 1);
    assert_eq!(loaded.labels[0].name.as_str(), "Verse");
}

#[tokio::test]
async fn the_client_writes_labels_and_regions_and_meets_a_conflict() {
    let url = start_server(store_with_recording()).await;
    let link = HttpServerLink::new(&url);
    let name = |text: &str| BookmarkName::new(text).unwrap();

    let label = add_label(&link, RECORDING, &name("Intro"), Millis::new(5_000))
        .await
        .unwrap();
    let renamed = edit_label(
        &link,
        label.id,
        label.version,
        &name("Entry"),
        Millis::new(6_000),
    )
    .await
    .unwrap();
    assert_eq!(renamed.version, Version::new(2));
    let stale = edit_label(
        &link,
        label.id,
        label.version,
        &name("Late"),
        Millis::new(7_000),
    )
    .await;
    assert!(matches!(
        stale,
        Err(LinkError::Conflict { current }) if current == Version::new(2)
    ));

    let range = TimeRange::new(Millis::new(1_000), Millis::new(4_000)).unwrap();
    let region = add_region(&link, RECORDING, &name("Verse"), range)
        .await
        .unwrap();
    let longer = TimeRange::new(Millis::new(1_000), Millis::new(9_000)).unwrap();
    let resized = edit_region(&link, region.id, region.version, &name("Verse"), longer)
        .await
        .unwrap();
    assert_eq!(resized.range, longer);

    let found = load_bookmarks(&link, RECORDING).await.unwrap();
    assert_eq!(found.labels.len(), 1);
    assert_eq!(found.regions.len(), 1);

    delete_label(&link, label.id, renamed.version)
        .await
        .unwrap();
    delete_region(&link, region.id, resized.version)
        .await
        .unwrap();
    let found = load_bookmarks(&link, RECORDING).await.unwrap();
    assert!(found.labels.is_empty() && found.regions.is_empty());
}

#[tokio::test]
async fn a_marker_changes_its_kind_in_one_step() {
    let url = start_server(store_with_recording()).await;
    let link = HttpServerLink::new(&url);
    let name = BookmarkName::new("Chorus").unwrap();

    let label = add_label(&link, RECORDING, &name, Millis::new(2_000))
        .await
        .unwrap();
    let range = TimeRange::new(Millis::new(2_000), Millis::new(8_000)).unwrap();
    let region = convert_label_to_region(&link, label.id, label.version, &name, range)
        .await
        .unwrap();
    let found = load_bookmarks(&link, RECORDING).await.unwrap();
    assert!(found.labels.is_empty());
    assert_eq!(found.regions, vec![region.clone()]);

    let stale =
        convert_region_to_label(&link, region.id, Version::new(9), &name, Millis::new(2_000)).await;
    assert!(matches!(stale, Err(LinkError::Conflict { .. })));
    assert_eq!(
        load_bookmarks(&link, RECORDING).await.unwrap().regions,
        vec![region.clone()]
    );

    let back = convert_region_to_label(&link, region.id, region.version, &name, Millis::new(2_000))
        .await
        .unwrap();
    let found = load_bookmarks(&link, RECORDING).await.unwrap();
    assert!(found.regions.is_empty());
    assert_eq!(found.labels, vec![back]);
}

#[tokio::test]
async fn regions_merge_into_one_or_not_at_all() {
    let url = start_server(store_with_recording()).await;
    let link = HttpServerLink::new(&url);
    let name = |text: &str| BookmarkName::new(text).unwrap();
    let span = |a, b| TimeRange::new(Millis::new(a), Millis::new(b)).unwrap();
    let first = add_region(&link, RECORDING, &name("A"), span(1_000, 4_000))
        .await
        .unwrap();
    let second = add_region(&link, RECORDING, &name("B"), span(6_000, 9_000))
        .await
        .unwrap();
    let third = add_region(&link, RECORDING, &name("C"), span(20_000, 30_000))
        .await
        .unwrap();

    // One of the regions is stale: nothing changes.
    let stale = [(first.id, first.version), (second.id, Version::new(9))];
    let refused = merge_regions(&link, RECORDING, &stale, &name("AB"), span(1_000, 9_000)).await;
    assert!(matches!(refused, Err(LinkError::Conflict { .. })));
    assert_eq!(
        load_bookmarks(&link, RECORDING)
            .await
            .unwrap()
            .regions
            .len(),
        3
    );

    // A region that does not exist: nothing changes either.
    let missing = [
        (first.id, first.version),
        (veltrunox_core::BookmarkId::new(999), Version::FIRST),
    ];
    assert!(
        merge_regions(&link, RECORDING, &missing, &name("AB"), span(1_000, 9_000))
            .await
            .is_err()
    );
    assert_eq!(
        load_bookmarks(&link, RECORDING)
            .await
            .unwrap()
            .regions
            .len(),
        3
    );

    let seen = [(first.id, first.version), (second.id, second.version)];
    let merged = merge_regions(&link, RECORDING, &seen, &name("AB"), span(1_000, 9_000))
        .await
        .unwrap();
    let found = load_bookmarks(&link, RECORDING).await.unwrap();
    assert_eq!(found.regions.len(), 2);
    assert!(found.regions.contains(&merged));
    assert!(found.regions.contains(&third));
}

#[tokio::test]
async fn a_stale_edit_is_a_conflict_and_loses_nothing() {
    let url = start_server(store_with_recording()).await;
    let http = reqwest::Client::new();
    let label: LabelDto = http
        .post(labels_url(&url))
        .json(&NewLabelDto {
            name: "Verse".into(),
            position_ms: 1_000,
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let edit = |name: &str, version: u64| UpdateLabelDto {
        expected_version: version,
        name: name.into(),
        position_ms: 1_000,
    };
    let label_url = format!("{url}/api/v1/labels/{}", label.id);

    let first = http
        .put(&label_url)
        .json(&edit("Verse 1", 1))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 200);
    let second = http
        .put(&label_url)
        .json(&edit("Verse 2", 1))
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), 409);
    let conflict: ConflictDto = second.json().await.unwrap();
    assert_eq!(conflict.current_version, 2);

    let all: BookmarksDto = http
        .get(format!("{url}/api/v1/recordings/1/bookmarks"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(all.labels[0].name, "Verse 1");
}

#[tokio::test]
async fn invalid_bookmarks_and_unknown_recordings_are_refused() {
    let url = start_server(store_with_recording()).await;
    let http = reqwest::Client::new();

    let blank = http
        .post(labels_url(&url))
        .json(&NewLabelDto {
            name: "  ".into(),
            position_ms: 1,
        })
        .send()
        .await
        .unwrap();
    assert_eq!(blank.status(), 400);

    let backwards = http
        .post(format!("{url}/api/v1/recordings/1/regions"))
        .json(&NewRegionDto {
            name: "Chorus".into(),
            range: RangeDto {
                start_ms: 9,
                end_ms: 3,
            },
        })
        .send()
        .await
        .unwrap();
    assert_eq!(backwards.status(), 400);

    let unknown = http
        .post(format!("{url}/api/v1/recordings/99/labels"))
        .json(&NewLabelDto {
            name: "Verse".into(),
            position_ms: 1,
        })
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn recordings_bookmarks_and_settings_survive_a_restart() {
    let directory = std::env::temp_dir().join(format!("veltrunox-restart-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("veltrunox.db");

    {
        let store = Arc::new(SqliteStore::open(&database).unwrap());
        veltrunox_server_app::RecordingRepository::add(
            store.as_ref(),
            veltrunox_core::RecordingTitle::new("Soundcheck").unwrap(),
        )
        .unwrap();
        let url = start_server(store).await;
        reqwest::Client::new()
            .post(labels_url(&url))
            .json(&NewLabelDto {
                name: "Verse".into(),
                position_ms: 5_000,
            })
            .send()
            .await
            .unwrap();
        let mut client = TestClient::connect(&url);
        client
            .connection
            .send_intent(Intent::SelectRecording(RECORDING));
        client
            .connection
            .send_intent(Intent::SetSpeed(Speed::new(0.8).unwrap()));
        client
            .connection
            .send_intent(Intent::SetKey(KeyShift::new(-2).unwrap()));
        client.connection.send_intent(Intent::Seek {
            position: Millis::new(30_000),
            scrubbing: false,
        });
        client.state_where(|state| state.reference.position == Millis::new(30_000));
    }

    // A new store and a new server over the same file: the restart.
    let url = start_server(Arc::new(SqliteStore::open(&database).unwrap())).await;
    let link = HttpServerLink::new(&url);
    assert_eq!(load_library(&link).await.unwrap().len(), 1);
    assert_eq!(
        load_bookmarks(&link, RECORDING).await.unwrap().labels.len(),
        1
    );

    let mut client = TestClient::connect(&url);
    let initial = client.state_where(|_| true);
    assert_eq!(initial.recording, None, "a restarted server is idle");
    client
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    let restored = client.state_where(|state| state.recording.is_some());
    assert_eq!(restored.settings.speed.factor(), 0.8);
    assert_eq!(restored.settings.key.semitones(), -2);
    assert!(!restored.running);
    std::fs::remove_dir_all(&directory).unwrap();
}
