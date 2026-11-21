//! A client survives the loss of the server and finds it again.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::net::SocketAddr;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use support::{WAIT, store_with_recording};
use tokio::runtime::Runtime;
use veltrunox_client_adapters::{SystemTimer, open_channel};
use veltrunox_client_app::{ChannelEvent, LinkEvent, ServerEvent};
use veltrunox_core::{Intent, Millis, RecordingId};
use veltrunox_server_adapters::{AppState, SystemClock, router};

/// Runs the server on its own runtime; dropping the runtime closes every connection.
fn serve_on(address: SocketAddr) -> (Runtime, SocketAddr) {
    let runtime = Runtime::new().unwrap();
    let app = AppState::new(store_with_recording(), Arc::new(SystemClock::new()));
    let listener = runtime
        .block_on(tokio::net::TcpListener::bind(address))
        .unwrap();
    let bound = listener.local_addr().unwrap();
    runtime.spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    (runtime, bound)
}

fn wait_for(events: &mpsc::Receiver<ChannelEvent>, wanted: impl Fn(&ChannelEvent) -> bool) {
    loop {
        let event = events.recv_timeout(WAIT).expect("event in time");
        if wanted(&event) {
            return;
        }
    }
}

fn is_greeting(event: &ChannelEvent) -> bool {
    matches!(event, ChannelEvent::Link(LinkEvent::Greeted))
}

#[test]
fn the_channel_reconnects_after_the_server_restarts() {
    let (server, address) = serve_on("127.0.0.1:0".parse().unwrap());
    let (sender, events) = mpsc::channel();
    let channel = open_channel(
        &format!("http://{address}"),
        Arc::new(SystemTimer),
        || Millis::new(0),
        move |event| {
            let _ = sender.send(event);
        },
    );
    wait_for(&events, is_greeting);

    drop(server);
    wait_for(&events, |event| {
        matches!(
            event,
            ChannelEvent::Link(LinkEvent::Closed | LinkEvent::Failed)
        )
    });
    // While the server is away, sending is refused instead of being lost silently.
    assert!(
        channel
            .send_intent(Intent::SelectRecording(RecordingId::new(1)))
            .is_err()
    );

    let (_restarted, _) = serve_on(address);
    wait_for(&events, is_greeting);
    wait_for(&events, |event| {
        matches!(event, ChannelEvent::Server(ServerEvent::State(_)))
    });
    channel
        .send_intent(Intent::SelectRecording(RecordingId::new(1)))
        .unwrap();
}

#[test]
fn the_channel_keeps_trying_until_the_server_appears() {
    // Reserve a port, then free it so that nothing listens when the channel starts.
    let (probe, address) = serve_on("127.0.0.1:0".parse().unwrap());
    drop(probe);
    let (sender, events) = mpsc::channel();
    let _channel = open_channel(
        &format!("http://{address}"),
        Arc::new(SystemTimer),
        || Millis::new(0),
        move |event| {
            let _ = sender.send(event);
        },
    );
    wait_for(&events, |event| {
        matches!(
            event,
            ChannelEvent::Link(LinkEvent::Closed | LinkEvent::Failed)
        )
    });
    std::thread::sleep(Duration::from_millis(200));

    let (_server, _) = serve_on(address);
    wait_for(&events, is_greeting);
}
