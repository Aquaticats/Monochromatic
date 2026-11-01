//! What:
//!  The pure decision of the commit-only transform:
//!  insert `-o` after `git commit`,
//!  or
//!       say why not.
//! Why:
//!  Every commit must name the paths it includes.
//!  Two decisions need a repository fact
//!      (sequencer state,
//!  index against `HEAD`);
//!  this module asks for the fact by returning
//!      a `Needs...` variant and the caller answers through a `resolve_...` function,
//!  so no
//!      process or file is touched here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const rule = makeCommitOnly({ checkIndexDiffersFromHead, checkSequencerInProgress });
//! ```

/// What:
///  Bring the commit facts,
///  the global-option boundary,
///  the tokenizer helpers and the
///       repository-fact types into this file.
/// Why:
///   The decision combines them;
///  it owns no parsing of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseCommitRegion } from '../parser/commit.ts';
/// import { parseGlobalOptions } from '../parse-global-options.ts';
/// ```
use super::command_commit::{CommitRegion, FixupKind, parse_commit_region};
use super::command_options::OptionError;
use super::command_options_query::without_tokens;
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::rule_argument_rewrite::insert_tokens;
use super::rule_commit_index::IndexVsHead;
use super::rule_commit_only_message::{
    ALL_FLAG_MESSAGE, NO_PATHSPEC_MESSAGE, ignored_index_message,
};
use super::rule_commit_sequencer::SequencerState;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Stable rejection codes.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Events carry `commit-only/<code>`;
///  consumers match the code,
///  not the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitOnlyViolationCode = 'all-flag' | 'pathspec-required' | 'staged-changes-ignored';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitOnlyViolationCode {
    /// `-a`/`--all` stages every tracked modification implicitly.
    AllFlag,
    /// No pathspec source and no mode that permits a pathless only-commit.
    PathspecRequired,
    /// A pathless `--amend`/`--allow-empty` commit would ignore a dirty index.
    StagedChangesIgnored,
}

/// What:
///  The kebab-case spelling of a code.
///  `&'static str` is text baked into the program.
/// Why:
///   The event format predates this module and its spellings must not drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = code; // the union member already is the string
/// ```
pub fn violation_code_text(code: CommitOnlyViolationCode) -> &'static str {
    // `match` picks one arm per variant; the compiler refuses a forgotten variant.
    return match code {
        CommitOnlyViolationCode::AllFlag => "all-flag",
        CommitOnlyViolationCode::PathspecRequired => "pathspec-required",
        CommitOnlyViolationCode::StagedChangesIgnored => "staged-changes-ignored",
    };
}

/// What:
///  One expected rejection.
///  `String` is owned UTF-8 text (sibling `&str` borrows).
/// Why:
///   The message is built per invocation,
///  so the violation owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitOnlyViolationError extends Error { readonly code: CommitOnlyViolationCode }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitOnlyViolation {
    /// Stable code.
    pub code: CommitOnlyViolationCode,
    /// Complete user-facing explanation naming every way forward.
    pub message: String,
}

/// What:
///  What `resolve_index_state` needs to finish the decision.
/// Why:
///   The first call already located `commit` and the pathless-allowed flags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingInjection = { commandIndex: number; flagText: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingInjection {
    /// Index of the `commit` token in the full argument list.
    pub command_index: usize,
    /// The pathless-allowed flags present,
    ///  echoed in the diagnostic.
    pub flag_text: String,
}

/// What:
///  The transform's answer,
///  or the repository fact it needs first.
/// Why:
///   The facts are fetched lazily:
///  most commands are decided from arguments alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitOnlyDecision = { kind: 'unchanged' } | { kind: 'rewritten'; args: string[] }
///   | { kind: 'rejected'; violation } | { kind: 'needs-sequencer-state' } | { kind: 'needs-index-state'; pending };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommitOnlyDecision {
    /// Forward the caller's arguments as they are.
    Unchanged,
    /// Forward this list:
    ///  `-o` inserted,
    ///  or the escape hatch removed.
    Rewritten(Vec<OsString>),
    /// Block the command with this finding.
    Rejected(CommitOnlyViolation),
    /// Ask whether a merge,
    ///  cherry-pick or revert awaits its commit,
    ///  then call
    /// `resolve_sequencer_state`.
    NeedsSequencerState,
    /// Ask whether the index differs from `HEAD`,
    ///  then call `resolve_index_state`.
    NeedsIndexState(PendingInjection),
}

/// What:
///  Build a rejection from a code and its message text.
/// Why:
///   `String::from` copies the borrowed text into storage the violation owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new CommitOnlyViolationError(code, message)
/// ```
fn rejected(code: CommitOnlyViolationCode, message: &str) -> CommitOnlyDecision {
    return CommitOnlyDecision::Rejected(CommitOnlyViolation {
        code,
        message: String::from(message),
    });
}

/// What:
///  The pathless-allowed flags of a region as diagnostic text,
///  such as `--amend`.
/// Why:
///   The rejection names the exact form that was refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flagText = [amend && '--amend', allowEmpty && '--allow-empty'].filter(Boolean).join(' ');
/// ```
fn pathless_flag_text(region: &CommitRegion) -> String {
    let mut parts: Vec<&str> = Vec::<&str>::new();
    if region.amend {
        parts.push("--amend");
    }
    if region.allow_empty {
        parts.push("--allow-empty");
    }
    if region.fixup == Some(FixupKind::Amend) {
        parts.push("--fixup=amend:<commit>");
    }
    // `.join(" ")` builds one owned `String` with spaces between the parts.
    return parts.join(" ");
}

/// What:
///  Decide from arguments alone,
///  or name the repository fact needed.
///       `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   The order mirrors the TypeScript rule:
///  escape hatch,
///  opt-out,
///  `-a`,
///  pathspec
///       source,
///  explicit only,
///  modes Git owns,
///  then the dirty-index guard.
/// Gotcha:
///  `Err` means Git 2.56.0 itself refuses the commit options;
///  the caller forwards
///         such a command unchanged so Git prints its own error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitOnly(args: readonly string[]): Promise<readonly string[]>; // throws
/// ```
pub fn decide_commit_only(
    arguments: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<CommitOnlyDecision, OptionError> {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command
        || arguments[layout.prefix_len].as_encoded_bytes() != b"commit"
    {
        // `Ok(x)` is the success case of `Result`.
        return Ok(CommitOnlyDecision::Unchanged);
    }
    let region_start: usize = layout.prefix_len + 1;
    // `?` returns Git's refusal to our caller, or unwraps the facts.
    let region: CommitRegion = parse_commit_region(&arguments[region_start..], wrapper_flags)?;
    if !region.wrapper.escape.is_empty() {
        // Only option-position hatch tokens go; an equal message or path stays.
        return Ok(CommitOnlyDecision::Rewritten(without_tokens(
            arguments,
            region_start,
            region.wrapper.escape.as_slice(),
        )));
    }
    let fixup_owns_only: bool = region.fixup == Some(FixupKind::Reword)
        || region.fixup == Some(FixupKind::UnknownSuboption);
    let pathless_allowed: bool =
        region.amend || region.allow_empty || region.fixup == Some(FixupKind::Amend);
    let has_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    // `--no-only` is the explicit choice to commit the whole index.
    if region.only != Some(false) {
        if region.all {
            return Ok(rejected(CommitOnlyViolationCode::AllFlag, ALL_FLAG_MESSAGE));
        }
        let has_source: bool =
            has_paths || pathless_allowed || fixup_owns_only || region.interactive || region.patch;
        if !has_source {
            return Ok(CommitOnlyDecision::NeedsSequencerState);
        }
    }
    if region.only.is_some() {
        return Ok(CommitOnlyDecision::Unchanged);
    }
    // Git forbids `--only` with these, or turns it on itself for `--fixup=reword:`.
    if region.include || region.interactive || region.patch || fixup_owns_only {
        return Ok(CommitOnlyDecision::Unchanged);
    }
    if pathless_allowed && !has_paths {
        return Ok(CommitOnlyDecision::NeedsIndexState(PendingInjection {
            command_index: layout.prefix_len,
            flag_text: pathless_flag_text(&region),
        }));
    }
    return Ok(CommitOnlyDecision::Rewritten(insert_tokens(
        arguments,
        region_start,
        &["-o"],
    )));
}

/// What:
///  Finish a `NeedsSequencerState` decision.
/// Why:
///   A pathless commit concludes a merge,
///  cherry-pick or revert;
///  otherwise it is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (sequencerState === 'in-progress') return args; throw new CommitOnlyViolationError('pathspec-required', ...);
/// ```
pub fn resolve_sequencer_state(state: SequencerState) -> CommitOnlyDecision {
    if state == SequencerState::InProgress {
        return CommitOnlyDecision::Unchanged;
    }
    return rejected(
        CommitOnlyViolationCode::PathspecRequired,
        NO_PATHSPEC_MESSAGE,
    );
}

/// What:
///  Finish a `NeedsIndexState` decision.
///  `&PendingInjection` borrows the pending data.
/// Why:
///   An injected `--only` on a pathless amend or empty commit reuses `HEAD`'s tree,
///  so a
///       dirty index would be ignored silently;
///  an equal or unknown index is safe to inject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (indexState === 'differs') throw new CommitOnlyViolationError('staged-changes-ignored', ...);
/// return [...args.slice(0, i + 1), '-o', ...args.slice(i + 1)];
/// ```
pub fn resolve_index_state(
    arguments: &[OsString],
    pending: &PendingInjection,
    state: IndexVsHead,
) -> CommitOnlyDecision {
    if state == IndexVsHead::Differs {
        // `.as_str()` lends the owned text as a borrowed view.
        return rejected(
            CommitOnlyViolationCode::StagedChangesIgnored,
            ignored_index_message(pending.flag_text.as_str()).as_str(),
        );
    }
    return CommitOnlyDecision::Rewritten(insert_tokens(
        arguments,
        pending.command_index + 1,
        &["-o"],
    ));
}

/// Every `commit-only.unit.test.ts` case.
#[cfg(test)]
#[path = "rule_commit_only_tests.rs"]
mod tests;

/// Cases beyond the incumbent test,
///  including readings where Git differs from it.
#[cfg(test)]
#[path = "rule_commit_only_divergence_tests.rs"]
mod divergence_tests;

/// Real Git 2.56.0 controls for the Git behavior the decision relies on.
#[cfg(test)]
#[path = "rule_commit_only_git_tests.rs"]
mod git_tests;
