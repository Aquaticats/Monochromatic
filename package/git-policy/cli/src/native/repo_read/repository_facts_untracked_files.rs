//! What:
//!  The untracked-and-tracked fact of the real-Git provider:
//!  the worktree's untracked files whose paths a revision's tree also tracks.
//! Why:
//!   An abort restores a tree with a reset,
//!  and Git refuses that reset while an untracked file stands where the tree has one,
//!  naming the files:
//!  "The following untracked working tree files would be overwritten by reset".
//!  Nothing is lost then,
//!  and the command does not run,
//!  so a wrapper that saved a copy and said the abort runs would be describing something that did not happen.
//!  Measured with Git 2.56.0 for every abort form,
//!  including an untracked file whose content is the tree's own.
//!  The query and its reading live beside the provider,
//!  as `repository_facts_ignored_files.rs` holds its sibling fact,
//!  so the provider's own file stays the interface and the order of its answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // untrackedFilesTheTreeTracks(facts, 'rebase-merge/orig-head') => ['victim.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The tree reading both facts share.
use super::repository_facts_ignored_files::tree_paths;
/// The captured-query runner's output type.
use crate::git_metadata::MetadataOutput;
/// The readings of one `git ls-files --others` answer over the asked paths.
use crate::rule_worktree_dirty::{nul_separated_paths, replaced_answers, untracked_paths_queries, with_ancestors};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;

/// What:
///  The worktree's untracked files whose paths the given revision's tree also tracks,
///  as Git printed them.
/// Why:
///   The revision is the one the command's own abort restores,
///  read by the caller from the same state the abort reads;
///  a tree that cannot be read is a failure,
///  never an empty answer,
///  because the refusal it earns must not read as clean.
///  A tree creates the directories its paths need,
///  so every directory on the way to a tracked path is asked about too:
///  an untracked file where one of them must go is lost to the same reset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function untrackedFilesTheTreeTracks(facts: GitFacts, revision: string): Promise<string[]>;
/// ```
pub(super) fn untracked_files_the_tree_tracks(facts: &mut GitFacts, revision: &OsStr) -> Result<Vec<String>, String> {
    let Some(tracked) = tree_paths(facts, revision)? else {
        return Err(format!(
            "monochromatic-git-policy-cli could not read the tree of {} , so the worktree's untracked files cannot be compared with the tree this command writes into.",
            revision.to_string_lossy()
        ));
    };
    if tracked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    let asked: Vec<Vec<u8>> = with_ancestors(tracked.as_slice());
    // Git answers for these paths alone, so no query walks the worktree.
    let mut shared: Vec<String> = Vec::<String>::new();
    for query in untracked_paths_queries(facts.global_prefix.as_slice(), asked.as_slice()) {
        let output: MetadataOutput = facts.query(query, "the untracked files")?;
        if !output.success {
            let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            return Err(format!("Git could not read the untracked files: {detail}"));
        }
        for path in replaced_answers(asked.as_slice(), tracked.as_slice(), nul_separated_paths(output.stdout.as_slice()).as_slice()) {
            shared.push(String::from_utf8_lossy(path.as_slice()).into_owned());
        }
    }
    return Ok(shared);
}

/// The tree readings and the answers over them.
#[cfg(test)]
#[path = "repository_facts_untracked_files_tests.rs"]
mod tests;
