//! Connection status as a stream: the natural model for state changing over time.

use rxrust::prelude::*;
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
}

/// What the link to the server reports about the real-time channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkEvent {
    /// The server greeted the client; the channel is usable.
    Greeted,
    Closed,
    Failed,
}

impl LinkEvent {
    fn status(self) -> ConnectionStatus {
        match self {
            Self::Greeted => ConnectionStatus::Connected,
            Self::Closed | Self::Failed => ConnectionStatus::Disconnected,
        }
    }
}

/// Turns link events into distinct status changes and hands each one to `on_status`.
///
/// Owns the event stream; the subscription ends when the monitor is dropped.
pub struct ConnectionMonitor {
    events: SharedSubject<'static, LinkEvent, Infallible>,
}

impl ConnectionMonitor {
    pub fn new(on_status: impl FnMut(ConnectionStatus) + Send + 'static) -> Self {
        let events = Shared::subject();
        events
            .clone()
            .map(LinkEvent::status)
            .distinct_until_changed()
            .subscribe(on_status);
        Self { events }
    }

    pub fn push(&mut self, event: LinkEvent) {
        self.events.next(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn repeated_status_is_reported_once() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let mut monitor = ConnectionMonitor::new(move |status| sink.lock().unwrap().push(status));

        for event in [
            LinkEvent::Closed,
            LinkEvent::Closed,
            LinkEvent::Greeted,
            LinkEvent::Greeted,
            LinkEvent::Failed,
        ] {
            monitor.push(event);
        }

        assert_eq!(
            *seen.lock().unwrap(),
            [
                ConnectionStatus::Disconnected,
                ConnectionStatus::Connected,
                ConnectionStatus::Disconnected
            ]
        );
    }
}
