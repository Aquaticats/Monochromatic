//! What:
//!  The clearing of stale reference locks for the person's own `git fetch` and `git push`, and the notice that goes before
//!  Git's own words.
//! Why:
//!  A forwarded plain fetch or push replaces the wrapper's process with Git, so there is no "after" to look at: the wrapper
//!  can only judge the locks before it hands over. Locks it removes are said, and locks it will not remove are described
//!  with the command that clears them, so the notice precedes Git's refusal instead of leaving the person with
//!  "File exists" alone. The walk reads directory entries of `refs` and two files, and starts no Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = forwardedLockNotice({ word: 'fetch', globalPrefix, environment, judge }); process.stderr.write(notice);
//! ```

use crate::child_environment::environment_value;
use crate::diagnostic_log::debug;
use crate::git_directory_roots::roots_for;
use crate::stale_ref_lock_judge::Judge;
use crate::stale_ref_locks::clear_stale_ref_locks;
use crate::worktree_add_plan::start_directory;
use std::ffi::OsString;
use std::path::PathBuf;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Whether the person's command is one that updates references from or to a remote and is covered.
pub fn is_covered(word: &[u8]) -> bool {
    return word == b"fetch" || word == b"push";
}

/// Clear what is provably stale before a forwarded `fetch` or `push` and return the notice to print; empty for every other
///  command, for a command whose repository is named by an option other than `-C` and `GIT_DIR`, and when nothing is found.
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
    let Some(start) = start_directory(global_prefix, current.as_path()) else {
        return String::new();
    };
    let mut named: Option<PathBuf> = None;
    if let Some(path) = environment_value(environment, "GIT_DIR") {
        named = Some(start.join(path));
    }
    let outcome = clear_stale_ref_locks(roots_for(start.as_path(), named).as_slice(), judge);
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
