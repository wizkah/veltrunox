//! Ends the playback at the end of the recording and at the end of the play range. The shared
//! state only knows a position and a rate, so something has to look at the clock.

use crate::state::AppState;
use std::time::Duration;
use tokio::task::JoinHandle;

/// Clients extrapolate past the end of the recording for at most this long.
const CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Starts the watch on the running tokio runtime; dropping the handle does not stop it. The end
/// of a play range is awaited to the millisecond; the end of the recording is polled.
pub fn watch_end(app: &AppState) -> JoinHandle<()> {
    let playback = app.playback.clone();
    let mut states = app.hub.subscribe_state();
    tokio::spawn(async move {
        let mut ticks = tokio::time::interval(CHECK_INTERVAL);
        loop {
            // Every published state may move the instant, so it is asked for again each round.
            let stop_in = playback.region_stop_in();
            tokio::select! {
                _ = ticks.tick() => {
                    playback.stop_at_end();
                }
                changed = states.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(stop_in.map_or(0, |wait| wait.value()))),
                    if stop_in.is_some() =>
                {
                    playback.stop_at_region_end();
                    // The clock may read a millisecond short; try again a moment later.
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        }
    })
}
