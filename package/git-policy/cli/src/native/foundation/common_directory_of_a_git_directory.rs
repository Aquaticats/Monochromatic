//! What:
//!  The common Git directory a worktree's own Git directory belongs to:
//!  the directory its `commondir` file names,
//!  else the directory itself.
//! Why:
//!  A linked worktree's own Git directory goes away with the worktree and is not where the repository keeps what belongs to
//!  all of it:
//!  the timing records,
//!  the repository scratch directory a commit's scratch tree lives in,
//!  and the discard audit.
//!  Three readers needed the same answer,
//!  and each wrote the few lines again
//!  (`commit_timing.rs` said so in its own copy),
//!  so the rule lives here once.
//!  Git strips every trailing carriage return and line feed and resolves a relative path against the Git directory.
//!  A path that cannot be resolved is used as written,
//!  so a Git directory whose `commondir` names a missing place still gets a record beside itself rather than none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
//! ```

/// Paths of directories.
use std::path::{Path, PathBuf};

/// What:
///  The common Git directory of `git_dir`.
/// Why:
///  Every caller asks the same question of a Git directory Git itself handed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDir = commonDirectoryOf(gitDir);
/// ```
pub fn common_directory_of(git_dir: &Path) -> PathBuf {
    let Ok(text) = std::fs::read_to_string(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let named: &str = text.trim_end_matches(['\r', '\n']);
    if named.is_empty() {
        return git_dir.to_path_buf();
    }
    let joined: PathBuf = git_dir.join(named);
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// The resolution is controlled in the test module.
#[cfg(test)]
#[path = "common_directory_of_a_git_directory_tests.rs"]
mod tests;
