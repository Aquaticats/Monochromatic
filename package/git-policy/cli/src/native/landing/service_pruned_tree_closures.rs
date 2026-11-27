//! What:
//!  Whether a tree,
//! a tree below it or a blob in any of them is gone from the object store,
//! for every root of a batch,
//! and what that walk cost.
//! Why:
//!   An amendment carries a whole tree and no list of blobs,
//! and a prune can remove a blob from under a tree that survives until the branch moves
//! (`service_pruned.rs`).
//! One work stack and one set of examined objects serve the whole batch,
//! the walk is linear in the batch's distinct objects with no recursion,
//! and the budget is what keeps an unbounded walk from holding a landing,
//! so the walk,
//! its ending and its cost stand in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const missing = treeClosuresMissing(database, roots, format) // logged and false when the budget ran out
//! ```

use crate::diagnostic_log::warn;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{
    ObjectKind,
    from_hex,
    name_length,
};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{
    MODE_GITLINK,
    is_tree_mode,
    parse_tree,
};
use std::collections::HashSet;
/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli-service";
/// How many tree entries (and trees) the walk of one batch examines before it stops asking and leaves the rest to Git.
pub const CLOSURE_ENTRY_BUDGET: usize = 5_000_000;
/// How the walk over the trees of a batch ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureEnd = 'intact' | 'missing' | 'out-of-budget';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ClosureEnd {
    /// Every tree and blob below every root is present,
    /// or is left to Git as unreadable.
    Intact,
    /// An object below a root is gone from the object store.
    Missing,
    /// The budget ran out first,
    /// so nothing is known about the rest.
    OutOfBudget,
}

/// What a walk found and what it cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ClosureWalk {
    /// How it ended.
    pub(super) end: ClosureEnd,
    /// Entries and trees examined,
    /// which is what the budget counts.
    pub(super) examined: usize,
}

/// What:
///  Whether a tree,
/// a tree below it or a blob in any of them is gone from the object store,
/// for every root of a batch.
/// Why:
///   An amendment carries a whole tree and no list of blobs,
/// and a prune can remove a blob from under a tree that survives
///  until the branch moves.
/// One work stack and one set of examined objects serve the whole batch,
/// so a tree that several
///  requests share (the usual amendment batch changes few files of one large tree) is walked once,
/// and a blob that several
///  trees list is looked up once:
/// the cost is linear in the batch's distinct objects,
/// with no recursion.
/// A found loss ends
///  the whole walk,
/// so a shared set never hides one.
/// The budget counts every tree and entry examined,
/// not only distinct
///  trees;
/// when it runs out the answer is "not known",
/// as for an object that cannot be read here,
/// because holding the
///  landing for an unbounded walk delays every later commit of the worktree.
/// A submodule entry names a commit of another
///  repository and is skipped,
/// and an object that is present but cannot be read here (unreadable,
/// beyond the reader's size
///  limit) is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// walkTrees(roots, { shared: true, budget }).some((entry) => !objectStore.has(entry.id))
/// ```
pub(super) fn walk_tree_closures(
    database: &ObjectDatabase,
    roots: &[&str],
    format: ObjectFormat,
    budget: usize,
) -> ClosureWalk {
    // A root that is not an object name of the repository's format is left out.
    let mut pending: Vec<Vec<u8>> = Vec::new();
    for root in roots {
        if let Some(bytes) = from_hex(root.as_bytes(), format) {
            pending.push(bytes);
        }
    }
    let mut visited: HashSet<Vec<u8>> = HashSet::new();
    let mut examined: usize = 0;
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        if examined >= budget {
            return ClosureWalk {
                end: ClosureEnd::OutOfBudget,
                examined,
            };
        }
        examined += 1;
        let Some(object) = database.read(name.as_slice()) else {
            if database.contains(name.as_slice()) {
                continue;
            }
            return ClosureWalk {
                end: ClosureEnd::Missing,
                examined,
            };
        };
        if object.kind != ObjectKind::Tree {
            continue;
        }
        let Some(entries) = parse_tree(object.bytes.as_slice(), name_length(format)) else {
            continue;
        };
        if entries.len() > budget - examined {
            return ClosureWalk {
                end: ClosureEnd::OutOfBudget,
                examined,
            };
        }
        examined += entries.len();
        for entry in entries {
            if entry.mode == MODE_GITLINK {
                continue;
            }
            if is_tree_mode(entry.mode) {
                pending.push(entry.id);
            } else if visited.insert(entry.id.clone()) && !database.contains(entry.id.as_slice()) {
                return ClosureWalk {
                    end: ClosureEnd::Missing,
                    examined,
                };
            }
        }
    }
    return ClosureWalk {
        end: ClosureEnd::Intact,
        examined,
    };
}

/// Whether the walk found a loss;
/// an exhausted budget is logged and counts as "not known",
/// which is `false`.
pub(super) fn tree_closures_missing(
    database: &ObjectDatabase,
    roots: &[&str],
    format: ObjectFormat,
) -> bool {
    let walk: ClosureWalk = walk_tree_closures(database, roots, format, CLOSURE_ENTRY_BUDGET);
    if walk.end == ClosureEnd::OutOfBudget {
        warn(
            TAG,
            &format!(
                "the trees of the batch hold more than {CLOSURE_ENTRY_BUDGET} entries; their objects are not checked here and are left to Git"
            ),
        );
    }
    return walk.end == ClosureEnd::Missing;
}
