//! The real-time channel that keeps itself open: reconnects with growing delays,
//! measures the clock offset again from time to time and reports the status.

use crate::events::{EventConnection, connect_events};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use veltrunox_client_app::{
    CLOCK_SYNC_INTERVAL, ChannelAccess, ChannelEvent, LinkError, LinkEvent, ReconnectPolicy, Timer,
};
use veltrunox_core::{Intent, Millis};

type EventSink = dyn Fn(ChannelEvent) + Send + Sync;

/// The open connections live in a process-wide registry: the browser's socket is
/// not `Send`, but the event callback of the socket has to be. Each connection is
/// touched only through the channel that owns it, identified by its id.
type Connections = Vec<(u64, EventConnection)>;

#[cfg(not(target_arch = "wasm32"))]
static CONNECTIONS: Mutex<Connections> = Mutex::new(Vec::new());

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CONNECTIONS: std::cell::RefCell<Connections> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(not(target_arch = "wasm32"))]
fn with_connections<T>(action: impl FnOnce(&mut Connections) -> T) -> T {
    action(&mut CONNECTIONS.lock().unwrap_or_else(PoisonError::into_inner))
}

#[cfg(target_arch = "wasm32")]
fn with_connections<T>(action: impl FnOnce(&mut Connections) -> T) -> T {
    CONNECTIONS.with(|connections| action(&mut connections.borrow_mut()))
}

fn remove_connection(id: u64) {
    with_connections(|connections| connections.retain(|(owner, _)| *owner != id));
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct Control {
    /// Counts connection attempts; events of older attempts are ignored.
    generation: u64,
    policy: ReconnectPolicy,
    stopped: bool,
}

struct Supervisor {
    id: u64,
    url: String,
    access: ChannelAccess,
    timer: Arc<dyn Timer>,
    clock: fn() -> Millis,
    on_event: Box<EventSink>,
    control: Mutex<Control>,
}

/// Owns the real-time channel; dropping it closes the connection for good.
pub struct EventChannel {
    supervisor: Arc<Supervisor>,
}

/// Opens the channel and keeps it open. Reports what happens on it to `on_event`;
/// a connection that cannot be made is retried, so this cannot fail.
pub fn open_channel(
    base_url: &str,
    access: ChannelAccess,
    timer: Arc<dyn Timer>,
    clock: fn() -> Millis,
    on_event: impl Fn(ChannelEvent) + Send + Sync + 'static,
) -> EventChannel {
    let supervisor = Arc::new(Supervisor {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        url: base_url.to_owned(),
        access,
        timer,
        clock,
        on_event: Box::new(on_event),
        control: Mutex::new(Control {
            generation: 0,
            policy: ReconnectPolicy::new(),
            stopped: false,
        }),
    });
    supervisor.open();
    EventChannel { supervisor }
}

impl EventChannel {
    /// Sends an intent; the server answers with the new state or a rejection.
    pub fn send_intent(&self, intent: Intent) -> Result<(), LinkError> {
        self.supervisor
            .with_connection(|connection| connection.send_intent(intent))
    }

    /// Measures the clock offset to the server now.
    pub fn sync_clock(&self) -> Result<(), LinkError> {
        self.supervisor.ping()
    }
}

impl Drop for EventChannel {
    fn drop(&mut self) {
        self.supervisor.lock().stopped = true;
        remove_connection(self.supervisor.id);
    }
}

impl Supervisor {
    fn lock(&self) -> MutexGuard<'_, Control> {
        self.control.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn with_connection(&self, action: impl FnOnce(&mut EventConnection)) -> Result<(), LinkError> {
        with_connections(|connections| {
            match connections.iter_mut().find(|(owner, _)| *owner == self.id) {
                Some((_, connection)) => {
                    action(connection);
                    Ok(())
                }
                None => Err(LinkError::NotConnected),
            }
        })
    }

    fn ping(&self) -> Result<(), LinkError> {
        let now = (self.clock)();
        self.with_connection(|connection| connection.send_ping(now))
    }

    /// Starts a connection attempt. The control lock is held until the connection
    /// is registered, so the callback cannot see events before it can be used.
    fn open(self: &Arc<Self>) {
        let mut control = self.lock();
        if control.stopped {
            return;
        }
        control.generation += 1;
        let generation = control.generation;
        let this = Arc::clone(self);
        let opened = connect_events(&self.url, move |event| {
            this.on_connection_event(generation, event);
        });
        match opened {
            Ok(connection) => {
                let id = self.id;
                with_connections(|connections| {
                    connections.retain(|(owner, _)| *owner != id);
                    connections.push((id, connection));
                });
            }
            Err(_) => {
                let delay = control.policy.on_link_event(LinkEvent::Failed);
                drop(control);
                (self.on_event)(ChannelEvent::Link(LinkEvent::Failed));
                self.schedule_retry(generation, delay);
            }
        }
    }

    fn on_connection_event(self: &Arc<Self>, generation: u64, event: ChannelEvent) {
        let delay = {
            let mut control = self.lock();
            if control.stopped || control.generation != generation {
                return;
            }
            match event {
                ChannelEvent::Link(link) => control.policy.on_link_event(link),
                ChannelEvent::Server(_) => None,
            }
        };
        (self.on_event)(event.clone());
        match event {
            ChannelEvent::Link(LinkEvent::Greeted) => {
                // Signing in comes first; a failed send only means the connection was lost again.
                let access = self.access.clone();
                let _ = self.with_connection(|connection| connection.send_access(&access));
                // A failed ping only means the connection was lost again; the retry follows.
                let _ = self.ping();
                self.schedule_clock_sync(generation);
            }
            ChannelEvent::Link(LinkEvent::Closed | LinkEvent::Failed) => {
                remove_connection(self.id);
                self.schedule_retry(generation, delay);
            }
            ChannelEvent::Server(_) => {}
        }
    }

    fn schedule_retry(self: &Arc<Self>, generation: u64, delay: Option<std::time::Duration>) {
        let Some(delay) = delay else { return };
        let this = Arc::clone(self);
        self.timer.after(
            delay,
            Box::new(move || {
                {
                    let mut control = this.lock();
                    if control.stopped || control.generation != generation {
                        return;
                    }
                    control.policy.retry_started();
                }
                this.open();
            }),
        );
    }

    fn schedule_clock_sync(self: &Arc<Self>, generation: u64) {
        let this = Arc::clone(self);
        self.timer.after(
            CLOCK_SYNC_INTERVAL,
            Box::new(move || {
                {
                    let control = this.lock();
                    if control.stopped || control.generation != generation {
                        return;
                    }
                }
                let _ = this.ping();
                this.schedule_clock_sync(generation);
            }),
        );
    }
}
