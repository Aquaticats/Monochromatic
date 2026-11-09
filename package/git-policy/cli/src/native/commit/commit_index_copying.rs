//! What:
//!  The copying of the real index into a private index:
//! a copy that keeps the source's timestamps,
//! the decision of
//!  whether the captured index may seed an explicit-path commit and which entries the copy leaves out,
//!  the step that leaves the intent-to-add entries out,
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
///  while an intent-to-add entry holds the empty blob's name and no stat data at all,
///  and a tree Git builds from an index that holds one leaves that path out.
///  So the copy is made for an index whose only marks are intent-to-add entries,
///  and those entries are left out of it:
///  the commit keeps every other entry's cached stat data,
///  and loses nothing,
/// because an intent-to-add entry carries none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateIndexSeed = 'copy' | { leaveOut: Uint8Array[] } | 'base';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivateIndexSeed {
    /// Copy the real index as it is.
    Copy,
    /// Copy the real index and leave these intent-to-add entries out of the copy.
    CopyWithoutIntentToAdd(Vec<Vec<u8>>),
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
///  and names the entries the copy leaves out.
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
        IndexFlags::IntentToAdd(paths) => PrivateIndexSeed::CopyWithoutIntentToAdd(paths),
        IndexFlags::Marked | IndexFlags::Unreadable => PrivateIndexSeed::Base,
    };
}

/// What:
///  Removes the given entries from the private index,
///  so no intent-to-add entry is left in the copy.
/// Why:
///   A tree Git builds from an index that holds an intent-to-add entry leaves that path out,
///  so a tracked file the person marked with `git add -N` would vanish from a commit that names other paths,
///  where native Git commits the base's content for it.
///  Git writes the file itself,
///  so the copy keeps Git's own checksum and its bookkeeping of the extensions,
///  and a path with a newline or a byte that is not UTF-8 needs no quoting because the names are NUL-separated.
///  The step runs only for an index that holds such an entry,
///  so a commit pays no Git start for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git update-index --force-remove -z --stdin <<< paths.map((path) => path + '\0').join('')
/// ```
pub fn drop_intent_to_add(
    context: &GitContext,
    cwd: &Path,
    private_index: &Path,
    object_directory: &Path,
    paths: &[Vec<u8>],
) -> Result<(), GitFailure> {
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let mut request: GitRequest = indexed_request(
        cwd,
        &["update-index", "--force-remove", "-z", "--stdin"],
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
