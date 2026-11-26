//! Stems of a recording: the endpoints and the manifest of a stem package (ADR-015).

use serde::{Deserialize, Serialize};

/// Takes a stem package (`POST`, the ZIP file as the raw body).
pub const STEM_PACKAGES_PATH: &str = "/api/v1/stem-packages";
/// The stems of one recording (`GET` lists them, `DELETE` removes them).
pub const STEMS_PATH: &str = "/api/v1/recordings/{id}/stems";
/// The stem package of one recording as a ZIP file.
pub const STEM_PACKAGE_PATH: &str = "/api/v1/recordings/{id}/stems/package";
/// Waveform description of one stem (`WaveformInfoDto`); `{index}` counts from 0.
pub const STEM_WAVEFORM_PATH: &str = "/api/v1/recordings/{id}/stems/{index}/waveform";
/// Peaks of one waveform level of a stem, as for a recording.
pub const STEM_WAVEFORM_LEVEL_PATH: &str = "/api/v1/recordings/{id}/stems/{index}/waveform/{level}";

/// Name of the manifest inside a package.
pub const MANIFEST_FILE: &str = "manifest.json";
/// Version of the package format.
pub const PACKAGE_FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifestDto {
    pub format_version: u32,
    pub recording: PackageRecordingDto,
    /// In the order in which they are shown.
    pub stems: Vec<PackageStemDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<PackageAnalysisDto>,
}

/// The recording the stems belong to, known by the SHA-256 of its audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRecordingDto {
    pub content_hash: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageStemDto {
    pub name: String,
    /// Path of the audio file inside the package.
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageAnalysisDto {
    #[serde(default)]
    pub bpm: Option<u32>,
    #[serde(default)]
    pub beats_ms: Vec<u64>,
    #[serde(default)]
    pub downbeats_ms: Vec<u64>,
    #[serde(default)]
    pub sections: Vec<PackageSectionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSectionDto {
    pub start_ms: u64,
    pub end_ms: u64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemDto {
    pub name: String,
    pub length_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemListDto {
    pub stems: Vec<StemDto>,
}

/// Answer to an imported package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedStemsDto {
    pub recording: u64,
    pub title: String,
    pub stems: u32,
    pub regions_added: u32,
}
