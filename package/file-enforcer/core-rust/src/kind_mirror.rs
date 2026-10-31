//! What:
//!  The `mirror` rule kind:
//! make one directory an exact copy of another,
//! owning the target outright.
//! Why:
//!  `.claude/skills` and `.factory/skills` are copies of `.agents/skills`;
//! anything in a target that the source does
//!  not hold is drift,
//! which `check` lists and `write` removes.
//! Source links and special files are rejected,
//! and
//!  target links are removed as links,
//! never followed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function mirrorRule(from: string, to: string): Rule
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Change, Rule, RulePlan};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;

/// What:
///  A planned `mirror` rule.
/// Why:
///  Holds the source and target directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MirrorRule implements Rule { id: string; from: string; to: string }
/// ```
pub struct MirrorRule {
    /// Rule id.
    pub id: String,
    /// Source directory.
    pub from: String,
    /// Target directory.
    pub to: String,
}

/// What:
///  The files and directories under one directory,
/// as relative paths.
///  `BTreeSet<String>` is a sorted set of text (TS:
/// `Set<string>`,
/// but always iterated in order).
/// Why:
///  Comparing the two trees is a set difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Tree = { files: Set<string>; directories: Set<string>; others: Set<string> };
/// ```
struct Tree {
    /// Paths of regular files under the walked root,
    /// relative to it.
    files: BTreeSet<String>,
    /// Paths of directories under the walked root,
    /// relative to it.
    directories: BTreeSet<String>,
    /// Paths of symbolic links and special files under the walked root,
    /// which a mirror refuses in its source and removes from its target.
    others: BTreeSet<String>,
}

/// What:
///  Walk a directory with an explicit work stack and collect its relative paths by kind.
/// Why:
///  A stack,
/// not recursion,
/// keeps the walk's depth from limiting what it can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(view: View, root: string): Tree
/// ```
fn walk(view: &mut View, root: &str) -> Result<Tree, Diagnostic> {
    let mut tree = Tree { files: BTreeSet::new(), directories: BTreeSet::new(), others: BTreeSet::new() };
    let mut pending: Vec<String> = vec![String::new()];
    while let Some(relative) = pending.pop() {
        let full = if relative.is_empty() { root.to_string() } else { format!("{root}/{relative}") };
        for (name, kind) in view.list(&full)? {
            let child = if relative.is_empty() { name } else { format!("{relative}/{name}") };
            match kind {
                EntryKind::Directory => {
                    tree.directories.insert(child.clone());
                    pending.push(child);
                }
                EntryKind::File => {
                    tree.files.insert(child);
                }
                EntryKind::Symlink | EntryKind::Other => {
                    tree.others.insert(child);
                }
            }
        }
    }
    return Ok(tree);
}

/// What:
///  Order two directory paths deepest first,
/// and by name descending among equals.
///  `Ordering` is the three-way result of a comparison (TS:
/// a negative,
/// zero or positive `number`).
///  `&&String` is a borrow of a borrowed string,
/// which is the item type of the list being sorted.
/// Why:
///  A directory can only be removed after everything inside it,
/// so the longest paths go first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deepestFirst(left: string, right: string): number { return right.length - left.length || (right < left ? -1 : 1); }
/// ```
fn deepest_first(left: &&String, right: &&String) -> Ordering {
    return right.len().cmp(&left.len()).then(right.cmp(left));
}

/// What:
///   How the `mirror` kind plugs into the engine:
/// its id,
/// its kind name and its plan.
/// Why:
///   The plan copies every source file into the target and removes target files and directories the source lacks,
///   and refuses a source that holds a link or special file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MirrorRule implements Rule { id(); kind(); plan(view) }
/// ```
impl Rule for MirrorRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "mirror";
    }

    /// What:
    ///  Copy every source file,
    /// remove every target entry the source lacks.
    /// Why:
    ///  Target contents equal the source;
    /// the removal list is deepest-first so directories empty before they go.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let source = walk(view, &self.from)?;
        if let Some(rejected) = source.others.iter().next() {
            let path = format!("{}/{rejected}", self.from);
            let message = format!("{path} is a symbolic link or special file; mirror sources must be plain files");
            return Err(Diagnostic::new("symlink-rejected", &message).with_path(&path));
        }
        let target = walk(view, &self.to)?;
        let mut plan = RulePlan::default();
        for file in &source.files {
            let from_path = format!("{}/{file}", self.from);
            let Some(content) = view.read_bytes(&from_path)? else { continue };
            // Only the permission bits are copied: the file-type bits that `mode_of` also reports are dropped.
            let mut mode = view.mode_of(&from_path)?;
            if let Some(bits) = mode {
                mode = Some(bits & 0o777);
            }
            plan.changes.push(Change::Write { path: format!("{}/{file}", self.to), content, mode });
        }
        for entry in target.files.iter().chain(target.others.iter()) {
            if !source.files.contains(entry) || target.others.contains(entry) {
                plan.changes.push(Change::Remove { path: format!("{}/{entry}", self.to) });
            }
        }
        let mut stale_directories: Vec<&String> = Vec::new();
        for directory in &target.directories {
            if !source.directories.contains(directory) {
                stale_directories.push(directory);
            }
        }
        stale_directories.sort_by(deepest_first);
        for directory in stale_directories {
            plan.changes.push(Change::RemoveDirectory { path: format!("{}/{directory}", self.to) });
        }
        return Ok(plan);
    }
}
