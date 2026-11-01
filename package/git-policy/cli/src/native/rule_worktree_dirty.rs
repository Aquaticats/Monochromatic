//! What:
//!  The question "does this worktree hold uncommitted state?" as one Git query and its reading.
//! Why:
//!  A command that can discard uncommitted state is judged by whether any exists: a clean linked worktree has
//!  nothing to lose, a dirty one does. Tracked changes (staged or not) always count; untracked files count only
//!  for the commands that delete them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git status --porcelain=v1 --untracked-files=no  => dirty when it prints anything
//! ```

use super::git_metadata::MetadataOutput;
use std::ffi::OsString;

/// What:
///  Which files count as uncommitted state.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A `reset --hard` loses tracked changes only, a `clean` loses untracked files only; asking for the
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
///  The arguments of the status query, after the invocation's global options.
///  `Vec<OsString>` is an owned list of byte-preserving text.
/// Why:
///   `--no-optional-locks` keeps a read from writing the index, and the porcelain form is stable across
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
///  Read the query: any output line means uncommitted state exists.
///  `Result<bool, String>` is "the answer, or why Git could not give one".
/// Why:
///   A status that failed says nothing, and a safety check must not read a failure as "clean".
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

/// Arguments of each scope and the reading of a successful, empty, non-empty and failed status.
#[cfg(test)]
#[path = "rule_worktree_dirty_tests.rs"]
mod tests;
