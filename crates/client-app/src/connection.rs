//! Connection status as a stream: the natural model for state changing over time.

use rxrust::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
}

/// Reports each distinct status change once.
pub fn report_changes(
    statuses: impl IntoIterator<Item = ConnectionStatus> + 'static,
) -> Vec<ConnectionStatus> {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    Local::from_iter(statuses)
        .distinct_until_changed()
        .subscribe(move |status| sink.borrow_mut().push(status));
    seen.take()
}

#[cfg(test)]
mod tests {
    use super::ConnectionStatus::{Connected, Disconnected};
    use super::*;

    #[test]
    fn repeated_status_is_reported_once() {
        let changes = report_changes([
            Disconnected,
            Disconnected,
            Connected,
            Connected,
            Disconnected,
        ]);
        assert_eq!(changes, [Disconnected, Connected, Disconnected]);
    }
}
