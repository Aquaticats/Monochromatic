//! What:
//!  The one Git start every private-index step makes,
//!  and the empty index a repository with no index file starts from.
//! Why:
//!  A private-index step runs Git against the private index and the shadow object store,
//!  never against the person's index,
//!  so one request shape is written once and every step asks through it.
//!  A repository nothing was ever staged in has no index file at all,
//!  and the empty index a commit needs there is written beside the private copies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args });
//! ```

/// One required Git start,
/// its request builder and the failure text.
use crate::private_index_request::indexed_request;
/// The transaction Git wrapper:
///  the context a step runs in,
///  the failure it reports,
///  the output it prints and the start that requires success.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    run_git_checked,
};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// What:
///  Runs one Git command against the private index and the shadow object store,
///  requiring success.
/// Why:
///   Every private-index step uses the same index and object directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args });
/// ```
pub(crate) fn private_git<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    object_directory: &Path,
    arguments: &[S],
) -> Result<GitOutput, GitFailure> {
    return run_git_checked(
        context,
        &indexed_request(context, cwd, arguments, Some(index), Some(object_directory)),
    );
}

/// The one start that writes an empty index,
/// for a repository whose index file does not exist yet.
/// It writes a private copy and never the person's index.
pub(crate) const EMPTY_INDEX: &[&str] = &["read-tree", "--empty"];
