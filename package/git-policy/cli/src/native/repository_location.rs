//! What:
//!  Ask real Git,
//!  in one query,
//!  which repository an invocation selects and where
//!       inside its worktree the command runs.
//! Why:
//!  The configuration root,
//!  the require-root policy and the worktree policies all need
//!      answers Git itself computes from `-C`,
//!  `--git-dir`,
//!  `--work-tree` and its
//!      environment.
//!  One `git rev-parse` prints all of them,
//!  so an invocation starts one
//!      Git process for these facts instead of one per fact,
//!  and the wrapper never
//!      re-implements Git's repository discovery or its `-C` chaining.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { identity, prefix } = parseLocationOutput(await git(locationQueryArguments(globalPrefix)));
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The identity part of the answer is read by the existing identity reader;
///  this
///       module adds the one extra line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseIdentityOutput } from './worktree_identity.ts';
/// ```
use super::git_metadata::{path_from_git_bytes, strip_git_line};
use super::worktree_identity::{IDENTITY_QUERY, WorktreeIdentity, parse_identity_output};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   The caller's global options are replayed to Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  The `rev-parse` option that prints the path from the worktree top level to the
///       directory the command runs in.
///  `&str` is borrowed text baked into the program.
/// Why:
///   Git prints an empty line at the top level,
///  so "is the command run from the
///       root" is Git's own answer,
///  with every `-C` and symbolic link already applied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PREFIX_OPTION = '--show-prefix';
/// ```
pub const PREFIX_OPTION: &str = "--show-prefix";

/// What:
///  Everything the one query answers.
///  A `struct` is a record with named fields;
///       `Vec<u8>` is an owned list of bytes (`u8` is one byte;
///  sibling `String` would
///       require UTF-8).
///  `#[derive(...)]` asks the compiler to generate cloning,
///  debug
///       printing and `==`.
/// Why:
///   The prefix is kept as the bytes Git printed,
///  because a directory name need not
///       be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepositoryLocation = { identity: WorktreeIdentity; prefix: Buffer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryLocation {
    /// Which repository shape the invocation selects.
    pub identity: WorktreeIdentity,
    /// Path from the worktree top level to the command's directory,
    ///  with Git's trailing
    /// slash;
    ///  empty at the top level,
    ///  and empty where the identity has no worktree.
    pub prefix: Vec<u8>,
}

/// Named predicate for finding the last line break;
///  `&u8` borrows one byte.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Interpret the location query's exit state and output.
///  `&[u8]` borrows the raw
///       output;
///  `std::io::Result<T>` is `Result<T, std::io::Error>`,
///  "a value or an
///       input/output error".
/// Why:
///   On success the last line is the prefix and everything before it is the identity
///       answer.
///  On failure Git stopped before the prefix,
///  so the output is the identity
///       answer of a place without a worktree.
///  Any other shape is rejected,
///  not guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLocationOutput(success: boolean, stdout: Buffer): RepositoryLocation; // throws
/// ```
pub fn parse_location_output(success: bool, stdout: &[u8]) -> std::io::Result<RepositoryLocation> {
    if !success {
        // A trailing `?` returns the reader's error to our caller, or unwraps the identity.
        let identity: WorktreeIdentity = parse_identity_output(false, stdout)?;
        // `Ok(...)` is the success variant; `Vec::<u8>::new()` is an empty owned byte list.
        return Ok(RepositoryLocation {
            identity,
            prefix: Vec::<u8>::new(),
        });
    }
    let body: &[u8] = strip_git_line(stdout);
    // `.iter().rposition(f)` is the index of the last byte for which `f` is true, or nothing.
    let Some(last_break) = body.iter().rposition(is_line_feed) else {
        // `Err(...)` is the failure variant; `std::io::Error::other` wraps a message.
        return Err(std::io::Error::other(
            "cli-git could not locate the command inside the Git worktree: git rev-parse \
             printed no line for the path below the top level.",
        ));
    };
    // `&body[..=n]` borrows up to and including index `n`; `&body[n..]` borrows from `n` on.
    let identity: WorktreeIdentity = parse_identity_output(true, &body[..=last_break])?;
    return Ok(RepositoryLocation {
        identity,
        // `.to_vec()` copies the borrowed bytes into an owned list.
        prefix: body[last_break + 1..].to_vec(),
    });
}

/// What:
///  Build the real-Git argument list of the location query:
///  the caller's global
///       options,
///  the identity options,
///  then the prefix option.
///  `&[OsString]` borrows the
///       arguments before the subcommand;
///  `Vec<OsString>` is the owned result.
/// Why:
///   Replaying the complete global prefix lets Git apply `-C` chains through symbolic
///       links,
///  `--git-dir`,
///  `--work-tree` and its environment exactly as it will for the
///       forwarded command.
///  The prefix option is last because Git stops at
///       `--show-toplevel` where there is no worktree,
///  before it would print a prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...globalPrefix, ...IDENTITY_QUERY, '--show-prefix'];
/// ```
pub fn location_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into an owned, growable argument list.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    // `for part in IDENTITY_QUERY` visits each compiled-in option of the identity query.
    for part in IDENTITY_QUERY {
        arguments.push(OsString::from(part));
    }
    arguments.push(OsString::from(PREFIX_OPTION));
    return arguments;
}

/// What:
///  The directory the command runs in:
///  the worktree top level plus the prefix.
///       `&Path` borrows the top level;
///  the result is an owned path.
/// Why:
///   Diagnostics name the directory,
///  and the require-root decision compares it with
///       the top level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const effectiveDirectory = prefix === '' ? root : join(root, prefix);
/// ```
pub fn effective_directory(worktree_root: &Path, prefix: &[u8]) -> PathBuf {
    // Git ends a prefix with a slash; `.strip_suffix` is `Some(shorter)` when it is there,
    // and `.unwrap_or(prefix)` keeps the bytes as they are otherwise.
    let relative_bytes: &[u8] = prefix.strip_suffix(b"/").unwrap_or(prefix);
    // `match` unpacks "a path or nothing": an empty prefix is no path at all.
    match path_from_git_bytes(relative_bytes) {
        // `.join` appends the relative path and returns an owned path.
        Some(relative) => return worktree_root.join(relative),
        // `.to_path_buf()` copies the borrowed path into an owned one.
        None => return worktree_root.to_path_buf(),
    }
}

/// Output-shape and disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_location_tests.rs"]
mod tests;
