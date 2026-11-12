//! What:
//!  The work whose destruction one guarded form asks about:
//!  the paths where the worktree or the index differs from `HEAD`,
//!  and the entries Git does not track and the ignore rules do not cover.
//! Why:
//!  An allowlisted repository is exempt from the worktree rule,
//!  and never from the rule that a command which could destroy a file existing nowhere else is refused,
//!  so a form there is refused over an untracked entry that stands where it writes
//!  (`policy_linked_worktree`).
//!  Every answer is Git's own listing of the one question asked,
//!  and a listing that never answered is a failure,
//!  never "nothing to save".
//!  The two readings live beside the provider,
//!  as `repository_facts_ignored_files.rs` holds its fact's,
//!  so the provider's own file stays the interface and the order of its answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // uncommittedPaths(facts) => ['a.rs']; untrackedPaths(facts) => ['notes.md']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The captured-query runner's output type.
use crate::git_metadata::MetadataOutput;
/// The two listings of the uncommitted changes and their readings,
///  which the abort carve-out asks as well.
use crate::repository_facts_abort_discard::{both_listings, listed_paths, uncommitted_queries};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The argument list of the untracked listing:
///  every file Git does not track that the worktree's ignore rules do not cover.
///  `&[OsString]` borrows the invocation's global options.
/// Why:
///   A clean that names ignored files as well (`-x`) deletes every untracked file,
///  and a clean whose `-e`/`--exclude` patterns pull one in (`-X`) deletes the ones they call ignored,
///  so an untracked file is what such a clean loses and what the copy must hold.
///  `--exclude-standard` is the worktree's own ignore stack,
///  which leaves out the ignored build output a clean may delete without a copy;
///  `--full-name` names every path from the top of the worktree,
///  which is where the copies are read from,
///  whatever `-C` did to the directory Git runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arguments = [...prefix, '--no-optional-locks', 'ls-files', '--others', '--exclude-standard', '--full-name', '-z'];
/// ```
fn untracked_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-optional-locks",
        "ls-files",
        "--others",
        "--exclude-standard",
        "--full-name",
        "-z",
    ] {
        arguments.push(OsString::from(token));
    }
    return arguments;
}

/// What:
///  The paths where the worktree or the index differs from `HEAD`,
///  each once and in a settled order,
///  or why Git could not list them.
/// Why:
///   Every form that writes a tree or replaces the index discards these,
///  and the listing is asked only where the form would otherwise lose them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uncommittedPaths(facts: GitFacts): Buffer[]; // throws
/// ```
pub(super) fn uncommitted_paths(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let queries: [Vec<OsString>; 2] = uncommitted_queries(facts.global_prefix.as_slice());
    let worktree = facts.query(queries[0].clone(), "the paths this command would discard")?;
    let index = facts.query(queries[1].clone(), "the staged paths this command would discard")?;
    return Ok(both_listings(listed_paths(&worktree)?, listed_paths(&index)?));
}

/// What:
///  Every entry Git does not track and the ignore rules do not cover,
///  as Git printed it,
///  including the directory entry Git prints for a nested Git repository,
///  or why Git could not list them.
/// Why:
///   A clean that deletes untracked files loses exactly these,
///  and a form that writes a tree replaces the ones that stand at its paths;
///  a nested repository arrives as one directory entry (`nested/`),
///  which is the entry a refusal must name whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untrackedPaths(facts: GitFacts): Buffer[]; // throws
/// ```
pub(super) fn untracked_paths(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let arguments: Vec<OsString> = untracked_query_arguments(facts.global_prefix.as_slice());
    let output: MetadataOutput = facts.query(arguments, "the files Git does not track")?;
    return listed_paths(&output);
}
