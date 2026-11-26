//! Stems (separate audio tracks of one recording) and the song sections that come with
//! them (ADR-015).

use crate::bookmark::{BookmarkName, NewRegion, Region};
use crate::units::{InvalidValue, Millis, TimeRange};
use std::collections::HashMap;

/// Most stems one recording can have.
pub const MAX_STEMS: usize = 8;
const MAX_NAME_CHARS: usize = 40;

/// A section boundary moves to a downbeat at most this far away.
const DOWNBEAT_REACH_MS: u64 = 2_000;
/// A boundary this close to the end of the recording stays: the last section ends with it.
const AT_END_MS: u64 = 500;
/// Regions whose ends differ by at most this are taken to be the same.
const SAME_RANGE_MS: u64 = 1_000;

/// The non-empty, trimmed name of a stem ("drums", "bass" …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemName(String);

impl StemName {
    pub fn new(text: &str) -> Result<Self, InvalidValue> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(InvalidValue::EmptyName);
        }
        Ok(Self(trimmed.chars().take(MAX_NAME_CHARS).collect()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A part of the song structure as an analysis found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub start: Millis,
    pub end: Millis,
    /// For example "verse" or "chorus"; "start" and "end" mark nothing.
    pub label: String,
}

/// The regions to add for the sections: named after their label with a running number
/// ("Verse 1"), their ends moved to the nearest downbeat and kept inside the recording.
/// A section is left out if a region with the same name or nearly the same range exists.
pub fn section_regions(
    sections: &[Section],
    downbeats: &[Millis],
    length: Millis,
    existing: &[Region],
) -> Vec<NewRegion> {
    let mut ordered: Vec<&Section> = sections
        .iter()
        .filter(|section| !matches!(section.label.trim(), "" | "start" | "end"))
        .collect();
    ordered.sort_by_key(|section| section.start);
    let mut counts: HashMap<String, u32> = HashMap::new();
    ordered
        .into_iter()
        .filter_map(|section| {
            let label = capitalised(section.label.trim());
            let number = counts.entry(label.clone()).or_default();
            *number += 1;
            let name = BookmarkName::new(&format!("{label} {number}")).ok()?;
            let start = snapped(section.start, downbeats).min(length);
            let end = if section.end.value() + AT_END_MS >= length.value() {
                length
            } else {
                snapped(section.end, downbeats).min(length)
            };
            let range = TimeRange::new(start, end).ok()?;
            let known = existing
                .iter()
                .any(|region| region.name == name || nearly_same(region.range, range));
            (!known).then_some(NewRegion { name, range })
        })
        .collect()
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn snapped(position: Millis, downbeats: &[Millis]) -> Millis {
    downbeats
        .iter()
        .copied()
        .min_by_key(|downbeat| downbeat.value().abs_diff(position.value()))
        .filter(|downbeat| downbeat.value().abs_diff(position.value()) <= DOWNBEAT_REACH_MS)
        .unwrap_or(position)
}

fn nearly_same(one: TimeRange, other: TimeRange) -> bool {
    one.start().value().abs_diff(other.start().value()) <= SAME_RANGE_MS
        && one.end().value().abs_diff(other.end().value()) <= SAME_RANGE_MS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmark::{BookmarkId, Version};

    fn section(start: u64, end: u64, label: &str) -> Section {
        Section {
            start: Millis::new(start),
            end: Millis::new(end),
            label: label.to_owned(),
        }
    }

    fn names(regions: &[NewRegion]) -> Vec<&str> {
        regions.iter().map(|region| region.name.as_str()).collect()
    }

    #[test]
    fn sections_are_numbered_per_label_and_markers_of_start_and_end_dropped() {
        let sections = [
            section(0, 500, "start"),
            section(500, 20_000, "verse"),
            section(20_000, 40_000, "chorus"),
            section(40_000, 60_000, "verse"),
        ];
        let regions = section_regions(&sections, &[], Millis::new(60_000), &[]);
        assert_eq!(names(&regions), ["Verse 1", "Chorus 1", "Verse 2"]);
    }

    #[test]
    fn ends_move_to_a_near_downbeat_and_stay_inside_the_recording() {
        let downbeats = [Millis::new(1_000), Millis::new(30_000)];
        let regions = section_regions(
            &[
                section(1_300, 12_000, "intro"),
                section(29_000, 70_000, "outro"),
            ],
            &downbeats,
            Millis::new(60_000),
            &[],
        );
        let ranges: Vec<_> = regions
            .iter()
            .map(|region| (region.range.start().value(), region.range.end().value()))
            .collect();
        assert_eq!(ranges, [(1_000, 12_000), (30_000, 60_000)]);
    }

    #[test]
    fn the_last_section_keeps_ending_with_the_recording() {
        let downbeats = [Millis::new(0), Millis::new(1_000)];
        let regions = section_regions(
            &[section(900, 2_000, "verse")],
            &downbeats,
            Millis::new(2_000),
            &[],
        );
        assert_eq!(regions[0].range.start().value(), 1_000);
        assert_eq!(regions[0].range.end().value(), 2_000);
    }

    #[test]
    fn known_names_and_ranges_are_not_added_again() {
        let existing = |name: &str, start: u64, end: u64| Region {
            id: BookmarkId::new(1),
            name: BookmarkName::new(name).unwrap(),
            range: TimeRange::new(Millis::new(start), Millis::new(end)).unwrap(),
            version: Version::FIRST,
        };
        let sections = [
            section(0, 10_000, "verse"),
            section(10_000, 20_000, "chorus"),
            section(20_000, 30_000, "bridge"),
        ];
        let regions = section_regions(
            &sections,
            &[],
            Millis::new(30_000),
            &[
                existing("Verse 1", 3_000, 5_000),
                existing("Refrain", 10_400, 19_600),
            ],
        );
        assert_eq!(names(&regions), ["Bridge 1"]);
    }
}
