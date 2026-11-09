//! What:
//!  The question "does this repository hold a stash entry?"
//! as one Git query and its reading.
//! Why:
//!  `git stash drop` and `git stash clear` delete entries of `refs/stash`,
//!  one stash that every worktree
//!       of the repository shares,
//!  so the cleanliness of the worktree the command runs in says nothing about them.
//!  One
//!       reference has to exist for either to delete anything,
//!  so that reference is the whole question and it costs
//!       no worktree walk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git rev-parse --verify --quiet refs/stash  => exit 0 means an entry exists
//! ```

use crate::git_metadata::MetadataOutput;
use std::ffi::OsString;

/// The reference a stash entry lives on,
///  in the common Git directory.
pub const STASH_REFERENCE: &str = "refs/stash";

/// What:
///  The arguments of the stash query,
/// after the invocation's global options.
///  `Vec<OsString>` is an owned list
///       of byte-preserving text.
/// Why:
///   `--verify` asks about that one reference and nothing else,
///  `--quiet` keeps Git from printing
///       anything when it is absent,
/// and neither reads the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...preSubcommandArgs, 'rev-parse', '--verify', '--quiet', 'refs/stash'];
/// ```
pub fn stash_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["rev-parse", "--verify", "--quiet", STASH_REFERENCE] {
        arguments.push(OsString::from(token));
    }
    return arguments;
}

/// What:
///  Read the query:
/// status 0 means an entry exists,
/// status 1 means none,
///  and anything else is a question Git could not answer.
///  `Result<bool, String>` is "the answer,
/// or why Git could
///       not give one".
/// Why:
///   A safety check must not read a failure as "no stash",
/// which would let a clear run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stashFromOutput(output: MetadataOutput): boolean; // throws
/// ```
pub fn stash_from_output(output: &MetadataOutput) -> Result<bool, String> {
    if output.success {
        return Ok(true);
    }
    if output.code == Some(1) {
        return Ok(false);
    }
    let detail: String = String::from_utf8_lossy(&output.stderr)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    return Err(format!("Git could not read the repository stash: {detail}"));
}

/// Query arguments,
///  both answers and the failed question stay out of the release executable.
#[cfg(test)]
#[path = "rule_repository_stash_tests.rs"]
mod tests;
