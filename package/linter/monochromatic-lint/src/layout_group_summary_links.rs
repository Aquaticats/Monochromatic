//! What:
//!  The `uses` summary of a group folder:
//!  one link for every other entry that a folder inside the group uses,
//!  and no other link.
//! Why:
//!  The owner's rule gives each group a `uses` folder that shows,
//!  from the file tree alone,
//!  which other groups and one-folder entries its child folders depend on.
//!  A use of a folder of the same group,
//!  of a group itself (a finding of its own),
//!  and a use by a group's own code (a finding of the shelf rule) are not part of a summary.
//!  A group inside a group has no summary,
//!  because the rule is one level of groups.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function summaryFindings(groups: Set<Path>, grouped: FolderUses, sources: Map<Path, Path>, base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the folder uses,
///  the group entries and the link reader.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_folder_uses::{FolderUses, Proof},
    layout_group_entries::{entry_of, owner_of},
    layout_link_entries::{Link, USES_FOLDER, link_command, read_links},
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// What:
///  One use of an entry by a folder inside a group,
///  kept for the finding about a missing summary link.
/// Why:
///  The finding names the folder that uses and the folder used,
///  and sits on the reference that proves it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SummaryUse = { from: Path; to: Path; proof: Proof };
/// ```
struct SummaryUse {
    /// The folder inside the group whose code uses.
    from: PathBuf,
    /// The folder it names,
    ///  which is inside the entry or is the entry.
    to: PathBuf,
    /// The reference that proves the use.
    proof: Proof,
}

/// What:
///  The entries each group's child folders use,
///  with the smallest proving use of each.
/// Why:
///  A group's `uses` folder must link exactly these.
///  A use of a folder of the same group,
///  of the group itself (reported on its own) and a use by the group's own code are not part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function summaries(groups: Set<Path>, grouped: FolderUses): Map<Path, Map<Path, SummaryUse>>;
/// ```
fn summaries(
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
) -> BTreeMap<PathBuf, BTreeMap<PathBuf, SummaryUse>> {
    let mut needed: BTreeMap<PathBuf, BTreeMap<PathBuf, SummaryUse>> =
        BTreeMap::<PathBuf, BTreeMap<PathBuf, SummaryUse>>::new();
    for (from, targets) in grouped {
        if groups.contains(from) {
            continue;
        }
        let Some(owner) = owner_of(from, groups) else {
            continue;
        };
        for (to, proof) in targets {
            if groups.contains(to) {
                continue;
            }
            let entry: PathBuf = entry_of(to, groups);
            if entry == owner {
                continue;
            }
            let slot: &mut BTreeMap<PathBuf, SummaryUse> = needed.entry(owner.clone()).or_default();
            let smaller: bool = match slot.get(&entry) {
                Some(existing) => *proof < existing.proof,
                None => true,
            };
            if smaller {
                slot.insert(
                    entry,
                    SummaryUse {
                        from: from.clone(),
                        to: to.clone(),
                        proof: proof.clone(),
                    },
                );
            }
        }
    }
    return needed;
}

/// What:
///  The finding for a summary link that no folder inside the group needs.
/// Why:
///  A link into another group names a folder inside the group,
///  and the summary links the group itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extraLink(group: Path, link: Link, groups: Set<Path>, base: Path): LayoutFinding;
/// ```
fn extra_link(group: &Path, link: &Link, groups: &BTreeSet<PathBuf>, base: &Path) -> LayoutFinding {
    let inside: Option<PathBuf> = match owner_of(&link.target, groups) {
        Some(other) if other != group => Some(other),
        _ => None,
    };
    let (message, help): (String, String) = match inside {
        Some(other) => (
            format!(
                "{} links to {}, which is inside the group {}, so a group links the group and not a folder inside it.",
                shown(&link.entry, base),
                shown(&link.target, base),
                shown(&other, base)
            ),
            format!("Link the group {} instead.", shown(&other, base)),
        ),
        None => (
            format!(
                "{} links to {}, but no folder inside {} uses {}.",
                shown(&link.entry, base),
                shown(&link.target, base),
                shown(group, base),
                if groups.contains(&link.target) {
                    "a folder of it"
                } else {
                    "it"
                }
            ),
            String::from(
                "Remove the link, or use the entry it points to from a folder of the group.",
            ),
        ),
    };
    return LayoutFinding {
        file: link.entry.clone(),
        line: 0,
        message,
        help: Some(help),
    };
}

/// What:
///  The finding for an entry that a folder inside the group uses and the group does not link.
/// Why:
///  It sits on the reference that proves the use and says how to add the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingSummary(group: Path, entry: Path, used: SummaryUse, base: Path): LayoutFinding;
/// ```
fn missing_summary(group: &Path, entry: &Path, used: &SummaryUse, base: &Path) -> LayoutFinding {
    let inside: String = if used.to == entry {
        String::new()
    } else {
        format!(", which is in the group {}", shown(entry, base))
    };
    return LayoutFinding {
        file: used.proof.file.clone(),
        line: used.proof.line,
        message: format!(
            "{} uses the folder {} (`{}`){inside}, but {}/{USES_FOLDER} has no link to {}.",
            shown(&used.from, base),
            shown(&used.to, base),
            used.proof.text,
            shown(group, base),
            shown(entry, base)
        ),
        help: Some(format!(
            "Create the link: {}",
            link_command(group, entry, base)
        )),
    };
}

/// What:
///  The findings for the `uses` folder of one group:
///  the bad entries,
///  the links no folder inside needs,
///  and the used entries without a link.
/// Why:
///  A summary is right exactly when it links the entries the group's child folders use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSummary(group: Path, needed: Map<Path, SummaryUse>, groups: Set<Path>, sources: Map<Path, Path>, base: Path): LayoutFinding[];
/// ```
fn check_summary(
    group: &Path,
    needed: Option<&BTreeMap<PathBuf, SummaryUse>>,
    groups: &BTreeSet<PathBuf>,
    sources: &BTreeMap<PathBuf, PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let read = read_links(group, sources, base);
    let mut findings: Vec<LayoutFinding> = read.findings;
    let empty: BTreeMap<PathBuf, SummaryUse> = BTreeMap::<PathBuf, SummaryUse>::new();
    let wanted: &BTreeMap<PathBuf, SummaryUse> = needed.unwrap_or(&empty);
    let mut linked: BTreeSet<&PathBuf> = BTreeSet::<&PathBuf>::new();
    for link in &read.links {
        linked.insert(&link.target);
        if !wanted.contains_key(&link.target) {
            findings.push(extra_link(group, link, groups, base));
        }
    }
    for (entry, used) in wanted {
        if !linked.contains(entry) {
            findings.push(missing_summary(group, entry, used, base));
        }
    }
    return findings;
}

/// What:
///  The findings for the `uses` folders of every group.
/// Why:
///  Each group is checked against the entries its own child folders use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function summaryFindings(groups: Set<Path>, grouped: FolderUses, sources: Map<Path, Path>, base: Path): LayoutFinding[];
/// ```
pub fn summary_findings(
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
    sources: &BTreeMap<PathBuf, PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let needed = summaries(groups, grouped);
    for group in groups {
        if owner_of(group, groups).is_some() {
            continue;
        }
        findings.extend(check_summary(
            group,
            needed.get(group),
            groups,
            sources,
            base,
        ));
    }
    return findings;
}
