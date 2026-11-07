//! What:
//!  The exit status of `git diff-index --cached --quiet <commit-or-HEAD> [-- <paths>]` without starting Git:
//! 0 when the index
//!  holds exactly the commit's tree for those paths,
//! 1 when it differs.
//! Why:
//!  The wrapper asks this before every merge,
//! before every commit's index check and when it proves a restored worktree;
//! each
//!  start cost a process (and,
//! under load,
//! more than forty seconds) for a question two files answer.
//! The rules are
//!  `diff-lib.c` (`diff_cache`,
//! `oneway_diff`,
//! `do_oneway_diff`) and `diff.c` (`diff_addremove`,
//! `diff_change`,
//!  `is_submodule_ignored`) of Git 2.56.0:
//! an unmerged entry is always a difference (`diff_unmerge`),
//! an intent-to-add entry
//!  is an ordinary entry with the empty blob (`diff-index` never sets `ita_invisible_in_index`),
//! and a submodule difference
//!  can be hidden by `.gitmodules`,
//! so that one case is left to Git when a `.gitmodules` exists anywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code: 0 | 1 | undefined = diffIndexQuiet({ commit, paths }); // undefined: start Git
//! ```

use crate::repo_read_diff::PathFilter;
use crate::repo_read_index::{INTENT_TO_ADD, IndexEntry, ParsedIndex};
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_request_answer_and_context::{Answer, Context, Parsed, flags_are};
use crate::repo_read_refs::{RefValue, Refs};
use crate::repo_read_tree::{FlatEntry, MODE_GITLINK, flatten_tree_shared, parse_commit};
use crate::repo_read_tree_cached::{UnequalParts, outside, unequal_parts};

/// The commit `HEAD` names,
/// for a branch under `refs/heads/` or a detached head.
fn head_commit(context: &Context) -> Option<String> {
    let refs: Refs = Refs::new(context.layout.common_dir.as_path(), context.layout.object_format.hex_length());
    return match refs.lookup_head(context.layout.git_dir.as_path())? {
        RefValue::Direct(object) => Some(object),
        RefValue::Symbolic(target) if target.starts_with("refs/heads/") => refs.resolve(target.as_str()),
        RefValue::Symbolic(_) => None,
    };
}

/// The root tree of the commit or tree `operand` names (a hexadecimal name or `HEAD`).
fn tree_of_operand(context: &Context, database: &ObjectDatabase, operand: &[u8]) -> Option<Vec<u8>> {
    let hex: Vec<u8> = if operand == b"HEAD" { head_commit(context)?.into_bytes() } else { operand.to_vec() };
    let name: Vec<u8> = from_hex(hex.as_slice(), context.layout.object_format)?;
    let object = database.read(name.as_slice())?;
    return match object.kind {
        ObjectKind::Tree => Some(name),
        ObjectKind::Commit => Some(parse_commit(object.bytes.as_slice(), context.layout.object_format)?.tree),
        ObjectKind::Blob | ObjectKind::Tag => None,
    };
}

/// Whether a `.gitmodules` file is in the worktree or in either list,
/// so a submodule difference might be configured away.
fn has_gitmodules(context: &Context, tree: &[FlatEntry], index: &[IndexEntry]) -> bool {
    if context.layout.worktree_root.join(".gitmodules").symlink_metadata().is_ok() {
        return true;
    }
    for entry in tree {
        if entry.path == b".gitmodules" {
            return true;
        }
    }
    for entry in index {
        if entry.path == b".gitmodules" {
            return true;
        }
    }
    return false;
}

/// Whether `mode` is the mode of a submodule link (a Git link).
fn is_link(mode: u32) -> bool {
    return mode & 0o170_000 == MODE_GITLINK;
}

/// The answer of a quiet comparison:
/// no output,
/// and the exit code `1` when the two sides differ.
fn quiet_answer(differs: bool) -> Option<Answer> {
    return Some(Answer { stdout: Vec::new(), stderr: Vec::new(), code: i32::from(differs) });
}

/// How the two sorted lists differ:
/// whether a difference Git always reports exists,
/// and whether one that only a submodule
/// setting could hide exists.
fn compare(tree: &[&FlatEntry], index: &[&IndexEntry]) -> (bool, bool) {
    let (mut left, mut right): (usize, usize) = (0, 0);
    let mut hideable: bool = false;
    while left < tree.len() || right < index.len() {
        let order = match (tree.get(left), index.get(right)) {
            (Some(old), Some(new)) => old.path.cmp(&new.path),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, _) => std::cmp::Ordering::Greater,
        };
        let mode: u32 = match order {
            std::cmp::Ordering::Less => {
                left += 1;
                tree[left - 1].mode
            }
            std::cmp::Ordering::Greater => {
                right += 1;
                index[right - 1].mode
            }
            std::cmp::Ordering::Equal => {
                let (old, new): (&FlatEntry, &IndexEntry) = (tree[left], index[right]);
                left += 1;
                right += 1;
                // A pair whose two sides are both submodules is the one `diff_change` can skip; any other pair is reported.
                if old.mode == new.mode && old.id == new.id {
                    continue;
                }
                if is_link(old.mode) && is_link(new.mode) {
                    hideable = true;
                    continue;
                }
                return (true, hideable);
            }
        };
        if is_link(mode) {
            hideable = true;
        } else {
            return (true, hideable);
        }
    }
    return (false, hideable);
}

/// `diff-index --cached --quiet <commit> [-- <paths>]`.
pub(crate) fn diff_index_quiet(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--cached", "--quiet"], &[]) || parsed.operands.len() != 1 {
        return None;
    }
    let filter: PathFilter = PathFilter::parse(parsed.specs.as_slice(), context.layout.prefix.is_empty())?;
    let database: ObjectDatabase = context.database()?;
    let tree_name: Vec<u8> = tree_of_operand(context, &database, parsed.operands[0])?;
    let whole_index: ParsedIndex = context.index_with_tree()?;
    // The cache-tree names the directories the index has not changed; with no unmerged or intent-to-add entry (which it says
    //  nothing of) those are left unread on both sides, and otherwise, or when its walk declines, the whole tree is read.
    let mut trusted: bool = true;
    let mut paths: Vec<&[u8]> = Vec::new();
    for entry in &whole_index.entries {
        if entry.stage != 0 || entry.extended & INTENT_TO_ADD != 0 {
            trusted = false;
        }
        paths.push(entry.path.as_slice());
    }
    let mut unequal: Option<UnequalParts> = None;
    if trusted && let Some(nodes) = whole_index.tree.as_ref() {
        unequal = unequal_parts(&database, tree_name.as_slice(), context.layout.object_format, nodes, paths.as_slice());
    }
    let whole_tree: std::rc::Rc<Vec<FlatEntry>>;
    let tree: &[FlatEntry] = if let Some(parts) = &unequal {
        parts.tree.as_slice()
    } else {
        whole_tree = flatten_tree_shared(&database, tree_name.as_slice(), context.layout.object_format)?;
        whole_tree.as_slice()
    };
    let mut kept: Vec<IndexEntry> = Vec::new();
    if let Some(parts) = unequal.as_ref() {
        for entry in outside(whole_index.entries.as_slice(), parts.skipped.as_slice()) {
            kept.push(entry.clone());
        }
    }
    let index: &[IndexEntry] = if unequal.is_some() { kept.as_slice() } else { whole_index.entries.as_slice() };
    let mut wanted_index: Vec<&IndexEntry> = Vec::new();
    for entry in index {
        if filter.matches(entry.path.as_slice()) {
            wanted_index.push(entry);
        }
    }
    // An unmerged path is a pair of its own (`diff_unmerge`), whatever the tree holds.
    for entry in &wanted_index {
        if entry.stage != 0 {
            return quiet_answer(true);
        }
    }
    let mut wanted_tree: Vec<&FlatEntry> = Vec::new();
    for entry in tree {
        if filter.matches(entry.path.as_slice()) {
            wanted_tree.push(entry);
        }
    }
    let (reported, hideable) = compare(wanted_tree.as_slice(), wanted_index.as_slice());
    if reported {
        return quiet_answer(true);
    }
    if hideable && has_gitmodules(context, tree, index) {
        return None;
    }
    return quiet_answer(hideable);
}

/// Controls for the quiet comparison against real Git stay out of the release executable.
#[cfg(test)]
#[path = "repo_read_query_quiet_tests.rs"]
mod tests;
