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
    layout_allow_list::{
        AllowKind, AllowList, dead_entry_findings, tolerated,
    },
    layout_finding::{LayoutFinding, key_path, shown, unique_keys},
    layout_folder_uses::{
        FolderUses, Proof, group_by_folder, loop_membership, record_use, use_loops,
    },
    layout_group_entries::{entry_of, find_groups},
    layout_group_folders::{entry_uses, group_findings, group_loops, links_group},
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
                key: format!(
                    "glob {} {}",
                    key_path(&module.file, base),
                    reference.raw.segments.join("::")
                ),
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
                key: format!("below-uses {}", key_path(folder, base)),
            });
        }
    }
    return findings;
}

/// What:
///  Which loops each folder is in:
///  its own folder loop and the loop of the entry it belongs to.
/// Why:
///  An allow-list entry tolerates one use,
///  and it stays alive only while the two folders it joins are in one loop together.
///  A folder can sit in both graphs at once:
///  two folders of a group loop with each other while their entry loops with a third,
///  and then a use from the folder to another entry's folder is held by the entry loop
///  while the folder's own loop holds a different use.
///  Keeping both answers is what makes that state satisfiable:
///  reading only the folder's own loop left its entry-loop uses with no loop at all,
///  so no list could keep them alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loopsOfEachFolder(folders: Path[], grouped: FolderUses, groups: Set<Path>): Map<Path, Set<[number, number]>>;
/// ```
fn loops_of_each_folder(
    folders: &[PathBuf],
    grouped: &FolderUses,
    groups: &BTreeSet<PathBuf>,
) -> BTreeMap<PathBuf, BTreeSet<(u8, usize)>> {
    let (entry_nodes, entry_grouped): (Vec<PathBuf>, FolderUses) = entry_uses(groups, grouped);
    let folder_loops: BTreeMap<PathBuf, usize> = loop_membership(folders, grouped);
    let entry_loops: BTreeMap<PathBuf, usize> = loop_membership(&entry_nodes, &entry_grouped);
    let mut loops: BTreeMap<PathBuf, BTreeSet<(u8, usize)>> =
        BTreeMap::<PathBuf, BTreeSet<(u8, usize)>>::new();
    for folder in folders {
        // The loops this folder is in, from either graph; zero means its own folders, one its entry.
        let mut held: BTreeSet<(u8, usize)> = BTreeSet::<(u8, usize)>::new();
        if let Some(number) = folder_loops.get(folder) {
            held.insert((0, *number));
        }
        let entry: PathBuf = entry_of(folder, groups);
        if let Some(number) = entry_loops.get(&entry) {
            held.insert((1, *number));
        }
        if !held.is_empty() {
            loops.insert(folder.clone(), held);
        }
    }
    return loops;
}

/// What:
///  The loops two folders are in together,
///  empty when they share none.
/// Why:
///  One use between two folders holds a loop only while both are in it,
///  and a folder may sit in a folder loop and in an entry loop at the same time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedLoops(loops: Map<Path, Set<[number, number]>>, from: Path, to: Path): Set<[number, number]>;
/// ```
fn shared_loops(
    loops: &BTreeMap<PathBuf, BTreeSet<(u8, usize)>>,
    from: &Path,
    to: &Path,
) -> BTreeSet<(u8, usize)> {
    let (Some(of_from), Some(of_to)) = (loops.get(from), loops.get(to)) else {
        return BTreeSet::<(u8, usize)>::new();
    };
    return of_from.intersection(of_to).copied().collect::<BTreeSet<(u8, usize)>>();
}


/// What:
///  The uses that stay in the loop checks:
///  every use except the ones the list tolerates.
/// Why:
///  A loop is reported only when one of its uses is not tolerated,
///  so a loop that is fully recorded reports nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutTolerated(grouped: FolderUses, list: AllowList): FolderUses;
/// ```
fn without_tolerated(grouped: &FolderUses, list: &AllowList, base: &Path) -> FolderUses {
    let mut kept: FolderUses = FolderUses::new();
    for (from, targets) in grouped {
        for (to, proof) in targets {
            // The list holds the paths findings show, which are relative to the package folder.
            let tolerated_use: bool = list.covers_use(
                Path::new(key_path(from, base).as_str()),
                Path::new(key_path(to, base).as_str()),
            );
            if tolerated_use {
                continue;
            }
            record_use(&mut kept, from.clone(), to.clone(), proof.clone());
        }
    }
    return kept;
}

/// What:
///  The loops one of whose uses the list already tolerates.
/// Why:
///  A loop the list knows nothing about is reported by the loop check,
///  so only a loop the list has started tolerating can hide a use added later.
///  A listed use between folders that share more than one loop records all of them,
///  because the list holds a line for one use and that use keeps every loop it joins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toleratedLoops(grouped: FolderUses, loops: Map<Path, Set<[number, number]>>, list: AllowList, base: Path): Set<[number, number]>;
/// ```
fn tolerated_loops(
    grouped: &FolderUses,
    loops: &BTreeMap<PathBuf, BTreeSet<(u8, usize)>>,
    list: &AllowList,
    base: &Path,
) -> BTreeSet<(u8, usize)> {
    let mut tolerated: BTreeSet<(u8, usize)> = BTreeSet::<(u8, usize)>::new();
    for (from, targets) in grouped {
        for to in targets.keys() {
            let listed: bool = list.covers_use(
                Path::new(key_path(from, base).as_str()),
                Path::new(key_path(to, base).as_str()),
            );
            if !listed {
                continue;
            }
            tolerated.extend(shared_loops(loops, from, to));
        }
    }
    return tolerated;
}

/// What:
///  The findings for uses that join two folders already inside one tolerated loop,
///  and that no loop finding reports.
/// Why:
///  A loop the list tolerates must only lose uses:
///  a new use inside it adds a loop edge that no entry records,
///  and no entry may be added for it,
///  so the run says to remove the use or untangle the loop first.
///  A use that a loop finding already lists is left to that finding,
///  or one added edge would be reported once per old use of the loop it joins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newLoopUses(grouped: FolderUses, loops: Map<Path, Set<[number, number]>>, reportedLoops: Map<Path, Set<[number, number]>>, tolerated: Set<[number, number]>, list: AllowList, base: Path): LayoutFinding[];
/// ```
fn new_loop_uses(
    grouped: &FolderUses,
    loops: &BTreeMap<PathBuf, BTreeSet<(u8, usize)>>,
    reported_loops: &BTreeMap<PathBuf, BTreeSet<(u8, usize)>>,
    tolerated: &BTreeSet<(u8, usize)>,
    list: &AllowList,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (from, targets) in grouped {
        for (to, proof) in targets {
            let tolerated_use: bool = list.covers_use(
                Path::new(key_path(from, base).as_str()),
                Path::new(key_path(to, base).as_str()),
            );
            if tolerated_use {
                continue;
            }
            // Every loop these two folders are in together, from either graph.
            let joined: BTreeSet<(u8, usize)> = shared_loops(loops, from, to);
            if joined.is_empty() {
                continue;
            }
            if joined.intersection(tolerated).next().is_none() {
                continue;
            }
            // A loop the loop check reports already lists this use.
            if !shared_loops(reported_loops, from, to).is_empty() {
                continue;
            }
            findings.push(LayoutFinding {
                file: proof.file.clone(),
                line: proof.line,
                message: format!(
                    "{} uses the folder {} (`{}`), and both sit inside one loop the allow list tolerates today; that loop holds this use, and no line of the list records it, so a tolerated loop may only lose uses.",
                    shown(from, base),
                    shown(to, base),
                    proof.text
                ),
                help: Some(String::from(
                    "Remove the use, or untangle that loop first: an allow-list entry records today's findings and a round never adds one.",
                )),
                key: String::new(),
            });
        }
    }
    return findings;
}

/// What:
///  The list entries about one use that still hold:
///  the use exists and its two ends are still in one loop together.
/// Why:
///  They are the entries that tolerated something,
///  and every other use entry is dead and must be deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveUseEntries(grouped: FolderUses, loops: Map<Path, Set<[number, number]>>, list: AllowList): Set<number>;
/// ```
fn live_use_entries(
    grouped: &FolderUses,
    loops: &BTreeMap<PathBuf, BTreeSet<(u8, usize)>>,
    list: &AllowList,
    base: &Path,
) -> BTreeSet<usize> {
    let mut used: BTreeSet<usize> = BTreeSet::<usize>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        let AllowKind::FolderUse { from, to } = &entry.kind else {
            continue;
        };
        let absolute_from: PathBuf = base.join(from);
        let absolute_to: PathBuf = base.join(to);
        let Some(targets) = grouped.get(&absolute_from) else {
            continue;
        };
        if !targets.contains_key(&absolute_to) {
            continue;
        }
        if shared_loops(loops, &absolute_from, &absolute_to).is_empty() {
            continue;
        }
        used.insert(index);
    }
    return used;
}

/// What:
///  Check the folder rule over a package.
/// Why:
///  `base` is the package folder;
///  folders are the folders of the production module files in the tree.
///  `allow` is the list the rule's `allow` option names,
///  and the links are always checked against every use,
///  because a `uses` folder shows who depends on whom however the list reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFolderLinks(tree: ModuleTree, uses: ModuleUse[], base: Path, allow?: AllowList): LayoutFinding[];
/// ```
pub fn check_folder_links(
    tree: &ModuleTree,
    uses: &[ModuleUse],
    base: &Path,
    allow: Option<&AllowList>,
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
    let reported: FolderUses = match allow {
        Some(list) => without_tolerated(&grouped, list, base),
        None => grouped.clone(),
    };
    findings.extend(use_loops(
        folder_list.as_slice(),
        &reported,
        base,
        "folders",
        None,
    ));
    findings.extend(group_findings(tree, &groups, &grouped, &sources, base));
    findings.extend(group_loops(&groups, &reported, base));
    findings.extend(glob_reexports(tree, base));
    findings.extend(folder_below_uses(folder_list.as_slice(), base));
    // Every key names one finding before a line can tolerate it, so a second finding of one shape is reported.
    unique_keys(&mut findings);
    let Some(list) = allow else {
        return findings;
    };
    let loops: BTreeMap<PathBuf, BTreeSet<(u8, usize)>> =
        loops_of_each_folder(&folder_list, &grouped, &groups);
    let reported_loops: BTreeMap<PathBuf, BTreeSet<(u8, usize)>> =
        loops_of_each_folder(&folder_list, &reported, &groups);
    let tolerated_loops: BTreeSet<(u8, usize)> = tolerated_loops(&grouped, &loops, list, base);
    findings.extend(new_loop_uses(
        &grouped,
        &loops,
        &reported_loops,
        &tolerated_loops,
        list,
        base,
    ));
    let (mut kept, mut used): (Vec<LayoutFinding>, BTreeSet<usize>) = tolerated(findings, list);
    used.extend(live_use_entries(&grouped, &loops, list, base));
    kept.extend(dead_entry_findings(list, &used, base));
    return kept;
}

/// Controls of the folder rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_folder_links_tests.rs"]
mod tests;
