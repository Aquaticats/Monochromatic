//! What:
//!  The two provisional limits of the clone rung:
//! a source whose tracked files sum below a whole-tree size threshold takes Git's own checkout,
//! and so does a
//!  source with more changed files than a cut derived from its tracked path count.
//! Why:
//!  A clone of a tree of small files costs more than Git's write of the same files,
//! because one clone a
//!  file costs more than Git's one write of it,
//! and a source with many changed paths pays the write step per path,
//! so the
//!  rung must decline both shapes or an add would be slower than the checkout it replaced.
//! Both limits send the add to
//!  Git's own checkout,
//! which is the baseline the decision record measures against,
//! so a declined add is never made slower
//!  than Git's own checkout by the rung;
//! what a declined add loses is the clone's gain.
//! Both values are
//!  provisional and open to the project's veto:
//! the decision record accepted the proposal's option "clone with limits" and left the
//!  values to this build, to be pinned at a measured edge with the measurement beside each
//!  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`, "Left to the build").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason = reasonTheTreeIsTooSmall(source) ?? reasonTooManyChangedPaths(source); // undefined when the rung may clone
//! ```

use crate::worktree_clean::stored_stats;
use crate::worktree_snapshot_source::SnapshotSource;
use std::path::Path;

/// The fewest tracked bytes a source must hold for the clone rung to serve its add.
///
/// Provenance:
/// pinned at the losing edge of the measured bracket,
/// rounded up so the tree that lost is clearly below it.
/// On the fenced rung this repository's own tree
/// (763,309,482 bytes of tracked files)
/// cloned at 0.73 to 0.77 of Git's own checkout in four balanced rounds,
/// and the waiver's tree of 10,001 files of 2,600 bytes
/// (about 26,005,200 bytes) lost at 1.30 of Git's own checkout (1.44 of the same build of `main`).
/// Every value above 26 MB and up to 763 MB fits both measurements
/// (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
/// "The measurements it rests on").
///
/// Why this edge and not the top of the bracket:
/// the limit applies to a clean source as well,
/// and a clean tree above this value keeps the clone rung's win that `main` already has there
/// (`main` is 0.99
/// to 1.19 of Git's own checkout on the clean small trees it measured, and 0.64 on this repository's tree),
/// while a top-of-bracket value would hand every clean tree below 763 MB to Git's checkout and give that win up
/// (`AGENTS.md`, rule `QJ1`).
/// What this edge leaves unmeasured is a dirty source of small files between 32 MB and 763 MB,
/// which clones and may lose where `main` would have taken Git's own checkout;
/// no round has measured that shape.
///
/// Provisional and open to the project's veto,
/// and to a lower value once more sizes are measured.
pub const THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG: u64 = 32_000_000;

/// What:
///  The changed paths the clone rung still clones a source with:
/// one quarter of its tracked paths,
/// which is the share the conversion fence
///  already declines at.
/// Why:
///   Three rounds measured this quantity:
/// 6,000 changed paths of 24,620 cloned at 1.00 to 1.05 of Git's own checkout,
///  which is parity,
/// and 20,000 of 24,620 (81 per cent) lost at 1.68 to 1.74 of it.
/// A quarter of 24,620 is 6,155,
///  which keeps the measured parity point and declines the measured loss,
/// and it reuses the fence's own decline share rather than
///  pinning a second share nobody measured.
/// Every value from 6,000 to below 20,000 fits the measurements
///  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`, "The measurements it rests on").
///
/// Provisional and open to the project's veto,
/// and to a cut in changed paths once one is measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cut = Math.floor(tracked * (1 / DECLINE_SHARE_DIVISOR));
/// ```
pub fn the_changed_file_cut(tracked: usize) -> usize {
    return tracked / crate::worktree_clone_conversion_fence::DECLINE_SHARE_DIVISOR;
}

/// What:
///  The bytes the source's index records for its tracked files,
/// or nothing when the index has a shape this reader does not know.
/// Why:
///   The size limit sums the sizes the index already holds,
/// so it costs one read of bytes the rung has in hand and no Git run,
///  and an index this reader cannot read is a state the source check has already refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const total = storedStats(indexBytes, hashLength)?.reduce((sum, entry) => sum + entry.stored.size, 0);
/// ```
pub fn tracked_bytes(source: &SnapshotSource) -> Option<u64> {
    let entries = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length)?;
    let total: u64 = entries.iter().map(|entry| { return u64::from(entry.stored.size); }).sum();
    return Some(total);
}

/// What:
///  The reason the whole-tree size limit sends this source to Git's own checkout,
/// in words,
/// or nothing when the rung may clone it.
/// Why:
///   A declined add is Git's own checkout,
/// so the person is owed the numbers the decision was made on rather than a bare refusal
/// (`AGENTS.md`, rule `DGT`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = total < THE_FEWEST_TRACKED_BYTES ? `...` : undefined;
/// ```
pub fn reason_the_tree_is_too_small(source: &SnapshotSource) -> Option<String> {
    let total: u64 = tracked_bytes(source)?;
    if total >= THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG {
        return None;
    }
    return Some(format!("the tracked files of {} sum to {} bytes, below the {} this rung needs to clone a tree at all, so Git's checkout writes the worktree", shown(source.root.as_path()), total, THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG));
}

/// What:
///  The reason the changed-file cut sends this source to Git's own checkout,
/// in words,
/// or nothing when the rung may clone it.
/// Why:
///   The cut is a fraction of the tracked paths,
/// so the sentence must name both numbers and the cut it applied:
/// a person reading it
///  decides whether the next measurement should move the cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = changed > theChangedFileCut(tracked) ? `...` : undefined;
/// ```
pub fn reason_too_many_changed_paths(source: &SnapshotSource) -> Option<String> {
    let tracked: usize = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length)?.len();
    let changed: usize = source.dirty.as_ref().map_or(0, |changes| { return changes.tracked; });
    let cut: usize = the_changed_file_cut(tracked);
    if changed <= cut {
        return None;
    }
    return Some(format!("{} of the {} tracked files of {} have changes, more than the {} this rung clones and has Git write the rest of, so Git's checkout writes the worktree", changed, tracked, shown(source.root.as_path()), cut));
}

/// What:
///  The path as the text of a sentence,
/// with every control character replaced.
/// Why:
///   A path on a terminal or in the journal must not carry an escape sequence or start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/\p{Cc}/gu, '?')
/// ```
fn shown(path: &Path) -> String {
    return crate::worktree_snapshot_source::plain(path.display().to_string().as_str());
}

/// Controls for the two limits stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_rung_limits_tests.rs"]
mod tests;
