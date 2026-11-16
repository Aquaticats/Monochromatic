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
//!  A command asks the same read through several callers,
//!  and storage that stopped answering would cost one ceiling per caller
//!  (measured:
//! about ten asks of one `git add`),
//!  so a read that gave no answer makes the rest of this process's reads answer nothing at once:
//!  the answer they would give is the one the first gave,
//!  and declining hands the question to Git,
//!  which is correct and only slower.
//!  The job is abandoned,
//! never cancelled:
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
use crate::work_awaited_within_a_ceiling::{TheAnswerOfAThreadOfWork, work_within_reporting};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
/// and reading that directory,
/// its device and its entries is storage work of the same job.
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
///  `None` when the read gave no answer,
/// which is the caller's own "ask Git".
/// Why:
///   `git_path` refuses a name that is a link,
/// because `--path-format=absolute` prints a link's target,
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
    return within(move || return is_a_link_at(owned));
}

/// What:
///  Whether the name at `path` is a symbolic link,
///  and `false` for a name that is not there or cannot be read.
/// Why:
///   The ceiling's job names one function,
/// so the work it runs is readable on its own
///  (`rust/avoidable-closure`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = lstatSync(path).isSymbolicLink(); // a name that cannot be read is not a link
/// ```
fn is_a_link_at(path: PathBuf) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else {
        return false;
    };
    return metadata.file_type().is_symlink();
}

/// What:
///  Run `job` on a thread of its own and wait for it no longer than `LOCATION_READ_CEILING`.
/// Why:
///  This is the one place the location read names its ceiling,
///  so a storage call added anywhere in the read is covered by it without a ceiling of its own,
///  and a read that gives no answer,
/// or a system that refuses the thread,
/// answers `None`.
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
    return within_the_ceiling(LOCATION_READ_CEILING, job);
}

/// Whether a read of this process already gave no answer at its ceiling.
/// Why:
///  One command asks this read through several callers (a fact provider,
///  a candidate preparation,
///  a transaction probe),
///  and each of them that waited a whole ceiling on storage that stopped answering would multiply the bound
///  the rule asks for once.
///  Measured through the wrapper's `git add` against a lookup that never returns:
///  the read is asked about ten times,
///  so ten ceilings passed before the command answered where one does now.
///  Only a wait that reached the ceiling sets it:
///  a thread the system refused leaves the read as it was,
///  because nothing was learned about the storage.
///  A process that once saw this keeps asking Git for its reads until it ends,
///  which is the slower correct answer,
/// and a commit service that saw it ends the sooner for being idle.
static A_READ_GAVE_NO_ANSWER: AtomicBool = AtomicBool::new(false);

/// What:
///  Run `job` for no longer than `ceiling`,
///  and answer nothing at once when an earlier read of this process gave no answer.
/// Why:
///  The controls shorten the ceiling;
///  the shipped read passes `LOCATION_READ_CEILING` and nothing else does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (aReadGaveNoAnswer) return undefined; const answer = workWithinReporting(ceiling, job);
/// ```
fn within_the_ceiling<Value, Job>(ceiling: Duration, job: Job) -> Option<Value>
where
    Value: Send + 'static,
    Job: FnOnce() -> Value + Send + 'static,
{
    if A_READ_GAVE_NO_ANSWER.load(Ordering::Relaxed) {
        return None;
    }
    return match work_within_reporting(ceiling, job) {
        TheAnswerOfAThreadOfWork::Ran(value) => Some(value),
        // Nothing ran, so nothing was learned about the storage and the next read may still have its answer.
        TheAnswerOfAThreadOfWork::NoThreadToStart => None,
        TheAnswerOfAThreadOfWork::NoAnswer => {
            A_READ_GAVE_NO_ANSWER.store(true, Ordering::Relaxed);
            None
        }
    };
}

/// What:
///  Forget that a read of this process gave no answer,
///  so the next read waits for its own one.
/// Why:
///  The seam of the controls above:
///  one control shows that a read after a no answer is not waited for,
///  and the controls beside it need their read to be asked again.
#[cfg(test)]
pub(crate) fn a_control_forgets_the_no_answer() {
    A_READ_GAVE_NO_ANSWER.store(false, Ordering::Relaxed);
}

/// The ceiling and its cover are controlled in the test module.
#[cfg(test)]
#[path = "repo_read_layout_within_tests.rs"]
mod tests;
