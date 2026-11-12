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
use crate::rule_worktree_dirty::worktree_entries_from_answer;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

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
    let mut found: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    // Two answers, one question: what is untracked and what is ignored, each asked once over the whole worktree
    //  because Git matches thousands of literal pathspecs far more slowly than it lists the entries itself.
    //  Both listings collapse a directory the worktree holds whole into one entry (`nested/`), which is the entry a
    //  save must keep whole, and a nested repository is reported exactly that way.
    for ignored in [false, true] {
        let output: MetadataOutput = facts.query(worktree_entry_query(facts.global_prefix.as_slice(), ignored), "the files this command writes over")?;
        for entry in worktree_entries_from_answer(&output)? {
            if !asked.iter().any(|path| return related(path.as_slice(), entry.as_slice())) {
                continue;
            }
            found.push(entry);
        }
    }
    return Ok(outermost_entries(found));
}

/// What:
///  Whether one worktree entry stands where the form writes:
///  at the written path,
///  below it,
///  or above it,
///  which is the case of a directory the tree needs a file inside of.
/// Why:
///   `--directory` collapses a directory the worktree holds whole into one entry,
///  and that entry holds every written path inside it;
///  an entry deeper than a written path is the file the form writes over;
///  and a name that merely starts the same letters names nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const related = (asked, entry) => asked === entry || holds(asked, entry) || holds(entry, asked);
/// ```
fn related(asked: &[u8], entry: &[u8]) -> bool {
    return asked == entry || holds(asked, entry) || holds(entry, asked);
}

/// What:
///  The one listing over the whole worktree that names what Git did not put there,
///  either the entries its ignore rules do not cover or the ones they do,
///  with each directory the worktree holds whole collapsed into one entry.
///  `bool` asks for the ignored ones.
/// Why:
///   Asking over the written paths alone costs one large argument list per bounded start,
///  and Git's own pathspec matching over thousands of literal paths is what made a
///  `reset --hard` slow:
///  measured on a worktree of this repository's size, ten such starts took 1.9 s where these two take 0.12 s.
///  The answers are intersected with the written paths in this process,
///  which is one pass over each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeEntryQuery(globalPrefix: string[], ignored: boolean): string[];
/// ```
fn worktree_entry_query(global_prefix: &[OsString], ignored: bool) -> Vec<OsString> {
    let selection: &[&str] = if ignored {
        &["--others", "--ignored", "--exclude-standard", "--directory"]
    } else {
        &["--others", "--exclude-standard", "--directory"]
    };
    return crate::rule_worktree_dirty::listing_arguments(global_prefix, selection);
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
        if kept.iter().any(|outer| return holds(outer.as_slice(), path.as_slice())) {
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
