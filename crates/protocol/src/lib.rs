//! Versioned wire messages for HTTP and the real-time channel. No system access.

use serde::{Deserialize, Serialize};

/// Path of the recording list endpoint.
pub const RECORDINGS_PATH: &str = "/api/v1/recordings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingSummaryDto {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingListDto {
    pub recordings: Vec<RecordingSummaryDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_round_trips_through_json() {
        let list = RecordingListDto {
            recordings: vec![RecordingSummaryDto {
                id: 1,
                title: "Intro".into(),
            }],
        };
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, r#"{"recordings":[{"id":1,"title":"Intro"}]}"#);
        assert_eq!(
            serde_json::from_str::<RecordingListDto>(&json).unwrap(),
            list
        );
    }
}
