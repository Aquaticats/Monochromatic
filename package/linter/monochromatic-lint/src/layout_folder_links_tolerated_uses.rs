//! What:
//!  What the allow list tolerates of a package's folder uses,
//!  and the use findings a tolerated loop hides:
//!  the uses that stay in the loop checks,
//!  the loops one listed use tolerates,
//!  the uses that join a tolerated loop,
//!  and the list entries that still hold.
//! Why:
//!  A loop the list tolerates may only lose uses,
//!  so a use added inside one is a finding no round may record,
//!  and an entry that tolerates nothing must be deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // withoutTolerated, toleratedLoops, useJoiningAToleratedLoop, newLoopUses and liveUseEntries
//! ```

/// Import the list and the entry shapes it holds,
///  the finding shape,
///  the loops of a package,
///  the use record,
///  the entry each folder belongs to,
///  and the one spelling of a listed path.
use crate::{
    layout_allow_list::{
        AllowKind,
        AllowList,
    },
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    layout_folder_links_loops::{
        FolderLoops,
        LoopGraph,
        shared_loops,
    },
    layout_folder_uses::{
        FolderUses,
        Proof,
        record_use,
    },
    layout_group_entries::entry_of,
    markdown_lfs_target::lexical_normal,
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
pub(crate) fn without_tolerated(
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
pub(crate) fn tolerated_loops(
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
///  The finding for a use that joins a loop the list tolerates,
///  and that no loop finding reports.
/// Why:
///  It carries no key,
///  so no round can record a new loop edge;
///  the loop is untangled first, or the use is removed.
///  A loop a group's own file closes held only the own-file line before alpha.16,
///  so a list written then tolerates such a loop and records none of its uses;
///  the help names the one way out, which is deleting those own-file lines of that loop
///  and writing every line the next run prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function useJoiningAToleratedLoop(from: Path, to: Path, proof: Proof, ownFileLoop: boolean, base: Path): LayoutFinding;
/// ```
pub(crate) fn use_joining_a_tolerated_loop(
    from: &Path,
    to: &Path,
    proof: &Proof,
    own_file_loop: bool,
    base: &Path,
) -> LayoutFinding {
    let help: String = if own_file_loop {
        String::from(
            "Remove the use, or untangle that loop first: an allow-list entry records today's findings and a round never adds one. If this list was written before alpha.16, it holds that loop's `own-file-loop` lines and none of its uses: delete those lines, run once with `--debug`, and write every line it prints.",
        )
    } else {
        String::from(
            "Remove the use, or untangle that loop first: an allow-list entry records today's findings and a round never adds one.",
        )
    };
    return LayoutFinding {
        file: proof.file.clone(),
        line: proof.line,
        message: format!(
            "{} uses the folder {} (`{}`), and both sit inside one loop the allow list tolerates today; that loop holds this use, and no line of the list records it, so a tolerated loop may only lose uses.",
            shown(from, base),
            shown(to, base),
            proof.text
        ),
        help: Some(help),
        key: String::new(),
    };
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
pub(crate) fn new_loop_uses(
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
            findings.push(use_joining_a_tolerated_loop(from, to, proof, false, base));
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
///  A loop a group's own file closes holds a use too:
///  the entries of that loop sit in one strongly connected set,
///  and the use's line is how the loop is recorded.
///  A line names its folders from the package folder,
///  and a folder outside the package is that walk with `..` in it,
///  so each listed path is read back into the tree's own spelling before the use is looked up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveUseEntries(grouped, loops, ownLoops, list, groups, base): Set<number>;
/// ```
pub(crate) fn live_use_entries(
    grouped: &FolderUses,
    loops: &FolderLoops,
    own_loops: &BTreeMap<PathBuf, usize>,
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
        let absolute_from: PathBuf = lexical_normal(base.join(from).as_path());
        let absolute_to: PathBuf = lexical_normal(base.join(to).as_path());
        let Some(targets) = grouped.get(&absolute_from) else {
            continue;
        };
        if !targets.contains_key(&absolute_to) {
            continue;
        }
        if !shared_loops(loops, &absolute_from, &absolute_to, groups).is_empty() {
            used.insert(index);
            continue;
        }
        // A group's own code makes no edge of the entry graph,
        // so the line that records such a use is the `own-file-loop` line and not a `uses` line.
        if groups.contains(&absolute_from) {
            continue;
        }
        let from_entry: PathBuf = entry_of(&absolute_from, groups);
        let to_entry: PathBuf = entry_of(&absolute_to, groups);
        if from_entry == to_entry {
            continue;
        }
        if own_loops.get(&from_entry).is_none()
            || own_loops.get(&from_entry) != own_loops.get(&to_entry)
        {
            continue;
        }
        used.insert(index);
    }
    return used;
}
