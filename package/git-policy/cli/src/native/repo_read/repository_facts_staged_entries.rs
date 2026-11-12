//! What:
//!  The listed paths whose staged content an abort would discard,
//!  whether or not the worktree still holds a file at them.
//! Why:
//!  An abort resets the index as well as the worktree,
//!  so the content the index holds at a listed path is work it discards.
//!  The save step copies worktree files and nothing else,
//!  and a path with no worktree file was reported as having nothing to copy,
//!  which read as clean for content that stood only in the index.
//!  A file the worktree still holds is no better off when it differs from the index:
//!  a resolution staged and then edited again leaves two things a person made,
//!  and the abort drops the staged one whatever the worktree holds.
//!  `git ls-files --stage -z` names the paths whose index entry is whole (stage 0),
//!  which is the entry a reset drops;
//!  an unresolved conflict's own stages are the operation's state and not the person's work.
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

/// Whether one byte ends a `-z` record.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte separates the header's words from a record's path.
fn is_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// Whether one byte separates the words of a `--stage` header.
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
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
    for record in answer.split(is_nul) {
        if record.is_empty() {
            continue;
        }
        let Some(tab) = record.iter().position(is_tab) else {
            continue;
        };
        let mut words = record[..tab].split(is_space);
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
///  The paths whose index holds a whole entry (stage 0),
///  among the ones the caller already found the abort would discard.
/// Why:
///   Those entries are the content a reset drops,
///  and no copy of a worktree file holds them:
///  the save step copies each of them from the index itself.
///  The answer is one settled order and each path once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stagedEntries(facts, listedPaths) => listedPaths with a stage-zero entry;
/// ```
pub(super) fn staged_entries(facts: &mut GitFacts, paths: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    if paths.is_empty() {
        return Ok(Vec::<Vec<u8>>::new());
    }
    let staged: MetadataOutput = listing(facts, "--stage", paths, "the index's entries")?;
    let present: BTreeSet<Vec<u8>> = staged_paths(staged.stdout.as_slice()).into_iter().collect();
    let listed: BTreeSet<Vec<u8>> = paths.iter().cloned().collect();
    return Ok(present.intersection(&listed).cloned().collect());
}

/// The listing and its reading.
#[cfg(test)]
#[path = "repository_facts_staged_entries_tests.rs"]
mod tests;
