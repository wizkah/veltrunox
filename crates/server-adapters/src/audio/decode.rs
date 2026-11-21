//! Decodes an audio file completely into stereo `f32` samples.

use super::engine::Pcm;
use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, TrackType, probe::Hint};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("cannot read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported or damaged audio: {0}")]
    Audio(String),
}

impl From<Error> for DecodeError {
    fn from(error: Error) -> Self {
        Self::Audio(error.to_string())
    }
}

/// Decodes the whole file; mono is doubled, channels beyond the second are dropped.
pub fn decode_file(path: &Path) -> Result<Pcm, DecodeError> {
    let file = Box::new(File::open(path)?);
    let stream = MediaSourceStream::new(file, Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }
    let mut format = symphonia::default::get_probe().probe(
        &hint,
        stream,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| DecodeError::Audio("no audio track".to_owned()))?;
    let parameters = track
        .codec_params
        .as_ref()
        .and_then(|parameters| parameters.audio())
        .ok_or_else(|| DecodeError::Audio("no audio parameters".to_owned()))?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(parameters, &AudioDecoderOptions::default())?;
    let track_id = track.id;

    let mut stereo = Vec::new();
    let mut interleaved: Vec<f32> = Vec::new();
    let mut sample_rate = 0;
    while let Some(packet) = format.next_packet()? {
        if packet.track_id != track_id {
            continue;
        }
        let buffer = match decoder.decode(&packet) {
            Ok(buffer) => buffer,
            Err(Error::DecodeError(_)) => continue,
            Err(error) => return Err(error.into()),
        };
        sample_rate = buffer.spec().rate();
        let channels = buffer.spec().channels().count();
        interleaved.resize(buffer.samples_interleaved(), 0.0);
        buffer.copy_to_slice_interleaved(&mut interleaved);
        append_stereo(&mut stereo, &interleaved, channels);
    }
    if sample_rate == 0 || stereo.is_empty() {
        return Err(DecodeError::Audio("no audio samples".to_owned()));
    }
    Ok(Pcm::new(stereo, sample_rate))
}

fn append_stereo(stereo: &mut Vec<f32>, interleaved: &[f32], channels: usize) {
    for frame in interleaved.chunks_exact(channels.max(1)) {
        let left = frame[0];
        let right = if channels > 1 { frame[1] } else { left };
        stereo.extend([left, right]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_is_doubled_and_extra_channels_dropped() {
        let mut stereo = Vec::new();
        append_stereo(&mut stereo, &[0.5, -0.5], 1);
        assert_eq!(stereo, [0.5, 0.5, -0.5, -0.5]);
        stereo.clear();
        append_stereo(&mut stereo, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        assert_eq!(stereo, [1.0, 2.0, 4.0, 5.0]);
    }

    #[test]
    fn decodes_a_wav_file() {
        let path =
            std::env::temp_dir().join(format!("veltrunox-decode-{}.wav", std::process::id()));
        std::fs::write(&path, wav_mono_16bit(8_000, &[0, 16_384, -16_384, 0])).unwrap();
        let pcm = decode_file(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(pcm.sample_rate(), 8_000);
        assert_eq!(pcm.frames(), 4);
    }

    #[test]
    fn a_text_file_is_not_audio() {
        let path =
            std::env::temp_dir().join(format!("veltrunox-decode-{}.txt", std::process::id()));
        std::fs::write(&path, "not audio at all").unwrap();
        let result = decode_file(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(matches!(result, Err(DecodeError::Audio(_))));
    }

    fn wav_mono_16bit(rate: u32, samples: &[i16]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::new();
        bytes.extend(b"RIFF");
        bytes.extend((36 + data_len).to_le_bytes());
        bytes.extend(b"WAVEfmt ");
        bytes.extend(16u32.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(rate.to_le_bytes());
        bytes.extend((rate * 2).to_le_bytes());
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(16u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(data_len.to_le_bytes());
        for sample in samples {
            bytes.extend(sample.to_le_bytes());
        }
        bytes
    }
}
