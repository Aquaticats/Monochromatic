//! What:
//!  The three limit rules of the clone rung:
//! a source with changes whose tracked files sum below a whole-tree size threshold takes Git's own checkout,
//! a clean source on ZFS whose tracked files average too few bytes a file takes it too,
//! and so does a
//!  source with more changed files than a cut derived from its tracked path count.
//! Why:
//!  A clone of a tree of small files costs more than Git's write of the same files,
//! because one clone a
//!  file costs more than Git's one write of it,
//! and a source with many changed paths pays the write step per path,
//! so the
//!  rung must decline both shapes,
//! or a clone would be slower than Git's own write of the same files.
//! A clean source is judged by its mean file size rather than its total,
//! because a clean tree of a few
//!  large files clones far quicker than Git's write of them
//! (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
//!  "The size limit on a clean source").
//! The mean rule has no value on btrfs:
//! no btrfs mean was measured to lose,
//! and the project answered "Clone as main does on btrfs;
//! keep the rule on ZFS and on filesystems nobody measured"
//! (the same record,
//! "No size rule on btrfs").
//! Every limit sends the add to Git's own checkout,
//! and a declined add costs what `main` paid for the same source:
//! the source check and then Git's own checkout,
//! never less and never much more.
//! A clean tree this rung declines measured 1.07 to 1.24 times Git's own checkout in the recheck's guest,
//! the source check and,
//! on some,
//! a Git call inside the ladder;
//! a dirty tree it declines measured 0.94 to 1.04 of it,
//! inside the round's spread
//! (`~/temp/agent/integration-recheck-clone-rung-partial-clone-2026-11-22/keep/review.md`,
//! "RM2" and "The owner's requirement per shape").
//! What a declined add loses is the clone's gain.
//! Every value is
//!  provisional and open to the project's veto:
//! the decision record accepted the proposal's option "clone with limits",
//! and left the values to this build,
//! to be pinned at a measured edge with the measurement beside each
//!  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
//!  "Left to the build").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = storedStats(source.indexBytes, source.hashLength);
//! const reason = entries && (source.dirty ? tooSmallATree(entries) : tooSmallAFilesMean(entries) ?? tooManyChangedPaths(source, entries));
//! ```

use crate::fs_kind::{FilesystemKind, filesystem_kind};
use crate::worktree_clean::{StatEntry, stored_stats};
use crate::worktree_snapshot_source::SnapshotSource;
use std::path::Path;

/// The fewest tracked bytes a source with changes must hold for the clone rung to serve its add.
///
/// Provenance:
/// pinned at the winning edge of the measured bracket.
/// On the fenced rung,
/// in position-balanced rounds against Git's own checkout with an identical-binary control,
/// a dirty source lost at every size below this repository's tree:
/// 1.30 at 10,001 files of 2,600 bytes (about 26,000,028 bytes),
/// and 1.32 (1.03 to 1.71,
/// identical control 1.40,
/// 8 counted rounds) at 10,001 files of 26,000 bytes (260,000,028 bytes,
/// measured 2026-11-21 for this pin);
/// this repository's own tree of 24,621 files and 763,289,228 bytes,
/// the tree that round measured,
/// won at 0.73 to 0.81 of it.
/// The rung's fence reads cost about as much whatever the tree holds
/// (`git ls-tree` alone is 275 ms at 24,621 paths,
/// 110 ms at 10,001),
/// while Git's own checkout grows with the bytes,
/// so the rung loses wherever Git's checkout is shorter than those reads.
/// No size between 260 MB and this tree was measured a win,
/// so the pin sits at the top of the bracket,
/// just under the tree that won,
/// and no unmeasured size is cloned.
///
/// The limit applies only to a source with changes,
/// where every measurement above was taken.
/// A clean source is judged by `the_smallest_mean_tracked_file`,
/// so a clean tree of a few large files clones whatever its total.
///
/// Provisional and open to the project's veto,
/// and to a lower value once a size below this tree is measured a win.
pub const THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG: u64 = 762_000_000;

/// The smallest mean tracked file size a clean source on ZFS may have and still be cloned.
///
/// Provenance:
/// pinned just above the only mean whose copy lost clearly in the pooled rounds.
/// The points are the recheck's pooling of this round's ZFS rounds
/// (`~/temp/agent/integration-recheck-clone-rung-partial-clone-2026-11-22/keep/`,
/// `analysis-edges.txt` and the review's "The measured points"),
/// each the clone's time over Git's own checkout in the same round,
/// median with the counted rounds above 1.0 in brackets:
/// 2,599 1.23 (11 of 12);
/// 3,599 1.03 (4 of 8);
/// 4,399 0.71 (2 of 8);
/// 5,199 1.23 (6 of 8);
/// 5,999 0.80 (2 of 8);
/// 6,999 0.80 (3 of 8);
/// 7,999 1.21 (10 of 12);
/// 8,799 1.02 (9 of 12);
/// 9,599 0.85 (4 of 12);
/// 10,998 0.60 (2 of 8);
/// 13,198 0.71;
/// 17,598 0.60;
/// 21,997 0.64;
/// 25,997 0.55;
/// 28,997 0.48;
/// 31,996 0.50;
/// 43,995 0.97 (3 of 8).
/// The means are not monotone:
/// 5,199 and 7,999 read 1.21 to 1.23 while 4,399,
/// 5,999 and 6,999 read 0.71 to 0.80,
/// so the data place the crossing only within a range,
/// about 3,000 to 9,600 bytes a file,
/// and the identical-binary control's own spread (0.26 to 3.15 a round) is wider than the differences in that range.
/// 2,599 is the one mean with a clear pooled loss,
/// and the project answered "lower it to about 3,000 bytes a file ... decline only below,
/// where the clone clearly
/// loses" (the decision record,
/// "The ZFS edge is lowered to about 3,000 bytes a file"),
/// so the edge sits at 2,600,
/// the smallest whole number of bytes above that mean.
/// It is a new pin on the same noisy points:
/// the sizes the project's answer names as costs,
/// 5,199 and 7,999,
/// clone
/// again,
/// and their own single-mean readings above say the clone may lose there.
///
/// Provisional and open to the project's veto.
pub const THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS: u64 = 2_600;

/// What:
///  The mean edge of one filesystem,
/// or nothing for a filesystem whose clean sources are cloned whatever their mean.
/// Why:
///   The measured filesystems differ:
/// ZFS's copy of a tree of small files costs more than Git's write of it,
/// while no btrfs mean measured lost,
/// and the pin of 49 bytes made a clean btrfs tree under it 1.36 to 1.62 times `main` in the recheck's guest
/// (its finding RH1,
/// and the project's answer "No size rule on btrfs").
/// A filesystem nobody measured takes the safest measured edge,
/// which is the ZFS one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const edge = kind === 'btrfs' ? undefined : kind === 'zfs' ? ZFS : safest();
/// ```
pub fn the_smallest_mean_tracked_file(kind: FilesystemKind) -> Option<u64> {
    return match kind {
        // The project answered "Clone as main does on btrfs; keep the rule on ZFS and on filesystems nobody measured"
        // (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`, "No size rule on btrfs").
        FilesystemKind::Btrfs => None,
        FilesystemKind::Zfs => Some(THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS),
        FilesystemKind::Overlayfs | FilesystemKind::Other => Some(the_safest_mean_tracked_file()),
    };
}

/// What:
///  The edge a filesystem nobody measured takes.
/// Why:
///   XFS with reflink and overlayfs were not measured,
/// and the ZFS edge is the only measured edge that still applies to a clean source now that btrfs has none,
/// so they take it:
/// a filesystem whose copy of small files costs more than Git's write is declined,
/// as on ZFS.
/// The alternative,
/// no rule on an unmeasured filesystem,
/// is put to the project in `SPEC.md` and is not decided.
/// A fixture that must reach the clone rung on any of these sizes its anchor from this edge,
/// so no control carries a per-filesystem number of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const safest = THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS;
/// ```
pub fn the_safest_mean_tracked_file() -> u64 {
    return THE_SMALLEST_MEAN_TRACKED_FILE_ON_ZFS;
}

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
///  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
///  "The measurements it rests on").
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
///  The source's index entries with their recorded stat data,
/// read once for every limit and for the rung itself.
/// Why:
///   The limits and the copies all read the same entries,
/// and each parse of this repository's 24,648-entry index costs about 2 ms,
/// so one read is shared rather than repeated
/// (the integration pass's low L1).
/// An index this reader cannot read is nothing here,
/// and the rung refuses it later with its own reason.
///
/// The index records a tracked file's size in 32 bits,
/// so a file of 4 GiB or more counts modulo 2^32,
/// exactly as Git's own index records it,
/// and a tree dominated by such files can read as small.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = storedStats(source.clean.indexBytes, source.clean.hashLength) ?? undefined;
/// ```
pub fn tracked_entries(source: &SnapshotSource) -> Option<Vec<StatEntry>> {
    return stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length);
}

/// What:
///  The bytes the source's index records for its tracked files.
/// Why:
///   The size limits sum the sizes the index already holds,
/// so they cost no Git run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const total = entries.reduce((sum, entry) => sum + entry.stored.size, 0);
/// ```
pub fn tracked_bytes(entries: &[StatEntry]) -> u64 {
    let total: u64 = entries.iter().map(|entry| { return u64::from(entry.stored.size); }).sum();
    return total;
}

/// What:
///  The mean tracked file size the index records,
/// or nothing for a tree of no tracked paths.
/// Why:
///   A clean source is declined or cloned by this mean,
/// and an empty tree is a state the source check has already refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mean = Math.floor(total / entries.length);
/// ```
pub fn mean_tracked_file(entries: &[StatEntry]) -> Option<u64> {
    if entries.is_empty() {
        return None;
    }
    return Some(tracked_bytes(entries) / entries.len() as u64);
}

/// What:
///  The reason the whole-tree size limit sends a source with changes to Git's own checkout,
/// in words,
/// or nothing when the rung may clone it.
/// Why:
///   A declined add costs what `main` paid for the same source:
/// the source check and then Git's own checkout,
/// so the person is owed the numbers the decision was made on rather than a bare refusal
/// (`AGENTS.md`,
/// rule `DGT`).
/// The words name the input and what happens,
/// and the route's own lead says who wrote the worktree
/// (the integration pass's low L4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = total < THE_FEWEST_TRACKED_BYTES ? `...` : undefined;
/// ```
pub fn reason_the_tree_is_too_small(source: &SnapshotSource, entries: &[StatEntry]) -> Option<String> {
    let total: u64 = tracked_bytes(entries);
    if total >= THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG {
        return None;
    }
    return Some(format!("the tracked files of {} sum to {} bytes, below the {} a tree with changes needs for the clone to be quicker than Git's own checkout", shown(source.root.as_path()), total, THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG));
}

/// What:
///  The reason the mean-file limit sends a clean source to Git's own checkout,
/// in words,
/// or nothing when the rung may clone it.
/// Why:
///   A clean tree of many small files costs more to clone than to write through Git,
/// and a clean tree of few large files costs far less,
/// so the mean file size decides rather than the total
/// (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
/// "The size limit on a clean source").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = mean < THE_SMALLEST_MEAN ? `...` : undefined;
/// ```
pub fn reason_the_mean_file_is_too_small(source: &SnapshotSource, entries: &[StatEntry]) -> Option<String> {
    let mean: u64 = mean_tracked_file(entries)?;
    // An unreadable filesystem takes the safest measured edge, as an unmeasured one does;
    // btrfs has none, and a clean btrfs source is cloned whatever its mean.
    let edge: u64 = the_smallest_mean_tracked_file(filesystem_kind(source.root.as_path()).unwrap_or(FilesystemKind::Other))?;
    if mean >= edge {
        return None;
    }
    return Some(format!("the tracked files of {} average {} bytes each, below the {} a clean tree needs for the clone to be quicker than Git's own checkout", shown(source.root.as_path()), mean, edge));
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
pub fn reason_too_many_changed_paths(source: &SnapshotSource, entries: &[StatEntry]) -> Option<String> {
    let tracked: usize = entries.len();
    let changed: usize = source.dirty.as_ref().map_or(0, |changes| { return changes.tracked; });
    let cut: usize = the_changed_file_cut(tracked);
    if changed <= cut {
        return None;
    }
    return Some(format!("{} of the {} tracked files of {} have changes, more than the {} whose changes the clone copies and lets Git write the rest of", changed, tracked, shown(source.root.as_path()), cut));
}

/// What:
///  The reason the clone rung declines this source for its size or its changes,
/// in words,
/// or nothing when the rung may clone it.
/// Why:
///   The limit is chosen by what the source is:
/// a source with changes is judged by its whole-tree size,
/// where every measurement of that limit was taken,
/// and a clean source by its mean file size;
/// the changed-file cut applies to both.
/// Both limits are asked before the rung's fence,
/// its claim on a stage name and its first copy,
/// so a declined add pays none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const size = source.dirty ? tooSmallATree(entries) : tooSmallAFilesMean(entries);
/// const reason = size ?? tooManyChangedPaths(source, entries);
/// ```
pub fn decline_before_cloning(source: &SnapshotSource, entries: &[StatEntry]) -> Option<String> {
    let by_size: Option<String> = match source.dirty.is_none() {
        true => reason_the_mean_file_is_too_small(source, entries),
        false => reason_the_tree_is_too_small(source, entries),
    };
    return by_size.or_else(|| { return reason_too_many_changed_paths(source, entries); });
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
