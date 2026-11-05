//! What:
//!  The copying of the real index into a private index: a copy that keeps the source's timestamps, the check of
//!  whether the captured index may seed an explicit-path commit, and the failure that reports a copy which failed.
//! Why:
//!  `commit_index.rs` initializes the private commit index for each selection mode and stays under the file-size
//!  limit; the copy steps need nothing from it, so they live in a file of their own that it imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyIndexFile({ sourcePath, destinationPath });
//! ```

/// The failure a step of the commit reports.
use crate::transaction_git::GitFailure;
/// What marks the captured index carries.
use crate::index_flags::{IndexFlags, read_index_flags};
/// `FileTimes` carries the copied timestamps.
use std::fs::FileTimes;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Copies an index and gives the copy the source's access and modification times.
/// Why:
///   A copy with fresh times would make Git trust stat data captured in the same second
///  as a later same-size edit (issue #544).
///  A missing source is an error, as the incumbent's `copyFile` reports it.
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
///  Whether the captured real index may seed the private index of an explicit-path commit.
/// Why:
///   Only an index whose every entry was read and carries no assume-unchanged or skip-worktree mark may;
///  one that cannot be read is treated like a marked one, and the commit builds its index from the base alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readIndexFlags(await readFile(capturedIndex).catch(() => undefined)) === 'plain'
/// ```
pub fn seeds_private_index(captured_index: &Path) -> bool {
    let Ok(bytes) = std::fs::read(captured_index) else {
        return false;
    };
    return read_index_flags(bytes.as_slice()) == IndexFlags::Plain;
}

/// What:
///  The failure that reports an index file which could not be copied.
/// Why:
///   Three copies of `initialize_commit_index` report the same failure, and `map_err` takes this function by path
///  where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copyIndexFailure = (error: Error) => new GitFailure(`copying the index failed: ${error}`);
/// ```
pub fn copy_index_failure(error: std::io::Error) -> GitFailure {
    return GitFailure(format!("copying the index failed: {error}"));
}
