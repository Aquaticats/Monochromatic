//! What:
//!  The writes the repair makes to the commit's own index:
//!  one path at a time,
//!  and the paths a landing could own together.
//! Why:
//!   The commit's own index describes the branch before the landing,
//!  so an entry left as it is would record the landed commit's own paths as their old content;
//!  setting those entries to the head is what makes the commit record what it is about,
//!  and nothing here writes the live index.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await setToHeadPaths(git, gitDir, commitIndex, paths);
//! ```

use crate::owed_index_entries::{
    Entry,
    head_entries,
    specs_of,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use std::collections::BTreeMap;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Sets an index to the worktree's head at `paths`,
/// removing whatever it held there and below them first,
/// and leaves the file exactly as it was when any step fails.
/// Why:
pub(super) fn set_to_head_with(
    git: &GitContext,
    repository: &Path,
    index: &Path,
    paths: &[Vec<u8>],
    _named: &Path,
) -> Result<(), FailureCode> {
    if paths.is_empty() {
        return Ok(());
    }
    let kept: PathBuf = index.with_extension("before-head");
    if let Err(error) = std::fs::copy(index, kept.as_path()) {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index cannot be copied aside before it is set to the head ({error})").as_str());
        return Err(FailureCode::Unavailable);
    }
    let restored: Result<(), FailureCode> = set_to_head_unchecked(git, repository, index, paths);
    if restored.is_err()
        && let Err(error) = std::fs::copy(kept.as_path(), index)
    {
        crate::diagnostic_log::warn(
            "servicePrepare",
            format!("the commit's own index could not be put back after a refused step ({error})")
                .as_str(),
        );
    }
    let _gone: std::io::Result<()> = std::fs::remove_file(kept.as_path());
    return restored;
}

/// The two steps of `set_to_head`,
/// each of which may refuse.
pub(super) fn set_to_head_unchecked(
    git: &GitContext,
    repository: &Path,
    index: &Path,
    paths: &[Vec<u8>],
) -> Result<(), FailureCode> {
    let specs: Vec<OsString> = specs_of(paths);
    let mut prune_words: Vec<&OsStr> = vec![
        OsStr::new("rm"),
        OsStr::new("--cached"),
        OsStr::new("-r"),
        OsStr::new("--force"),
        OsStr::new("--ignore-unmatch"),
        OsStr::new("--quiet"),
        OsStr::new("--"),
    ];
    prune_words.extend(specs.iter().map(OsString::as_os_str));
    let mut prune: GitRequest = GitRequest::new(repository, prune_words.as_slice()).own_pathspecs();
    prune.index = Some(index.to_path_buf());
    match run_git(git, &prune) {
        Ok(output) if output.succeeded() => {}
        _ => return Err(FailureCode::Unavailable),
    }
    if crate::service_land_follow::reset_entries(git, repository, Some(index), "HEAD", paths)
        .is_ok()
    {
        return Ok(());
    }
    // A `reset` this wrapper refuses is not the only way to the head: the entries are written where the head's tree has them.
    let head: BTreeMap<Vec<u8>, Entry> = head_entries(git, repository, paths)?;
    let mut listing: Vec<u8> = Vec::new();
    for path in paths {
        let (mode, object) = match head.get(path) {
            Some((mode, object)) => (mode.clone(), object.clone()),
            // The head holds nothing there, so the entry goes.
            None => (String::from("0"), String::from("0".repeat(40))),
        };
        listing.extend_from_slice(format!("{mode} {object} 0\t").as_bytes());
        listing.extend_from_slice(path.as_slice());
        listing.push(0);
    }
    let mut write: GitRequest =
        GitRequest::new(repository, &["update-index", "-z", "--index-info"]);
    write.index = Some(index.to_path_buf());
    write.input = Some(listing);
    return match run_git(git, &write) {
        Ok(answer) if answer.succeeded() => Ok(()),
        _ => Err(FailureCode::Unavailable),
    };
}

/// `set_to_head` for paths whose old entry is no record's,
/// with the worktree's head as the entry to take.
pub(super) fn set_to_head_paths(
    git: &GitContext,
    repository: &Path,
    index: &Path,
    paths: &[Vec<u8>],
) -> Result<(), FailureCode> {
    return set_to_head_with(git, repository, index, paths, index);
}
