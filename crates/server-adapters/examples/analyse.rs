//! Prints the analysis of audio files: `cargo run --release -p veltrunox-server-adapters --example analyse -- FILE...`

use std::path::Path;
use std::time::Instant;
use veltrunox_server_adapters::{analyse, decode_file};

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn main() {
    for argument in std::env::args().skip(1) {
        let path = Path::new(&argument);
        let started = Instant::now();
        let result = decode_file(path)
            .map_err(|error| error.to_string())
            .and_then(|pcm| {
                let decoded = started.elapsed();
                analyse(&pcm)
                    .map(|found| (pcm.duration_ms(), decoded, found))
                    .map_err(|error| error.to_string())
            });
        match result {
            Ok((duration_ms, decoded, found)) => println!(
                "{}\n  {:.0} s, decode {:.1} s, analysis {:.1} s\n  A4 {:.2} Hz (x{:.2}), key {} {:?} (x{:.3}), {:.1} bpm (x{:.2})",
                path.display(),
                duration_ms as f64 / 1000.0,
                decoded.as_secs_f64(),
                (started.elapsed() - decoded).as_secs_f64(),
                found.tuning.hertz(),
                found.tuning_confidence.ratio(),
                NOTE_NAMES[usize::from(found.key.tonic.index())],
                found.key.mode,
                found.key_confidence.ratio(),
                found.tempo.value(),
                found.tempo_confidence.ratio(),
            ),
            Err(error) => println!("{}: {error}", path.display()),
        }
    }
}
