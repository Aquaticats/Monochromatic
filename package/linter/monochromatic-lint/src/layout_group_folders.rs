//! What:
//!  Group folders of the folder rule:
//!  a folder that holds child folders holds nothing else,
//!  its `uses` folder summarizes the other groups and one-folder entries its child folders use,
//!  a child folder never links or names a group,
//!  and no two entries use each other.
//! Why:
//!  The owner's rule is a shelf with summaries (2026-11-07T11:05:21-05:00):
//!  the tree shows who depends on whom at two scales,
//!  the entries of a package and the exact folders inside each group.
//!  Decisions:
//!  a group is recognized by its child folders and needs no marker,
//!  so a group that grows code is still a group and the code is a finding;
//!  an **entry** is a group,
//!  or a source folder outside every group,
//!  and a folder inside a group has that group as its entry;
//!  a group's own code makes no use and no link,
//!  because the shelf rule reports the code and the summary is what its child folders use;
//!  a group inside a group is a finding and has no summary,
//!  because the rule is one level of groups.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findGroups(tree: ModuleTree): Set<Path>;
//! function entryOf(folder: Path, groups: Set<Path>): Path;
//! function groupFindings(tree: ModuleTree, groups: Set<Path>, grouped: FolderUses, sources: Map<Path, Path>, base: Path): LayoutFinding[];
//! function groupLoops(groups: Set<Path>, grouped: FolderUses, base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the file and index readers,
///  the use records and loops,
///  the link reader,
///  the index item check and the tree.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_folder_index_files::{file_name, files_by_folder, index_file},
    layout_folder_uses::{FolderUses, Proof, record_use, use_loops},
    layout_index_file_items::{IndexRole, check_index_items},
    layout_link_entries::{Link, USES_FOLDER, link_command, read_links},
    layout_module_tree::ModuleTree,
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
///  The group folders of a package.
/// Why:
///  A group is a folder whose index is a `mod.rs` and that has a source folder directly inside it;
///  the folder of a crate root is where the entries sit and is never a group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findGroups(tree: ModuleTree): Set<Path>;
/// ```
pub fn find_groups(tree: &ModuleTree) -> BTreeSet<PathBuf> {
    let (by_folder, roots) = files_by_folder(tree);
    let mut parents: BTreeSet<&Path> = BTreeSet::<&Path>::new();
    for folder in by_folder.keys() {
        if let Some(parent) = folder.parent() {
            parents.insert(parent);
        }
    }
    let mut groups: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for (folder, files) in &by_folder {
        let Some(index) = index_file(folder, files, &roots) else {
            continue;
        };
        if file_name(&index) == "mod.rs" && parents.contains(folder.as_path()) {
            groups.insert(folder.clone());
        }
    }
    return groups;
}

/// What:
///  The outermost group above a folder,
///  not counting the folder itself.
/// Why:
///  Every folder inside a group counts for that group's summary,
///  and the rule has one level of groups,
///  so a nested group counts for the group around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerOf(folder: Path, groups: Set<Path>): Path | undefined;
/// ```
fn owner_of(folder: &Path, groups: &BTreeSet<PathBuf>) -> Option<PathBuf> {
    return folder
        .ancestors()
        .skip(1)
        .filter(|ancestor| return groups.contains(*ancestor))
        .last()
        .map(Path::to_path_buf);
}

/// What:
///  The entry a folder belongs to:
///  its outermost group,
///  or the folder itself outside every group.
/// Why:
///  The groups and the one-folder entries are the nodes of the loop check and of the summaries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(folder: Path, groups: Set<Path>): Path;
/// ```
pub fn entry_of(folder: &Path, groups: &BTreeSet<PathBuf>) -> PathBuf {
    return match owner_of(folder, groups) {
        Some(group) => group,
        None => folder.to_path_buf(),
    };
}

/// What:
///  The findings about what a group folder holds.
/// Why:
///  A group holds only child folders:
///  its `mod.rs` holds only its top doc and `mod` lines,
///  and it holds no other module file.
///  A group inside a group is its own finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shelfFindings(tree: ModuleTree, groups: Set<Path>, base: Path): LayoutFinding[];
/// ```
fn shelf_findings(
    tree: &ModuleTree,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let (by_folder, _roots) = files_by_folder(tree);
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for group in groups {
        let index: PathBuf = group.join("mod.rs");
        if let Some(outer) = owner_of(group, groups) {
            findings.push(LayoutFinding {
                file: index.clone(),
                line: 0,
                message: format!(
                    "{} holds folders, so it is a group folder, but it sits inside the group folder {}, and groups go one level deep.",
                    shown(group, base),
                    shown(&outer, base)
                ),
                help: Some(format!(
                    "Move the folders of {} into {}, or into a group of their own beside it.",
                    shown(group, base),
                    shown(&outer, base)
                )),
            });
        }
        for file in by_folder.get(group).into_iter().flatten() {
            if *file == index {
                continue;
            }
            findings.push(LayoutFinding {
                file: file.clone(),
                line: 0,
                message: format!(
                    "{} holds the file {}, but a group folder holds only its child folders and its mod.rs.",
                    shown(group, base),
                    file_name(file)
                ),
                help: Some(String::from(
                    "Move the file into a child folder named for what it does.",
                )),
            });
        }
        findings.extend(check_index_items(&index, IndexRole::Group, base));
    }
    return findings;
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
    let inside: Option<PathBuf> =
        owner_of(&link.target, groups).filter(|other| return other != group);
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
///  The finding for a folder whose code names a group itself.
/// Why:
///  A folder names the folders inside a group,
///  because the group holds no code and stands for none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesGroup(from: Path, group: Path, proof: Proof, base: Path): LayoutFinding;
/// ```
fn names_group(from: &Path, group: &Path, proof: &Proof, base: &Path) -> LayoutFinding {
    return LayoutFinding {
        file: proof.file.clone(),
        line: proof.line,
        message: format!(
            "{} uses the group folder {} itself (`{}`), but a folder names the folders inside a group that it uses, never the group itself.",
            shown(from, base),
            shown(group, base),
            proof.text
        ),
        help: Some(format!(
            "Name the folder inside {} that holds what is needed, and link that folder.",
            shown(group, base)
        )),
    };
}

/// What:
///  The finding for a link from a folder to a group.
/// Why:
///  A folder links the folders inside a group that it uses,
///  and only a group's own summary links groups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linksGroup(link: Link, base: Path): LayoutFinding;
/// ```
pub fn links_group(link: &Link, base: &Path) -> LayoutFinding {
    return LayoutFinding {
        file: link.entry.clone(),
        line: 0,
        message: format!(
            "{} links to the group folder {}, but a folder links the folders inside a group that it uses, never the group itself.",
            shown(&link.entry, base),
            shown(&link.target, base)
        ),
        help: Some(format!(
            "Remove the link, and link the folder inside {} that the code uses.",
            shown(&link.target, base)
        )),
    };
}

/// What:
///  Every group finding except the loops:
///  the shelf rules,
///  the summaries,
///  and the folders that name a group.
/// Why:
///  They all start from the same group set and the same folder uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupFindings(tree: ModuleTree, groups: Set<Path>, grouped: FolderUses, sources: Map<Path, Path>, base: Path): LayoutFinding[];
/// ```
pub fn group_findings(
    tree: &ModuleTree,
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
    sources: &BTreeMap<PathBuf, PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = shelf_findings(tree, groups, base);
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
    for (from, targets) in grouped {
        if groups.contains(from) {
            continue;
        }
        for (to, proof) in targets {
            if groups.contains(to) {
                findings.push(names_group(from, to, proof, base));
            }
        }
    }
    return findings;
}

/// What:
///  The findings for entries that use each other in a loop.
/// Why:
///  A use of a folder inside a group is a use of the group,
///  so two groups,
///  or a group and a one-folder entry,
///  can form a loop although no two folders do.
///  A loop of one-folder entries only is a folder loop and is reported there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupLoops(groups: Set<Path>, grouped: FolderUses, base: Path): LayoutFinding[];
/// ```
pub fn group_loops(
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut entries: FolderUses = FolderUses::new();
    let mut nodes: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for (from, targets) in grouped {
        if groups.contains(from) {
            continue;
        }
        let source: PathBuf = entry_of(from, groups);
        for (to, proof) in targets {
            let target: PathBuf = entry_of(to, groups);
            if source == target {
                continue;
            }
            nodes.insert(source.clone());
            nodes.insert(target.clone());
            record_use(&mut entries, source.clone(), target, proof.clone());
        }
    }
    let node_list: Vec<PathBuf> = nodes.into_iter().collect::<Vec<PathBuf>>();
    return use_loops(
        &node_list,
        &entries,
        base,
        "groups and folders",
        Some(groups),
    );
}

/// Controls of the group checks stay outside release artifacts.
#[cfg(test)]
#[path = "layout_group_folders_tests.rs"]
mod tests;
