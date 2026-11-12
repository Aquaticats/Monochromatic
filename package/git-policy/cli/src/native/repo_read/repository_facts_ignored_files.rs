//! What:
//!  The ignored-and-tracked fact of the real-Git provider:
//!  the worktree's ignored files whose paths a revision's tree also tracks.
//! Why:
//!   A forced `checkout`,
//!  a forced `switch` and a hard `reset` replace an ignored file whose path the tree they
//!       write tracks,
//!  whatever `--no-overwrite-ignore` says,
//!  and an ignored file is where a person keeps
//!       their environment and their local secrets.
//!  The query and its reading live beside the provider,
//!       as `repository_facts_staged_selection.rs` holds the `git add` fact's,
//!  so the provider's own file stays the interface and the order of its answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ignoredFilesTheTreeTracks(facts, 'HEAD') => ['debug.log']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The captured-query runner's output type and its byte helpers.
use crate::git_metadata::MetadataOutput;
/// The tree paths one written revision holds,
///  read beside the provider as this fact reads its own answer.
use super::repository_facts_paths_the_form_writes::tree_paths;
/// The reading of one `git check-ignore` answer over the asked paths.
use crate::rule_worktree_dirty::{
    ignored_paths_from_answer, ignored_paths_queries, replaced_answers, with_ancestors,
};
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::OsStr;

/// What:
///  The worktree's ignored files whose paths the given revision's tree also tracks.
/// Why:
///   The revision the command names is read first,
///  and every form that names no tree writes `HEAD`'s
///       content;
///  a tree that cannot be read is a failure,
///  never an empty answer,
///  because the refusal it earns
///       must not read as clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ignoredFilesTheTreeTracks(facts: GitFacts, revision?: string): Promise<string[]>;
/// ```
pub(super) fn ignored_files_the_tree_tracks(
    facts: &mut GitFacts,
    revision: Option<&OsStr>,
) -> Result<Vec<String>, String> {
    // The revision the command names, when Git can resolve it; every form that names no tree writes `HEAD`'s.
    let named: Option<Vec<Vec<u8>>> = match revision {
        Some(name) => tree_paths(facts, Some(name), &[])?,
        None => None,
    };
    let tracked: Vec<Vec<u8>> = match named {
        Some(paths) => paths,
        None => match tree_paths(facts, Some(OsStr::new("HEAD")), &[])? {
            Some(paths) => paths,
            None => {
                return Err(String::from(
                    "monochromatic-git-policy-cli could not read the tree of HEAD, so the ignored files \
                     cannot be compared with the tree this command writes into.",
                ));
            }
        },
    };
    if tracked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    // A tree creates the directories its paths need, so a file the worktree holds where one of them must go
    //  is lost too: every directory on the way to a tracked path is asked about as well.
    let asked: Vec<Vec<u8>> = with_ancestors(tracked.as_slice());
    // Git answers for these paths alone, so no query walks the worktree's ignored files.
    let mut shared: Vec<String> = Vec::<String>::new();
    for query in ignored_paths_queries(facts.global_prefix.as_slice(), asked.as_slice()) {
        let output: MetadataOutput = facts.query(query, "the ignored files")?;
        for path in replaced_answers(
            asked.as_slice(),
            tracked.as_slice(),
            ignored_paths_from_answer(&output)?.as_slice(),
        ) {
            shared.push(String::from_utf8_lossy(path.as_slice()).into_owned());
        }
    }
    return Ok(shared);
}
