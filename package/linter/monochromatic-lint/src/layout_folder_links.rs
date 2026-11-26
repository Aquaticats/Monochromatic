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
        AllowKind,
        AllowList,
        dead_entry_findings,
        tolerated,
    },
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
        unique_keys,
    },
    layout_folder_uses::{
        FolderUses,
        Proof,
        group_by_folder,
        loop_membership,
        record_use,
        use_loops,
    },
    layout_group_entries::{
        entry_of,
        find_groups,
    },
    layout_group_folders::{
        entry_uses,
        group_findings,
        group_loops,
        links_group,
    },
    layout_link_entries::{
        USES_FOLDER,
        link_help,
        read_links,
    },
    layout_module_tree::ModuleTree,
    layout_module_uses::{
        ModuleUse,
        folder_of,
    },
    layout_path_resolution::resolve_path,
};
/// Import ordered collections, the queue of one path search and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
        VecDeque,
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
///  One line can hold two of them (`pub use crate::{a::*, b::*};`),
///  so a finding is keyed by the module it names and not by the line it sits on:
///  a list line for the first glob must never tolerate the second.
///  One source line is one finding however many `mod` declarations reach its file:
///  a library and an executable that declare the same modules give the tree two nodes
///  with one file and the same reference offsets,
///  so the rule reads one node per file and set of offsets,
///  and never one node per declaration that reached it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function globReexports(tree: ModuleTree, base: Path): LayoutFinding[];
/// ```
fn glob_reexports(
    tree: &ModuleTree,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    // What: the node shapes this rule has already read, a shape being the file and every reference offset in it.
    // Why: a file two `mod` declarations reach is read once per declaration, and one node of it is one module.
    let mut seen: BTreeSet<(PathBuf, Vec<usize>)> = BTreeSet::<(PathBuf, Vec<usize>)>::new();
    for (index, module) in tree.modules.iter().enumerate() {
        // What: this node's file and the offsets it holds references at, which no other module node of the file shares.
        // Why: a repeated module scans the same bytes at the same offsets and reports the same globs again.
        let shape: (PathBuf, Vec<usize>) = (
            module.file.clone(),
            module
                .references
                .iter()
                .map(|reference| return reference.raw.offset)
                .collect::<Vec<usize>>(),
        );
        if !seen.insert(shape) {
            continue;
        }
        for reference in &module.references {
            if !(reference.raw.glob && reference.raw.reexport) {
                continue;
            }
            let Some(target) = resolve_path(tree, index, reference.raw.segments.as_slice(), false)
            else {
                continue;
            };
            if target == index {
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
fn folder_below_uses(
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

/// What:
///  Which graph a loop was found in.
/// Why:
///  A folder belongs to one loop of folders and to the loop of its entry,
///  and the two graphs put different folders together,
///  so a loop number alone would say that two members of one group share a loop they never make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LoopGraph = 'folders' | 'entries';
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum LoopGraph {
    /// The source folders,
    /// each using the other.
    Folders,
    /// The entries:
    /// a group,
    /// or a folder outside every group.
    Entries,
}

/// What:
///  Which loops each folder sits in.
/// Why:
///  A loop is one of the folder graph or one of the entry graph,
///  and the two graphs put different folders together,
///  so the pair to the left of the count says which graph made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderLoops = Map<Path, Set<[LoopGraph, number]>>;
/// ```
type FolderLoops = BTreeMap<PathBuf, BTreeSet<(LoopGraph, usize)>>;

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
/// function loopsOfEachFolder(folders: Path[], grouped: FolderUses, groups: Set<Path>): FolderLoops;
/// ```
fn loops_of_each_folder(
    folders: &[PathBuf],
    grouped: &FolderUses,
    groups: &BTreeSet<PathBuf>,
) -> FolderLoops {
    let (entry_nodes, entry_grouped): (Vec<PathBuf>, FolderUses) = entry_uses(groups, grouped);
    let folder_loops: BTreeMap<PathBuf, usize> = loop_membership(folders, grouped);
    let entry_loops: BTreeMap<PathBuf, usize> = loop_membership(&entry_nodes, &entry_grouped);
    let mut loops: FolderLoops = BTreeMap::<PathBuf, BTreeSet<(LoopGraph, usize)>>::new();
    for folder in folders {
        // The loops this folder is in, from either graph; the graph is part of the answer.
        let mut held: BTreeSet<(LoopGraph, usize)> = BTreeSet::<(LoopGraph, usize)>::new();
        if let Some(number) = folder_loops.get(folder) {
            held.insert((LoopGraph::Folders, *number));
        }
        let entry: PathBuf = entry_of(folder, groups);
        if let Some(number) = entry_loops.get(&entry) {
            held.insert((LoopGraph::Entries, *number));
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
///  An entry loop holds a use only when the use reaches outside its entry:
///  two folders of one group share that group as their entry,
///  so a use between them makes no edge of the entry graph and no entry loop can hold it.
///  A group's own code makes no edge of the entry graph:
///  the shelf rule reports the file that holds the code,
///  a group's summary is what its child folders use,
///  and `entry_uses` skips such a folder as a source too,
///  so a use from a group's own file joins no entry loop and owes no `uses` line.
///  The folder graph still sees that use,
///  because the group folder is one of the package's folders;
///  a loop of plain folders through it is reported by the folder loop check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedLoops(loops: FolderLoops, from: Path, to: Path, groups: Set<Path>): Set<[LoopGraph, number]>;
/// ```
fn shared_loops(
    loops: &FolderLoops,
    from: &Path,
    to: &Path,
    groups: &BTreeSet<PathBuf>,
) -> BTreeSet<(LoopGraph, usize)> {
    let (Some(of_from), Some(of_to)) = (loops.get(from), loops.get(to)) else {
        return BTreeSet::<(LoopGraph, usize)>::new();
    };
    let mut shared: BTreeSet<(LoopGraph, usize)> = BTreeSet::<(LoopGraph, usize)>::new();
    for held in of_from.intersection(of_to) {
        match held.0 {
            LoopGraph::Folders => {
                shared.insert(*held);
            }
            LoopGraph::Entries => {
                // A group's own code makes no use (the shelf rule reports the file), and two members of one group make no entry edge.
                let inside_one_entry: bool =
                    groups.contains(from) || entry_of(from, groups) == entry_of(to, groups);
                if !inside_one_entry {
                    shared.insert(*held);
                }
            }
        }
    }
    return shared;
}

/// What:
///  The entry graph the own-file check searches:
///  every use between two entries,
///  and every use written in a group's own file as an edge from that group.
/// Why:
///  A group's own code makes no edge of the entry graph
///  (the shelf rule reports the file and a group's summary is what its child folders use),
///  so the loop checks of the entries never see such a use.
///  A walk back from the entry one own file uses must be able to step over another group's own-file use,
///  or a mutual dependency that two own-file uses close is never seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownLoopSearch(groups: Set<Path>, grouped: FolderUses): FolderUses;
/// ```
fn own_loop_search(
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
) -> FolderUses {
    let (_, entries): (Vec<PathBuf>, FolderUses) = entry_uses(groups, grouped);
    let mut search: FolderUses = entries;
    for (from, targets) in grouped {
        if !groups.contains(from) {
            continue;
        }
        for (to, proof) in targets {
            let target_entry: PathBuf = entry_of(to, groups);
            // A use of the group's own child folders joins no two entries.
            if target_entry == *from {
                continue;
            }
            record_use(&mut search, from.clone(), target_entry, proof.clone());
        }
    }
    return search;
}

/// What:
///  The uses that lead each entry back to one target entry,
///  with the reference that proves the first step of each.
/// Why:
///  One walk back answers every use written in the own file of the entry it starts from.
///  A walk forward per use walks the whole graph once per use,
///  so a group whose own file uses many folders costs that many passes over the uses;
///  one walk per such group costs one.
///  The walk is breadth first,
///  so each message spells the fewest uses that lead back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function backSteps(entries: FolderUses, target: Path): Map<Path, [Path, Proof]>;
/// ```
fn back_steps(
    entries: &FolderUses,
    target: &Path,
) -> BTreeMap<PathBuf, (PathBuf, Proof)> {
    // What: every use filed under the entry it reaches, which is the step a walk back takes.
    // Why: a walk against the direction of the uses needs each target's sources together.
    let mut before: BTreeMap<PathBuf, Vec<(PathBuf, Proof)>> =
        BTreeMap::<PathBuf, Vec<(PathBuf, Proof)>>::new();
    for (from, targets) in entries {
        for (to, proof) in targets {
            before
                .entry(to.clone())
                .or_default()
                .push((from.clone(), proof.clone()));
        }
    }
    // What: the first step from each entry toward the target, and the entries already reached.
    // Why: the first walk to reach an entry is its shortest way back, and the set keeps a loop in the graph finite.
    let mut next: BTreeMap<PathBuf, (PathBuf, Proof)> =
        BTreeMap::<PathBuf, (PathBuf, Proof)>::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::<PathBuf>::new();
    queue.push_back(target.to_path_buf());
    seen.insert(target.to_path_buf());
    while let Some(node) = queue.pop_front() {
        for (step_from, proof) in before.get(&node).into_iter().flatten() {
            if !seen.insert(step_from.clone()) {
                continue;
            }
            next.insert(step_from.clone(), (node.clone(), proof.clone()));
            queue.push_back(step_from.clone());
        }
    }
    return next;
}

/// What:
///  The uses one entry takes to reach the searched one,
///  in order and with the reference that proves each.
/// Why:
///  The finding about a group's own file names the use that closes the loop
///  and every use that closes it back, the way a loop finding does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathBack(next: Map<Path, [Path, Proof]>, target: Path, start: Path, base: Path): string[] | undefined;
/// ```
fn path_back(
    next: &BTreeMap<PathBuf, (PathBuf, Proof)>,
    target: &Path,
    start: &Path,
    base: &Path,
) -> Option<Vec<String>> {
    let mut steps: Vec<String> = Vec::<String>::new();
    let mut node: PathBuf = start.to_path_buf();
    while node != *target {
        // No step back means the walk never reached this entry, so the use closes no loop.
        let (after, proof) = next.get(&node)?;
        steps.push(format!(
            "{} uses {} (`{}` at {}:{})",
            shown(&node, base),
            shown(after, base),
            proof.text,
            shown(&proof.file, base),
            proof.line
        ));
        node = after.clone();
    }
    return Some(steps);
}

/// What:
///  The findings for a use written in a group's own file
///  that closes a mutual dependency between its group and another entry.
/// Why:
///  A group's own code makes no edge of the entry graph
///  (the shelf rule reports the file and a group's summary is what its child folders use),
///  so the loop checks of the entries never see such a use.
///  Without this check, new code in a tolerated group file could add a loop,
///  and the allow list promises that new code cannot add one.
///  The search runs over the entry graph with every own-file use added as an edge,
///  so a loop that two own-file uses close is seen as one that a single use closes,
///  and it walks back once per group whose own file holds a use rather than once per use.
///  A use the folder loop already holds is left to the loop checks,
///  which report it or the use added inside a tolerated loop.
///  A use of the group's own child folders joins no two entries and owes nothing.
///  The finding is keyed by the use it reports, `own-file-loop <group> -> <used folder>`,
///  so a tree that already holds one can record it on a list;
///  a list of every finding never grows,
///  so a use added later still has no line to hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupOwnLoopUses(grouped: FolderUses, search: FolderUses, loops: FolderLoops, groups: Set<Path>, base: Path): LayoutFinding[];
/// ```
fn group_own_loop_uses(
    grouped: &FolderUses,
    search: &FolderUses,
    loops: &FolderLoops,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    // What: the walk back from each group that its own file's uses need, built the first time one asks.
    // Why: every use of one group needs the same answer, and one walk answers all of them.
    let mut searched: BTreeMap<PathBuf, BTreeMap<PathBuf, (PathBuf, Proof)>> =
        BTreeMap::<PathBuf, BTreeMap<PathBuf, (PathBuf, Proof)>>::new();
    for (from, targets) in grouped {
        if !groups.contains(from) {
            continue;
        }
        for (to, proof) in targets {
            let target_entry: PathBuf = entry_of(to, groups);
            // A use inside the group itself joins no two entries.
            if target_entry == *from {
                continue;
            }
            // A folder loop holding both folders already reports this use, or the use added inside a tolerated loop.
            if !shared_loops(loops, from, to, groups).is_empty() {
                continue;
            }
            let next: &BTreeMap<PathBuf, (PathBuf, Proof)> = searched
                .entry(from.clone())
                .or_insert_with(|| return back_steps(search, from));
            let Some(back) = path_back(next, from, &target_entry, base) else {
                continue;
            };
            findings.push(LayoutFinding {
                file: proof.file.clone(),
                line: proof.line,
                message: format!(
                    "These entries use each other in a loop: {} uses {} (`{}`); {}.",
                    shown(from, base),
                    shown(to, base),
                    proof.text,
                    back.join("; ")
                ),
                help: Some(format!(
                    "Remove the use in the own file of {}, or move that code into a child folder of it, so it and {} no longer use each other.",
                    shown(from, base),
                    shown(&target_entry, base)
                )),
                key: format!(
                    "own-file-loop {} -> {}",
                    key_path(from, base),
                    key_path(to, base)
                ),
            });
        }
    }
    return findings;
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
fn without_tolerated(
    grouped: &FolderUses,
    list: &AllowList,
    base: &Path,
) -> FolderUses {
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
/// function toleratedLoops(grouped: FolderUses, loops: FolderLoops, list: AllowList, groups: Set<Path>, base: Path): Set<[LoopGraph, number]>;
/// ```
fn tolerated_loops(
    grouped: &FolderUses,
    loops: &FolderLoops,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> BTreeSet<(LoopGraph, usize)> {
    let mut tolerated: BTreeSet<(LoopGraph, usize)> = BTreeSet::<(LoopGraph, usize)>::new();
    for (from, targets) in grouped {
        for to in targets.keys() {
            let listed: bool = list.covers_use(
                Path::new(key_path(from, base).as_str()),
                Path::new(key_path(to, base).as_str()),
            );
            if !listed {
                continue;
            }
            tolerated.extend(shared_loops(loops, from, to, groups));
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
/// function newLoopUses(grouped: FolderUses, loops: FolderLoops, reportedLoops: FolderLoops, tolerated: Set<[LoopGraph, number]>, list: AllowList, groups: Set<Path>, base: Path): LayoutFinding[];
/// ```
fn new_loop_uses(
    grouped: &FolderUses,
    loops: &FolderLoops,
    reported_loops: &FolderLoops,
    tolerated: &BTreeSet<(LoopGraph, usize)>,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
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
            let joined: BTreeSet<(LoopGraph, usize)> = shared_loops(loops, from, to, groups);
            if joined.is_empty() {
                continue;
            }
            if joined.intersection(tolerated).next().is_none() {
                continue;
            }
            // A loop the loop check reports already lists this use.
            if !shared_loops(reported_loops, from, to, groups).is_empty() {
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
/// function liveUseEntries(grouped: FolderUses, loops: FolderLoops, list: AllowList, groups: Set<Path>, base: Path): Set<number>;
/// ```
fn live_use_entries(
    grouped: &FolderUses,
    loops: &FolderLoops,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> BTreeSet<usize> {
    let mut used: BTreeSet<usize> = BTreeSet::<usize>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        let AllowKind::FolderUse {
            from,
            to,
        } = &entry.kind
        else {
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
        if shared_loops(loops, &absolute_from, &absolute_to, groups).is_empty() {
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
    let loops: FolderLoops = loops_of_each_folder(&folder_list, &grouped, &groups);
    // A group's own file makes no entry edge, so only this check sees a loop it closes;
    // the searched graph holds those uses as edges, so a loop that two own-file uses close is seen too.
    let search: FolderUses = own_loop_search(&groups, &grouped);
    findings.extend(group_own_loop_uses(
        &grouped, &search, &loops, &groups, base,
    ));
    // Every key names one finding before a line can tolerate it, so a second finding of one shape is reported.
    unique_keys(&mut findings);
    let Some(list) = allow else {
        return findings;
    };
    let reported_loops: FolderLoops = loops_of_each_folder(&folder_list, &reported, &groups);
    let tolerated_loops: BTreeSet<(LoopGraph, usize)> =
        tolerated_loops(&grouped, &loops, list, &groups, base);
    findings.extend(new_loop_uses(
        &grouped,
        &loops,
        &reported_loops,
        &tolerated_loops,
        list,
        &groups,
        base,
    ));
    let (mut kept, mut used): (Vec<LayoutFinding>, BTreeSet<usize>) = tolerated(findings, list);
    used.extend(live_use_entries(&grouped, &loops, list, &groups, base));
    kept.extend(dead_entry_findings(list, &used, base));
    return kept;
}

/// Controls of the folder rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_folder_links_tests.rs"]
mod tests;
