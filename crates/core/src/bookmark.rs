//! Bookmarks of a recording: labels (a point in time) and regions (a span).

use crate::units::{InvalidValue, Millis, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookmarkId(u64);

impl BookmarkId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// Per-record counter for optimistic concurrency: a write names the version it saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

impl Version {
    pub const FIRST: Self = Self(1);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// A non-empty, trimmed bookmark name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkName(String);

impl BookmarkName {
    pub fn new(text: &str) -> Result<Self, InvalidValue> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(InvalidValue::EmptyName);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub id: BookmarkId,
    pub name: BookmarkName,
    pub position: Millis,
    pub version: Version,
}

/// Regions may overlap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub id: BookmarkId,
    pub name: BookmarkName,
    pub range: TimeRange,
    pub version: Version,
}

/// All bookmarks of one recording, ordered by position.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bookmarks {
    pub labels: Vec<Label>,
    pub regions: Vec<Region>,
}

/// Outcome of a write whose expected version did not match the stored one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionConflict {
    pub current: Version,
}

/// Decides whether a write that saw `expected` may replace a record at `current`.
pub fn check_version(current: Version, expected: Version) -> Result<Version, VersionConflict> {
    if current == expected {
        Ok(current.next())
    } else {
        Err(VersionConflict { current })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_version_advances() {
        assert_eq!(
            check_version(Version::new(3), Version::new(3)),
            Ok(Version::new(4))
        );
    }

    #[test]
    fn stale_version_is_a_conflict_that_names_the_current_one() {
        assert_eq!(
            check_version(Version::new(4), Version::new(3)),
            Err(VersionConflict {
                current: Version::new(4)
            })
        );
    }

    #[test]
    fn name_is_trimmed_and_must_not_be_blank() {
        assert_eq!(BookmarkName::new(" Chorus ").unwrap().as_str(), "Chorus");
        assert_eq!(BookmarkName::new("  "), Err(InvalidValue::EmptyName));
    }
}
