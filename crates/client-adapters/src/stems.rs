//! Stem requests (ADR-015).

use crate::{HttpServerLink, checked};
use reqwest::header::CONTENT_TYPE;
use veltrunox_client_app::{ImportedPackage, LinkError, RecordingWaveform, StemLink, StemSummary};
use veltrunox_core::RecordingId;
use veltrunox_protocol::{
    ImportedStemsDto, STEM_PACKAGE_PATH, STEM_PACKAGES_PATH, STEM_WAVEFORM_PATH, STEMS_PATH,
    StemListDto,
};

fn invalid(error: reqwest::Error) -> LinkError {
    LinkError::InvalidResponse(error.to_string())
}

fn unreachable(error: reqwest::Error) -> LinkError {
    LinkError::Unreachable(error.to_string())
}

impl StemLink for HttpServerLink {
    async fn fetch_stems(&self, recording: RecordingId) -> Result<Vec<StemSummary>, LinkError> {
        let response = self
            .get(self.url(STEMS_PATH, recording.value()))
            .send()
            .await
            .map_err(unreachable)
            .and_then(checked)?;
        let list: StemListDto = response.json().await.map_err(invalid)?;
        Ok(list
            .stems
            .into_iter()
            .map(|stem| StemSummary {
                name: stem.name,
                key: stem.key,
            })
            .collect())
    }

    async fn fetch_stem_waveform(
        &self,
        recording: RecordingId,
        index: usize,
    ) -> Result<RecordingWaveform, LinkError> {
        let url = self
            .url(STEM_WAVEFORM_PATH, recording.value())
            .replace("{index}", &index.to_string());
        self.waveform_at(&url).await
    }

    async fn send_stem_package(&self, package: Vec<u8>) -> Result<ImportedPackage, LinkError> {
        let request = self
            .post(format!("{}{}", self.base_url, STEM_PACKAGES_PATH))
            .header(CONTENT_TYPE, "application/zip")
            .body(package);
        // An unknown recording answers 404 with its reason.
        let imported: ImportedStemsDto = Self::account_json(request).await?;
        Ok(ImportedPackage {
            recording: RecordingId::new(imported.recording),
            title: imported.title,
            stems: imported.stems,
            regions_added: imported.regions_added,
        })
    }

    async fn fetch_stem_package(&self, recording: RecordingId) -> Result<Vec<u8>, LinkError> {
        let response = self
            .get(self.url(STEM_PACKAGE_PATH, recording.value()))
            .send()
            .await
            .map_err(unreachable)
            .and_then(checked)?;
        Ok(response.bytes().await.map_err(unreachable)?.to_vec())
    }

    async fn remove_stems(&self, recording: RecordingId) -> Result<(), LinkError> {
        let request = self.delete(self.url(STEMS_PATH, recording.value()));
        Self::send_write(request).await.map(|_| ())
    }
}
