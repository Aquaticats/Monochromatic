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
//!  Where each part of the rule lives:
//!  `layout_folder_links_entries` holds the findings about one folder's links,
//!  `layout_folder_links_loops` holds the loops each folder and each entry sits in,
//!  `layout_folder_links_own_file_loops` holds the loop a group's own file closes,
//!  and `layout_folder_links_tolerated_uses` holds what the allow list tolerates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkFolderLinks(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the list that tolerates findings,
///  the folder uses,
///  the group checks,
///  the tree and the uses.
use crate::{
    layout_allow_list::{
        AllowList,
        dead_entry_findings,
        tolerated,
    },
    layout_finding::{
        LayoutFinding,
        unique_keys,
    },
    layout_folder_uses::{
        FolderUses,
        group_by_folder,
        loop_membership,
        use_loops,
    },
    layout_group_entries::find_groups,
    layout_group_folders::{
        group_findings,
        group_loops,
    },
    layout_module_tree::ModuleTree,
    layout_module_uses::{
        ModuleUse,
        folder_of,
    },
};
/// Import the findings about one folder's links,
///  the loops each folder sits in,
///  the loop a group's own file closes,
///  and what the allow list tolerates.
use crate::{
    layout_folder_links_entries::{
        check_one_folder,
        folder_below_uses,
        glob_reexports,
    },
    layout_folder_links_loops::{
        FolderLoops,
        LoopGraph,
        loops_of_each_folder,
        own_loop_search,
    },
    layout_folder_links_own_file_loops::{
        OwnFileLoop,
        group_own_loop_uses,
        held_own_file_loops,
        uses_inside_a_reported_own_file_loop,
        uses_joining_a_tolerated_own_file_loop,
    },
    layout_folder_links_tolerated_uses::{
        live_use_entries,
        new_loop_uses,
        tolerated_loops,
        without_tolerated,
    },
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
///  How much work one run of the folder rule did,
///  for the controls that pin a cost.
/// Why:
///  A cost has no test of its own,
///  and a control that timed one would read this machine's load rather than the code,
///  so a control counts the work instead and fails when the walk or the index is done more often.
///  Test builds only, so a release artifact carries no counter and no branch on one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let reverseIndexes = 0;
/// let walks = 0;
/// ```
#[cfg(test)]
pub(crate) mod counted_work {
    /// Import the cell the counters live in.
    use std::cell::Cell;
    // What: one counter per thread, because the test runner runs several tests at once in one process.
    // Why: a counter shared between tests would add two tests' work together and make both controls meaningless;
    // the siblings a reader might expect are a plain `static` and a lock, and neither keeps the tests apart.
    thread_local! {
        /// The times the reverse index of the searched graph was built.
        static REVERSE_INDEXES: Cell<usize> = const { Cell::new(0) };
        /// The times one group's uses were read backwards to their shortest routes.
        static WALKS: Cell<usize> = const { Cell::new(0) };
    }

    /// Count one building of the reverse index.
    pub(crate) fn note_reverse_index() {
        REVERSE_INDEXES.with(|count| return count.set(count.get() + 1));
    }

    /// Count one backward walk from a group.
    pub(crate) fn note_walk() {
        WALKS.with(|count| return count.set(count.get() + 1));
    }

    /// How many reverse indexes this thread built.
    pub(crate) fn reverse_indexes() -> usize {
        return REVERSE_INDEXES.with(|count| return count.get());
    }

    /// How many backward walks this thread ran.
    pub(crate) fn walks() -> usize {
        return WALKS.with(|count| return count.get());
    }

    /// Start one control from nothing counted.
    pub(crate) fn reset() {
        REVERSE_INDEXES.with(|count| return count.set(0));
        WALKS.with(|count| return count.set(0));
    }
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
    // What: the list as this package's rule asks it, already in the one spelling a key writes.
    // Why: the reader read every listed path back against this package folder,
    // so one spelling decides tolerance, death and a repeat, in both rules alike.
    let listed: Option<&AllowList> = allow;
    let untolerated: FolderUses = match listed {
        Some(list) => without_tolerated(&grouped, list, base),
        None => grouped.clone(),
    };
    findings.extend(use_loops(
        folder_list.as_slice(),
        &untolerated,
        base,
        "folders",
        None,
    ));
    findings.extend(group_findings(tree, &groups, &grouped, &sources, base));
    findings.extend(group_loops(&groups, &untolerated, base));
    findings.extend(glob_reexports(tree, base));
    findings.extend(folder_below_uses(folder_list.as_slice(), base));
    let loops: FolderLoops = loops_of_each_folder(&folder_list, &grouped, &groups);
    // A group's own file makes no entry edge, so only this check sees a loop it closes;
    // the searched graph holds those uses as edges, so a loop that two own-file uses close is seen too.
    let (search_nodes, search): (Vec<PathBuf>, FolderUses) = own_loop_search(&groups, &grouped);
    // What: the loop each entry of the searched graph is in.
    // Why: two entries close a loop exactly when they share one, so one pass over the graph answers every use.
    let own_loops: BTreeMap<PathBuf, usize> = loop_membership(search_nodes.as_slice(), &search);
    let own_file_loops: Vec<OwnFileLoop> =
        group_own_loop_uses(&grouped, &search, &own_loops, &loops, &groups, base);
    findings.extend(
        own_file_loops
            .iter()
            .map(|item| return item.finding.clone()),
    );
    // Every key names one finding before a line can tolerate it, so a second finding of one shape is reported.
    unique_keys(&mut findings);
    let Some(list) = listed else {
        return findings;
    };
    let reported_loops: FolderLoops = loops_of_each_folder(&folder_list, &untolerated, &groups);
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
    // A line for an own-file use holds the loop the entries make,
    // so a use that joins that loop is a finding too, and every other use of the loop is recorded by a `uses` line.
    let (held, reported): (BTreeSet<usize>, BTreeSet<usize>) =
        held_own_file_loops(&own_file_loops, &own_loops, list);
    findings.extend(uses_inside_a_reported_own_file_loop(
        &grouped, &own_loops, &reported, &loops, list, &groups, base,
    ));
    findings.extend(uses_joining_a_tolerated_own_file_loop(
        &grouped, &own_loops, &held, &loops, list, &groups, base,
    ));
    let (mut kept, mut used): (Vec<LayoutFinding>, BTreeSet<usize>) = tolerated(findings, list);
    used.extend(live_use_entries(
        &grouped, &loops, &own_loops, list, &groups, base,
    ));
    kept.extend(dead_entry_findings(list, &used, &folders, base));
    return kept;
}

/// Controls of the folder rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_folder_links_tests.rs"]
mod tests;
