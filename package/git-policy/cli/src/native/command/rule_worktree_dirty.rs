//! What:
//!  The question "does this worktree hold uncommitted state?"
//! as one Git query and its reading.
//! Why:
//!  A command that can discard uncommitted state is judged by whether any exists:
//! a clean linked worktree has
//!  nothing to lose,
//! a dirty one does.
//! Tracked changes (staged or not) always count;
//! untracked files count only
//!  for the commands that delete them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git status --porcelain=v1 --untracked-files=no  => dirty when it prints anything
//! ```

use crate::git_metadata::MetadataOutput;
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  Which files count as uncommitted state.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A `reset --hard` loses tracked changes only,
/// a `clean` loses untracked files only;
/// asking for the
///  wrong set would reject a harmless command or let a destructive one through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtyScope = 'tracked' | 'tracked-and-untracked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirtyScope {
    /// Staged and unstaged changes to tracked files.
    Tracked,
    /// Tracked changes and untracked files that are not ignored.
    TrackedAndUntracked,
}

/// What:
///  The arguments of the status query,
/// after the invocation's global options.
///  `Vec<OsString>` is an owned list of byte-preserving text.
/// Why:
///   `--no-optional-locks` keeps a read from writing the index,
/// and the porcelain form is stable across
///  Git versions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...preSubcommandArgs, '--no-optional-locks', 'status', '--porcelain=v1', '--untracked-files=no'];
/// ```
pub fn dirty_query_arguments(global_prefix: &[OsString], scope: DirtyScope) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "status", "--porcelain=v1"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(OsString::from(match scope {
        DirtyScope::Tracked => "--untracked-files=no",
        DirtyScope::TrackedAndUntracked => "--untracked-files=normal",
    }));
    return arguments;
}

/// What:
///  Read the query:
/// any output line means uncommitted state exists.
///  `Result<bool, String>` is "the answer,
/// or why Git could not give one".
/// Why:
///   A status that failed says nothing,
/// and a safety check must not read a failure as "clean".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyFromOutput(output: MetadataOutput): boolean; // throws
/// ```
pub fn dirty_from_output(output: &MetadataOutput) -> Result<bool, String> {
    if !output.success {
        let detail: String = String::from_utf8_lossy(&output.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read the worktree status: {detail}"));
    }
    return Ok(!output.stdout.is_empty());
}

/// What:
///  Whether a Git directory has never had an index file: the state `git clone --no-checkout` leaves.
///  `Option<&Path>` is "a borrowed path or nothing"; the second argument is the value of `GIT_INDEX_FILE`, injected so tests
///  never read the process environment.
/// Why:
///   With no index file Git sees an initial checkout and has nothing of the user's to protect: `builtin/checkout.c` sets
///  `initial_checkout = is_index_unborn(...)` and `is_index_unborn` in `read-cache.c` is `!istate->cache_nr && !istate->timestamp.sec`,
///  so `unpack-trees.c` skips the rejection of a staged deletion (`if (oldtree && !o->initial_checkout)`). `git status` still lists every
///  committed file as a staged deletion there, which the guard read as uncommitted work. The definition stays as narrow as Git's: an index
///  file that exists, empty or not, is never absent, so a staged deletion of every file still counts; a `GIT_INDEX_FILE` that names
///  another index, an unreadable Git directory and a Git directory that does not exist all answer false and leave the question to `git status`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexFileIsAbsent(gitDirectory?: string, indexFileVariable?: string): boolean;
/// ```
pub fn index_file_is_absent(git_directory: Option<&Path>, index_file_variable: Option<&OsStr>) -> bool {
    if index_file_variable.is_some() {
        return false;
    }
    let Some(directory) = git_directory else {
        return false;
    };
    // `symlink_metadata` does not follow a link, so a dangling link named `index` is present, never absent.
    return match std::fs::symlink_metadata(directory.join("index")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => directory.is_dir(),
        Err(_) | Ok(_) => false,
    };
}

/// Arguments of each scope and the reading of a successful,
/// empty,
/// non-empty and failed status.
#[cfg(test)]
#[path = "rule_worktree_dirty_tests.rs"]
mod tests;
