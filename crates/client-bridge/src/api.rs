//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.

use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::load_library;

/// A recording as the user interface needs it.
pub struct RecordingItem {
    pub id: u64,
    pub title: String,
}

/// Loads the recording list from the server at `server_url`.
pub async fn list_recordings(server_url: String) -> Result<Vec<RecordingItem>, String> {
    let link = HttpServerLink::new(&server_url);
    let recordings = load_library(&link)
        .await
        .map_err(|error| error.to_string())?;
    Ok(recordings
        .into_iter()
        .map(|recording| RecordingItem {
            id: recording.id.value(),
            title: recording.title.as_str().to_owned(),
        })
        .collect())
}
