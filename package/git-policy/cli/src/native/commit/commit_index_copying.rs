//! What:
//!  The copying of the real index into a private index:
//! a copy that keeps the source's timestamps,
//! the decision of
//!  whether the captured index may seed an explicit-path commit,
//! and the failure that reports a copy which failed.
//! Why:
//!  `commit_index.rs` initializes the private commit index for each selection mode and stays under the file-size
//!  limit;
//! the copy steps need nothing from it,
//! so they live in a file of their own that it imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyIndexFile({ sourcePath, destinationPath });
//! ```

/// What marks the captured index carries.
use crate::index_flags::{IndexFlags, read_index_flags};
/// The failure a step of the commit reports.
use crate::transaction_git::GitFailure;
/// `FileTimes` carries the copied timestamps.
use std::fs::FileTimes;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Copies an index and gives the copy the source's access and modification times.
/// Why:
///   A copy with fresh times would make Git trust stat data captured in the same second
///  as a later same-size edit (issue #544).
///  A missing source is an error,
/// as the incumbent's `copyFile` reports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile({ sourcePath, destinationPath });
/// ```
pub fn copy_index_with_timestamps(source: &Path, destination: &Path) -> std::io::Result<()> {
    // The times come from the open file, not from its path, so a rename of another index over the path between the
    // read and the stat cannot give this content that file's times.
    let mut opened: std::fs::File = std::fs::File::open(source)?;
    let metadata: std::fs::Metadata = opened.metadata()?;
    let times: FileTimes = FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    let mut copy: std::fs::File = std::fs::File::create(destination)?;
    std::io::copy(&mut opened, &mut copy)?;
    copy.set_permissions(metadata.permissions())?;
    return copy.set_times(times);
}

/// What:
///  How the private index of a commit is built from the captured real index.
/// Why:
///   Two marks stop the copy and one does not:
///  an assume-unchanged or skip-worktree entry makes `git add --all` skip or refuse a file the commit named,
///  where native Git stages it,
///  while an intent-to-add entry carries no stat data and copies as it is.
///  The copy is then reset to the base as native Git's own partial commit does:
/// the reset leaves a mark whose mode and object name match the base's entry,
///  replaces one that differs,
///  and drops one the base does not hold,
/// so the marks decide nothing here,
///  the commit keeps the real index's cached stat data,
///  and the reset and the staging that follow it produce Git's tree
/// (`commit_index.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateIndexSeed = 'copy' | 'copy-with-intent-to-add' | 'base';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivateIndexSeed {
    /// Copy the real index as it is.
    Copy,
    /// Copy the real index as it is,
    /// marks included:
    /// its intent-to-add entries are selections a later step stages,
    /// so no early return may skip them.
    CopyWithIntentToAdd,
    /// Build the private index from the base alone.
    Base,
}

/// What:
///  Whether a build seeded this way keeps the cached stat data of the entries its tree shares with the real index.
/// Why:
///  The commit's timing record states this fact,
/// and only a build that starts from a copy keeps any.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// seed !== 'base'
/// ```
impl PrivateIndexSeed {
    /// The answer the timing record carries for a build seeded this way.
    pub fn keeps_stat_data(&self) -> bool {
        return !matches!(self, PrivateIndexSeed::Base);
    }
}

/// What:
///  Which way the captured real index may seed the private index of an explicit-path commit.
/// Why:
///   Only an index whose every entry was read and carries no assume-unchanged or skip-worktree mark may seed it,
/// and one that cannot be read is treated like a marked one.
///  An index whose only marks are intent-to-add entries seeds it too,
///  and is copied as it is:
/// its marks carry no stat data to lose,
///  and the reset and staging that follow produce native Git's tree for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readIndexFlags(await readFile(capturedIndex).catch(() => undefined))
/// ```
pub fn private_index_seed(captured_index: &Path) -> PrivateIndexSeed {
    let Ok(bytes) = std::fs::read(captured_index) else {
        return PrivateIndexSeed::Base;
    };
    return match read_index_flags(bytes.as_slice()) {
        IndexFlags::Plain => PrivateIndexSeed::Copy,
        IndexFlags::IntentToAdd(_paths) => PrivateIndexSeed::CopyWithIntentToAdd,
        IndexFlags::Marked | IndexFlags::Unreadable => PrivateIndexSeed::Base,
    };
}

/// What:
///  The failure that reports an index file which could not be copied.
/// Why:
///   Three copies of `initialize_commit_index` report the same failure,
/// and `map_err` takes this function by path
///  where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copyIndexFailure = (error: Error) => new GitFailure(`copying the index failed: ${error}`);
/// ```
pub fn copy_index_failure(error: std::io::Error) -> GitFailure {
    return GitFailure(format!("copying the index failed: {error}"));
}
