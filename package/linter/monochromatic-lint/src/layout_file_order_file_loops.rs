//! What:
//!  The uses between the listed files of one folder,
//!  with the smallest reference that proves each,
//!  and the loops those uses make.
//! Why:
//!  A loop makes an order impossible,
//!  so the loop is reported first and the order check skips the uses inside it,
//!  and a use between two files needs the line that proves it before either finding can name it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Proof = { file: Path; line: number; text: string };
//! function usesBetween(tree: ModuleTree, uses: ModuleUse[], listed: Path[]): Map<[number, number], Proof>;
//! function loopFindings(listed, edges, base): [LayoutFinding[], Map<number, number>];
//! function loopOfEachFile(listed, edges): Map<number, number>;
//! ```

/// Import the finding shape,
///  the index file reader,
///  the loop finder,
///  the tree,
///  and the uses.
use crate::{
    layout_finding::{
        LayoutFinding,
        shown,
    },
    layout_folder_index_files::file_name,
    layout_loops::find_loops,
    layout_module_tree::ModuleTree,
    layout_module_uses::ModuleUse,
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeMap,
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
pub(crate) struct Proof {
    /// The file with the reference.
    pub(crate) file: PathBuf,
    /// Its line.
    pub(crate) line: usize,
    /// The path as written.
    pub(crate) text: String,
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
pub(crate) fn uses_between(
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
pub(crate) fn loop_findings(
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
///  Which loop each listed file of a folder is in.
/// Why:
///  An allow-list entry about one use between two files stays alive only while its two files are in one loop together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loopOfEachFile(listed: Path[], edges: Map<[number, number], Proof>): Map<number, number>;
/// ```
pub(crate) fn loop_of_each_file(
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
