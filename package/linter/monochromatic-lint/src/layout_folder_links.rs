//! What:
//!  The folder rule:
//!  every source folder links,
//! in its `uses` folder,
//! to exactly the folders its code uses,
//!  and no two folders use each other.
//! Why:
//!  "Who depends on whom" must be visible in the file tree itself.
//!  A folder uses another when code in one of its module files names a module that the other folder defines.
//!  Decisions:
//!  a folder that holds child folders is a group,
//! which has its own checks in `layout_group_folders.rs`
//!  (a shelf of folders with a `uses` summary,
//! never linked or named by a folder);
//!  a folder that uses nothing needs no `uses` folder because Git cannot store an empty one;
//!  a glob re-export (`pub use folder::*`) is a finding of its own,
//!  because it hides which folder an item comes from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkFolderLinks(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the link reader,
///  the group checks,
///  the folder uses and loops,
///  the tree and the resolver.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_folder_uses::{FolderUses, Proof, group_by_folder, use_loops},
    layout_group_entries::find_groups,
    layout_group_folders::{group_findings, group_loops, links_group},
    layout_link_entries::{USES_FOLDER, link_command, read_links},
    layout_module_tree::ModuleTree,
    layout_module_uses::{ModuleUse, folder_of},
    layout_path_resolution::resolve_path,
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// What:
///  The finding for a folder that uses another with no link.
/// Why:
///  It is anchored on the reference that proves the use and says how to add the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingLink(from: Path, to: Path, proof: Proof, base: Path): LayoutFinding;
/// ```
fn missing_link(from: &Path, to: &Path, proof: &Proof, base: &Path) -> LayoutFinding {
    return LayoutFinding {
        file: proof.file.clone(),
        line: proof.line,
        message: format!(
            "{} uses the folder {} (`{}`) but {}/{USES_FOLDER} has no link to it.",
            shown(from, base),
            shown(to, base),
            proof.text,
            shown(from, base)
        ),
        help: Some(format!("Create the link: {}", link_command(from, to, base))),
    };
}

/// What:
///  The findings for the links of one folder:
///  the bad entries,
///  the links to folders the code does not use,
///  and the used folders without a link.
/// Why:
///  A link is right exactly when the code uses its target.
///  A group is never a target:
///  a link to one is a finding of the group checks' kind,
///  and a group the code names is reported by the group checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOneFolder(folder: Path, used: Map<Path, Proof>, links: LinkRead, groups: Set<Path>, base: Path): LayoutFinding[];
/// ```
fn check_one_folder(
    folder: &Path,
    used: Option<&BTreeMap<PathBuf, Proof>>,
    sources: &BTreeMap<PathBuf, PathBuf>,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let read = read_links(folder, sources, base);
    let mut findings: Vec<LayoutFinding> = read.findings;
    let mut linked: BTreeSet<&PathBuf> = BTreeSet::<&PathBuf>::new();
    for link in &read.links {
        linked.insert(&link.target);
    }
    let empty: BTreeMap<PathBuf, Proof> = BTreeMap::<PathBuf, Proof>::new();
    let needed: &BTreeMap<PathBuf, Proof> = used.unwrap_or(&empty);
    for link in &read.links {
        if needed.contains_key(&link.target) {
            continue;
        }
        if groups.contains(&link.target) {
            findings.push(links_group(link, base));
            continue;
        }
        findings.push(LayoutFinding {
            file: link.entry.clone(),
            line: 0,
            message: format!(
                "{} links to the folder {}, but no code in {} uses it.",
                shown(&link.entry, base),
                shown(&link.target, base),
                shown(folder, base)
            ),
            help: Some(String::from(
                "Remove the link, or use the folder it points to.",
            )),
        });
    }
    for (target, proof) in needed {
        // A group the code names is reported by the group checks.
        if groups.contains(target) {
            continue;
        }
        if !linked.contains(target) {
            findings.push(missing_link(folder, target, proof, base));
        }
    }
    return findings;
}

/// What:
///  The findings for glob re-exports that name a module of this package.
/// Why:
///  After `pub use folder::*;` a path through the re-exporting module names the re-exporter,
///  while the item lives in `folder`;
///  the folders' links then cannot show who really depends on whom.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function globReexports(tree: ModuleTree, base: Path): LayoutFinding[];
/// ```
fn glob_reexports(tree: &ModuleTree, base: &Path) -> Vec<LayoutFinding> {
    let mut seen: BTreeSet<(PathBuf, usize)> = BTreeSet::<(PathBuf, usize)>::new();
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (index, module) in tree.modules.iter().enumerate() {
        for reference in &module.references {
            if !(reference.raw.glob && reference.raw.reexport) {
                continue;
            }
            let Some(target) = resolve_path(tree, index, reference.raw.segments.as_slice(), false)
            else {
                continue;
            };
            if target == index || !seen.insert((module.file.clone(), reference.line)) {
                continue;
            }
            findings.push(LayoutFinding {
                file: module.file.clone(),
                line: reference.line,
                message: format!(
                    "`pub use {}::*` re-exports every item of the module in {}, so a path through {} no longer shows which folder defines an item.",
                    reference.raw.segments.join("::"),
                    shown(&folder_of(tree, target), base),
                    shown(&module.file, base)
                ),
                help: Some(String::from(
                    "Name the items (`pub use path::{A, B};`), or let callers use the defining folder's path.",
                )),
            });
        }
    }
    return findings;
}

/// What:
///  The findings for source folders that sit below a folder named `uses`.
/// Why:
///  `uses` holds links only;
///  a module file below it means the tree was reached through a link or the layout is confused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function folderBelowUses(folders: Path[], base: Path): LayoutFinding[];
/// ```
fn folder_below_uses(folders: &[PathBuf], base: &Path) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for folder in folders {
        let Ok(below) = folder.strip_prefix(base) else {
            continue;
        };
        let mut named_uses: bool = false;
        for part in below.components() {
            if part.as_os_str() == USES_FOLDER {
                named_uses = true;
            }
        }
        if named_uses {
            findings.push(LayoutFinding {
                file: folder.clone(),
                line: 0,
                message: format!(
                    "{} holds module files but is below a folder named {USES_FOLDER}, which may hold links only.",
                    shown(folder, base)
                ),
                help: None,
            });
        }
    }
    return findings;
}

/// What:
///  Check the folder rule over a package.
/// Why:
///  `base` is the package folder;
///  folders are the folders of the production module files in the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFolderLinks(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
/// ```
pub fn check_folder_links(
    tree: &ModuleTree,
    uses: &[ModuleUse],
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut folders: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for index in 0..tree.modules.len() {
        folders.insert(folder_of(tree, index));
    }
    let folder_list: Vec<PathBuf> = folders.iter().cloned().collect::<Vec<PathBuf>>();
    let mut sources: BTreeMap<PathBuf, PathBuf> = BTreeMap::<PathBuf, PathBuf>::new();
    for folder in &folder_list {
        if let Ok(canonical) = std::fs::canonicalize(folder) {
            sources.insert(canonical, folder.clone());
        }
    }
    let grouped: FolderUses = group_by_folder(tree, uses);
    let groups: BTreeSet<PathBuf> = find_groups(tree);
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for folder in &folder_list {
        // A group's `uses` folder is a summary, checked with the group checks.
        if groups.contains(folder) {
            continue;
        }
        findings.extend(check_one_folder(
            folder,
            grouped.get(folder),
            &sources,
            &groups,
            base,
        ));
    }
    findings.extend(use_loops(
        folder_list.as_slice(),
        &grouped,
        base,
        "folders",
        None,
    ));
    findings.extend(group_findings(tree, &groups, &grouped, &sources, base));
    findings.extend(group_loops(&groups, &grouped, base));
    findings.extend(glob_reexports(tree, base));
    findings.extend(folder_below_uses(folder_list.as_slice(), base));
    return findings;
}

/// Controls of the folder rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_folder_links_tests.rs"]
mod tests;
