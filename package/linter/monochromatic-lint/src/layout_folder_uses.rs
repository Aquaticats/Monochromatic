//! What:
//!  The folders each folder uses,
//!  with the reference that proves each use,
//!  and the loops among a set of folders or entries.
//! Why:
//!  The folder rule and the group checks both start from these uses.
//!  The folder rule compares them with the `uses` links of each folder,
//!  and both report a loop with one proving reference per use inside it.
//!  The smallest proving reference is kept so the output does not depend on discovery order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Proof = { file: Path; line: number; text: string };
//! type FolderUses = Map<Path, Map<Path, Proof>>;
//! function groupByFolder(tree: ModuleTree, uses: ModuleUse[]): FolderUses;
//! function useLoops(nodes: Path[], grouped: FolderUses, base: Path, subject: string, onlyWith?: Set<Path>): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the loop finder,
///  the tree and the uses.
use crate::{
    layout_finding::{LayoutFinding, shown},
    layout_loops::find_loops,
    layout_module_tree::ModuleTree,
    layout_module_uses::{ModuleUse, folder_of},
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// What:
///  The reference that proves one folder uses another.
/// Why:
///  Every finding about a use names a file and a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { file: Path; line: number; text: string };
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Proof {
    /// The file with the reference.
    pub file: PathBuf,
    /// Its line.
    pub line: usize,
    /// The path as written.
    pub text: String,
}

/// What:
///  The folders each folder uses,
///  with the first reference that proves each use.
/// Why:
///  The smallest reference is kept so the output does not depend on discovery order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderUses = Map<Path, Map<Path, Proof>>;
/// ```
pub type FolderUses = BTreeMap<PathBuf, BTreeMap<PathBuf, Proof>>;

/// What:
///  Record one use,
///  keeping the smaller proof when the pair is already known.
/// Why:
///  The output must not depend on the order uses are found in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function record(into: FolderUses, from: Path, to: Path, proof: Proof): void;
/// ```
pub fn record_use(into: &mut FolderUses, from: PathBuf, to: PathBuf, proof: Proof) {
    let targets: &mut BTreeMap<PathBuf, Proof> = into.entry(from).or_default();
    let smaller: bool = match targets.get(&to) {
        Some(existing) => proof < *existing,
        None => true,
    };
    if smaller {
        targets.insert(to, proof);
    }
}

/// What:
///  Group module uses by the folders of the two modules.
/// Why:
///  A use inside one folder is not a folder dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupByFolder(tree: ModuleTree, uses: ModuleUse[]): FolderUses;
/// ```
pub fn group_by_folder(tree: &ModuleTree, uses: &[ModuleUse]) -> FolderUses {
    let mut grouped: FolderUses = FolderUses::new();
    for found in uses {
        let from: PathBuf = folder_of(tree, found.from);
        let to: PathBuf = folder_of(tree, found.to);
        if from == to {
            continue;
        }
        let proof: Proof = Proof {
            file: found.file.clone(),
            line: found.line,
            text: found.text.clone(),
        };
        record_use(&mut grouped, from, to, proof);
    }
    return grouped;
}

/// What:
///  The findings for nodes that use each other,
///  directly or through others.
/// Why:
///  One finding per loop lists the nodes and one reference per use inside it.
///  `subject` names what the nodes are in the message;
///  with `only_with`,
///  a loop is reported only when it holds one of those nodes,
///  so a loop of plain folders is not reported twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function useLoops(nodes: Path[], grouped: FolderUses, base: Path, subject: string, onlyWith?: Set<Path>): LayoutFinding[];
/// ```
pub fn use_loops(
    nodes: &[PathBuf],
    grouped: &FolderUses,
    base: &Path,
    subject: &str,
    only_with: Option<&BTreeSet<PathBuf>>,
) -> Vec<LayoutFinding> {
    let mut position: BTreeMap<&PathBuf, usize> = BTreeMap::<&PathBuf, usize>::new();
    for (index, node) in nodes.iter().enumerate() {
        position.insert(node, index);
    }
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::<usize>::new(); nodes.len()];
    for (from, targets) in grouped {
        for to in targets.keys() {
            if let (Some(a), Some(b)) = (position.get(from), position.get(to)) {
                adjacency[*a].push(*b);
            }
        }
    }
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for members in find_loops(adjacency.as_slice()) {
        let wanted: bool = match only_with {
            Some(held) => members
                .iter()
                .any(|member| return held.contains(&nodes[*member])),
            None => true,
        };
        if !wanted {
            continue;
        }
        let inside: BTreeSet<usize> = members.iter().copied().collect::<BTreeSet<usize>>();
        let mut lines: Vec<String> = Vec::<String>::new();
        let mut anchor: Option<Proof> = None;
        for member in &members {
            let from: &PathBuf = &nodes[*member];
            for (to, proof) in grouped.get(from).into_iter().flatten() {
                let Some(target) = position.get(to) else {
                    continue;
                };
                if !inside.contains(target) {
                    continue;
                }
                lines.push(format!(
                    "{} uses {} ({}:{} `{}`)",
                    shown(from, base),
                    shown(to, base),
                    shown(&proof.file, base),
                    proof.line,
                    proof.text
                ));
                if anchor.is_none() {
                    anchor = Some(proof.clone());
                }
            }
        }
        let Some(first) = anchor else {
            continue;
        };
        findings.push(LayoutFinding {
            file: first.file,
            line: first.line,
            message: format!(
                "These {subject} use each other in a loop: {}.",
                lines.join("; ")
            ),
            help: Some(String::from(
                "Move the shared code into a folder both can use, or pass it in, so that one direction of use disappears.",
            )),
        });
    }
    return findings;
}
