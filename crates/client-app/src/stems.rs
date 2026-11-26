//! The stems of a recording (ADR-015): the port to the server's stem requests.

use crate::{LinkError, RecordingWaveform};
use std::future::Future;
use veltrunox_core::RecordingId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemSummary {
    pub name: String,
    /// Changes with the content of the stem.
    pub key: String,
}

/// What importing a stem package came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedPackage {
    pub recording: RecordingId,
    pub title: String,
    pub stems: u32,
    pub regions_added: u32,
}

/// Port: stem requests to the server. As with `ServerLink` the futures need not be `Send`.
pub trait StemLink {
    /// Empty if the recording has no stems.
    fn fetch_stems(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Vec<StemSummary>, LinkError>>;
    /// Length and peaks of the stem at `index`, counted from 0.
    fn fetch_stem_waveform(
        &self,
        recording: RecordingId,
        index: usize,
    ) -> impl Future<Output = Result<RecordingWaveform, LinkError>>;
    /// Sends a stem package; its content says which recording it belongs to.
    fn send_stem_package(
        &self,
        package: Vec<u8>,
    ) -> impl Future<Output = Result<ImportedPackage, LinkError>>;
    fn fetch_stem_package(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Vec<u8>, LinkError>>;
    fn remove_stems(&self, recording: RecordingId) -> impl Future<Output = Result<(), LinkError>>;
}
