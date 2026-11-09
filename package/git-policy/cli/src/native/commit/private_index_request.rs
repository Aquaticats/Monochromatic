//! What:
//!  The private index and object store a commit's preparation reads and writes,
//! and the Git request that selects them.
//! Why:
//!  Staging and listing read the private commit index,
//! and only staging writes objects,
//! into the scratch object store;
//!  `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` select that private state while the invocation's global options are ignored,
//!  as the incumbent does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args })
//! ```

use crate::transaction_git::GitRequest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  The index and object-store inputs of one listing.
/// Why:
///   Staging and listing read the private index,
///  and only staging writes objects into the scratch store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Workspace = Pick<CommitTransactionWorkspace, 'capturedIndexPath' | 'commitIndexPath' | 'objectDirectory'>;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureIndexes {
    /// Real index copied at invocation.
    pub captured_index: PathBuf,
    /// Private commit index.
    pub commit_index: PathBuf,
    /// Scratch object store.
    pub object_directory: PathBuf,
}

/// What:
///  A Git request that reads or writes one private index,
///  ignoring the invocation's global options as the incumbent does.
/// Why:
///   `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` select the private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args })
/// ```
pub fn indexed_request<S: AsRef<OsStr>>(
    cwd: &Path,
    arguments: &[S],
    index: Option<&Path>,
    object_directory: Option<&Path>,
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
    request.index = index.map(Path::to_path_buf);
    request.object_directory = object_directory.map(Path::to_path_buf);
    return request;
}

/// What:
///  A private-index request whose pathspecs are the caller's own,
/// so every pathspec-mode option of the invocation reaches Git with them.
/// Why:
///   `indexed_request` ignores the global prefix,
/// which is right for a pathspec the wrapper wrote and wrong for one the caller wrote:
///  `--noglob-pathspecs commit -- 'st*r.txt'` would then read the pathspec under the default glob mode and stage a
///  never-added `stxr.txt`,
///  and `--icase-pathspecs commit -- a.txt` would miss a staged `A.TXT`.
///  The modes the caller set as variables are already inherited;
/// this adds the ones written as options, before the subcommand where Git reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callerPathspecRequest = (cwd, modes, arguments, index, objects) => ({ ...indexedRequest(cwd, [...modes, ...arguments], index, objects) });
/// ```
pub fn caller_pathspec_request(
    context: &crate::transaction_git::GitContext,
    cwd: &Path,
    arguments: &[std::ffi::OsString],
    index: Option<&Path>,
    object_directory: Option<&Path>,
) -> GitRequest {
    let mut words: Vec<std::ffi::OsString> = crate::global_arguments::pathspec_mode_options(context.global_prefix.as_slice());
    words.extend(arguments.iter().cloned());
    return indexed_request(cwd, words.as_slice(), index, object_directory);
}

/// Request controls stay out of the release executable.
#[cfg(test)]
#[path = "private_index_request_tests.rs"]
mod tests;
