//! What:
//!  What the allow list tolerates of one folder's file uses:
//!  the uses that stay in the loop check,
//!  the loops one listed use tolerates,
//!  the uses that join a tolerated loop,
//!  and the entries that still hold.
//! Why:
//!  A loop the list tolerates may only lose uses,
//!  so a use added inside one is a finding no round may record,
//!  and an entry that tolerates nothing must be deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // withoutToleratedFiles, positionOf, newLoopFileUses, toleratedFileLoops and liveFileUseEntries
//! ```

/// Import the reference that proves one use out of the module the file rule re-exports it from.
use crate::layout_file_order_file_loops::Proof;
/// Import the list and the entry shapes it holds,
///  the finding shape,
///  the index file reader,
///  the proof of one use,
///  and the words for a use that joins a tolerated loop.
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
    layout_folder_index_files::file_name,
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
///  The uses that stay in the loop check of one folder:
///  every one except the pairs the list tolerates for that folder.
/// Why:
///  A loop is reported only when one of its uses is not tolerated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutToleratedFiles(folder: Path, edges: Map<[number, number], Proof>, listed: Path[], list: AllowList): Map<[number, number], Proof>;
/// ```
pub(crate) fn without_tolerated_files(
    folder: &Path,
    base: &Path,
    edges: &BTreeMap<(usize, usize), Proof>,
    listed: &[PathBuf],
    list: &AllowList,
) -> BTreeMap<(usize, usize), Proof> {
    let mut kept: BTreeMap<(usize, usize), Proof> = BTreeMap::<(usize, usize), Proof>::new();
    for ((from, to), proof) in edges {
        let tolerated_use: bool = list.covers_file_use(
            Path::new(key_path(folder, base).as_str()),
            file_name(&listed[*from]).as_str(),
            file_name(&listed[*to]).as_str(),
        );
        if tolerated_use {
            continue;
        }
        kept.insert((*from, *to), proof.clone());
    }
    return kept;
}

/// What:
///  The index of a listed file by its name,
///  because an entry names files and the rule works on positions.
/// Why:
///  A name that the folder does not hold names no use and is a dead entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function positionOf(listed: Path[]): Map<string, number>;
/// ```
fn position_of(listed: &[PathBuf]) -> BTreeMap<String, usize> {
    let mut positions: BTreeMap<String, usize> = BTreeMap::<String, usize>::new();
    for (index, file) in listed.iter().enumerate() {
        positions.insert(file_name(file), index);
    }
    return positions;
}

/// What:
///  The findings for uses that join two files of a folder that already sit in one tolerated loop.
/// Why:
///  A tolerated loop may only lose uses,
///  so a new use inside it is a loop edge no entry records and no round may record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newLoopFileUses(folder: Path, edges: Map<[number, number], Proof>, listed: Path[], loops: Map<number, number>, list: AllowList, base: Path): LayoutFinding[];
/// ```
pub(crate) fn new_loop_file_uses(
    folder: &Path,
    edges: &BTreeMap<(usize, usize), Proof>,
    listed: &[PathBuf],
    loops: &BTreeMap<usize, usize>,
    reported_loops: &BTreeMap<usize, usize>,
    tolerated: &BTreeSet<usize>,
    list: &AllowList,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let key_of_folder: String = key_path(folder, base);
    for ((from, to), proof) in edges {
        let tolerated_use: bool = list.covers_file_use(
            Path::new(key_of_folder.as_str()),
            file_name(&listed[*from]).as_str(),
            file_name(&listed[*to]).as_str(),
        );
        if tolerated_use {
            continue;
        }
        let Some(loop_of_from) = loops.get(from) else {
            continue;
        };
        if loops.get(to) != Some(loop_of_from) {
            continue;
        }
        if !tolerated.contains(loop_of_from) {
            continue;
        }
        // A loop the loop check reports already lists this use.
        if let Some(reported) = reported_loops.get(from)
            && reported_loops.get(to) == Some(reported)
        {
            continue;
        }
        findings.push(LayoutFinding {
            file: proof.file.clone(),
            line: proof.line,
            message: format!(
                "`{}` uses `{}` (`{}`), and both files sit inside one loop the allow list tolerates today; that loop holds this use, and no line of the list records it, so a tolerated loop may only lose uses.",
                file_name(&listed[*from]),
                file_name(&listed[*to]),
                proof.text
            ),
            help: Some(format!(
                "Remove the use, or untangle the loop in {} first: an allow-list entry records today's findings and a round never adds one.",
                shown(folder, base)
            )),
            key: String::new(),
        });
    }
    return findings;
}

/// What:
///  The loops of one folder that one of the list's entries already tolerates a use of.
/// Why:
///  A loop the list knows nothing about is reported by the loop check,
///  so only a loop the list has started tolerating can hide a use added later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toleratedFileLoops(folder: Path, edges: Map<[number, number], Proof>, listed: Path[], loops: Map<number, number>, list: AllowList, base: Path): Set<number>;
/// ```
pub(crate) fn tolerated_file_loops(
    folder: &Path,
    edges: &BTreeMap<(usize, usize), Proof>,
    listed: &[PathBuf],
    loops: &BTreeMap<usize, usize>,
    list: &AllowList,
    base: &Path,
) -> BTreeSet<usize> {
    let mut tolerated: BTreeSet<usize> = BTreeSet::<usize>::new();
    let key_of_folder: String = key_path(folder, base);
    for (from, to) in edges.keys() {
        let listed_use: bool = list.covers_file_use(
            Path::new(key_of_folder.as_str()),
            file_name(&listed[*from]).as_str(),
            file_name(&listed[*to]).as_str(),
        );
        if !listed_use {
            continue;
        }
        let Some(loop_of_from) = loops.get(from) else {
            continue;
        };
        if loops.get(to) == Some(loop_of_from) {
            tolerated.insert(*loop_of_from);
        }
    }
    return tolerated;
}

/// What:
///  The entries of one folder that still hold:
///  the use exists and its two files are still in one loop together.
/// Why:
///  They are the entries that tolerated something;
///  every other entry about a folder is dead and must be deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveFileUseEntries(folder: Path, edges: Map<[number, number], Proof>, listed: Path[], loops: Map<number, number>, list: AllowList): Set<number>;
/// ```
pub(crate) fn live_file_use_entries(
    folder: &Path,
    base: &Path,
    edges: &BTreeMap<(usize, usize), Proof>,
    listed: &[PathBuf],
    loops: &BTreeMap<usize, usize>,
    list: &AllowList,
    used: &mut BTreeSet<usize>,
) {
    let positions: BTreeMap<String, usize> = position_of(listed);
    let key_of_folder: String = key_path(folder, base);
    for (index, entry) in list.entries().iter().enumerate() {
        let AllowKind::FileUse {
            folder: listed_folder,
            from,
            to,
        } = &entry.kind
        else {
            continue;
        };
        if *listed_folder != Path::new(key_of_folder.as_str()) {
            continue;
        }
        let (Some(from_index), Some(to_index)) = (positions.get(from), positions.get(to)) else {
            continue;
        };
        if !edges.contains_key(&(*from_index, *to_index)) {
            continue;
        }
        let Some(loop_of_from) = loops.get(from_index) else {
            continue;
        };
        if loops.get(to_index) != Some(loop_of_from) {
            continue;
        }
        used.insert(index);
    }
}
