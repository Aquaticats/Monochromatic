//! What:
//!  The findings about the links of one folder:
//!  a link to a folder no code uses,
//!  a used folder without a link,
//!  a glob re-export that hides which folder defines an item,
//!  and a source folder that sits below a `uses` folder.
//! Why:
//!  The folder rule reports each of them about one folder at a time,
//!  so they read the links of one folder and the uses its code makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkOneFolder(folder: Path, used: Map<Path, Proof>, links: LinkRead, groups: Set<Path>, base: Path): LayoutFinding[];
//! function globReexports(tree: ModuleTree, base: Path): LayoutFinding[];
//! function folderBelowUses(folders: Path[], base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the reference that proves a use,
///  the group's link finding,
///  the link reader,
///  the tree,
///  the folder of a module,
///  and the resolver.
use crate::{
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    layout_folder_uses::Proof,
    layout_group_folders::links_group,
    layout_link_entries::{
        USES_FOLDER,
        link_help,
        read_links,
    },
    layout_module_tree::ModuleTree,
    layout_module_uses::folder_of,
    layout_path_resolution::resolve_path,
};
/// Import ordered collections and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    path::{
        Path,
        PathBuf,
    },
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
fn missing_link(
    from: &Path,
    to: &Path,
    proof: &Proof,
    base: &Path,
) -> LayoutFinding {
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
        help: Some(link_help(from, to, base)),
        key: format!(
            "missing-link {} -> {}",
            key_path(from, base),
            key_path(to, base)
        ),
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
pub(crate) fn check_one_folder(
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
            key: format!("unused-link {}", key_path(&link.entry, base)),
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
///  One line can hold two of them (`pub use crate::{a::*, b::*};`),
///  so a finding is keyed by the module it names and not by the line it sits on:
///  a list line for the first glob must never tolerate the second.
///  One source line is one finding however many `mod` declarations reach its file:
///  a library and an executable that declare the same modules give the tree two nodes
///  with one file and the same reference offsets,
///  so the rule reads one node per file and set of offsets,
///  and never one node per declaration that reached it.
///  A file that an inline module splits into two modules gives two shapes,
///  so a glob inside the inline module is a finding of its own.
///  A path is resolved against the module that holds it,
///  and the two declarations of one file are two modules,
///  so a glob can name a folder in one of them and nothing in the other;
///  the rule reports the finding when any of them names one,
///  so the answer never depends on which `mod` line stands first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function globReexports(tree: ModuleTree, base: Path): LayoutFinding[];
/// ```
pub(crate) fn glob_reexports(
    tree: &ModuleTree,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    // What: the module nodes of each shape, a shape being the file and every reference offset in it.
    // Why: a file that two `mod` declarations reach holds one node per declaration and one module,
    // while a file that holds an inline module holds one node for each of its modules.
    let mut shapes: BTreeMap<(PathBuf, Vec<usize>), Vec<usize>> =
        BTreeMap::<(PathBuf, Vec<usize>), Vec<usize>>::new();
    for (index, module) in tree.modules.iter().enumerate() {
        let shape: (PathBuf, Vec<usize>) = (
            module.file.clone(),
            module
                .references
                .iter()
                .map(|reference| return reference.raw.offset)
                .collect::<Vec<usize>>(),
        );
        shapes.entry(shape).or_default().push(index);
    }
    // What: each shape paired with the module node it starts at, in that node's order.
    // Why: a finding belongs where the tree first reads the file that holds it,
    // and a file that two `mod` declarations reach is still read once.
    let mut order: Vec<(&(PathBuf, Vec<usize>), usize)> =
        Vec::<(&(PathBuf, Vec<usize>), usize)>::new();
    for (shape, nodes) in &shapes {
        let first: usize = match nodes.first() {
            Some(index) => *index,
            None => usize::MAX,
        };
        order.push((shape, first));
    }
    order.sort_by_key(|pair| return pair.1);
    for pair in order {
        let shape: &(PathBuf, Vec<usize>) = pair.0;
        // What: every node of this shape, which is every context the file is read in.
        // Why: a glob can resolve in one context and not in another, so every one of them is asked.
        let nodes: &Vec<usize> = match shapes.get(shape) {
            Some(found) => found,
            None => continue,
        };
        // What: the one finding of each glob of this shape,
        // keyed by where its source path starts, what it names and how many globs before it name the same module.
        // Why: the same glob read in the two contexts of one file is one finding,
        // and two globs on one line, or on two lines, keep one finding each.
        let mut found: BTreeMap<(usize, String, usize), LayoutFinding> =
            BTreeMap::<(usize, String, usize), LayoutFinding>::new();
        for index in nodes {
            let module = &tree.modules[*index];
            // What: how many globs of one start and one path came before, inside this module.
            // Why: `pub use crate::{a::*, a::*};` is two re-exports that start where the path starts,
            // so only their order apart keeps them two findings.
            let mut repeated: BTreeMap<(usize, String), usize> =
                BTreeMap::<(usize, String), usize>::new();
            for reference in &module.references {
                if !(reference.raw.glob && reference.raw.reexport) {
                    continue;
                }
                let Some(target) =
                    resolve_path(tree, *index, reference.raw.segments.as_slice(), false)
                else {
                    continue;
                };
                if target == *index {
                    continue;
                }
                let finding: LayoutFinding = LayoutFinding {
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
                    key: format!(
                        "glob {} {}",
                        key_path(&module.file, base),
                        reference.raw.segments.join("::")
                    ),
                };
                let start: (usize, String) =
                    (reference.raw.offset, reference.raw.segments.join("::"));
                let place: usize = repeated.get(&start).copied().unwrap_or(0);
                repeated.insert(start.clone(), place + 1);
                let slot: (usize, String, usize) = (start.0, start.1, place);
                // Two contexts that name two different folders keep the one whose message sorts first,
                // so the report never depends on the order the `mod` lines stand in.
                let smaller: bool = match found.get(&slot) {
                    Some(seen) => finding.message < seen.message,
                    None => true,
                };
                if smaller {
                    found.insert(slot, finding);
                }
            }
        }
        findings.extend(found.into_values());
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
pub(crate) fn folder_below_uses(
    folders: &[PathBuf],
    base: &Path,
) -> Vec<LayoutFinding> {
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
                key: format!("below-uses {}", key_path(folder, base)),
            });
        }
    }
    return findings;
}
