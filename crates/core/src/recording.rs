use std::fmt;

/// Identifies a recording in the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordingId(u64);

impl RecordingId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for RecordingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A non-empty, trimmed recording title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingTitle(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRecording {
    #[error("recording title must not be empty")]
    EmptyTitle,
}

impl RecordingTitle {
    pub fn new(text: &str) -> Result<Self, InvalidRecording> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(InvalidRecording::EmptyTitle);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A recording as listed in the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    pub id: RecordingId,
    pub title: RecordingTitle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_trimmed() {
        let title = RecordingTitle::new("  Blue Monday ").unwrap();
        assert_eq!(title.as_str(), "Blue Monday");
    }

    #[test]
    fn blank_title_is_rejected() {
        assert_eq!(
            RecordingTitle::new("   "),
            Err(InvalidRecording::EmptyTitle)
        );
    }
}
