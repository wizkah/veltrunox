//! Analysis in the background: import, notice to clients, result in the library, persistence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::path::{Path, PathBuf};
use support::{TestClient, start_server};
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::{ChannelEvent, ServerEvent, analyse_again, load_library};
use veltrunox_core::{AnalysisProgress, KeyMode, LibraryEntry, RecordingId};
use veltrunox_server_adapters::{SqliteStore, import_directory};
use veltrunox_server_app::AnalysisRepository;

const RATE: u32 = 44_100;

/// A 30 s WAV: chords struck on every beat of 100 bpm in C major, A4 at 440 Hz.
fn write_music(path: &Path) {
    let chords: [[f64; 3]; 4] = [
        [0.0, 4.0, 7.0],
        [5.0, 9.0, 12.0],
        [7.0, 11.0, 14.0],
        [0.0, 4.0, 7.0],
    ];
    let beat = 0.6;
    let mut samples = Vec::new();
    for frame in 0..(30 * RATE) {
        let time = f64::from(frame) / f64::from(RATE);
        let index = (time / beat) as usize;
        let envelope = (-(time - index as f64 * beat) / 0.2).exp();
        let value: f64 = chords[index / 4 % 4]
            .iter()
            .flat_map(|semitones| {
                let base = 440.0 * 2.0_f64.powf((semitones - 9.0) / 12.0);
                (1..=3).map(move |harmonic| (base * f64::from(harmonic), 1.0 / f64::from(harmonic)))
            })
            .map(|(hertz, amplitude)| amplitude * (2.0 * std::f64::consts::PI * hertz * time).sin())
            .sum();
        samples.push((0.1 * envelope * value * f64::from(i16::MAX)) as i16);
    }
    std::fs::write(path, wav(&samples)).unwrap();
}

/// Mono 16-bit PCM WAV.
fn wav(samples: &[i16]) -> Vec<u8> {
    let data_bytes = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + data_bytes).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(RATE.to_le_bytes());
    bytes.extend((RATE * 2).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data_bytes.to_le_bytes());
    bytes.extend(samples.iter().flat_map(|sample| sample.to_le_bytes()));
    bytes
}

fn directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("veltrunox-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

async fn library(url: &str) -> Vec<LibraryEntry> {
    load_library(&HttpServerLink::new(url)).await.unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn an_imported_recording_is_analysed_in_the_background_and_clients_are_told() {
    let dir = directory("background");
    write_music(&dir.join("take.wav"));
    let store = std::sync::Arc::new(SqliteStore::in_memory().unwrap());
    import_directory(&dir, store.as_ref()).unwrap();
    let url = start_server(store.clone()).await;
    let mut watcher = TestClient::connect(&url);

    // Playable at once; the analysis is not there yet.
    assert!(library(&url).await[0].analysis.result.is_none());

    analyse_again(&HttpServerLink::new(&url), RecordingId::new(1))
        .await
        .unwrap();
    let announced = watcher.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::AnalysisChanged(id)) => Some(*id),
        _ => None,
    });
    assert_eq!(announced, RecordingId::new(1));

    let found = loop {
        let entry = library(&url).await.remove(0);
        if entry.analysis.progress == AnalysisProgress::Done {
            break entry.analysis.result.unwrap();
        }
        assert!(
            !matches!(entry.analysis.progress, AnalysisProgress::Failed(_)),
            "{:?}",
            entry.analysis.progress
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    };
    assert_eq!(
        (found.key.tonic.index(), found.key.mode),
        (0, KeyMode::Major)
    );
    assert!(
        (found.tempo.value() - 100.0).abs() < 1.0,
        "{}",
        found.tempo.value()
    );
    assert!(
        (found.tuning.hertz() - 440.0).abs() < 1.0,
        "{}",
        found.tuning.hertz()
    );
    // The result is stored, not only held in memory (compared without the last bits of the
    // floating-point values, which the wire format does not keep).
    let stored = store.analysis(RecordingId::new(1)).unwrap().unwrap();
    assert_eq!(stored.key, found.key);
    assert!((stored.tempo.value() - found.tempo.value()).abs() < 1e-9);
    assert!((stored.tuning.hertz() - found.tuning.hertz()).abs() < 1e-9);
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn a_recording_without_audio_reports_why_it_cannot_be_analysed() {
    let url = start_server(support::store_with_recording()).await;
    analyse_again(&HttpServerLink::new(&url), RecordingId::new(1))
        .await
        .unwrap();

    let progress = loop {
        let progress = library(&url).await.remove(0).analysis.progress;
        if matches!(progress, AnalysisProgress::Failed(_)) {
            break progress;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    assert_eq!(
        progress,
        AnalysisProgress::Failed("the recording has no audio file".into())
    );
}

#[tokio::test]
async fn analysing_an_unknown_recording_is_not_found() {
    let url = start_server(support::store_with_recording()).await;
    let response = reqwest::Client::new()
        .post(format!("{url}/api/v1/recordings/99/analysis"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
}
