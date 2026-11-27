//! What:
//!  The fence over a clone rung's bytes:
//! which tracked paths the clones may carry,
//! which paths Git must write,
//! the attributes each decision reads,
//! and the one Git run that writes the rest.
//! Why:
//!  Cloning a source's working files and installing the source's index
//! makes a worktree that every check calls clean,
//!  even where Git's checkout would have written other bytes
//! (`package/git-policy/cli/SPEC.md`,
//! rung 2;
//! GitHub issue 6323).
//! The clone may carry a path only where the checkout's conversion of its blob is the blob itself,
//! with the source's file proven to hold that blob:
//! `worktree_clone_conversion_rules::decide` decides that from Git's own attributes,
//! `core.autocrlf`,
//! `core.eol` and the recorded sizes.
//! The attributes come from one `git check-attr --all`,
//! so every source Git reads is answered by Git,
//! macros and precedence included;
//! sizes come from one `git ls-tree -r -l`,
//! for every path the fence may carry,
//! because a path with no rule can still hold an earlier rule's form;
//! and the paths Git must write go through one `git checkout-index` in the destination,
//! which applies the destination's own conversion.
//! The source's bytes must be the blob's,
//! so an untracked or ignored `.gitattributes` above a tracked path refuses the rung:
//! Git read it when the files were added,
//! the new worktree will not have it,
//! and no answer Git gives today can prove what it said then.
//!
//! The attributes a decision reads and the one Git run that writes the rest
//! are the sibling files `worktree_clone_conversion_fence_attributes.rs` and
//! `worktree_clone_conversion_fence_write.rs`.
//!

use crate::diagnostic_log::debug;
use crate::transaction_git::GitContext;
/// The change a rung cannot serve,
/// and why,
/// re-exported so every caller of this fence names it here.
pub use crate::worktree_change_a_rung_cannot_serve::{
    ChangeNotServed,
    WhyNotServed,
};
/// How many paths a rule of their own converts,
/// and the share that declines the rung,
/// re-exported so every caller of this fence names them here.
pub use crate::worktree_clone_conversion_decline::{
    DECLINE_PATHS_FLOOR,
    DECLINE_SHARE_DIVISOR,
    converting_paths,
    declines,
};
use crate::worktree_clone_conversion_decline::{
    MODE_SYMLINK,
    MODE_TYPE_MASK,
};
/// The attributes half of the fence,
/// re-exported so every caller of this fence names it here.
pub use crate::worktree_clone_conversion_fence_attributes::{
    AttributeFence,
    fence_the_attributes,
    refuse_untracked_attributes,
};
/// The one Git run that writes the paths the fence leaves to Git,
/// re-exported so every caller of this fence names it here.
pub use crate::worktree_clone_conversion_fence_write::{
    checkout_index_through_git,
    checkout_index_through_the_index_at,
    write_through_git,
};
use crate::worktree_clone_conversion_git_reads::{
    BlobSizes,
    blob_sizes,
};
use crate::worktree_clone_conversion_rules::{
    FenceVerdict,
    PathAttributes,
    decide,
};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashSet;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCloneConversionFence";

/// The paths beyond which the fence logs no more reasons,
/// because the count says the rest and a line per path would bury it.
const WRITTEN_PATHS_REPORTED: usize = 8;

/// What:
///  What one clone rung's fence decided.
/// Why:
///   The caller clones what it may carry and hands the rest to one Git run,
/// whose paths it also names in a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fence = { gitWrites: Buffer[]; carried: number };
/// ```
pub struct Fence {
    /// The paths Git writes,
    /// in index order.
    pub git_writes: Vec<Vec<u8>>,
    /// How many tracked paths the clones may carry.
    pub carried: usize,
}

/// What:
///  Decide every tracked path the attributes left:
/// what the clones may carry and what Git must write.
/// Why:
///   This is the half that costs `git ls-tree`,
/// so it runs where the copies are,
/// and a control can price it apart from
///  the reads the decline needed.
/// `sizes` lets a caller that already has the map hand it in
///  (`decide_the_paths_with_sizes`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideThePaths(source, context, read): Fence; // throws the reason the rung cannot be proven
/// ```
pub fn decide_the_paths(
    source: &SnapshotSource,
    context: &GitContext,
    read: AttributeFence,
) -> Result<Fence, String> {
    // One `git ls-tree` for every path whose checkout can write the blob unchanged, which is every path this fence may
    // carry: the blob's size is the proof the file holds it, and a file can hold another rule's form even when today it
    // has no rule at all (an unset `core.autocrlf`, a tracked attributes file a later commit deleted).
    let sizes: Option<BlobSizes> = match read.needs_sizes() {
        true => Some(blob_sizes(
            context,
            source.root.as_path(),
            source.clean.head_commit.as_str(),
        )?),
        false => None,
    };
    return decide_the_paths_with_sizes(read, sizes);
}

/// What:
///  `decide_the_paths` with the sizes already answered,
///  for a rung that started the read before its source check.
/// Why:
///   The sizes may come from a read that ran beside the check
/// (`worktree_clone_conversion_size_read.rs`),
///  so this half takes the map rather than running Git for it,
///  and it needs nothing else from the source:
/// `None` means no path needed one,
/// which is `needs_sizes`'s own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideThePathsWithSizes(read, sizes): Fence;
/// ```
pub fn decide_the_paths_with_sizes(
    read: AttributeFence,
    sizes: Option<BlobSizes>,
) -> Result<Fence, String> {
    let AttributeFence {
        entries,
        attributes,
        rules,
        a_changed_path_could_matter: _matters,
    }: AttributeFence = read;
    let mut git_writes: Vec<Vec<u8>> = Vec::new();
    let mut carried: usize = 0;
    for entry in &entries {
        let held: PathAttributes = attributes
            .get(entry.path.as_slice())
            .cloned()
            .unwrap_or_default();
        let is_link: bool = entry.mode & MODE_TYPE_MASK == MODE_SYMLINK;
        let file_size: u64 = u64::from(entry.stored.size);
        let listed: Option<u64> = sizes
            .as_ref()
            .and_then(|known| return known.get(entry.path.as_slice()).copied());
        match decide(&held, &rules, is_link, file_size, listed) {
            FenceVerdict::Carry => carried += 1,
            FenceVerdict::WriteThroughGit(reason) => {
                if git_writes.len() < WRITTEN_PATHS_REPORTED {
                    debug(
                        TAG,
                        fence_sentence(entry.path.as_slice(), reason.as_str()).as_str(),
                    );
                }
                git_writes.push(entry.path.clone());
            }
        }
    }
    debug(
        TAG,
        format!(
            "the fence carries {carried} paths and leaves {} to Git",
            git_writes.len()
        )
        .as_str(),
    );
    return Ok(Fence {
        git_writes,
        carried,
    });
}

/// What:
///  The first of `changed` the whole fence hands to Git,
///  in the source's index order,
///  or nothing when every changed path is one the clones may carry.
/// Why:
///   The snapshot rung decides both halves of the fence before its snapshot,
/// so `git_writes` is the exact set of paths its index installs without stat data,
///  which is the set `git read-tree -m -u` refuses when the target tree changes one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstWrittenChange = (fence, changed) => fence.gitWrites.find((path) => changed.has(path));
/// ```
pub fn first_written_change(
    fence: &Fence,
    changed: &[Vec<u8>],
) -> Option<ChangeNotServed> {
    if changed.is_empty() {
        return None;
    }
    let changed_paths: HashSet<&[u8]> = changed.iter().map(|path| return path.as_slice()).collect();
    for path in &fence.git_writes {
        if changed_paths.contains(path.as_slice()) {
            return Some(ChangeNotServed {
                path: path.clone(),
                why: WhyNotServed::TheFenceWritesIt,
            });
        }
    }
    return None;
}

/// What:
///  Both halves of the fence,
/// one after the other,
/// for a rung that decides it before its copy (rung 1's snapshot).
/// Why:
///   The snapshot's index is installed by the snapshot itself,
/// so the written paths must be known before it runs,
///  and a snapshot rung copies everything anyway:
/// there is no second thread for the reads to run beside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fence = decideThePaths(source, context, fenceTheAttributes(source, context));
/// ```
pub fn fence_the_clone(
    source: &SnapshotSource,
    context: &GitContext,
) -> Result<Fence, String> {
    return decide_the_paths(source, context, fence_the_attributes(source, context)?);
}

/// What:
///  One path and the rule that makes a clone of its source file unprovable,
/// as a log line.
/// Why:
///   The reason names the rule,
/// so a report can be checked against `convert.c` without a second run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = (path, reason) => `${path}: ${reason}`;
/// ```
fn fence_sentence(
    path: &[u8],
    reason: &str,
) -> String {
    return format!("{}: {reason}", String::from_utf8_lossy(path));
}

#[cfg(test)]
#[path = "worktree_clone_conversion_fence_tests.rs"]
mod tests;
