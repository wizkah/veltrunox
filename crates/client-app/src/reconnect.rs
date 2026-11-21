//! When to reopen the real-time channel and when to ask the server for its time.
//! Pure decisions; the clock and the timer are supplied by adapters.

use crate::connection::LinkEvent;
use std::time::Duration;

/// Delay before the first reconnection attempt.
const INITIAL_DELAY: Duration = Duration::from_millis(500);
/// Reconnection delays stop growing here, so a returning server is found quickly.
const MAX_DELAY: Duration = Duration::from_secs(30);

/// How often the clock offset to the server is measured again while connected.
pub const CLOCK_SYNC_INTERVAL: Duration = Duration::from_secs(60);

/// Port: runs an action once after a delay, on native targets and in the browser.
pub trait Timer: Send + Sync {
    fn after(&self, delay: Duration, action: Box<dyn FnOnce() + Send>);
}

/// Delays that double with every failed attempt up to a cap.
#[derive(Debug)]
struct Backoff {
    next: Duration,
}

impl Backoff {
    fn new() -> Self {
        Self {
            next: INITIAL_DELAY,
        }
    }

    fn next_delay(&mut self) -> Duration {
        let delay = self.next;
        self.next = (self.next * 2).min(MAX_DELAY);
        delay
    }

    fn reset(&mut self) {
        self.next = INITIAL_DELAY;
    }
}

/// Decides, per link event, whether a reconnection attempt has to be scheduled.
#[derive(Debug)]
pub struct ReconnectPolicy {
    backoff: Backoff,
    retry_pending: bool,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectPolicy {
    pub fn new() -> Self {
        Self {
            backoff: Backoff::new(),
            retry_pending: false,
        }
    }

    /// The delay after which to try again, if an attempt has to be scheduled now.
    /// A failure and the close that follows it count as one loss of the connection.
    pub fn on_link_event(&mut self, event: LinkEvent) -> Option<Duration> {
        match event {
            LinkEvent::Greeted => {
                self.backoff.reset();
                self.retry_pending = false;
                None
            }
            LinkEvent::Closed | LinkEvent::Failed if self.retry_pending => None,
            LinkEvent::Closed | LinkEvent::Failed => {
                self.retry_pending = true;
                Some(self.backoff.next_delay())
            }
        }
    }

    /// The scheduled attempt starts now; a failure of it schedules the next one.
    pub fn retry_started(&mut self) {
        self.retry_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lose(policy: &mut ReconnectPolicy) -> Duration {
        let delay = policy.on_link_event(LinkEvent::Failed).expect("a retry");
        policy.retry_started();
        delay
    }

    #[test]
    fn delays_grow_up_to_the_cap() {
        let mut policy = ReconnectPolicy::new();
        let delays: Vec<_> = (0..9).map(|_| lose(&mut policy)).collect();
        assert_eq!(delays[0], Duration::from_millis(500));
        assert_eq!(delays[1], Duration::from_secs(1));
        assert_eq!(delays[2], Duration::from_secs(2));
        assert_eq!(delays[7], MAX_DELAY);
        assert_eq!(delays[8], MAX_DELAY);
    }

    #[test]
    fn a_greeting_starts_the_delays_over() {
        let mut policy = ReconnectPolicy::new();
        lose(&mut policy);
        lose(&mut policy);
        assert_eq!(policy.on_link_event(LinkEvent::Greeted), None);
        assert_eq!(lose(&mut policy), INITIAL_DELAY);
    }

    #[test]
    fn a_failure_and_its_close_schedule_one_retry() {
        let mut policy = ReconnectPolicy::new();
        assert!(policy.on_link_event(LinkEvent::Failed).is_some());
        assert_eq!(policy.on_link_event(LinkEvent::Closed), None);
    }
}
