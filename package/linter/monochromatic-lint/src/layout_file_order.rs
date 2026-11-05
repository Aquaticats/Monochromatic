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
//!  executable roots next to a library root are not listed either;
//!  a folder with module files but no `mod.rs` is a finding,
//! because it has no top doc.
//!  Known limit:
//!  an item that `mod.rs` re-exports and a sibling file reaches through `super::` hides the file that defines it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkFileOrder(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
/// the loop finder,
/// the tree,
/// the uses and the top doc reader.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_loops::find_loops,
    layout_module_tree::ModuleTree,
    layout_module_uses::ModuleUse,
    layout_top_doc::{LIST_MARKER, TopDoc, read_top_doc},
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
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
///  The name of a file without its folder.
/// Why:
///  The top doc lists bare file names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileName(file: Path): string;
/// ```
fn file_name(file: &Path) -> String {
    return match file.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(),
    };
}

/// What:
///  The files of each folder,
///  and the files that are roots of a crate.
/// Why:
///  A file is the unit of the rule,
/// however many modules it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filesByFolder(tree: ModuleTree): { byFolder: Map<Path, Set<Path>>; roots: Set<Path> };
/// ```
fn files_by_folder(tree: &ModuleTree) -> (BTreeMap<PathBuf, BTreeSet<PathBuf>>, BTreeSet<PathBuf>) {
    let mut by_folder: BTreeMap<PathBuf, BTreeSet<PathBuf>> =
        BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
    for module in &tree.modules {
        if let Some(folder) = module.file.parent() {
            by_folder
                .entry(folder.to_path_buf())
                .or_default()
                .insert(module.file.clone());
        }
    }
    let mut roots: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for root in &tree.roots {
        roots.insert(tree.modules[*root].file.clone());
    }
    return (by_folder, roots);
}

/// What:
///  The file that holds a folder's top doc.
/// Why:
///  `mod.rs` for a module folder;
/// `lib.rs`,
/// else `main.rs`,
/// for the folder of a crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexFile(folder: Path, files: Set<Path>, roots: Set<Path>): Path | undefined;
/// ```
fn index_file(
    folder: &Path,
    files: &BTreeSet<PathBuf>,
    roots: &BTreeSet<PathBuf>,
) -> Option<PathBuf> {
    for name in ["mod.rs", "lib.rs", "main.rs"] {
        let candidate: PathBuf = folder.join(name);
        if files.contains(&candidate) && (name == "mod.rs" || roots.contains(&candidate)) {
            return Some(candidate);
        }
    }
    return None;
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
        });
    }
    return (findings, component);
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
    let Some(marker) = doc.marker_line else {
        findings.push(LayoutFinding {
            file: index.to_path_buf(),
            line: doc.first_line,
            message: format!(
                "The top doc of {} has no file list, so its files are not listed in order.",
                shown(index, base)
            ),
            help: Some(format!(
                "Add the line `//! {LIST_MARKER}` to the top doc, then one line `//! - `name.rs`: what it does` per file, each file after the files it uses."
            )),
        });
        return findings;
    };
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
            });
        }
    }
    for name in &names {
        if !first_place.contains_key(name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: marker,
                message: format!(
                    "The top doc of {} does not list the file `{name}`.",
                    shown(index, base)
                ),
                help: Some(format!(
                    "Add a line `//! - `{name}`: what it does` after the files it uses."
                )),
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
        });
    }
    return findings;
}

/// What:
///  Check the file rule over a package.
/// Why:
///  A folder with fewer than one listed file has nothing to order and nothing to list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFileOrder(tree: ModuleTree, uses: ModuleUse[], base: Path): LayoutFinding[];
/// ```
pub fn check_file_order(tree: &ModuleTree, uses: &[ModuleUse], base: &Path) -> Vec<LayoutFinding> {
    let (by_folder, roots) = files_by_folder(tree);
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (folder, files) in &by_folder {
        let index: Option<PathBuf> = index_file(folder, files, &roots);
        let mut listed: Vec<PathBuf> = Vec::<PathBuf>::new();
        for file in files {
            if Some(file) == index.as_ref() || roots.contains(file) {
                continue;
            }
            listed.push(file.clone());
        }
        if listed.is_empty() {
            continue;
        }
        let edges = uses_between(tree, uses, listed.as_slice());
        let (loops, component) = loop_findings(listed.as_slice(), &edges, base);
        findings.extend(loops);
        let Some(index_path) = index else {
            findings.push(LayoutFinding {
                file: listed[0].clone(),
                line: 0,
                message: format!(
                    "The folder {} holds module files but no mod.rs, so it has no top doc to list them in.",
                    shown(folder, base)
                ),
                help: Some(String::from("Add a mod.rs that declares the modules and carries the top doc.")),
            });
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
                });
                continue;
            }
        };
        let doc: TopDoc = read_top_doc(text.as_str());
        findings.extend(list_findings(
            &doc,
            &index_path,
            listed.as_slice(),
            &edges,
            &component,
            base,
        ));
    }
    return findings;
}

/// Controls of the file rule stay outside release artifacts.
#[cfg(test)]
#[path = "layout_file_order_tests.rs"]
mod tests;
