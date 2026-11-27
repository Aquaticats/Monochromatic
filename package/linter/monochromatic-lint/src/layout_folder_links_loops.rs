//! What:
//!  Which loops each source folder and each entry sits in,
//!  and the searches over the uses that answer it.
//! Why:
//!  A folder may sit in a folder loop and in the loop of its entry at the same time,
//!  and a use holds a loop only while both its ends are in it,
//!  so the rule needs both answers before it can say which line tolerates a use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LoopGraph = 'folders' | 'entries';
//! type FolderLoops = Map<Path, Set<[LoopGraph, number]>>;
//! // loopsOfEachFolder, sharedLoops, ownLoopSearch, usesLeadingIn, routesTo and routeFrom
//! ```

/// Import the counters that pin the cost of one search,
///  in test builds only.
#[cfg(test)]
use crate::layout_folder_links::counted_work;
/// Import the uses between folders,
///  the entry each folder belongs to,
///  the loop membership and the record a use is added through.
use crate::{
    layout_folder_uses::{
        FolderUses,
        Proof,
        loop_membership,
        record_use,
    },
    layout_group_entries::entry_of,
    layout_group_folders::entry_uses,
};
/// Import the queue of one path search,
///  ordered collections and native paths.
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
pub(crate) enum LoopGraph {
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
pub(crate) type FolderLoops = BTreeMap<PathBuf, BTreeSet<(LoopGraph, usize)>>;

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
pub(crate) fn loops_of_each_folder(
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
pub(crate) fn shared_loops(
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
///  and every use written in a group's own file as an edge from its entry,
/// with the entries of that graph.
/// Why:
///  A group's own code makes no edge of the entry graph
///  (the shelf rule reports the file and a group's summary is what its child folders use),
///  so the loop checks of the entries never see such a use.
///  A walk back from the entry one own file uses must be able to step over another group's own-file use,
///  or a mutual dependency that two own-file uses close is never seen.
///  A group nested in another group is its outer group's entry,
///  so its own file's uses leave from that entry and its loop is found like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownLoopSearch(groups: Set<Path>, grouped: FolderUses): [Path[], FolderUses];
/// ```
pub(crate) fn own_loop_search(
    groups: &BTreeSet<PathBuf>,
    grouped: &FolderUses,
) -> (Vec<PathBuf>, FolderUses) {
    let (_, entries): (Vec<PathBuf>, FolderUses) = entry_uses(groups, grouped);
    let mut search: FolderUses = entries;
    for (from, targets) in grouped {
        if !groups.contains(from) {
            continue;
        }
        // What: the entry the group's own code belongs to, which is the outer group of a nested one.
        // Why: the search holds entries only, so a nested group's own use must leave from its entry.
        let source: PathBuf = entry_of(from, groups);
        for (to, proof) in targets {
            let target_entry: PathBuf = entry_of(to, groups);
            // A use of the group's own child folders joins no two entries.
            if target_entry == source {
                continue;
            }
            record_use(&mut search, source.clone(), target_entry, proof.clone());
        }
    }
    // What: every entry the searched graph names, a use's source or its target.
    // Why: the loop membership of the graph needs a node for each of them.
    let mut nodes: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for (from, targets) in &search {
        nodes.insert(from.clone());
        for to in targets.keys() {
            nodes.insert(to.clone());
        }
    }
    return (nodes.into_iter().collect::<Vec<PathBuf>>(), search);
}

/// What:
///  The uses that lead into each entry of the searched graph,
///  so a walk back from an entry can step along them.
/// Why:
///  The index is the same table for every group of one package,
///  and building it inside the walk that reads it cloned every use of the graph once per group:
///  on a package of 300 folders that each use every later one,
///  with a hundred groups whose own files close a loop, that read the graph a hundred times
///  and cost 3.67 to 4.03 times alpha.16's CPU time (measured 2026-11-27).
///  A caller builds it once for the package, and only when some group closes a loop at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesLeadingIn(search: FolderUses): Map<Path, Map<Path, Proof>>;
/// ```
pub(crate) fn uses_leading_in(search: &FolderUses) -> BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>> {
    // What: the uses that lead into each entry, so the walk can step back along them.
    // Why: the walk goes from the group towards the entries that reach it, and a use is read from its source.
    let mut leading_in: BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>> =
        BTreeMap::<PathBuf, BTreeMap<PathBuf, Proof>>::new();
    for (from, targets) in search {
        for (to, proof) in targets {
            leading_in
                .entry(to.clone())
                .or_default()
                .insert(from.clone(), proof.clone());
        }
    }
    // What: the one index every walk of this package reads.
    // Why: a control counts this building, which a walk per group would do once per group.
    #[cfg(test)]
    counted_work::note_reverse_index();
    return leading_in;
}

/// What:
///  How many uses lead from each entry to one entry, the entry itself at zero,
///  for every entry the walk reached before each entry it was asked about had its distance.
/// Why:
///  A finding about a group's own file names the use that closes the loop
///  and every use that leads back, the way a loop finding does.
///  One table answers every own-file use that closes a loop through the same group,
///  where a walk per use read the graph once for each of them:
///  on a package of 2,000 folders whose one group's own file uses all of them,
///  each use closing a loop, a walk per use cost 14 to 16 times alpha.15's time (measured 2026-11-26).
///  The walk reads the uses backwards from the group over the index it is given,
///  so it visits only what reaches the group,
///  and it stops as soon as every entry it was asked about has its distance:
///  an entry that reaches the group directly is answered by the uses that reach the group,
///  and the graph behind it is never read.
///  The caller runs it only for a group whose own file closes a loop at all;
///  a package whose own files close none walks nothing.
///  It is breadth first, so each message spells the fewest uses that lead back,
///  and the distance is what decides a tie: a step takes the first of an entry's successors,
///  read in ascending path order, that sits one use closer to the group,
///  so two routes of one length spell the one whose entries sort first,
///  on every machine and after every edit above the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function distancesTo(reverse: Map<Path, Map<Path, Proof>>, goal: Path, asked: Set<Path>): Map<Path, number>;
/// ```
pub(crate) fn distances_to(
    reverse: &BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>>,
    goal: &Path,
    asked: &BTreeSet<PathBuf>,
) -> BTreeMap<PathBuf, usize> {
    // What: how many uses lead from each entry to the group, the group itself at zero.
    // Why: a step may only go to an entry one use closer, so no route it spells can be longer than the shortest one.
    let mut distance: BTreeMap<PathBuf, usize> = BTreeMap::<PathBuf, usize>::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::<PathBuf>::new();
    distance.insert(goal.to_path_buf(), 0);
    // What: one entry this walk reached, which a control counts.
    // Why: a walk that reaches everything that leads to the group costs that whole graph once for every group.
    #[cfg(test)]
    counted_work::note_reach();
    queue.push_back(goal.to_path_buf());
    // What: how many of the entries the walk answers still have no distance.
    // Why: once every one of them has its distance, no use further back can answer a use of this group.
    let mut unanswered: usize = asked.len();
    if asked.contains(goal) {
        unanswered -= 1;
    }
    while unanswered > 0 {
        // What: the entry whose uses lead into the group are read next, or nothing once the queue is empty.
        // Why: an entry that no use reaches has no route, so the walk ends without one for it.
        let Some(node) = queue.pop_front() else {
            break;
        };
        // What: this entry's distance, which every entry that reaches it shares one more of.
        // Why: the queue is first in first out, so an entry is reached by the shortest walk that finds it.
        let Some(here) = distance.get(&node).copied() else {
            continue;
        };
        for (before, _) in reverse.get(&node).into_iter().flatten() {
            if distance.contains_key(before) {
                continue;
            }
            distance.insert(before.clone(), here + 1);
            // What: one entry this walk reached, which a control counts.
            // Why: how many of them it reaches is what a group costs, and the shape it needs decides that number.
            #[cfg(test)]
            counted_work::note_reach();
            if asked.contains(before) {
                unanswered -= 1;
                if unanswered == 0 {
                    // Every use this group answers has its distance, so nothing further back is read.
                    break;
                }
            }
            queue.push_back(before.clone());
        }
    }
    // What: one walk of this group's uses, which a control counts.
    // Why: a walk per use would count once per use of the group, and the table it reads answers them all.
    #[cfg(test)]
    counted_work::note_walk();
    return distance;
}

/// What:
///  The route one entry takes to the goal,
///  in order and with the reference that proves each use,
///  or nothing when no route reaches it.
/// Why:
///  The finding's message spells the steps in the order a reader reads them,
///  from the used entry to the group.
///  Each step is read where the route needs it,
///  from the successors of the entry it leaves,
///  so a route costs its own length and never a table over every use the walk reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function routeFrom(search: Map<Path, Map<Path, Proof>>, distance: Map<Path, number>, start: Path, goal: Path): [Path, Path, Proof][] | undefined;
/// ```
pub(crate) fn route_from(
    search: &FolderUses,
    distance: &BTreeMap<PathBuf, usize>,
    start: &Path,
    goal: &Path,
) -> Option<Vec<(PathBuf, PathBuf, Proof)>> {
    let mut steps: Vec<(PathBuf, PathBuf, Proof)> = Vec::<(PathBuf, PathBuf, Proof)>::new();
    let mut node: PathBuf = start.to_path_buf();
    while node != *goal {
        // What: the `?` operator hands back `None` from the whole function when the value before it is the absent variant of `Option`.
        // Why: an entry the walk never reached has no route, so no finding is spelled for it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const here = distance.get(node); if (here === undefined) return undefined;
        // ```
        // What: how many uses still lead from this entry to the group.
        // Why: a step may only go to an entry one use closer, so no route spells a longer way than the shortest one.
        let here: usize = *distance.get(&node)?;
        // What: the uses this entry can take.
        // Why: a reached entry was reached along a use, so the graph holds the uses it leaves.
        let targets: &BTreeMap<PathBuf, Proof> = search.get(&node)?;
        // What: the distance a successor of this entry must hold to be a step of the shortest route.
        // Why: an entry at zero is the goal itself, so no entry before the goal has a step to look for.
        let closer: usize = here.checked_sub(1)?;
        // What: the first successor in ascending path order that sits one use closer to the group.
        // Why: that order is what spells the smallest of two equally short routes on every machine.
        let mut next: Option<(PathBuf, PathBuf, Proof)> = None;
        for (after, proof) in targets {
            if distance.get(after) != Some(&closer) {
                continue;
            }
            next = Some((node.clone(), after.clone(), proof.clone()));
            break;
        }
        // What: the step this route takes, which an entry the walk reached always has.
        // Why: such an entry was reached along a use, so one of its successors is one use closer.
        let (from, after, proof) = next?;
        steps.push((from, after.clone(), proof));
        node = after;
    }
    return Some(steps);
}
