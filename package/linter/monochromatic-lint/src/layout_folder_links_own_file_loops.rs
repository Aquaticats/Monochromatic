//! What:
//!  The loop a group's own file closes:
//!  the finding for the use that closes it,
//!  the loops a line for such a use holds,
//!  and the uses that sit inside a loop the list holds or reports.
//! Why:
//!  A group's own code makes no edge of the entry graph,
//!  so no loop check of the entries sees such a use,
//!  and new code in a tolerated group file could add a loop no line records.
//!  The search runs over the entry graph with every own-file use added as an edge,
//!  and one walk from a group answers every own-file use that closes a loop through it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OwnFileLoop = { finding: LayoutFinding; entries: [Path, Path] };
//! // groupOwnLoopUses, heldOwnFileLoops, usesInsideOwnFileLoops,
//! // usesJoiningAToleratedOwnFileLoop and usesInsideAReportedOwnFileLoop
//! ```

/// Import the list,
///  the finding shape,
///  the loops of a package,
///  the search that answers a route,
///  the words for a use that joins a tolerated loop,
///  the folder uses,
///  and the entry each folder belongs to.
use crate::{
    layout_allow_list::AllowList,
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    layout_folder_links_loops::{
        FolderLoops,
        distances_to,
        route_from,
        shared_loops,
        uses_leading_in,
    },
    layout_folder_links_tolerated_uses::use_joining_a_tolerated_loop,
    layout_folder_uses::{
        FolderUses,
        Proof,
    },
    layout_group_entries::entry_of,
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
///  One finding about a use written in a group's own file,
///  with the entry pair its loop joins.
/// Why:
///  A list line for the use holds the loop the entries make,
///  so the check for a use that joins that loop needs to know which loop it is.
///  Every other use of that loop is recorded by its own `uses` line,
///  because a listed loop may only lose uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnFileLoop = { finding: LayoutFinding; entries: [Path, Path] };
/// ```
pub(crate) struct OwnFileLoop {
    /// The finding the rule reports.
    pub(crate) finding: LayoutFinding,
    /// The two entries the use joins.
    entries: (PathBuf, PathBuf),
}

/// What:
///  The findings for a use written in a group's own file
///  that closes a mutual dependency between its entry and another entry.
/// Why:
///  A group's own code makes no edge of the entry graph
///  (the shelf rule reports the file and a group's summary is what its child folders use),
///  so the loop checks of the entries never see such a use.
///  Without this check, new code in a tolerated group file could add a loop,
///  and the allow list promises that new code cannot add one.
///  The search runs over the entry graph with every own-file use added as an edge,
///  so a loop that two own-file uses close is seen as one that a single use closes.
///  A group nested in another group leaves its uses from that outer entry,
///  so a nested group is searched like any other.
///  Two entries close a loop exactly when the searched graph puts them in one strongly connected set,
///  which is computed once for the package;
///  only a use that closes one is reported, so a package of groups that close none walks nothing.
///  One walk answers every such use of one group:
///  it reads the uses backwards from the group once,
///  and each use takes its own shortest route from that table,
///  where a walk per use read the graph once for each of them.
///  The reverse index the walks read is built once for the package, as `uses_leading_in` says.
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
/// function groupOwnLoopUses(grouped: FolderUses, search: FolderUses, ownLoops: Map<Path, number>, loops: FolderLoops, groups: Set<Path>, base: Path): OwnFileLoop[];
/// ```
pub(crate) fn group_own_loop_uses(
    grouped: &FolderUses,
    search: &FolderUses,
    own_loops: &BTreeMap<PathBuf, usize>,
    loops: &FolderLoops,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<OwnFileLoop> {
    // What: the uses that close a loop, grouped by the entry the group's own file leaves from.
    // Why: one walk from that entry answers all of them, and only such a group pays a walk.
    let mut closing: BTreeMap<PathBuf, Vec<(PathBuf, PathBuf, Proof)>> =
        BTreeMap::<PathBuf, Vec<(PathBuf, PathBuf, Proof)>>::new();
    for (from, targets) in grouped {
        if !groups.contains(from) {
            continue;
        }
        // What: the entry the group's own code belongs to, which is the outer group of a nested one.
        // Why: the walk stops at the source entry, so a nested group's own use must leave from its entry.
        let source: PathBuf = entry_of(from, groups);
        for (to, proof) in targets {
            let target_entry: PathBuf = entry_of(to, groups);
            // A use inside the group itself joins no two entries.
            if target_entry == source {
                continue;
            }
            // A folder loop holding both folders already reports this use, or the use added inside a tolerated loop.
            if !shared_loops(loops, from, to, groups).is_empty() {
                continue;
            }
            // One strongly connected set means the used entry reaches the group back, so the use closes a loop.
            let Some(group_loop) = own_loops.get(&source) else {
                continue;
            };
            if own_loops.get(&target_entry) != Some(group_loop) {
                continue;
            }
            closing.entry(source.clone()).or_default().push((
                from.clone(),
                to.clone(),
                proof.clone(),
            ));
        }
    }
    // What: the reverse index of the searched graph, built once for the package.
    // Why: it is one table for every group, and building it inside each walk cloned the whole graph per group;
    // a package whose own files close no loop at all still builds none.
    let reverse: Option<BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>>> = if closing.is_empty() {
        None
    } else {
        Some(uses_leading_in(search))
    };
    let mut findings: Vec<OwnFileLoop> = Vec::<OwnFileLoop>::new();
    for (source, uses) in &closing {
        // What: the one index this group's walk reads.
        // Why: every group reads the same table, so only the walk below is done once per group.
        let Some(index) = reverse.as_ref() else {
            continue;
        };
        // What: the entries this group's own uses are answered for.
        // Why: the walk stops once every one of them has its route, so a use that reaches the group directly
        // costs the uses that reach the group and not the graph behind the entry it uses.
        let mut asked: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
        for (_, to, _) in uses {
            asked.insert(entry_of(to, groups));
        }
        // What: how many uses lead from every entry the walk had to reach, to this group.
        // Why: one table answers every use below, and it is read only for a group that closes a loop at all.
        let distance: BTreeMap<PathBuf, usize> = distances_to(index, source, &asked);
        for (from, to, proof) in uses {
            let target_entry: PathBuf = entry_of(to, groups);
            let Some(route) = route_from(search, &distance, &target_entry, source) else {
                continue;
            };
            // What: the steps the finding's message spells, from the used entry to the group.
            // Why: the finding shows the shortest route the walk found, so a reader can check that the loop is real.
            let mut steps: Vec<String> = Vec::<String>::new();
            for (step_from, step_to, step_proof) in &route {
                steps.push(format!(
                    "{} uses {} (`{}` at {}:{})",
                    shown(step_from, base),
                    shown(step_to, base),
                    step_proof.text,
                    shown(&step_proof.file, base),
                    step_proof.line
                ));
            }
            findings.push(OwnFileLoop {
                finding: LayoutFinding {
                    file: proof.file.clone(),
                    line: proof.line,
                    message: format!(
                        "These entries use each other in a loop: {} uses {} (`{}`); {}.",
                        shown(from, base),
                        shown(to, base),
                        proof.text,
                        steps.join("; ")
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
                },
                entries: (source.clone(), target_entry),
            });
        }
    }
    return findings;
}

/// What:
///  The loops a list line for an own-file use holds,
///  and the loops one of whose own-file findings no line holds.
/// Why:
///  A line holds the loop the finding reported,
///  so only a loop whose every own-file finding is on the list may hide a use that joined it;
///  a loop one of whose findings is missing is reported by that finding,
///  and its uses are printed as the lines that record them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function heldOwnFileLoops(ownFileLoops: OwnFileLoop[], ownLoops: Map<Path, number>, list: AllowList): [Set<number>, Set<number>];
/// ```
pub(crate) fn held_own_file_loops(
    own_file_loops: &[OwnFileLoop],
    own_loops: &BTreeMap<PathBuf, usize>,
    list: &AllowList,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut held: BTreeSet<usize> = BTreeSet::<usize>::new();
    let mut reported: BTreeSet<usize> = BTreeSet::<usize>::new();
    for item in own_file_loops {
        let Some(number) = own_loops.get(&item.entries.0) else {
            continue;
        };
        if list.covers_key(item.finding.key.as_str()) {
            held.insert(*number);
            continue;
        }
        reported.insert(*number);
    }
    // What: the loops one of whose own-file findings no line holds.
    // Why: such a loop is reported by that finding, so it holds no use of its own.
    for number in &reported {
        held.remove(number);
    }
    return (held, reported);
}

/// What:
///  The uses that sit inside one of the loops named,
///  with the reference that proves each.
/// Why:
///  One answer serves both checks about a loop a group's own file closes:
///  whether the list holds the loop, and whether the list reports it.
///  A use of a group's own file is left out,
///  because its own finding records it;
///  a use between two folders of one entry makes no edge of the entry graph;
///  a use the list tolerates is recorded already;
///  and a use a loop of folders or entries holds is reported by that loop's check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesInsideOwnFileLoops(grouped, ownLoops, which: Set<number>, loops, list, groups, base): [Path, Path, Proof][];
/// ```
fn uses_inside_own_file_loops(
    grouped: &FolderUses,
    own_loops: &BTreeMap<PathBuf, usize>,
    which: &BTreeSet<usize>,
    loops: &FolderLoops,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<(PathBuf, PathBuf, Proof)> {
    let mut found: Vec<(PathBuf, PathBuf, Proof)> = Vec::<(PathBuf, PathBuf, Proof)>::new();
    for (from, targets) in grouped {
        // A group's own use is reported by its own keyed finding.
        if groups.contains(from) {
            continue;
        }
        let from_entry: PathBuf = entry_of(from, groups);
        let Some(number) = own_loops.get(&from_entry) else {
            continue;
        };
        if !which.contains(number) {
            continue;
        }
        for (to, proof) in targets {
            let to_entry: PathBuf = entry_of(to, groups);
            // A use between two folders of one entry makes no edge of the entry graph.
            if to_entry == from_entry {
                continue;
            }
            if own_loops.get(&to_entry) != Some(number) {
                continue;
            }
            // The list holds the paths findings show, which are relative to the package folder.
            if list.covers_use(
                Path::new(key_path(from, base).as_str()),
                Path::new(key_path(to, base).as_str()),
            ) {
                continue;
            }
            // A loop of folders or entries holding both folders reports this use, or the use added inside it.
            if !shared_loops(loops, from, to, groups).is_empty() {
                continue;
            }
            found.push((from.clone(), to.clone(), proof.clone()));
        }
    }
    return found;
}

/// What:
///  The findings for uses that join a loop an own-file line tolerates,
///  and that no line of the list records.
/// Why:
///  A tolerated loop may only lose uses:
///  a use added inside it joins a loop edge that no line records,
///  and no line may be added for it,
///  so the run says to remove the use or to untangle the loop first.
///  The keyless finding is what makes the new use unrecordable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesJoiningAToleratedOwnFileLoop(grouped, ownLoops, held, loops, list, groups, base): LayoutFinding[];
/// ```
pub(crate) fn uses_joining_a_tolerated_own_file_loop(
    grouped: &FolderUses,
    own_loops: &BTreeMap<PathBuf, usize>,
    held: &BTreeSet<usize>,
    loops: &FolderLoops,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (from, to, proof) in
        uses_inside_own_file_loops(grouped, own_loops, held, loops, list, groups, base)
    {
        findings.push(use_joining_a_tolerated_loop(&from, &to, &proof, true, base));
    }
    return findings;
}

/// What:
///  The findings for the uses inside a loop a group's own file closes
///  that no line of the list records yet.
/// Why:
///  A loop is tolerated only while every use inside it is listed,
///  and a `uses` line is what records one use,
///  except the use written in the group's own file, which `own-file-loop` records.
///  The finding prints that line as its key,
///  so a round records the loop in one run of the rule;
///  once the loop's own-file findings are listed the use carries no key,
///  and a use added later still has no line to hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesInsideAReportedOwnFileLoop(grouped, ownLoops, reported, loops, list, groups, base): LayoutFinding[];
/// ```
pub(crate) fn uses_inside_a_reported_own_file_loop(
    grouped: &FolderUses,
    own_loops: &BTreeMap<PathBuf, usize>,
    reported: &BTreeSet<usize>,
    loops: &FolderLoops,
    list: &AllowList,
    groups: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (from, to, proof) in
        uses_inside_own_file_loops(grouped, own_loops, reported, loops, list, groups, base)
    {
        findings.push(use_inside_a_reported_own_file_loop(
            &from, &to, &proof, base,
        ));
    }
    return findings;
}

/// What:
///  The finding for a use inside a loop a group's own file closes that no line records yet.
/// Why:
///  The loop is reported, so its uses do not need a finding of their own to be visible;
///  the finding is how the rule prints the `uses` line that records the use,
///  and a round writes that line so the loop is held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function useInsideAReportedOwnFileLoop(from: Path, to: Path, proof: Proof, base: Path): LayoutFinding;
/// ```
fn use_inside_a_reported_own_file_loop(
    from: &Path,
    to: &Path,
    proof: &Proof,
    base: &Path,
) -> LayoutFinding {
    return LayoutFinding {
        file: proof.file.clone(),
        line: proof.line,
        message: format!(
            "{} uses the folder {} (`{}`), and both sit inside one loop a group's own file closes, which the list does not record yet; a loop is tolerated only while every use inside it is on the list, so this use and every other one in that loop need their own `uses` lines.",
            shown(from, base),
            shown(to, base),
            proof.text
        ),
        help: Some(format!(
            "Add `uses {} -> {}` to the list, or untangle that loop first.",
            key_path(from, base),
            key_path(to, base)
        )),
        key: format!("uses {} -> {}", key_path(from, base), key_path(to, base)),
    };
}
