//! What:
//!  Pure decision of the require-root policy:
//!  commands run from the repository root.
//! Why:
//!  Running Git from a subdirectory makes pathspecs mean something else.
//!  The policy
//!      rejects that,
//!  except for commands that need no repository.
//!  This file decides from
//!      the argument list alone which commands are exempt,
//!  and from two paths the caller
//!      measured whether the directory is the root.
//!  It reads no file and starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideRequireRoot(args): 'exempt' | 'needs-root'
//! // resolveRequireRoot({ effectiveDirectory, repositoryRoot }): 'pass' | Violation
//! ```

/// What:
///  Bring the `git config` facts and the global-option boundary into this file.
/// Why:
///   The exemptions are keyed on the subcommand word and on `git config`'s file scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseConfigRegion } from './command_config.ts';
/// ```
use super::command_config::{ConfigRegion, parse_config_region};
use super::command_options::OptionError;
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `PathBuf` is an owned file path of raw bytes (sibling `String` must be UTF-8).
use std::path::PathBuf;

/// What:
///  The finding code of a require-root rejection.
///  `&str` is borrowed text baked into
///       the program.
/// Why:
///   Callers and transcripts identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const NOT_AT_ROOT_CODE = 'not-at-root';
/// ```
pub const NOT_AT_ROOT_CODE: &str = "not-at-root";

/// What:
///  Subcommands that create a repository or need none.
///  `&[&[u8]]` is a borrowed list
///       of byte spellings.
/// Why:
///   `init` and `clone` run before a root exists;
///  `version` and `help` read none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXEMPT_SUBCOMMANDS = new Set(['init', 'clone', 'version', 'help']);
/// ```
const EXEMPT_SUBCOMMANDS: &[&[u8]] = &[b"init", b"clone", b"version", b"help"];

/// What:
///  Why a command is exempt.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The caller logs the reason;
///  the decision itself is the same for all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequireRootExemption = 'no-command' | 'subcommand' | 'config-scope';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequireRootExemption {
    /// Git runs no subcommand:
    ///  none was written,
    ///  or Git prints help,
    ///  a version or its own
    /// error for the global options.
    NoCommand,
    /// The subcommand is `init`,
    ///  `clone`,
    ///  `version` or `help`.
    Subcommand,
    /// `git config` on the per-user or system file,
    ///  or listing with `-l`/`--list`.
    ConfigScope,
}

/// What:
///  The outcome of looking at the argument list only.
/// Why:
///   Most commands need a measurement the caller makes;
///  exempt ones never do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequireRootDecision = { kind: 'exempt'; reason: RequireRootExemption } | { kind: 'needs-repository-root' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequireRootDecision {
    /// Forward unchanged without measuring anything.
    Exempt(RequireRootExemption),
    /// The caller must measure the effective directory and the repository root,
    ///  then call
    /// `resolve_require_root`.
    NeedsRepositoryRoot,
}

/// What:
///  The two measurements the decision needs.
///  `Option<PathBuf>` is "a path or nothing".
/// Why:
///   Finding them needs the filesystem and Git's `-C` handling,
///  which stay outside
///       this file.
///  `repository_root` is nothing when the directory is in no repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequireRootFacts = { effectiveDirectory: string; repositoryRoot: string | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequireRootFacts {
    /// The directory the subcommand runs in,
    ///  after every global `-C <path>`.
    pub effective_directory: PathBuf,
    /// The root of the repository that directory is in,
    ///  if any.
    pub repository_root: Option<PathBuf>,
}

/// What:
///  A require-root rejection with its user-facing text.
///  `String` is owned text.
/// Why:
///   The text names both directories and both ways forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequireRootViolation = { code: 'not-at-root'; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequireRootViolation {
    /// Complete diagnostic for the person who ran the command.
    pub message: String,
}

/// What:
///  The outcome once the two paths are known.
/// Why:
///   Either the command is forwarded unchanged or it is rejected with a message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequireRootVerdict = { kind: 'pass' } | { kind: 'not-at-root'; violation: RequireRootViolation };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequireRootVerdict {
    /// Outside any repository,
    ///  or at the root:
    ///  forward unchanged.
    Pass,
    /// Inside a repository below its root.
    NotAtRoot(RequireRootViolation),
}

/// What:
///  Whether a `git config` region works on a file outside the repository or lists.
///       `&[OsString]` borrows the tokens after `config`.
/// Why:
///   Those forms need no repository root.
///  A region Git itself refuses earns no
///       exemption,
///  so the command is treated like any other.
/// Gotcha:
///  Divergence from the incumbent,
///  which exempted on these spellings anywhere in the
///         argument list (`git config user.name --global` stores a value in the
///         repository) and missed `git config list`,
///  the 2.56.0 spelling of `--list`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exempt = region.global || region.system || region.lists;
/// ```
fn config_is_exempt(region: &[OsString], wrapper_flags: &[&[u8]]) -> bool {
    let parsed: Result<ConfigRegion, OptionError> = parse_config_region(region, wrapper_flags);
    // `if let Ok(found) = ...` runs only for a region Git accepts.
    if let Ok(found) = parsed {
        return found.global || found.system || found.lists;
    }
    return false;
}

/// What:
///  Decide from the argument list whether the command is exempt.
///  `wrapper_flags` are
///       the caller's wrapper-only spellings,
///  which Git must not see as unknown options.
/// Why:
///   An exempt command is forwarded without touching the filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideRequireRoot(args: string[], wrapperFlags: string[]): RequireRootDecision;
/// ```
pub fn decide_require_root(arguments: &[OsString], wrapper_flags: &[&[u8]]) -> RequireRootDecision {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return RequireRootDecision::Exempt(RequireRootExemption::NoCommand);
    }
    // `.as_encoded_bytes()` borrows the raw bytes of the subcommand word.
    let word: &[u8] = arguments[layout.prefix_len].as_encoded_bytes();
    if EXEMPT_SUBCOMMANDS.contains(&word) {
        return RequireRootDecision::Exempt(RequireRootExemption::Subcommand);
    }
    // `&arguments[n..]` borrows the tokens after the subcommand word.
    if word == b"config" && config_is_exempt(&arguments[layout.prefix_len + 1..], wrapper_flags) {
        return RequireRootDecision::Exempt(RequireRootExemption::ConfigScope);
    }
    return RequireRootDecision::NeedsRepositoryRoot;
}

/// What:
///  Decide from the measured paths whether to forward or reject.
///  `&RequireRootFacts`
///       borrows the measurements read-only.
/// Why:
///   Outside a repository Git reports its own error if the subcommand needs one.
/// Gotcha:
///  Paths compare by components,
///  so `/repo/` and `/repo` are the same directory;
///         symbolic links are the caller's concern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRequireRoot(facts: RequireRootFacts): RequireRootVerdict;
/// ```
pub fn resolve_require_root(facts: &RequireRootFacts) -> RequireRootVerdict {
    // `match &facts.repository_root` borrows the optional path without moving it.
    let root: &PathBuf = match &facts.repository_root {
        Some(found) => found,
        None => return RequireRootVerdict::Pass,
    };
    if *root == facts.effective_directory {
        return RequireRootVerdict::Pass;
    }
    // `.display()` shows a path as text, replacing bytes that are not UTF-8.
    let message: String = format!(
        "cli-git: not at the root of the git repository. \
         Repo root is {root} but effective cwd is {directory}. \
         This rule exists because agents and humans frequently forget which directory they are in. \
         Tip: cd to {root} or pass -C {root} before the subcommand.",
        root = root.display(),
        directory = facts.effective_directory.display(),
    );
    return RequireRootVerdict::NotAtRoot(RequireRootViolation { message });
}

/// Exemptions and verdicts,
///  including every case of the incumbent's unit tests.
#[cfg(test)]
#[path = "rule_require_root_tests.rs"]
mod tests;
