//! What:
//!  What Git's own two-way update left at each path a merge changes,
//!  as the file system named it at the instant the checkout moved:
//!  the entry the path holds,
//!  and that entry's change time;
//!  and whether a path still holds exactly that entry.
//! Why:
//!  A carry decides from a path's bytes whether the file standing there is the merge's own,
//!  and a person's save whose bytes
//!  accept every line the merge brought answers yes,
//! so their newer decision was replaced with the older layer the merge captured
//!  (a cross-vendor review of the second round,
//! its second finding).
//!  The entry Git wrote is the only thing that tells the two apart:
//! the device and the inode name the file,
//!  and its size and modification time say whether it was written again.
//!  The change time is asked too,
//! because a write that keeps the length and puts the modification time back still changes it,
//! and a program that saves that way is not this merge's own step.
//!  A change time is not part of the entry's identity,
//! because a rename changes it:
//! every take compares the identity it read
//!  before the rename with the file afterwards,
//! so a change time there would make every take refuse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landed = landedOf(root, paths); const still = theFileGitLanded(landed, landedFileAt(location, path));
//! ```

use crate::merge_isolation_changes::{FileIdentity, file_identity, worktree_path};
use std::path::{Path, PathBuf};

/// One worktree path as the file system named it when Git's update finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedFile = { path: Uint8Array; identity?: FileIdentity; changedSeconds: bigint; changedNanos: bigint };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedFile {
    /// The path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// The entry that stood there,
    /// or nothing when nothing did.
    pub identity: Option<FileIdentity>,
    /// The change time of that entry,
    /// whole seconds since the epoch.
    pub changed_seconds: i64,
    /// The change time's nanoseconds part.
    pub changed_nanos: i64,
}

/// The change time of one entry,
/// as the platform keeps it.
#[cfg(unix)]
fn changed_time_of(found: &std::fs::Metadata) -> (i64, i64) {
    use std::os::unix::fs::MetadataExt;
    return (found.ctime(), found.ctime_nsec());
}

/// The change time of one entry on a platform that keeps none.
#[cfg(not(unix))]
fn changed_time_of(found: &std::fs::Metadata) -> (i64, i64) {
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    let _ = found;
    return (0, 0);
}

/// What:
///  The entry at `location`,
///  named as the repository path `path`,
///  with the one thing a rename changes and a write cannot put back.
/// Why:
///   The identity and the change time are one read of the file system,
/// so the two comparisons that decide ownership cannot see two different files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function landedFileAt(location: string, path: Uint8Array): LandedFile; // lstat, no link followed
/// ```
pub fn landed_file_at(location: &Path, path: &[u8]) -> LandedFile {
    let found: Option<std::fs::Metadata> = std::fs::symlink_metadata(location).ok();
    let changed: (i64, i64) = found.as_ref().map(changed_time_of).unwrap_or_default();
    return LandedFile { path: path.to_vec(), identity: file_identity(location), changed_seconds: changed.0, changed_nanos: changed.1 };
}

/// What:
///  Read what Git's update left at each of these paths.
/// Why:
///   The merge's apply reads this once the checkout has moved and before the carry after it,
/// so a save that arrives in between is a file the
///  carry did not write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function landedOf(root: string, paths: Uint8Array[]): LandedFile[];
/// ```
pub fn landed_of(root: &Path, paths: &[Vec<u8>]) -> Vec<LandedFile> {
    let mut landed: Vec<LandedFile> = Vec::with_capacity(paths.len());
    for path in paths {
        let location: PathBuf = worktree_path(root, path.as_slice());
        landed.push(landed_file_at(location.as_path(), path.as_slice()));
    }
    return landed;
}

/// What:
///  Whether the file standing at a path is still the one Git's own update wrote there.
/// Why:
///   The identity is the device,
/// the inode,
/// the size and the modification time of the entry the file system hands out,
/// and the
///  change time says whether it was written again at all,
/// so a save from an editor is a different entry even when it accepts every
///  line the merge brought.
/// A path the landed reading does not name was read by nothing,
/// so its bytes are all the evidence
///  there is,
/// which is what a recovery of a killed merge has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theFileGitLanded = (landed: LandedFile[], standing: LandedFile): boolean => landed.find((left) => sameBytes(left.path, standing.path)) === undefined ? true : deepEqual(left, standing);
/// ```
pub fn the_file_git_landed(landed: &[LandedFile], standing: &LandedFile) -> bool {
    for left in landed {
        if left.path == standing.path {
            return left == standing;
        }
    }
    return true;
}
