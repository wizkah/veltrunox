//! The stems of a recording (ADR-015): the port to the server's stem requests.

use crate::{LinkError, RecordingWaveform};
use std::future::Future;
use veltrunox_core::{RecordingId, SplitStems};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemSummary {
    pub name: String,
    /// Changes with the content of the stem.
    pub key: String,
}

/// The analysis job of a recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StemJob {
    None,
    Queued,
    Running(u8),
    /// The code of the reason.
    Failed(String),
}

/// The stems of a recording and what can become of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemOverview {
    pub stems: Vec<StemSummary>,
    /// Whether the server can analyse recordings.
    pub analysis_available: bool,
    pub job: StemJob,
    /// Whether the job creates regions rather than stems.
    pub creating_regions: bool,
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
    fn fetch_stems(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<StemOverview, LinkError>>;
    /// Asks the server to analyse the recording into stems; the job shows in the overview.
    fn request_stem_analysis(
        &self,
        recording: RecordingId,
        stems: SplitStems,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// Asks the server to create regions from the structure of the stems.
    fn request_stem_regions(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<(), LinkError>>;
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
