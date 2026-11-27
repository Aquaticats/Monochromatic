//! What:
//!  What the repair reads about one path:
//!  the entries in which a commit's own index differs from the live index,
//!  and the object the worktree holds at a path.
//! Why:
//!   A writer replaces the index file whole,
//!  so the copy the commit made may be older than the live index,
//!  and a path whose entry moved there is left out;
//!  and the worktree is what tells the person's own staging from the landing's.
//!  Neither question changes anything,
//!  so they are the reads the repair decides on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const moved = movedSince(liveIndex, commitIndex);
//! const held = worktreeObject(repository, name);
//! ```

use crate::owed_index_entries::names_in;
use crate::service_protocol::FailureCode;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::Path;

/// What:
///  The paths in which a commit's own index differs from the live index.
/// Why:
///   A writer replaces the index file whole,
/// so a live index that is another file than the one the commit copied may
///  hold entries the commit's copy does not:
/// the paths that differ are where the copy is older.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// movedSince(liveIndex, commitIndex) // `git write-tree` then `diff-index --cached <tree>` on the copy
/// ```
pub(super) fn moved_since(
    git: &GitContext,
    repository: &Path,
    live_index: &Path,
    commit_index: &Path,
) -> Result<Vec<Vec<u8>>, FailureCode> {
    let mut tree: GitRequest = GitRequest::new(repository, &["write-tree"]);
    tree.index = Some(live_index.to_path_buf());
    let written = match run_git(git, &tree) {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(FailureCode::Unavailable),
    };
    let held: String = String::from_utf8_lossy(written.stdout.as_slice())
        .trim()
        .to_string();
    let mut moved: GitRequest = GitRequest::new(
        repository,
        &["diff-index", "--cached", "--name-only", "-z", held.as_str()],
    );
    moved.index = Some(commit_index.to_path_buf());
    let listing = match run_git(git, &moved) {
        Ok(output) if output.succeeded() => output,
        _ => return Err(FailureCode::Unavailable),
    };
    return Ok(names_in(listing.stdout.as_slice()));
}

/// What:
///  The object the worktree holds at one path,
/// or nothing when it holds no file there.
/// Why:
///   A worktree that still holds the entry the index holds means the person staged what they have,
/// so the
///  entry is their own statement and nothing may be changed there;
/// a worktree that holds something
///  else is an index that no longer describes it,
/// which is what a failed index update leaves.
/// The object is
///  asked of the file itself,
/// so no stat cache can answer for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeObject(name) // `git hash-object -- <path>`
/// ```
pub(super) fn worktree_object(
    git: &GitContext,
    repository: &Path,
    name: &[u8],
) -> Result<Option<(String, String)>, FailureCode> {
    let path: OsString = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.to_vec());
    let held: std::path::PathBuf = repository.join(path.as_os_str());
    // A file that is not there is absent, which is evidence; one that is there and cannot be read is no evidence at all.
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(held.as_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(FailureCode::Unavailable),
    };
    // Git stores a link as the text of its target, so that text is what is hashed: hashing through the link would read the
    //  referent (when it exists) or fail (when it does not), neither of which is the entry Git made.
    let linking: bool = metadata.file_type().is_symlink();
    let mut request: GitRequest = GitRequest::new(
        repository,
        &[OsStr::new("hash-object"), OsStr::new("--stdin")],
    );
    if linking {
        let target: std::path::PathBuf = match std::fs::read_link(held.as_path()) {
            Ok(target) => target,
            Err(_) => return Err(FailureCode::Unavailable),
        };
        use std::os::unix::ffi::OsStringExt as _;
        request.input = Some(target.into_os_string().into_vec());
    } else {
        request = GitRequest::new(
            repository,
            &[
                OsStr::new("hash-object"),
                OsStr::new("--"),
                path.as_os_str(),
            ],
        );
    }
    let object: String = match run_git(git, &request) {
        Ok(answer) if answer.succeeded() => String::from_utf8_lossy(answer.stdout.as_slice())
            .trim()
            .to_string(),
        _ => return Err(FailureCode::Unavailable),
    };
    #[cfg(unix)]
    let mode: String = {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.file_type().is_symlink() {
            String::from("120000")
        } else if metadata.permissions().mode() & 0o111 != 0 {
            String::from("100755")
        } else {
            String::from("100644")
        }
    };
    #[cfg(not(unix))]
    let mode: String = String::from("100644");
    return Ok(Some((mode, object)));
}
