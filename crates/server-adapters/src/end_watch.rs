//! Ends the playback at the end of the recording. The shared state only knows
//! a position and a rate, so something has to look at the clock.

use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use veltrunox_server_app::PlaybackService;

/// Clients extrapolate past the end for at most this long.
const CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Starts the check on the running tokio runtime; dropping the handle does not stop it.
pub fn watch_end(playback: Arc<PlaybackService>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticks = tokio::time::interval(CHECK_INTERVAL);
        loop {
            ticks.tick().await;
            playback.stop_at_end();
        }
    })
}
