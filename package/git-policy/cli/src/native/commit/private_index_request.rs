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

use crate::transaction_git::{GitContext, GitRequest};
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
///  keeping none of the invocation's global options as the incumbent does.
/// Why:
///   `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` select the private state,
///  and the objects belong to the invocation's repository:
///  a Git directory beside its work tree,
///  or one an environment variable sent elsewhere,
///  is named here because this request keeps none of the caller's other options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args, repository: selection({ cwd, globalPrefix }) })
/// ```
pub fn indexed_request<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    arguments: &[S],
    index: Option<&Path>,
    object_directory: Option<&Path>,
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    request.index = index.map(Path::to_path_buf);
    request.object_directory = object_directory.map(Path::to_path_buf);
    return request;
}

/// What:
///  A private-index request whose pathspecs are the caller's own,
/// so every pathspec-mode option of the invocation reaches Git with them,
///  and whose messages are English,
/// so the wrapper can read the one sentence it decides from.
/// Why:
///   `indexed_request` ignores the global prefix,
/// which is right for a pathspec the wrapper wrote and wrong for one the caller wrote:
///  `--noglob-pathspecs commit -- 'st*r.txt'` would then read the pathspec under the default glob mode and stage a
///  never-added `stxr.txt`,
///  and `--icase-pathspecs commit -- a.txt` would miss a staged `A.TXT`.
///  The modes the caller set as variables are already inherited;
/// this adds the ones written as options,
/// before the subcommand where Git reads them.
///  Git words its refusal of a pathspec in the language the person's locale asks for,
/// and `commit_pathspec_matching_nothing.rs` knows the English ones,
///  so every start of this kind runs under the C locale:
/// the reading then holds whatever language the person's shell asks for,
///  where inheriting the locale turned a German misspelling into an engine failure (issue 6283).
///  Only the message language is pinned,
/// and only for these starts:
/// the interactive selection keeps the person's own language,
/// because the terminal is the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callerPathspecRequest = (cwd, modes, arguments, index, objects) => ({ ...indexedRequest(cwd, [...modes, ...arguments], index, objects), environment: [['LC_ALL', 'C']] });
/// ```
pub fn caller_pathspec_request(
    context: &GitContext,
    cwd: &Path,
    arguments: &[std::ffi::OsString],
    index: Option<&Path>,
    object_directory: Option<&Path>,
) -> GitRequest {
    let mut words: Vec<std::ffi::OsString> = crate::global_arguments_pathspec_modes::pathspec_mode_options(context.global_prefix.as_slice());
    words.extend(arguments.iter().cloned());
    let mut request: GitRequest = indexed_request(context, cwd, words.as_slice(), index, object_directory);
    // `LC_ALL` wins over `LANG`, `LC_MESSAGES` and `LANGUAGE` in the C library's own lookup, so the one variable is enough.
    request.environment.push((std::ffi::OsString::from("LC_ALL"), std::ffi::OsString::from("C")));
    return request;
}

/// Request controls stay out of the release executable.
#[cfg(test)]
#[path = "private_index_request_tests.rs"]
mod tests;
