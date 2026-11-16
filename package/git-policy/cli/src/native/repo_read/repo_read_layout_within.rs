//! What:
//!  The read of a repository's location,
//!  run as one job on a thread of its own and awaited no longer than `LOCATION_READ_CEILING`.
//! Why:
//!  `repo_read_layout.rs` finds the location from the file system,
//! and every guarded command asks for it
//!  (`repo_read/repository_facts.rs`).
//!  Its calls wait in the kernel with no end on a mount that stopped answering:
//!  a `canonicalize` or a `symlink_metadata` on a network mount,
//!  on a FUSE mount whose server has stopped,
//!  or on a disk that stalls.
//!  The owner's rule is "nothing should ever wait forever or indefinitely",
//!  so the read runs as a job and the caller stops waiting at the ceiling,
//!  whatever the kernel does.
//!  One ceiling covers the whole read and not each call:
//!  the walk up from the start directory,
//!  the configuration files it reads and the directory entries it checks are one job,
//!  so one thread starts where the read begins and no storage call starts one of its own.
//!  The job is abandoned, never cancelled:
//!  a call the kernel never returns is released only when the kernel lets go,
//!  and the process's own end waits for it (`work_awaited_within_a_ceiling.rs`).
//!  A read that gives no answer is `None`,
//!  which is what the reader already answers for every layout it does not model:
//!  the caller asks Git,
//!  and Git's own read has a bound of its own and its own words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout = readTheLayoutWithin({ start, scope, typed }); // undefined: ask Git
//! ```

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::{RepoLayout, walk_up};
use crate::repo_read_layout_disk_and_directory_facts::start_directory;
use crate::wait_ceilings::LOCATION_READ_CEILING;
use crate::work_awaited_within_a_ceiling::work_within;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The layout of the repository whose read starts in `start`,
///  found within `LOCATION_READ_CEILING`.
/// Why:
///  The callers that name a directory (a transaction Git asks for,
///  a merge probe,
///  the two query-driven LFS pushes)
///  hand it here,
///  so the whole read is one job and the answer or the ceiling is what they wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = fromADirectory({ start, scope, typed }); // undefined: ask Git
/// ```
pub(crate) fn from_a_directory(start: PathBuf, scope: ScopeEnvironment, typed: bool) -> Option<RepoLayout> {
    return within(move || {
        // With `--path-format=absolute` Git prints the real path of its start directory, so the walk begins at one.
        let real_start: PathBuf = std::fs::canonicalize(start).ok()?;
        return walk_up(real_start.as_path(), &scope, typed);
    })
    .flatten();
}

/// What:
///  The layout of the repository a command starts in with `global_prefix`,
///  found within `LOCATION_READ_CEILING`.
/// Why:
///  A command's location starts where its leading `-C` options leave it,
/// and reading that directory, its device and its entries is storage work of the same job.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = fromTheGlobalPrefix({ globalPrefix, scope, typed }); // undefined: ask Git
/// ```
pub(crate) fn from_the_global_prefix(global_prefix: Vec<OsString>, scope: ScopeEnvironment, typed: bool) -> Option<RepoLayout> {
    return within(move || {
        let start: PathBuf = start_directory(global_prefix.as_slice())?;
        return walk_up(start.as_path(), &scope, typed);
    })
    .flatten();
}

/// What:
///  Whether `path` is a symbolic link,
///  read within `LOCATION_READ_CEILING`;
///  `None` when the read gave no answer, which is the caller's own "ask Git".
/// Why:
///   `git_path` refuses a name that is a link, because `--path-format=absolute` prints a link's target,
///  and that one lookup waits in the kernel on a mount that stopped answering like every other read here.
/// One job per asked name is the price of bounding it.
/// A name that is not there is not a link,
/// which is the answer Git's own `--git-path` gives for it,
/// so only a read that gave no answer leaves the question to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = isALink(path); // undefined: the caller asks Git
/// ```
pub(crate) fn is_a_link(path: &Path) -> Option<bool> {
    let owned: PathBuf = path.to_path_buf();
    return within(move || std::fs::symlink_metadata(owned.as_path()).is_ok_and(|metadata| metadata.file_type().is_symlink()));
}

/// What:
///  Run `job` on a thread of its own and wait for it no longer than `LOCATION_READ_CEILING`.
/// Why:
///  This is the one place the location read names its ceiling,
///  so a storage call added anywhere in the read is covered by it without a ceiling of its own,
///  and a read that gives no answer, or a system that refuses the thread, answers `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = workWithin(LOCATION_READ_CEILING, job);
/// ```
fn within<Value, Job>(job: Job) -> Option<Value>
where
    Value: Send + 'static,
    Job: FnOnce() -> Value + Send + 'static,
{
    return work_within(LOCATION_READ_CEILING, job);
}

/// The ceiling and its cover are controlled in the test module.
#[cfg(test)]
#[path = "repo_read_layout_within_tests.rs"]
mod tests;
