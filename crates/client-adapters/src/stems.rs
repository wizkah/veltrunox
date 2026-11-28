//! Stem requests (ADR-015).

use crate::{HttpServerLink, checked};
use reqwest::header::CONTENT_TYPE;
use veltrunox_client_app::{
    ImportedPackage, LinkError, RecordingWaveform, StemJob, StemLink, StemOverview, StemSummary,
};
use veltrunox_core::{RecordingId, SplitModel};
use veltrunox_protocol::{
    ImportedStemsDto, STEM_ANALYSIS_PATH, STEM_PACKAGE_PATH, STEM_PACKAGES_PATH, STEM_REGIONS_PATH,
    STEM_SELECTION_PATH, STEM_WAVEFORM_PATH, STEMS_PATH, SelectStemsDto, StemAnalysisDto,
    StemListDto,
};

fn invalid(error: reqwest::Error) -> LinkError {
    LinkError::InvalidResponse(error.to_string())
}

fn unreachable(error: reqwest::Error) -> LinkError {
    LinkError::Unreachable(error.to_string())
}

impl StemLink for HttpServerLink {
    async fn fetch_stems(&self, recording: RecordingId) -> Result<StemOverview, LinkError> {
        let response = self
            .get(self.url(STEMS_PATH, recording.value()))
            .send()
            .await
            .map_err(unreachable)
            .and_then(checked)?;
        let list: StemListDto = response.json().await.map_err(invalid)?;
        Ok(StemOverview {
            creating_regions: list.creating_regions,
            sets: list.sets,
            selected: list.selected,
            stems: list
                .stems
                .into_iter()
                .map(|stem| StemSummary {
                    name: stem.name,
                    key: stem.key,
                })
                .collect(),
            analysis_available: list.analysis_available,
            job: match list.analysis {
                StemAnalysisDto::None => StemJob::None,
                StemAnalysisDto::Queued => StemJob::Queued,
                StemAnalysisDto::Running { percent } => StemJob::Running(percent),
                StemAnalysisDto::Failed { code } => StemJob::Failed(code),
            },
        })
    }

    async fn request_stem_analysis(
        &self,
        recording: RecordingId,
        model: SplitModel,
    ) -> Result<(), LinkError> {
        let url = self.url(STEM_ANALYSIS_PATH, recording.value());
        let request = self.post(format!("{url}?model={}", model.id()));
        Self::account_call(request).await.map(|_| ())
    }

    async fn request_stem_regions(&self, recording: RecordingId) -> Result<(), LinkError> {
        let request = self.post(self.url(STEM_REGIONS_PATH, recording.value()));
        Self::account_call(request).await.map(|_| ())
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

    async fn select_stems(&self, recording: RecordingId, set: &str) -> Result<(), LinkError> {
        let url = self.url(STEM_SELECTION_PATH, recording.value());
        let request = self.put(url).json(&SelectStemsDto {
            model: set.to_owned(),
        });
        Self::send_write(request).await.map(|_| ())
    }

    async fn remove_stems(
        &self,
        recording: RecordingId,
        set: Option<&str>,
    ) -> Result<(), LinkError> {
        let mut request = self.delete(self.url(STEMS_PATH, recording.value()));
        if let Some(set) = set {
            request = request.query(&[("model", set)]);
        }
        Self::send_write(request).await.map(|_| ())
    }
}
