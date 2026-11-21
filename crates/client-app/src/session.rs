//! What a client knows about the shared session: the latest server state, the
//! clock offset to the server and the throttled scrubbing input. Pure logic
//! with time passed in, so it runs the same natively and in the browser.

use veltrunox_core::{Intent, Millis, PlaybackState, RecordingId};

/// What the server tells a client over the real-time channel.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    State(PlaybackState),
    Pong {
        client_time: Millis,
        server_time: Millis,
    },
    Rejected(String),
    BookmarksChanged(RecordingId),
}

/// What changed for the user interface after an event.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewChange {
    Nothing,
    PlaybackState,
    BookmarksChanged(RecordingId),
    Rejected(String),
}

/// Estimated difference `server time - client time` in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClockOffset {
    millis: i64,
    round_trip: u64,
}

impl ClockOffset {
    /// Assumes the server stamped its time halfway through the round trip.
    fn from_round_trip(sent: Millis, server_time: Millis, received: Millis) -> Self {
        let round_trip = received.saturating_sub(sent).value();
        let midpoint = sent.value() as i64 + (round_trip / 2) as i64;
        Self {
            millis: server_time.value() as i64 - midpoint,
            round_trip,
        }
    }
}

/// Holds the latest state and extrapolates the position from it.
#[derive(Debug, Default)]
pub struct SessionView {
    state: Option<PlaybackState>,
    offset: Option<ClockOffset>,
}

impl SessionView {
    pub fn new() -> Self {
        Self::default()
    }

    /// The latest state; `None` until the server has sent one.
    pub fn state(&self) -> Option<&PlaybackState> {
        self.state.as_ref()
    }

    /// Applies an event. A state older than the one held is ignored.
    pub fn handle(&mut self, event: ServerEvent, local_now: Millis) -> ViewChange {
        match event {
            ServerEvent::State(state) => self.adopt(state),
            ServerEvent::Pong {
                client_time,
                server_time,
            } => {
                self.adopt_offset(ClockOffset::from_round_trip(
                    client_time,
                    server_time,
                    local_now,
                ));
                ViewChange::Nothing
            }
            ServerEvent::Rejected(reason) => ViewChange::Rejected(reason),
            ServerEvent::BookmarksChanged(recording) => ViewChange::BookmarksChanged(recording),
        }
    }

    fn adopt(&mut self, state: PlaybackState) -> ViewChange {
        match &self.state {
            Some(held) if held.revision >= state.revision => ViewChange::Nothing,
            _ => {
                self.state = Some(state);
                ViewChange::PlaybackState
            }
        }
    }

    /// The sample with the shortest round trip is the most reliable one.
    fn adopt_offset(&mut self, sample: ClockOffset) {
        match self.offset {
            Some(best) if best.round_trip <= sample.round_trip => {}
            _ => self.offset = Some(sample),
        }
    }

    /// The server's clock at the local time `local_now`; before the first
    /// round trip the clocks are assumed to agree.
    pub fn server_time(&self, local_now: Millis) -> Millis {
        let offset = self.offset.map_or(0, |offset| offset.millis);
        let shifted = (local_now.value() as i64).saturating_add(offset);
        Millis::new(u64::try_from(shifted).unwrap_or(0))
    }

    /// The playback position to show at `local_now`.
    pub fn position_now(&self, local_now: Millis) -> Option<Millis> {
        self.state
            .as_ref()
            .map(|state| state.position_at(self.server_time(local_now)))
    }
}

/// Limits `seek` intents while the position handle is dragged. Releasing
/// always sends the final position, so the last value is never lost.
#[derive(Debug)]
pub struct ScrubThrottle {
    min_interval: Millis,
    last_sent: Option<Millis>,
}

impl ScrubThrottle {
    /// About 20 intents per second (ADR-005).
    pub const DEFAULT_INTERVAL: Millis = Millis::new(50);

    pub fn new(min_interval: Millis) -> Self {
        Self {
            min_interval,
            last_sent: None,
        }
    }

    /// The intent to send for a drag to `position`, or `None` if it is too soon.
    pub fn drag(&mut self, position: Millis, now: Millis) -> Option<Intent> {
        let due = match self.last_sent {
            Some(last) => now.saturating_sub(last) >= self.min_interval,
            None => true,
        };
        due.then(|| {
            self.last_sent = Some(now);
            Intent::Seek {
                position,
                scrubbing: true,
            }
        })
    }

    /// The final intent when the handle is released; starts the next drag fresh.
    pub fn release(&mut self, position: Millis) -> Intent {
        self.last_sent = None;
        Intent::Seek {
            position,
            scrubbing: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{PlaybackSettings, PositionReference};

    fn ms(value: u64) -> Millis {
        Millis::new(value)
    }

    fn running_state(revision: u64, position: u64, server_time: u64) -> PlaybackState {
        PlaybackState {
            revision,
            recording: Some(RecordingId::new(1)),
            reference: PositionReference {
                position: ms(position),
                server_time: ms(server_time),
            },
            running: true,
            settings: PlaybackSettings::default(),
            loop_range: None,
            scrubbing: false,
        }
    }

    #[test]
    fn newer_states_replace_and_older_ones_are_ignored() {
        let mut view = SessionView::new();
        assert_eq!(
            view.handle(ServerEvent::State(running_state(5, 0, 0)), ms(0)),
            ViewChange::PlaybackState
        );
        assert_eq!(
            view.handle(ServerEvent::State(running_state(4, 99, 0)), ms(0)),
            ViewChange::Nothing
        );
        assert_eq!(
            view.handle(ServerEvent::State(running_state(5, 99, 0)), ms(0)),
            ViewChange::Nothing
        );
        assert_eq!(view.state().map(|state| state.revision), Some(5));
    }

    #[test]
    fn position_runs_on_with_the_estimated_clock_offset() {
        let mut view = SessionView::new();
        // The server clock is 10_000 ms ahead; the round trip took 40 ms.
        view.handle(
            ServerEvent::Pong {
                client_time: ms(1_000),
                server_time: ms(11_020),
            },
            ms(1_040),
        );
        view.handle(ServerEvent::State(running_state(1, 500, 11_000)), ms(1_040));
        // Local 3_000 is server 13_000, 2_000 ms after the reference.
        assert_eq!(view.position_now(ms(3_000)), Some(ms(2_500)));
    }

    #[test]
    fn the_shortest_round_trip_wins() {
        let mut view = SessionView::new();
        let pong = |client: u64, server: u64| ServerEvent::Pong {
            client_time: ms(client),
            server_time: ms(server),
        };
        view.handle(pong(0, 5_100), ms(200));
        view.handle(pong(1_000, 6_005), ms(1_010));
        view.handle(pong(2_000, 7_500), ms(2_900));
        assert_eq!(view.server_time(ms(10_000)), ms(15_000));
    }

    #[test]
    fn without_a_state_there_is_no_position() {
        assert_eq!(SessionView::new().position_now(ms(5)), None);
    }

    #[test]
    fn notices_and_rejections_are_passed_on() {
        let mut view = SessionView::new();
        assert_eq!(
            view.handle(ServerEvent::BookmarksChanged(RecordingId::new(2)), ms(0)),
            ViewChange::BookmarksChanged(RecordingId::new(2))
        );
        assert_eq!(
            view.handle(ServerEvent::Rejected("no".into()), ms(0)),
            ViewChange::Rejected("no".into())
        );
    }

    #[test]
    fn dragging_sends_at_most_one_intent_per_interval() {
        let mut throttle = ScrubThrottle::new(ms(50));
        assert!(throttle.drag(ms(1), ms(0)).is_some());
        assert!(throttle.drag(ms(2), ms(30)).is_none());
        assert!(throttle.drag(ms(3), ms(50)).is_some());
    }

    #[test]
    fn release_always_sends_the_final_position() {
        let mut throttle = ScrubThrottle::new(ms(50));
        throttle.drag(ms(1), ms(0));
        assert_eq!(
            throttle.release(ms(9)),
            Intent::Seek {
                position: ms(9),
                scrubbing: false
            }
        );
        assert!(throttle.drag(ms(10), ms(1)).is_some());
    }
}
