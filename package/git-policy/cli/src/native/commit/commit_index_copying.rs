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
/// The request built against a private index.
use crate::private_index_request::indexed_request;
/// The failure a step of the commit reports.
use crate::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
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
    /// so no early return may skip them,
    /// and these are the paths they name,
    /// which a retry writes as the requested tree's own entries.
    CopyWithIntentToAdd(Vec<Vec<u8>>),
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
        IndexFlags::IntentToAdd(paths) => PrivateIndexSeed::CopyWithIntentToAdd(paths),
        IndexFlags::Marked | IndexFlags::Unreadable => PrivateIndexSeed::Base,
    };
}

/// What:
///  Writes the requested tree's own entry over each given intent-to-add entry,
///  so the index holds that tree and not a mark that leaves the path out.
/// Why:
///   A reset leaves an intent-to-add mark whose mode and object name match the tree's entry,
/// and a tree built from an index holding such a mark leaves that path out:
///  a scratch-commit hook that restages an unrelated file invalidates the cached tree,
///  and a tracked file nobody named would then become a deletion in the tree the index writes.
///  `update-index --index-info` reads these names as index paths,
///  so a name relative to the repository root keeps that meaning under an invocation started in a subdirectory,
///  and the record names the object with the repository's own hash length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git update-index -z --index-info <<< paths.map((path) => `0 ${'0'.repeat(oidLength)} 0\t${path}\0`).join('')
/// ```
pub fn drop_intent_to_add(
    context: &GitContext,
    cwd: &Path,
    private_index: &Path,
    object_directory: &Path,
    paths: &[Vec<u8>],
    oid_length: usize,
) -> Result<(), GitFailure> {
    let zero: Vec<u8> = vec![b'0'; oid_length];
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(b"0 ");
        input.extend_from_slice(zero.as_slice());
        input.extend_from_slice(b" 0\t");
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let mut request: GitRequest = indexed_request(
        cwd,
        &["update-index", "-z", "--index-info"],
        Some(private_index),
        Some(object_directory),
    );
    request.input = Some(input);
    run_git_checked(context, &request)?;
    return Ok(());
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
