//! What:
//!  The file rule:
//!  the files of one folder never use each other in a loop,
//!  and the folder's top doc lists them in an order where each file uses only files listed before it.
//! Why:
//!  A folder's reader should be able to read its files top to bottom,
//!  each one building only on those already read.
//!  A loop makes such an order impossible,
//! so a loop is reported first
//!  and the order check skips the uses inside a loop.
//!  Decisions:
//!  the file that holds the top doc (`mod.rs`,
//! or `lib.rs` or `main.rs` in the folder of a crate root)
//!  is not listed and is not part of the loops,
//! because it declares the folder's modules and every file may
//!  name it with `super::`;
//!  that is safe because an index file may hold only its top doc,
//! `mod` lines and `pub use` re-exports,
//!  and any other item in it is a finding that says to move it into a file of the folder
//!  (`main.rs` that is the index may also hold `fn main` and plain `use` lines);
//!  a use of a name that the index re-exports counts as a use of the sibling file that defines it,
//!  so a re-export hides neither a loop nor an order;
//!  executable roots next to a library root (a `main.rs` beside `lib.rs`,
//!  a single file under `src/bin`) are not listed either,
//!  and are read for code like an entry point;
//!  a folder with module files but no `mod.rs` is a finding,
//! because it has no top doc;
//!  a group folder (a folder that holds child folders) is left to the folder rule,
//! which lets it hold only its child folders and its `mod.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkFileOrder(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
/// the folder and index file readers,
/// the loop finder,
/// the tree,
/// the uses and the top doc reader.
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
    layout_folder_index_files::{
        file_name,
        files_by_folder,
        index_file,
        index_file_findings,
        name_holds_space,
        space_finding,
    },
    layout_group_entries::find_groups,
    layout_loops::find_loops,
    layout_module_tree::ModuleTree,
    layout_module_uses::ModuleUse,
    layout_top_doc::{
        LIST_MARKER,
        TopDoc,
        read_top_doc,
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
///  A reference that proves one file uses another.
/// Why:
///  Findings name a file and a line.
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
///  The uses between the listed files of one folder,
///  with the smallest proving reference for each pair.
/// Why:
///  Uses of other folders,
/// of the index file and of oneself are not part of this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesBetween(tree: ModuleTree, uses: ModuleUse[], listed: Path[]): Map<[number, number], Proof>;
/// ```
fn uses_between(
    tree: &ModuleTree,
    uses: &[ModuleUse],
    listed: &[PathBuf],
) -> BTreeMap<(usize, usize), Proof> {
    let mut position: BTreeMap<&PathBuf, usize> = BTreeMap::<&PathBuf, usize>::new();
    for (index, file) in listed.iter().enumerate() {
        position.insert(file, index);
    }
    let mut edges: BTreeMap<(usize, usize), Proof> = BTreeMap::<(usize, usize), Proof>::new();
    for found in uses {
        let (Some(from), Some(to)) = (
            position.get(&tree.modules[found.from].file),
            position.get(&tree.modules[found.to].file),
        ) else {
            continue;
        };
        if from == to {
            continue;
        }
        let proof: Proof = Proof {
            file: found.file.clone(),
            line: found.line,
            text: found.text.clone(),
        };
        // Keep the smallest proof so the output does not depend on discovery order.
        let smaller: bool = match edges.get(&(*from, *to)) {
            Some(existing) => proof < *existing,
            None => true,
        };
        if smaller {
            edges.insert((*from, *to), proof);
        }
    }
    return edges;
}

/// What:
///  The loop findings of one folder,
///  and which files are in a loop together.
/// Why:
///  The order check must not complain about uses a loop makes unavoidable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loopFindings(listed: Path[], edges: Map<[number, number], Proof>, base: Path): { findings: LayoutFinding[]; component: Map<number, number> };
/// ```
fn loop_findings(
    listed: &[PathBuf],
    edges: &BTreeMap<(usize, usize), Proof>,
    base: &Path,
) -> (Vec<LayoutFinding>, BTreeMap<usize, usize>) {
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::<usize>::new(); listed.len()];
    for (from, to) in edges.keys() {
        adjacency[*from].push(*to);
    }
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let mut component: BTreeMap<usize, usize> = BTreeMap::<usize, usize>::new();
    for (number, members) in find_loops(adjacency.as_slice()).into_iter().enumerate() {
        let mut lines: Vec<String> = Vec::<String>::new();
        let mut anchor: Option<Proof> = None;
        for member in &members {
            component.insert(*member, number);
        }
        for (pair, proof) in edges {
            if !(members.contains(&pair.0) && members.contains(&pair.1)) {
                continue;
            }
            lines.push(format!(
                "{} uses {} ({}:{} `{}`)",
                file_name(&listed[pair.0]),
                file_name(&listed[pair.1]),
                shown(&proof.file, base),
                proof.line,
                proof.text
            ));
            if anchor.is_none() {
                anchor = Some(proof.clone());
            }
        }
        let Some(first) = anchor else {
            continue;
        };
        findings.push(LayoutFinding {
            file: first.file,
            line: first.line,
            message: format!("These files use each other in a loop: {}.", lines.join("; ")),
            help: Some(String::from(
                "Move what they share into a file of its own that both use, so that one direction of use disappears.",
            )),
            key: String::new(),
        });
    }
    return (findings, component);
}

/// What:
///  The key of a finding whose last word is the name a top doc lists.
/// Why:
///  An entry whose backticks hold no name would end the key in the space a list reader trims away,
///  so an entry that names nothing is keyed by its shape and index alone,
///  and `unique_keys` numbers a second such entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryKey(shape: string, index: Path, name: string, base: Path): string;
/// ```
fn entry_key(
    shape: &str,
    index: &Path,
    name: &str,
    base: &Path,
) -> String {
    let named: String = format!("{shape} {}", key_path(index, base));
    // What: an entry that names nothing contributes no word.
    // Why: no module file is named by an empty name, so the shape and the index name the finding alone.
    if name.is_empty() {
        return named;
    }
    return format!("{named} {name}");
}

/// What:
///  The findings about how a top doc lists a folder's files.
/// Why:
///  A file must be listed once,
/// a listed name must be a file of the folder,
///  and a file must come after every file it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listFindings(doc: TopDoc, index: Path, listed: Path[], edges: Map<[number, number], Proof>, component: Map<number, number>, base: Path): LayoutFinding[];
/// ```
fn list_findings(
    doc: &TopDoc,
    index: &Path,
    listed: &[PathBuf],
    edges: &BTreeMap<(usize, usize), Proof>,
    component: &BTreeMap<usize, usize>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    // The line a missing entry is reported on: the marker, or the first doc line when there is no list at all.
    let list_line: usize = doc.marker_line.unwrap_or(doc.first_line);
    if doc.marker_line.is_none() {
        // The help shows the marker broken after its comma, which is where the Markdown rule breaks it.
        let (head, tail) = LIST_MARKER.split_once(", ").unwrap_or((LIST_MARKER, ""));
        findings.push(LayoutFinding {
            file: index.to_path_buf(),
            line: doc.first_line,
            message: format!(
                "The top doc of {} has no file list, so its files are not listed in order.",
                shown(index, base)
            ),
            help: Some(format!(
                "Add the two lines `//! {head},` and `//! {tail}` to the top doc, so the marker is wrapped where the Markdown rule `markdown/semantic-line-breaks` breaks it (it may wrap at any other word too), then one line `//! - `name.rs`: what it does` per file, each file after the files it uses."
            )),
            key: format!("no-file-list {}", key_path(index, base)),
        });
    }
    // A doc with no list still owes the reader one, so every unlisted file is reported on its own:
    // a file added to this folder later is a finding that no `no-file-list` line tolerates.
    let mut first_place: BTreeMap<String, usize> = BTreeMap::<String, usize>::new();
    for (place, entry) in doc.entries.iter().enumerate() {
        if first_place.contains_key(&entry.name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: entry.line,
                message: format!(
                    "The top doc of {} lists `{}` twice.",
                    shown(index, base),
                    entry.name
                ),
                help: None,
                key: entry_key("listed-twice", index, entry.name.as_str(), base),
            });
            continue;
        }
        first_place.insert(entry.name.clone(), place);
    }
    let mut names: Vec<String> = Vec::<String>::new();
    for file in listed {
        names.push(file_name(file));
    }
    for entry in &doc.entries {
        if !names.contains(&entry.name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: entry.line,
                message: format!(
                    "The top doc of {} lists `{}`, which is not a module file of this folder.",
                    shown(index, base),
                    entry.name
                ),
                help: Some(String::from("Remove the line, or correct the file name.")),
                key: entry_key("foreign-name", index, entry.name.as_str(), base),
            });
        }
    }
    for name in &names {
        if !first_place.contains_key(name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: list_line,
                message: format!(
                    "The top doc of {} does not list the file `{name}`.",
                    shown(index, base)
                ),
                help: Some(format!(
                    "Add a line `//! - `{name}`: what it does` after the files it uses."
                )),
                key: format!("missing-entry {} {}", key_path(index, base), name),
            });
        }
    }
    for ((from, to), proof) in edges {
        let same_loop: bool =
            component.get(from).is_some() && component.get(from) == component.get(to);
        let (Some(at_from), Some(at_to)) =
            (first_place.get(&names[*from]), first_place.get(&names[*to]))
        else {
            continue;
        };
        if same_loop || at_to < at_from {
            continue;
        }
        findings.push(LayoutFinding {
            file: proof.file.clone(),
            line: proof.line,
            message: format!(
                "`{}` uses `{}` (`{}`) but the top doc of {} lists it first; a file may use only files listed before it.",
                names[*from],
                names[*to],
                proof.text,
                shown(index, base)
            ),
            help: Some(format!("List `{}` before `{}`.", names[*to], names[*from])),
            key: format!(
                "order {} {} {}",
                key_path(index, base),
                names[*from],
                names[*to]
            ),
        });
    }
    return findings;
}

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
fn without_tolerated_files(
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
///  Which loop each listed file of a folder is in.
/// Why:
///  An allow-list entry about one use between two files stays alive only while its two files are in one loop together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loopOfEachFile(listed: Path[], edges: Map<[number, number], Proof>): Map<number, number>;
/// ```
fn loop_of_each_file(
    listed: &[PathBuf],
    edges: &BTreeMap<(usize, usize), Proof>,
) -> BTreeMap<usize, usize> {
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::<usize>::new(); listed.len()];
    for (from, to) in edges.keys() {
        adjacency[*from].push(*to);
    }
    let mut loops: BTreeMap<usize, usize> = BTreeMap::<usize, usize>::new();
    for (number, members) in find_loops(adjacency.as_slice()).into_iter().enumerate() {
        for member in members {
            loops.insert(member, number);
        }
    }
    return loops;
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
fn new_loop_file_uses(
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
fn tolerated_file_loops(
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
fn live_file_use_entries(
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

/// What:
///  Why a folder's files cannot be read as a list.
/// Why:
///  The folder owes its reader a list either way,
///  and the two cases have to say different things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unreadable = 'no-top-doc' | 'unreadable-index';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Unreadable {
    /// The folder holds module files and no `mod.rs` that could list them.
    NoTopDoc,
    /// The folder's index file exists and cannot be read.
    Index,
}

/// What:
///  One finding per file of a folder whose top doc could not show the list.
/// Why:
///  A `no-top-doc` or `unreadable-index` entry tolerates that one finding,
///  so a list that stopped there would tolerate every file the folder gained afterwards.
///  Each file is therefore a finding of its own,
///  keyed like every other unlisted file,
///  and a file added later is reported until the folder's top doc lists it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistedFiles(doc: Path, reason: Unreadable, listed: Path[], base: Path): LayoutFinding[];
/// ```
fn unlisted_files(
    doc: &Path,
    reason: Unreadable,
    listed: &[PathBuf],
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for file in listed {
        let name: String = file_name(file);
        let message: String = match reason {
            Unreadable::NoTopDoc => format!(
                "The folder {} holds module files but no mod.rs to list them in, so the file `{name}` is not listed in any top doc.",
                shown(doc, base)
            ),
            Unreadable::Index => format!(
                "The top doc of {} cannot be read, so the file `{name}` is not listed in it.",
                shown(doc, base)
            ),
        };
        let help: String = match reason {
            Unreadable::NoTopDoc => format!(
                "Add a mod.rs that declares this folder's modules and holds a top doc, then one line `//! - `{name}`: what it does` after the files it uses."
            ),
            Unreadable::Index => format!(
                "Make the top doc readable, then list `{name}` in it after the files it uses."
            ),
        };
        findings.push(LayoutFinding {
            file: file.clone(),
            line: 0,
            message,
            help: Some(help),
            key: format!("missing-entry {} {name}", key_path(doc, base)),
        });
    }
    return findings;
}

/// What:
///  Check the file rule over a package.
/// Why:
///  The index file of every folder is checked for code first,
///  even when the folder has no other file;
///  a folder with fewer than one listed file has nothing to order and nothing to list.
///  `allow` is the list the rule's `allow` option names,
///  and the top doc is still expected to list every file,
///  because a list records findings and never changes what a top doc owes its reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFileOrder(tree: ModuleTree, uses: ModuleUse[], base: Path, allow?: AllowList): LayoutFinding[];
/// ```
pub fn check_file_order(
    tree: &ModuleTree,
    uses: &[ModuleUse],
    base: &Path,
    allow: Option<&AllowList>,
) -> Vec<LayoutFinding> {
    let (by_folder, roots) = files_by_folder(tree);
    let groups: BTreeSet<PathBuf> = find_groups(tree);
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let mut used: BTreeSet<usize> = BTreeSet::<usize>::new();
    for (folder, files) in &by_folder {
        // A group folder has no files to order and is checked by the folder rule.
        if groups.contains(folder) {
            continue;
        }
        let index: Option<PathBuf> = index_file(folder, files, &roots);
        findings.extend(index_file_findings(index.as_ref(), files, &roots, base));
        let mut listed: Vec<PathBuf> = Vec::<PathBuf>::new();
        for file in files {
            if Some(file) == index.as_ref() || roots.contains(file) {
                continue;
            }
            // A name holding a space forbids a `file-uses` line, which a loop needs to be tolerated.
            // The file stays in every check, so a loop through it is reported rather than hidden.
            if name_holds_space(file) {
                findings.push(space_finding(file, folder, base));
            }
            listed.push(file.clone());
        }
        if listed.is_empty() {
            continue;
        }
        let edges = uses_between(tree, uses, listed.as_slice());
        let reported: BTreeMap<(usize, usize), Proof> = match allow {
            Some(list) => without_tolerated_files(folder, base, &edges, listed.as_slice(), list),
            None => edges.clone(),
        };
        let (loops, component) = loop_findings(listed.as_slice(), &reported, base);
        findings.extend(loops);
        if let Some(list) = allow {
            let loops_of_files: BTreeMap<usize, usize> =
                loop_of_each_file(listed.as_slice(), &edges);
            let tolerated: BTreeSet<usize> = tolerated_file_loops(
                folder,
                &edges,
                listed.as_slice(),
                &loops_of_files,
                list,
                base,
            );
            let reported_loops_of_files: BTreeMap<usize, usize> =
                loop_of_each_file(listed.as_slice(), &reported);
            findings.extend(new_loop_file_uses(
                folder,
                &edges,
                listed.as_slice(),
                &loops_of_files,
                &reported_loops_of_files,
                &tolerated,
                list,
                base,
            ));
            live_file_use_entries(
                folder,
                base,
                &edges,
                listed.as_slice(),
                &loops_of_files,
                list,
                &mut used,
            );
        }
        let Some(index_path) = index else {
            // What: the sentence about a `mod.rs` on disk that no `mod` line declares, when the folder holds one.
            // Why: the folder owes its reader a top doc, and the file that looks like one is no part of the package.
            let stray: String = if folder.join("mod.rs").is_file() {
                String::from(
                    " The mod.rs on disk is not part of the package, because no mod line declares it.",
                )
            } else {
                String::new()
            };
            findings.push(LayoutFinding {
                file: listed[0].clone(),
                line: 0,
                message: format!(
                    "The folder {} holds module files but no mod.rs that declares them, so it has no top doc to list them in.{stray}",
                    shown(folder, base)
                ),
                help: Some(String::from("Add a mod.rs that declares the modules and carries the top doc.")),
                key: format!("no-top-doc {}", key_path(folder, base)),
            });
            // A folder with no top doc lists none of its files, so each file is a finding of its own.
            findings.extend(unlisted_files(
                folder,
                Unreadable::NoTopDoc,
                listed.as_slice(),
                base,
            ));
            continue;
        };
        let text: String = match std::fs::read_to_string(&index_path) {
            Ok(read) => read,
            Err(error) => {
                findings.push(LayoutFinding {
                    file: index_path.clone(),
                    line: 0,
                    message: format!("{} cannot be read: {error}.", shown(&index_path, base)),
                    help: None,
                    key: format!("unreadable-index {}", key_path(&index_path, base)),
                });
                // A top doc that cannot be read lists nothing, so each file is a finding of its own too.
                findings.extend(unlisted_files(
                    &index_path,
                    Unreadable::Index,
                    listed.as_slice(),
                    base,
                ));
                continue;
            }
        };
        let doc: TopDoc = read_top_doc(text.as_str());
        findings.extend(list_findings(
            &doc,
            &index_path,
            listed.as_slice(),
            &reported,
            &component,
            base,
        ));
    }
    // Every key names one finding before a line can tolerate it, so a second finding of one shape is reported.
    unique_keys(&mut findings);
    let Some(list) = allow else {
        return findings;
    };
    let (mut kept, mut entries_used): (Vec<LayoutFinding>, BTreeSet<usize>) =
        tolerated(findings, list);
    entries_used.extend(used);
    // What: the folders of this package, so a dead line that names one of them says whether the tree holds it.
    // Why: a mistyped or hand-aligned line names a folder no tree holds, and naming it is what tells its reader to re-space the line.
    let folders: BTreeSet<PathBuf> = by_folder.keys().cloned().collect::<BTreeSet<PathBuf>>();
    kept.extend(dead_entry_findings(list, &entries_used, &folders, base));
    return kept;
}

/// Controls of the file rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_file_order_tests.rs"]
mod tests;
