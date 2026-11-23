//! Two clients against the real server: shared playback state (AC-001, AC-002).

mod support;

use std::time::{Duration, Instant};
use support::{TestClient, start_server, store_with_recording};
use veltrunox_client_app::{ChannelEvent, ServerEvent};
use veltrunox_core::{Intent, Millis, PlayMode, RecordingId, Speed, TimeRange};

const RECORDING: RecordingId = RecordingId::new(1);

// The tests block their thread while waiting, so the server needs worker threads.
#[tokio::test(flavor = "multi_thread")]
async fn a_seek_by_one_client_reaches_the_other() {
    let url = start_server(store_with_recording()).await;
    let mut first = TestClient::connect(&url);
    let mut second = TestClient::connect(&url);

    first
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    first.connection.send_intent(Intent::Seek {
        position: Millis::new(42_000),
        scrubbing: false,
    });

    let seen = second.state_where(|state| state.reference.position == Millis::new(42_000));
    assert_eq!(seen.recording, Some(RECORDING));
    assert!(!seen.running);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_joining_during_playback_gets_the_current_state_at_once() {
    let url = start_server(store_with_recording()).await;
    let mut controller = TestClient::connect(&url);
    controller
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    controller
        .connection
        .send_intent(Intent::SetSpeed(Speed::new(0.5).unwrap()));
    controller.connection.send_intent(Intent::Play);
    controller.state_where(|state| state.running);

    let mut late = TestClient::connect(&url);
    let joined = late.state_where(|_| true);

    assert!(joined.running);
    assert_eq!(joined.recording, Some(RECORDING));
    assert_eq!(joined.settings.speed.factor(), 0.5);
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_intents_give_every_client_the_same_final_state() {
    let url = start_server(store_with_recording()).await;
    let mut first = TestClient::connect(&url);
    let mut second = TestClient::connect(&url);
    first
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    // Without a selection the server would reject the seeks and fewer intents apply.
    second.state_where(|state| state.recording.is_some());

    first.connection.send_intent(Intent::Seek {
        position: Millis::new(1_000),
        scrubbing: false,
    });
    second.connection.send_intent(Intent::Seek {
        position: Millis::new(2_000),
        scrubbing: false,
    });
    // A last common marker: whoever's state carries the highest revision wins.
    first.connection.send_intent(Intent::Pause);

    let last_of = |client: &mut TestClient| {
        let mut latest = client.state_where(|_| true);
        // Select, two seeks and the pause make four applied intents, in some order.
        while latest.revision < 4 {
            latest = client.state_where(|state| state.revision > latest.revision);
        }
        latest
    };
    let (a, b) = (last_of(&mut first), last_of(&mut second));
    assert_eq!(a, b);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rejected_intent_reaches_only_its_sender() {
    let url = start_server(store_with_recording()).await;
    let mut sender = TestClient::connect(&url);
    let mut bystander = TestClient::connect(&url);

    sender.connection.send_intent(Intent::Play); // nothing selected yet
    sender.next_matching(|event| {
        matches!(
            event,
            ChannelEvent::Server(ServerEvent::Rejected(reason)) if reason.code == "no_recording_selected"
        )
        .then_some(())
    });

    // The bystander sees no rejection; its next event is the following state.
    sender
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    let next = bystander.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::Authenticated(_)) => None,
        ChannelEvent::Server(event) => Some(event.clone()),
        _ => None,
    });
    assert!(matches!(next, ServerEvent::State(_)), "got {next:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_invalid_value_is_rejected_without_changing_anything() {
    let url = start_server(store_with_recording()).await;
    let mut client = TestClient::connect(&url);
    // Bypass the typed API: the server must also guard its own boundary.
    let mut raw = ewebsock::ws_connect(
        format!("ws://{}/api/v1/events", url.trim_start_matches("http://")),
        ewebsock::Options::default(),
        Box::new(|_| std::ops::ControlFlow::Continue(())),
    )
    .unwrap();
    raw.send(ewebsock::WsMessage::Text(
        r#"{"type":"set_speed","speed":9.0}"#.into(),
    ));
    raw.send(ewebsock::WsMessage::Text("not json".into()));

    client
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    let state = client.state_where(|state| state.recording.is_some());
    assert_eq!(state.revision, 1);
    assert_eq!(state.settings.speed, Speed::NORMAL);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_clock_estimate_comes_from_a_ping_round_trip() {
    let url = start_server(store_with_recording()).await;
    let mut client = TestClient::connect(&url);
    client.connection.send_ping(Millis::new(1_234));
    let (sent, server_time) = client.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::Pong {
            client_time,
            server_time,
        }) => Some((*client_time, *server_time)),
        _ => None,
    });
    assert_eq!(sent, Millis::new(1_234));
    assert!(
        server_time.value() < 60_000,
        "server time counts from start"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_joining_client_learns_that_the_server_has_no_audio_output() {
    // `AppState::new` starts no audio playback, as on a server without a device.
    let url = start_server(store_with_recording()).await;
    let mut client = TestClient::connect(&url);
    let available = client.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::AudioOutput(available)) => Some(*available),
        _ => None,
    });
    assert!(!available);
}

#[tokio::test(flavor = "multi_thread")]
async fn both_clients_see_the_pause_at_the_region_end_at_the_right_time() {
    let url = start_server(store_with_recording()).await;
    let mut first = TestClient::connect(&url);
    let mut second = TestClient::connect(&url);
    let range = TimeRange::new(Millis::new(1_000), Millis::new(1_600)).unwrap();

    first
        .connection
        .send_intent(Intent::SelectRecording(RECORDING));
    first
        .connection
        .send_intent(Intent::SetPlayMode(PlayMode::ToRegionEnd));
    first.connection.send_intent(Intent::PlayRegion(range));
    let started = Instant::now();

    for client in [&mut first, &mut second] {
        let playing = client.state_where(|state| state.running);
        assert_eq!(playing.play_range, Some(range));
        assert_eq!(playing.play_mode, PlayMode::ToRegionEnd);
        let ended = client.state_where(|state| !state.running);
        assert_eq!(ended.reference.position, range.end());
        assert_eq!(ended.play_range, None);
        assert_eq!(ended.play_mode, PlayMode::ToRegionEnd);
    }
    let elapsed = started.elapsed();
    assert!(
        (Duration::from_millis(550)..Duration::from_millis(800)).contains(&elapsed),
        "paused after {elapsed:?}"
    );
}
