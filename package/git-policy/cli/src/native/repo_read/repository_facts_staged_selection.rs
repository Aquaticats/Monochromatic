//! What:
//!  What one `git add` selects,
//!  counted from real Git's own listings.
//! Why:
//!  A pathspec reaches by Git's grammar,
//!  so only Git can say which paths a spelling selects,
//!  and the add-explicit policy refuses a spelling for that answer and no reading of its own.
//!  Two listings answer it:
//!  the caller's own spellings with the caller's own pathspec mode,
//!  and every path the worktree gives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const stagedSelection = async (facts, spellings) => ({ selected: (await listed(facts, spellings)).length, /* ... */ });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The provider owns the Git start and the caller's global options;
///  this file owns the question's shape and its reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { stagedSelection } from './repository_facts.ts';
/// ```
use crate::git_metadata::MetadataOutput;
use crate::repository_facts::{GitFacts, RepositoryFacts};
use crate::rule_add_explicit::{
    StagedSelection, selection_query_arguments, whole_worktree_query_arguments,
};
use crate::rule_worktree_dirty::nul_separated_paths;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  How many paths the given positional pathspecs select,
///  and how many the whole worktree and the caller's own directory give.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A selection is a subset of the worktree's paths,
///  so equal counts are equal sets,
///  and the caller's directory is read from the paths Git printed from the top level.
///  A first listing that found nothing asks for no second one:
///  nothing selected is nothing swept up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stagedSelection(facts: GitFacts, pathspecs: string[]): Promise<StagedSelection>;
/// ```
pub(super) fn staged_selection(
    facts: &mut GitFacts,
    pathspecs: Vec<OsString>,
) -> Result<StagedSelection, String> {
    let asked: Vec<OsString> =
        selection_query_arguments(facts.global_prefix.as_slice(), pathspecs.as_slice());
    let mut selected: Vec<Vec<u8>> = listed(facts, asked, "the paths a git add selects")?;
    // An unmerged path is one path listed at two or three stages in a row, and the count names paths.
    selected.dedup();
    // A command that selects nothing is no bulk command, and one process answers it.
    if selected.is_empty() {
        return Ok(StagedSelection {
            selected: 0,
            repository: 0,
            directory: 0,
        });
    }
    let directory_prefix: Vec<u8> = facts.location()?.prefix;
    let mut whole: Vec<Vec<u8>> = listed(
        facts,
        whole_worktree_query_arguments(facts.global_prefix.as_slice()),
        "the paths of the worktree",
    )?;
    // The same deduplication, so both counts name paths and the equality still holds.
    whole.dedup();
    let mut directory: usize = 0;
    for path in &whole {
        // `--full-name` prints each path from the top level, so the caller's own directory is a prefix of it.
        if directory_prefix.is_empty() || path.starts_with(directory_prefix.as_slice()) {
            directory += 1;
        }
    }
    return Ok(StagedSelection {
        selected: selected.len(),
        repository: whole.len(),
        directory,
    });
}

/// What:
///  Every path one `ls-files` list prints,
///  with the bytes Git wrote.
///  `Vec<Vec<u8>>` is an owned list of owned byte paths.
/// Why:
///  A listing that fails is a failed read,
///  never an empty answer:
///  an empty answer would prove a command stages nothing,
///  and the policy would let a bulk add through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async listed(facts: GitFacts, args: string[], fact: string): Promise<Buffer[]>;
/// ```
fn listed(facts: &mut GitFacts, arguments: Vec<OsString>, fact: &str) -> Result<Vec<Vec<u8>>, String> {
    let output: MetadataOutput = facts.query(arguments, fact)?;
    if !output.success {
        return Err(format!(
            "monochromatic-git-policy-cli could not list {fact}: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}
