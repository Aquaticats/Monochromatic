//! What:
//!  Group folders of the folder rule:
//!  a folder that holds child folders holds nothing else,
//!  a child folder never links or names a group,
//!  and no two entries use each other.
//! Why:
//!  The owner's rule is a shelf with summaries (2026-11-07T11:05:21-05:00):
//!  the tree shows who depends on whom at two scales,
//!  the entries of a package and the exact folders inside each group.
//!  `layout_group_entries.rs` recognizes groups and entries,
//!  and `layout_group_summary_links.rs` checks each group's `uses` summary.
//!  Decisions:
//!  a group is recognized by its child folders and needs no marker,
//!  so a group that grows code is still a group and the code is a finding;
//!  a group's own code makes no use and no link,
//!  because the shelf rule reports the code and the summary is what its child folders use;
//!  a group inside a group is a finding,
//!  because the rule is one level of groups.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function groupFindings(tree: ModuleTree, groups: Set<Path>, grouped: FolderUses, sources: Map<Path, Path>, base: Path): LayoutFinding[];
//! function linksGroup(link: Link, base: Path): LayoutFinding;
//! function groupLoops(groups: Set<Path>, grouped: FolderUses, base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the file readers,
///  the use records and loops,
///  the group entries and summaries,
///  the index item check,
///  the link type and the tree.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_folder_index_files::{file_name, files_by_folder},
    layout_folder_uses::{FolderUses, Proof, record_use, use_loops},
    layout_group_entries::{entry_of, owner_of},
    layout_group_summary_links::summary_findings,
    layout_index_file_items::{IndexRole, check_index_items},
    layout_link_entries::Link,
    layout_module_tree::ModuleTree,
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

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
    findings.extend(summary_findings(groups, grouped, sources, base));
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
