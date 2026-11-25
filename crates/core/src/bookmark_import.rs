//! Planning an import: what a set of bookmarks coming from outside would change, and whether
//! someone else changed the bookmarks since the set was exported.

use crate::bookmark::{
    BookmarkId, BookmarkName, Bookmarks, EntryRef, NewBookmarks, NewLabel, NewRegion, Version,
};
use std::collections::HashSet;

/// The id and version of the stored entry an imported entry was exported from.
pub type KnownEntry = (BookmarkId, Version);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedLabel {
    /// Present when the entry carries the id and version of a stored label.
    pub known: Option<KnownEntry>,
    pub label: NewLabel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedRegion {
    pub known: Option<KnownEntry>,
    pub region: NewRegion,
}

/// The bookmarks a recording is to have after an import.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportedBookmarks {
    pub labels: Vec<ImportedLabel>,
    pub regions: Vec<ImportedRegion>,
}

impl ImportedBookmarks {
    /// The set to store, without the references to stored entries.
    pub fn to_new(&self) -> NewBookmarks {
        NewBookmarks {
            labels: self
                .labels
                .iter()
                .map(|entry| entry.label.clone())
                .collect(),
            regions: self
                .regions
                .iter()
                .map(|entry| entry.region.clone())
                .collect(),
        }
    }
}

/// How an import relates to what is stored. Entries of the import that name a stored entry
/// are `changed` or `unchanged`; the others are `added`; stored entries the import leaves out
/// are `removed`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportDiff {
    pub added: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub removed: usize,
}

impl ImportDiff {
    fn plus(self, other: Self) -> Self {
        Self {
            added: self.added + other.added,
            changed: self.changed + other.changed,
            unchanged: self.unchanged + other.unchanged,
            removed: self.removed + other.removed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    /// The stored entry the import was made from is gone.
    Missing,
    /// The stored entry changed after the export; this is its version now.
    Stale(Version),
}

/// An imported entry whose stored original is gone or was changed meanwhile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportConflict {
    pub entry: EntryRef,
    pub kind: ConflictKind,
}

/// Compares an import with the stored bookmarks. Unless `overwrite` is set, an entry that names
/// a stored entry must find it at the same version; the first one that does not is a conflict.
pub fn plan_import(
    stored: &Bookmarks,
    incoming: &ImportedBookmarks,
    overwrite: bool,
) -> Result<ImportDiff, ImportConflict> {
    let stored_labels: Vec<_> = stored
        .labels
        .iter()
        .map(|label| ((label.id, label.version), (&label.name, label.position)))
        .collect();
    let incoming_labels: Vec<_> = incoming
        .labels
        .iter()
        .map(|entry| (entry.known, (&entry.label.name, entry.label.position)))
        .collect();
    let labels = plan_kind(&stored_labels, &incoming_labels, overwrite, EntryRef::Label)?;

    let stored_regions: Vec<_> = stored
        .regions
        .iter()
        .map(|region| ((region.id, region.version), (&region.name, region.range)))
        .collect();
    let incoming_regions: Vec<_> = incoming
        .regions
        .iter()
        .map(|entry| (entry.known, (&entry.region.name, entry.region.range)))
        .collect();
    let regions = plan_kind(
        &stored_regions,
        &incoming_regions,
        overwrite,
        EntryRef::Region,
    )?;
    Ok(labels.plus(regions))
}

type Content<'a, T> = (&'a BookmarkName, T);

/// The planning of one kind of bookmark; `T` is what makes an entry of that kind equal to
/// another (a position or a range).
fn plan_kind<T: PartialEq + Copy>(
    stored: &[(KnownEntry, Content<'_, T>)],
    incoming: &[(Option<KnownEntry>, Content<'_, T>)],
    overwrite: bool,
    entry: fn(usize) -> EntryRef,
) -> Result<ImportDiff, ImportConflict> {
    let mut diff = ImportDiff::default();
    let mut matched = HashSet::new();
    for (index, (known, content)) in incoming.iter().enumerate() {
        let original = known.and_then(|(id, version)| {
            let found = stored.iter().find(|((stored_id, _), _)| *stored_id == id);
            Some((version, found?))
        });
        match (known, original) {
            (Some(_), None) if !overwrite => {
                return Err(ImportConflict {
                    entry: entry(index),
                    kind: ConflictKind::Missing,
                });
            }
            (Some(_), Some((version, ((_, current), _)))) if !overwrite && version != *current => {
                return Err(ImportConflict {
                    entry: entry(index),
                    kind: ConflictKind::Stale(*current),
                });
            }
            _ => {}
        }
        match original {
            Some((_, ((id, _), stored_content))) if matched.insert(*id) => {
                if stored_content == content {
                    diff.unchanged += 1;
                } else {
                    diff.changed += 1;
                }
            }
            _ => diff.added += 1,
        }
    }
    diff.removed = stored.len() - matched.len();
    Ok(diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmark::{Label, Region};
    use crate::units::{Millis, TimeRange};

    fn name(text: &str) -> BookmarkName {
        BookmarkName::new(text).unwrap()
    }

    fn range(start: u64, end: u64) -> TimeRange {
        TimeRange::new(Millis::new(start), Millis::new(end)).unwrap()
    }

    fn stored() -> Bookmarks {
        Bookmarks {
            labels: vec![Label {
                id: BookmarkId::new(1),
                name: name("Verse"),
                position: Millis::new(10),
                version: Version::new(2),
            }],
            regions: vec![Region {
                id: BookmarkId::new(7),
                name: name("Chorus"),
                range: range(20, 30),
                version: Version::new(1),
            }],
        }
    }

    fn label(known: Option<(u64, u64)>, text: &str, at: u64) -> ImportedLabel {
        ImportedLabel {
            known: known.map(|(id, version)| (BookmarkId::new(id), Version::new(version))),
            label: NewLabel {
                name: name(text),
                position: Millis::new(at),
            },
        }
    }

    fn region(known: Option<(u64, u64)>, text: &str, span: (u64, u64)) -> ImportedRegion {
        ImportedRegion {
            known: known.map(|(id, version)| (BookmarkId::new(id), Version::new(version))),
            region: NewRegion {
                name: name(text),
                range: range(span.0, span.1),
            },
        }
    }

    #[test]
    fn the_stored_set_imported_unchanged_changes_nothing() {
        let incoming = ImportedBookmarks {
            labels: vec![label(Some((1, 2)), "Verse", 10)],
            regions: vec![region(Some((7, 1)), "Chorus", (20, 30))],
        };
        assert_eq!(
            plan_import(&stored(), &incoming, false),
            Ok(ImportDiff {
                unchanged: 2,
                ..ImportDiff::default()
            })
        );
    }

    #[test]
    fn entries_are_counted_as_changed_added_and_removed() {
        let incoming = ImportedBookmarks {
            labels: vec![label(Some((1, 2)), "Verse", 11), label(None, "Bridge", 40)],
            regions: vec![],
        };
        assert_eq!(
            plan_import(&stored(), &incoming, false),
            Ok(ImportDiff {
                added: 1,
                changed: 1,
                unchanged: 0,
                removed: 1,
            })
        );
    }

    #[test]
    fn an_entry_whose_original_changed_since_the_export_is_a_conflict() {
        let incoming = ImportedBookmarks {
            labels: vec![label(Some((1, 1)), "Verse", 10)],
            regions: vec![],
        };
        assert_eq!(
            plan_import(&stored(), &incoming, false),
            Err(ImportConflict {
                entry: EntryRef::Label(0),
                kind: ConflictKind::Stale(Version::new(2)),
            })
        );
    }

    #[test]
    fn an_entry_whose_original_is_gone_is_a_conflict() {
        let incoming = ImportedBookmarks {
            labels: vec![],
            regions: vec![region(Some((9, 1)), "Gone", (1, 2))],
        };
        assert_eq!(
            plan_import(&stored(), &incoming, false),
            Err(ImportConflict {
                entry: EntryRef::Region(0),
                kind: ConflictKind::Missing,
            })
        );
    }

    #[test]
    fn overwriting_ignores_versions_and_missing_originals() {
        let incoming = ImportedBookmarks {
            labels: vec![label(Some((1, 1)), "Verse", 10)],
            regions: vec![region(Some((9, 1)), "Gone", (1, 2))],
        };
        assert_eq!(
            plan_import(&stored(), &incoming, true),
            Ok(ImportDiff {
                added: 1,
                changed: 0,
                unchanged: 1,
                removed: 1,
            })
        );
    }

    #[test]
    fn an_original_named_twice_counts_once() {
        let incoming = ImportedBookmarks {
            labels: vec![
                label(Some((1, 2)), "Verse", 10),
                label(Some((1, 2)), "Verse", 10),
            ],
            regions: vec![],
        };
        let diff = plan_import(&stored(), &incoming, false).unwrap();
        assert_eq!((diff.unchanged, diff.added, diff.removed), (1, 1, 1));
    }
}
