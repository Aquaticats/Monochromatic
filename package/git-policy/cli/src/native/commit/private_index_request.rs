//! What:
//!  The private index and object store a commit's preparation reads and writes, and the Git request that selects them.
//! Why:
//!  Staging and listing read the private commit index, and only staging writes objects, into the scratch object store;
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

/// Request controls stay out of the release executable.
#[cfg(test)]
#[path = "private_index_request_tests.rs"]
mod tests;
