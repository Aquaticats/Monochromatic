//! What:
//!  The worktree entries that stand where one guarded form writes,
//!  whatever Git's status calls them.
//! Why:
//!  A forced tree write replaces an untracked file,
//!  turns it into a directory,
//!  and deletes the work inside a nested repository at that path,
//!  while the status the guard reads names only the worktree's own tracked changes.
//!  An ignored file is in neither reading,
//!  and an ignored file is where a person keeps their environment and their local secrets.
//!  The question is Git's own: the paths the form writes,
//!  then every entry of the worktree at or below them that Git did not put there,
//!  asked over those paths alone so a worktree with a million untracked files costs
//!  what its written paths cost.
//!  A path Git collapses into one directory name (`nested/`) is the directory itself:
//!  a nested repository is reported that way, and moving or copying it whole is what keeps it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsInTheWay(facts, { kind: 'tree', revision: 'other' }) === ['b.txt', 'nested']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The written-path reading the question starts from.
use crate::command_written_paths::WrittenPaths;
/// The paths the form writes,
///  each with each directory on the way to it.
use crate::repository_facts_written_paths::paths_the_tree_writes;
/// The captured-query runner's output type.
use crate::git_metadata::MetadataOutput;
/// The queries over one set of paths,
///  and the reading of each answer.
use crate::rule_worktree_dirty::{worktree_entry_queries, worktree_entries_from_answer};

/// What:
///  Every path at or below a written path that the worktree holds and Git did not put there,
///  each as Git printed it,
///  with the entries below another entry dropped.
///  `Result<Vec<String>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   Only the outermost entry is saved:
///  a directory holds everything below it,
///  and saving a file inside it as well would name a place it is not in.
///  A failure is a failure:
///  a reading that could not be made says nothing about what the command overwrites,
///  and the caller refuses rather than spending a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsInTheWay(facts, written): Promise<string[]>; // throws
/// ```
pub(super) fn paths_in_the_way(facts: &mut GitFacts, written: &WrittenPaths) -> Result<Vec<String>, String> {
    let asked: Vec<Vec<u8>> = paths_the_tree_writes(facts, written)?;
    if asked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    // Only the outermost of the written paths is asked about: a pathspec that names a path inside a nested
    //  repository makes Git answer nothing at all for every path in the same query, and a path below another one
    //  is inside that one's answer anyway.
    let outermost: Vec<Vec<u8>> = outermost_asked(asked.as_slice());
    let mut found: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    // Two answers, one question: what is untracked and what is ignored. Both are things Git did not write.
    for ignored in [false, true] {
        for query in worktree_entry_queries(facts.global_prefix.as_slice(), outermost.as_slice(), ignored) {
            let output: MetadataOutput = facts.query(query, "the files this command writes over")?;
            found.extend(worktree_entries_from_answer(&output)?);
        }
    }
    return Ok(outermost_entries(found));
}

/// What:
///  The written paths that no other written path holds,
///  in the order they were given.
/// Why:
///   A query that names both `nested` and `nested/file.txt` answers nothing at all,
///  because the second is a path inside a nested repository and Git skips the whole question;
///  and a path below another one is inside that one's own answer,
///  so nothing is lost by not naming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outermostAsked(asked: Buffer[]): Buffer[];
/// ```
fn outermost_asked(asked: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut kept: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in asked {
        let above: bool = asked.iter().any(|other| other.len() < path.len() && holds(other.as_slice(), path.as_slice()));
        if above {
            continue;
        }
        kept.push(path.clone());
    }
    return kept;
}

/// What:
///  The entries that no other entry holds,
///  each as bytes and in one stable order.
/// Why:
///   Saving both a directory and a file inside it names a place the file is not in,
///  and a move of the directory has already kept it;
///  the shortest path first makes the answer the same on every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outermostEntries(found: Buffer[]): string[];
/// ```
fn outermost_entries(found: Vec<Vec<u8>>) -> Vec<String> {
    let mut sorted: Vec<Vec<u8>> = found;
    sorted.sort();
    sorted.dedup();
    let mut kept: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in sorted {
        if kept.iter().any(|outer| holds(outer.as_slice(), path.as_slice())) {
            continue;
        }
        kept.push(path);
    }
    let mut named: Vec<String> = Vec::<String>::new();
    for path in kept {
        named.push(String::from_utf8_lossy(path.as_slice()).into_owned());
    }
    return named;
}

/// What:
///  Whether one entry is the other or holds it,
///  at a component boundary.
/// Why:
///  `a/b` and `a` are one entry's worth of saving,
///  while `ab` is a name of its own beside `a`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holds = (outer, inner) => outer === inner || inner.startsWith(outer + '/');
/// ```
fn holds(outer: &[u8], inner: &[u8]) -> bool {
    if outer == inner {
        return true;
    }
    return inner.starts_with(outer) && inner.get(outer.len()) == Some(&b'/');
}

/// The paths the worktree holds where the form writes, and the answer reading.
#[cfg(test)]
#[path = "repository_facts_paths_in_the_way_tests.rs"]
mod tests;
