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
//!  a parent folder and a child folder are folders like any other,
//! so using one needs a link
//!  (a child that reaches `super::super` into its parent's folder depends on it);
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
///  the loop finder,
///  the tree,
///  the resolver and the uses.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_link_entries::{USES_FOLDER, link_command, read_links},
    layout_loops::find_loops,
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
///  The reference that proves one folder uses another.
/// Why:
///  Every finding about a use names a file and a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { file: Path; line: number; text: string };
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Proof {
    /// The file with the reference.
    file: PathBuf,
    /// Its line.
    line: usize,
    /// The path as written.
    text: String,
}

/// What:
///  The folders each folder uses,
///  with the first reference that proves each use.
/// Why:
///  The smallest reference is kept so the output does not depend on discovery order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderUses = Map<Path, Map<Path, Proof>>;
/// ```
type FolderUses = BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>>;

/// What:
///  Group module uses by the folders of the two modules.
/// Why:
///  A use inside one folder is not a folder dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupByFolder(tree: ModuleTree, uses: ModuleUse[]): FolderUses;
/// ```
fn group_by_folder(tree: &ModuleTree, uses: &[ModuleUse]) -> FolderUses {
    let mut grouped: FolderUses = FolderUses::new();
    for found in uses {
        let from: PathBuf = folder_of(tree, found.from);
        let to: PathBuf = folder_of(tree, found.to);
        if from == to {
            continue;
        }
        let proof: Proof = Proof {
            file: found.file.clone(),
            line: found.line,
            text: found.text.clone(),
        };
        // Keep the smallest proof so the output does not depend on discovery order.
        let targets: &mut BTreeMap<PathBuf, Proof> = grouped.entry(from).or_default();
        let smaller: bool = match targets.get(&to) {
            Some(existing) => proof < *existing,
            None => true,
        };
        if smaller {
            targets.insert(to, proof);
        }
    }
    return grouped;
}

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOneFolder(folder: Path, used: Map<Path, Proof>, links: LinkRead, base: Path): LayoutFinding[];
/// ```
fn check_one_folder(
    folder: &Path,
    used: Option<&BTreeMap<PathBuf, Proof>>,
    sources: &BTreeMap<PathBuf, PathBuf>,
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
        if !needed.contains_key(&link.target) {
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
    }
    for (target, proof) in needed {
        if !linked.contains(target) {
            findings.push(missing_link(folder, target, proof, base));
        }
    }
    return findings;
}

/// What:
///  The findings for folders that use each other.
/// Why:
///  One finding per loop lists the folders and one reference per use inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function folderLoops(folders: Path[], uses: FolderUses, base: Path): LayoutFinding[];
/// ```
fn folder_loops(folders: &[PathBuf], grouped: &FolderUses, base: &Path) -> Vec<LayoutFinding> {
    let mut position: BTreeMap<&PathBuf, usize> = BTreeMap::<&PathBuf, usize>::new();
    for (index, folder) in folders.iter().enumerate() {
        position.insert(folder, index);
    }
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::<usize>::new(); folders.len()];
    for (from, targets) in grouped {
        for to in targets.keys() {
            if let (Some(a), Some(b)) = (position.get(from), position.get(to)) {
                adjacency[*a].push(*b);
            }
        }
    }
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for members in find_loops(adjacency.as_slice()) {
        let inside: BTreeSet<usize> = members.iter().copied().collect::<BTreeSet<usize>>();
        let mut lines: Vec<String> = Vec::<String>::new();
        let mut anchor: Option<Proof> = None;
        for member in &members {
            let from: &PathBuf = &folders[*member];
            for (to, proof) in grouped.get(from).into_iter().flatten() {
                let Some(target) = position.get(to) else {
                    continue;
                };
                if !inside.contains(target) {
                    continue;
                }
                lines.push(format!(
                    "{} uses {} ({}:{} `{}`)",
                    shown(from, base),
                    shown(to, base),
                    shown(&proof.file, base),
                    proof.line,
                    proof.text
                ));
                if anchor.is_none() {
                    anchor = Some(proof.clone());
                }
            }
        }
        let Some(first) = anchor else {
            continue;
        };
        findings.push(LayoutFinding {
            file: first.file,
            line: first.line,
            message: format!("These folders use each other in a loop: {}.", lines.join("; ")),
            help: Some(String::from(
                "Move the shared code into a folder both can use, or pass it in, so that one direction of use disappears.",
            )),
        });
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
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for folder in &folder_list {
        findings.extend(check_one_folder(
            folder,
            grouped.get(folder),
            &sources,
            base,
        ));
    }
    findings.extend(folder_loops(folder_list.as_slice(), &grouped, base));
    findings.extend(glob_reexports(tree, base));
    findings.extend(folder_below_uses(folder_list.as_slice(), base));
    return findings;
}

/// Controls of the folder rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_folder_links_tests.rs"]
mod tests;
