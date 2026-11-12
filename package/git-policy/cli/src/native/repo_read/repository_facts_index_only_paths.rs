//! What:
//!  The listed paths whose only content is the index,
//!  because the worktree no longer holds a file at them.
//! Why:
//!  An abort resets the index as well as the worktree,
//!  so a path the worktree lost while the index kept its staged content is work an
//!  abort would discard.
//!  The save step copies worktree files and nothing else,
//!  and a path with no worktree file was reported as having nothing to copy,
//!  which read as clean for content that stood only in the index.
//!  Two Git listings answer it:
//!  `git ls-files --stage -z` names the paths with an index entry,
//!  and `git ls-files --deleted -z` the paths whose worktree file is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // indexOnlyPaths(facts, ['new.txt', 'a.txt']) => ['new.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the two listings.
use crate::repository_facts::GitFacts;
/// The captured-query runner's output type,
///  the path of raw Git bytes and its reading.
use crate::git_metadata::{MetadataOutput, path_from_git_bytes};
/// The reading of one `-z` listing of paths.
use crate::rule_worktree_dirty::nul_separated_paths;
use std::collections::BTreeSet;
use std::ffi::OsString;

/// What:
///  The argument list of one `git ls-files` listing over exactly these paths.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///  `:(top,literal)` makes Git match a repository-relative name byte for byte from any directory,
///  whatever wildcard or magic characters it holds,
///  and `--` ends the options,
///  so a path that looks like one is read as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...pre, 'ls-files', '--stage', '-z', '--', ':(top,literal)new.txt'];
/// ```
fn listing_arguments(global_prefix: &[OsString], option: &str, paths: &[Vec<u8>]) -> Result<Vec<OsString>, String> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["ls-files", option, "-z", "--"] {
        arguments.push(OsString::from(token));
    }
    for path in paths {
        let Some(native) = path_from_git_bytes(path.as_slice()) else {
            return Err(format!("could not read the path {} as a path here.", String::from_utf8_lossy(path.as_slice())));
        };
        let mut literal: OsString = OsString::from(":(top,literal)");
        literal.push(native.as_os_str());
        arguments.push(literal);
    }
    return Ok(arguments);
}

/// What:
///  The paths whose index entry is at stage 0,
///  from one `git ls-files --stage -z` answer.
///  `&[u8]` borrows the query's standard output.
/// Why:
///  Each record is `mode SP oid SP stage TAB path`,
///  and `-z` keeps a path's bytes exactly,
///  a tab inside it included,
///  because the header's first tab is the one that ends the header.
///  Stage 0 is the entry Git resets;
///  the other stages are an unresolved conflict's own state,
///  which is not the person's work to copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stagedPaths('100644 abc 0\tnew.txt\0') === ['new.txt'];
/// ```
fn staged_paths(answer: &[u8]) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for record in answer.split(|byte| { return *byte == 0; }) {
        if record.is_empty() {
            continue;
        }
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            continue;
        };
        let mut words = record[..tab].split(|byte| { return *byte == b' '; });
        let (Some(_mode), Some(_oid), Some(stage)) = (words.next(), words.next(), words.next()) else {
            continue;
        };
        if stage == b"0" {
            found.push(record[tab + 1..].to_vec());
        }
    }
    return found;
}

/// What:
///  One listing's answer over `paths`,
///  or why Git could not give it.
/// Why:
///  A listing that failed says nothing,
///  and a safety check must never read a failure as "no path lost its worktree file".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listing(facts: GitFacts, option: string, paths: Buffer[], subject: string): MetadataOutput; // throws
/// ```
fn listing(facts: &mut GitFacts, option: &str, paths: &[Vec<u8>], subject: &str) -> Result<MetadataOutput, String> {
    let arguments: Vec<OsString> = listing_arguments(facts.global_prefix.as_slice(), option, paths)?;
    let answer: MetadataOutput = facts.query(arguments, subject)?;
    if !answer.success {
        let detail: String = String::from_utf8_lossy(answer.stderr.as_slice())
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("could not read {subject}: {detail}"));
    }
    return Ok(answer);
}

/// What:
///  The paths whose index entry holds content while the worktree holds no file at the path.
/// Why:
///   The abort resets the index too,
///  so that content is discarded,
///  and the save step cannot copy it from a worktree file that is not there:
///  the guard refuses the command naming these paths instead.
///  The answer is one settled order and each path once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// indexOnlyPaths(facts, paths) => present ∩ missing;
/// ```
pub(super) fn index_only_paths(facts: &mut GitFacts, paths: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    if paths.is_empty() {
        return Ok(Vec::<Vec<u8>>::new());
    }
    let staged: MetadataOutput = listing(facts, "--stage", paths, "the index's entries")?;
    let deleted: MetadataOutput = listing(facts, "--deleted", paths, "the files the worktree no longer holds")?;
    let present: BTreeSet<Vec<u8>> = staged_paths(staged.stdout.as_slice()).into_iter().collect();
    let missing: BTreeSet<Vec<u8>> = nul_separated_paths(deleted.stdout.as_slice()).into_iter().collect();
    return Ok(present.intersection(&missing).cloned().collect());
}

/// The listings and their readings.
#[cfg(test)]
#[path = "repository_facts_index_only_paths_tests.rs"]
mod tests;
