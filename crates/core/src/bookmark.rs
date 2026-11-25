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

/// A label that is not stored yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLabel {
    pub name: BookmarkName,
    pub position: Millis,
}

/// A region that is not stored yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRegion {
    pub name: BookmarkName,
    pub range: TimeRange,
}

/// The complete set of bookmarks a recording is to have; it replaces what is stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewBookmarks {
    pub labels: Vec<NewLabel>,
    pub regions: Vec<NewRegion>,
}

impl NewBookmarks {
    /// Every label must lie inside the recording and every region must end within it.
    pub fn check_within(&self, length: Millis) -> Result<(), InvalidValue> {
        let labels_inside = self.labels.iter().all(|label| label.position <= length);
        let regions_inside = self
            .regions
            .iter()
            .all(|region| region.range.end() <= length);
        if labels_inside && regions_inside {
            Ok(())
        } else {
            Err(InvalidValue::OutsideRecording)
        }
    }
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

    fn set(label_at: u64, region_end: u64) -> NewBookmarks {
        NewBookmarks {
            labels: vec![NewLabel {
                name: BookmarkName::new("A").unwrap(),
                position: Millis::new(label_at),
            }],
            regions: vec![NewRegion {
                name: BookmarkName::new("B").unwrap(),
                range: TimeRange::new(Millis::new(0), Millis::new(region_end)).unwrap(),
            }],
        }
    }

    #[test]
    fn bookmarks_up_to_the_end_of_the_recording_are_inside() {
        assert_eq!(set(1_000, 1_000).check_within(Millis::new(1_000)), Ok(()));
    }

    #[test]
    fn a_label_or_a_region_past_the_end_is_refused() {
        let length = Millis::new(1_000);
        assert_eq!(
            set(1_001, 500).check_within(length),
            Err(InvalidValue::OutsideRecording)
        );
        assert_eq!(
            set(500, 1_001).check_within(length),
            Err(InvalidValue::OutsideRecording)
        );
    }

    #[test]
    fn name_is_trimmed_and_must_not_be_blank() {
        assert_eq!(BookmarkName::new(" Chorus ").unwrap().as_str(), "Chorus");
        assert_eq!(BookmarkName::new("  "), Err(InvalidValue::EmptyName));
    }
}
