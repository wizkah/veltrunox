//! The waveform of recordings as the client keeps it: fetched once, then served
//! from memory.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{Millis, RecordingId, Waveform};

/// A recording's length together with its peaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingWaveform {
    pub length: Millis,
    pub waveform: Waveform,
}

/// Waveforms already fetched. A recording's peaks do not change once scanned, so
/// an entry stays for the life of the client.
#[derive(Debug, Default)]
pub struct WaveformCache {
    entries: HashMap<RecordingId, Arc<RecordingWaveform>>,
}

impl WaveformCache {
    pub fn get(&self, recording: RecordingId) -> Option<Arc<RecordingWaveform>> {
        self.entries.get(&recording).cloned()
    }

    pub fn insert(
        &mut self,
        recording: RecordingId,
        waveform: RecordingWaveform,
    ) -> Arc<RecordingWaveform> {
        let shared = Arc::new(waveform);
        self.entries.insert(recording, Arc::clone(&shared));
        shared
    }
}

/// Returns the cached waveform, or fetches and caches it. The lock is not held
/// while fetching; two simultaneous first requests both fetch, the later one
/// replaces the earlier (same content).
pub async fn cached_waveform<F, E>(
    cache: &Mutex<WaveformCache>,
    recording: RecordingId,
    fetch: F,
) -> Result<Arc<RecordingWaveform>, E>
where
    F: AsyncFnOnce() -> Result<RecordingWaveform, E>,
{
    if let Some(found) = cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(recording)
    {
        return Ok(found);
    }
    let fetched = fetch().await?;
    Ok(cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(recording, fetched))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn sample() -> RecordingWaveform {
        RecordingWaveform {
            length: Millis::new(1_000),
            waveform: Waveform::from_mono(&[0.5; 2_048], 44_100),
        }
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => unreachable!("the test futures never wait"),
        }
    }

    #[test]
    fn the_second_request_is_served_from_memory() {
        let cache = Mutex::new(WaveformCache::default());
        let fetches = Cell::new(0);
        let id = RecordingId::new(1);
        for _ in 0..2 {
            let found: Result<_, ()> = block_on(cached_waveform(&cache, id, async || {
                fetches.set(fetches.get() + 1);
                Ok(sample())
            }));
            assert_eq!(found.unwrap().length, Millis::new(1_000));
        }
        assert_eq!(fetches.get(), 1);
    }

    #[test]
    fn a_failed_fetch_is_not_cached() {
        let cache = Mutex::new(WaveformCache::default());
        let id = RecordingId::new(2);
        let failed: Result<_, &str> =
            block_on(cached_waveform(&cache, id, async || Err("not scanned")));
        assert_eq!(failed.unwrap_err(), "not scanned");
        let found: Result<_, &str> = block_on(cached_waveform(&cache, id, async || Ok(sample())));
        assert!(found.is_ok());
    }

    #[test]
    fn recordings_are_cached_apart() {
        let cache = Mutex::new(WaveformCache::default());
        let fetches = Cell::new(0);
        for id in [3, 4] {
            let _: Result<_, ()> =
                block_on(cached_waveform(&cache, RecordingId::new(id), async || {
                    fetches.set(fetches.get() + 1);
                    Ok(sample())
                }));
        }
        assert_eq!(fetches.get(), 2);
    }
}
