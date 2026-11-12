//! What:
//!  The paths one guarded form writes into the worktree,
//!  asked of the tree or the index the form names.
//! Why:
//!  Saving a file that stands in the way of a write needs the written paths first,
//!  and only Git knows which paths a pathspec reaches in a tree:
//!  `git checkout other -- dir` writes every path of `other` below `dir`,
//!  which a glob or a directory name would have to be re-read to guess.
//!  Every directory on the way to a written path is asked about as well,
//!  because a tree creates the directories it needs,
//!  and a worktree that holds a file where one of them must go loses that file.
//!  A revision Git cannot resolve writes no path,
//!  which is the reading `git checkout -f <a path>` needs:
//!  Git writes that path from the index and no tree at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsTheTreeWrites(facts, { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }) === ['b.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The written-path reading this fact answers.
use crate::command_written_paths::WrittenPaths;
/// The captured-query runner's output type,
///  and the NUL-separated reading of an answer.
use crate::git_metadata::MetadataOutput;
use crate::rule_worktree_dirty::{nul_separated_paths, with_ancestors};
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};

/// What:
///  Every path the form writes,
///  each with each directory on the way to it.
///  `Result<Vec<Vec<u8>>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   A query that failed says nothing,
///  and a safety check must never read a failure as "nothing is written".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsTheTreeWrites(facts, written): Promise<Buffer[]>; // throws
/// ```
pub(super) fn paths_the_tree_writes(facts: &mut GitFacts, written: &WrittenPaths) -> Result<Vec<Vec<u8>>, String> {
    let paths: Vec<Vec<u8>> = match written {
        WrittenPaths::Tree { revision } => tree_paths(facts, revision.as_deref(), &[])?,
        WrittenPaths::MatchingTree { revision, pathspecs } => tree_paths(facts, revision.as_deref(), pathspecs.as_slice())?,
        WrittenPaths::Index { pathspecs } => index_paths(facts, pathspecs.as_slice())?,
    };
    if paths.is_empty() {
        return Ok(paths);
    }
    return Ok(with_ancestors(paths.as_slice()));
}

/// What:
///  The paths of one revision's tree that the given pathspecs match,
///  or every path of it when there are none.
/// Why:
///   `git ls-tree -r -z --name-only` prints each path of the tree,
///  and Git itself matches the pathspecs,
///  so a directory, a glob and a literal name reach exactly what the form reaches.
///  A revision that does not resolve writes no path;
///  a query that could not run at all is a failure.
///  `-` is the previous branch only to `switch` and `checkout`,
///  so it is read as `@{-1}` before the tree is listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treePaths(facts, revision: string | undefined, pathspecs: string[]): Promise<Buffer[]>; // throws
/// ```
fn tree_paths(facts: &mut GitFacts, revision: Option<&OsStr>, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-tree", "-r", "-z", "--name-only"] {
        arguments.push(OsString::from(token));
    }
    // `-` is the previous branch only to `switch` and `checkout`, so it is read as the expression Git resolves
    //  wherever an object name is read; `@{-N}` needs no such reading.
    let named: OsString = match revision {
        Some(written) => match crate::command_discard::previous_branch_expression(written.as_encoded_bytes()) {
            Some(expression) => OsString::from(String::from_utf8_lossy(expression).into_owned()),
            None => written.to_os_string(),
        },
        None => OsString::from("HEAD"),
    };
    arguments.push(named);
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query(arguments, "the tree this command writes")?;
    if !output.success {
        return Ok(Vec::<Vec<u8>>::new());
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  or every path of it when there are none.
/// Why:
///  A restore without a source writes the index's content over the worktree,
///  and a query that failed is a failure rather than an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexPaths(facts, pathspecs: string[]): Promise<Buffer[]>; // throws
/// ```
fn index_paths(facts: &mut GitFacts, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-files", "-z", "--cached", "--full-name"] {
        arguments.push(OsString::from(token));
    }
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query(arguments, "the paths the index holds")?;
    if !output.success {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read the paths the index holds: {detail}"));
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}
