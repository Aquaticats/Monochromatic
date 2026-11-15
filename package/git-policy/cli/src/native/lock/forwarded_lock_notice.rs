//! What:
//!  The clearing of stale reference locks for the person's own `git fetch` and `git push`,
//! and the notice that goes before
//!  Git's own words.
//! Why:
//!  A forwarded plain fetch or push replaces the wrapper's process with Git,
//! so there is no "after" to look at:
//! the wrapper
//!  can only judge the locks before it hands over.
//! Locks it removes are said,
//! and locks it will not remove are described
//!  with the command that clears them,
//! so the notice precedes Git's refusal instead of leaving the person with
//!  "File exists" alone.
//! The walk reads directory entries of `refs` and two files,
//! and starts no Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = forwardedLockNotice({ word: 'fetch', globalPrefix, environment, judge }); process.stderr.write(notice);
//! ```

use crate::diagnostic_log::debug;
use crate::git_directory_roots::repository_a_command_works_on;
use crate::stale_ref_lock_judge::Judge;
use crate::stale_ref_locks::clear_stale_ref_locks;
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::PathBuf;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Whether the person's command is one that updates references from or to a remote and is covered.
pub fn is_covered(word: &[u8]) -> bool {
    return word == b"fetch" || word == b"push";
}

/// Clear what is provably stale before a forwarded `fetch` or `push` and return the notice to print;
/// empty for every other
///  command,
/// for a command whose repository no option and no variable names,
/// and when nothing is found.
/// The repository is the one every other reader of this invocation resolves
/// (`repository_a_command_works_on`):
/// a relative `-C` moves the directory once,
/// `--git-dir` and `GIT_DIR` name the Git directory,
/// and `GIT_COMMON_DIR` names the common one,
/// so a fetch or push that names its repository is judged in the repository Git uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forwardedLockNotice({ word, globalPrefix, environment, judge }): string;
/// ```
pub fn forwarded_lock_notice(word: &[u8], global_prefix: &[OsString], environment: &[(OsString, OsString)], judge: &Judge) -> String {
    if !is_covered(word) {
        return String::new();
    }
    let Ok(current) = std::env::current_dir() else {
        return String::new();
    };
    let roots: Vec<PathBuf> = repository_a_command_works_on(current.as_path(), global_prefix, environment).roots();
    if roots.is_empty() {
        return String::new();
    }
    let outcome = clear_stale_ref_locks(roots.as_slice(), judge);
    let notice: String = outcome.notice();
    if !notice.is_empty() {
        // The notice itself is printed on standard error, so the log keeps only a one-line record of it.
        debug(TAG, "reference locks were judged before a forwarded fetch or push");
    }
    return notice;
}

/// Controls for the clearing before the person's forwarded fetch and push.
#[cfg(test)]
#[path = "forwarded_lock_notice_tests.rs"]
mod tests;
