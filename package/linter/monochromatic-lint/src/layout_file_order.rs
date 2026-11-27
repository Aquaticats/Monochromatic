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
//!  Where each part of the rule lives:
//!  `layout_file_order_file_loops` holds the uses between a folder's files and the loops they make,
//!  `layout_file_order_top_doc_list` holds the findings about how a top doc lists the files,
//!  and `layout_file_order_tolerated_uses` holds what the allow list tolerates of those uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkFileOrder(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
//! ```

/// Import the uses between a folder's files and the loops they make,
///  re-exported so a caller asks the file rule for them.
pub(crate) use crate::layout_file_order_file_loops::{
    Proof,
    loop_findings,
    loop_of_each_file,
    uses_between,
};
/// Import the finding shape,
///  the list that tolerates findings,
///  the folder and index file readers,
///  the group finder,
///  the tree,
///  the uses,
///  and the top doc reader.
use crate::{
    layout_allow_list::{
        AllowList,
        dead_entry_findings,
        tolerated,
    },
    layout_file_order_tolerated_uses::{
        live_file_use_entries,
        new_loop_file_uses,
        tolerated_file_loops,
        without_tolerated_files,
    },
    layout_file_order_top_doc_list::list_findings,
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
    layout_module_tree::ModuleTree,
    layout_module_uses::ModuleUse,
    layout_top_doc::{
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
