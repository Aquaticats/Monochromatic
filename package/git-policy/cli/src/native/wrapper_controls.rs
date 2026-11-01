//! What:
//!  The wrapper-only control tokens,
//!  what each one means,
//!  and their removal from the
//!       arguments that come before the Git subcommand.
//! Why:
//!  Git refuses options it does not know,
//!  so every control must be gone before Git runs,
//!      and every policy decision must read the arguments without them.
//!  A control is
//!      recognized only where Git itself would look for an option,
//!  and it is removed by its
//!      position,
//!  never by its spelling,
//!  so an equal-looking value or path is kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { args, keepGoing, escapedPolicyIds } = parsePolicyControls({ args: rawArgs });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The scan asks the Git 2.56.0 global-option reader where an unknown option sits,
///       and removes tokens through the one position-based removal function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { withoutTokens } from './command_options_query.ts';
/// import { globalLayout } from './global_arguments.ts';
/// ```
use super::command_add::ADD_ESCAPE_HATCH;
use super::command_options_query::without_tokens;
use super::escape_hatch::{
    BRANCH_WORKTREE_ESCAPE_HATCH, WORKTREE_COPY_ESCAPE_HATCH, WORKTREE_ENFORCEMENT_ESCAPE_HATCH,
};
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::policy_registry::PolicyId;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments may hold bytes that are not UTF-8 and are forwarded unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The control that lets later policies run after an error finding.
///  `&str` is borrowed
///       text;
///  here it is baked into the program.
/// Why:
///   A caller who wants every finding of one command in a single run passes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEEP_GOING_FLAG = '--cli-git-keep-going';
/// ```
pub const KEEP_GOING_FLAG: &str = "--cli-git-keep-going";

/// What:
///  What one control asks for.
///  An `enum` is a closed set of named alternatives;
///       `Escape` carries the policy it switches off.
///  `#[derive(...)]` asks the compiler to
///       generate copying (`Clone`,
///  `Copy`),
///  debug printing (`Debug`) and `==`.
/// Why:
///   Recognition (a spelling) and effect (a meaning) are kept apart,
///  so two spellings
///       of one escape cannot drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ControlMeaning = { kind: 'keep-going' } | { kind: 'escape'; policy: PolicyId } | { kind: 'skip-worktree-copy' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlMeaning {
    /// Continue with later policies after an error finding.
    KeepGoing,
    /// Skip one policy for this invocation.
    Escape(PolicyId),
    /// Skip ignored-state synchronization for this invocation.
    SkipWorktreeCopy,
}

/// What:
///  One accepted control:
///  its exact spelling and its meaning.
///  A `struct` is a record
///       with named fields;
///  `&'static str` is text baked into the program for its whole run.
/// Why:
///   The table below is the single list of spellings the wrapper recognizes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ControlSpelling = { flag: string; meaning: ControlMeaning };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlSpelling {
    /// The exact token;
    ///  controls are never abbreviated and never carry `=value`.
    pub flag: &'static str,
    /// What the token asks for.
    pub meaning: ControlMeaning,
}

/// What:
///  Every control spelling.
///  `&[ControlSpelling]` is a borrowed,
///  read-only list (siblings:
///       `Vec<T>`,
///  owned and growable;
///  `[T; N]`,
///  fixed length in the type).
/// Why:
///   One `--no-enforce-<policy id>` per shipped policy,
///  the three spellings that predate
///       policy ids,
///  the keep-going flag and the worktree-copy opt-out.
///  A compiled-in list
///       cannot be reordered or extended at run time.
/// Gotcha:
///  `wrapper_controls_tests.rs` pins that every shipped policy has exactly the
///         spelling `--no-enforce-` followed by its registry name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONTROL_SPELLINGS: readonly ControlSpelling[] = [ ... ];
/// ```
pub const CONTROL_SPELLINGS: &[ControlSpelling] = &[
    ControlSpelling {
        flag: KEEP_GOING_FLAG,
        meaning: ControlMeaning::KeepGoing,
    },
    ControlSpelling {
        flag: "--no-enforce-require-root",
        meaning: ControlMeaning::Escape(PolicyId::RequireRoot),
    },
    ControlSpelling {
        flag: "--no-enforce-linked-worktree-only",
        meaning: ControlMeaning::Escape(PolicyId::LinkedWorktreeOnly),
    },
    ControlSpelling {
        flag: "--no-enforce-branch-worktree-only",
        meaning: ControlMeaning::Escape(PolicyId::BranchWorktreeOnly),
    },
    ControlSpelling {
        flag: "--no-enforce-add-explicit",
        meaning: ControlMeaning::Escape(PolicyId::AddExplicit),
    },
    ControlSpelling {
        flag: "--no-enforce-final-newline",
        meaning: ControlMeaning::Escape(PolicyId::FinalNewline),
    },
    ControlSpelling {
        flag: "--no-enforce-markdown/autofix",
        meaning: ControlMeaning::Escape(PolicyId::MarkdownAutofix),
    },
    ControlSpelling {
        flag: "--no-enforce-mono/forbidden-root-context",
        meaning: ControlMeaning::Escape(PolicyId::ForbiddenRootContext),
    },
    ControlSpelling {
        flag: "--no-enforce-mono/dependent-version-bump",
        meaning: ControlMeaning::Escape(PolicyId::DependentVersionBump),
    },
    ControlSpelling {
        flag: "--no-enforce-security/forbidden-strings",
        meaning: ControlMeaning::Escape(PolicyId::ForbiddenStrings),
    },
    ControlSpelling {
        flag: WORKTREE_ENFORCEMENT_ESCAPE_HATCH,
        meaning: ControlMeaning::Escape(PolicyId::LinkedWorktreeOnly),
    },
    ControlSpelling {
        flag: BRANCH_WORKTREE_ESCAPE_HATCH,
        meaning: ControlMeaning::Escape(PolicyId::BranchWorktreeOnly),
    },
    ControlSpelling {
        flag: ADD_ESCAPE_HATCH,
        meaning: ControlMeaning::Escape(PolicyId::AddExplicit),
    },
    ControlSpelling {
        flag: WORKTREE_COPY_ESCAPE_HATCH,
        meaning: ControlMeaning::SkipWorktreeCopy,
    },
];

/// What:
///  Everything the controls of one invocation asked for.
///  `bool` is true or false;
///       `Vec<PolicyId>` is an owned,
///  growable list of policy identities.
/// Why:
///   The engine reads the effect of the removed tokens from this record,
///  because the
///       tokens themselves are gone from the arguments it sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Controls = { keepGoing: boolean; escaped: PolicyId[]; commitOnlyEscaped: boolean; skipWorktreeCopy: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Controls {
    /// Later policies run after an error finding.
    pub keep_going: bool,
    /// Policies skipped for this invocation,
    ///  each listed once,
    ///  in first-use order.
    pub escaped: Vec<PolicyId>,
    /// `--no-enforce-only` was written in option position of `git commit`.
    pub commit_only_escaped: bool,
    /// Ignored-state synchronization is skipped.
    pub skip_worktree_copy: bool,
}

/// What:
///  The record of an invocation that used no control.
///  `Vec::<PolicyId>::new()` is an
///       empty owned list.
/// Why:
///   Every scan starts from "nothing was asked" and only adds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noControls = (): Controls => ({ keepGoing: false, escaped: [], commitOnlyEscaped: false, skipWorktreeCopy: false });
/// ```
pub fn no_controls() -> Controls {
    return Controls {
        keep_going: false,
        escaped: Vec::<PolicyId>::new(),
        commit_only_escaped: false,
        skip_worktree_copy: false,
    };
}

/// What:
///  The meaning of a token that is exactly a control spelling,
///  or nothing.
///  `&[u8]`
///       borrows raw bytes;
///  `Option<T>` is "a value or nothing".
/// Why:
///   Arguments are compared as bytes,
///  so a non-UTF-8 argument is never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const meaning = CONTROL_SPELLINGS.find(s => s.flag === token)?.meaning;
/// ```
pub fn control_meaning(token: &[u8]) -> Option<ControlMeaning> {
    // `for spelling in CONTROL_SPELLINGS` borrows each row in table order.
    for spelling in CONTROL_SPELLINGS {
        // `.as_bytes()` lends the spelling's bytes for the comparison.
        if spelling.flag.as_bytes() == token {
            // `Some(x)` is the "present" case of `Option`.
            return Some(spelling.meaning);
        }
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  The control spellings as byte strings,
///  in table order.
///  `Vec<&'static [u8]>` is an
///       owned list of borrowed byte strings baked into the program.
/// Why:
///   Every Git option-table reader takes this list,
///  so it reports a control found in
///       option position instead of refusing it as an unknown option.
///  The position in this
///       list is the position in `CONTROL_SPELLINGS`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flags = CONTROL_SPELLINGS.map(s => s.flag);
/// ```
pub fn control_flags() -> Vec<&'static [u8]> {
    // `with_capacity(n)` reserves room for `n` items; `mut` allows pushing.
    let mut flags: Vec<&'static [u8]> =
        Vec::<&'static [u8]>::with_capacity(CONTROL_SPELLINGS.len());
    for spelling in CONTROL_SPELLINGS {
        flags.push(spelling.flag.as_bytes());
    }
    return flags;
}

/// What:
///  Add one control's effect to the record.
///  `&mut Controls` lends the record for
///       writing (sibling `&Controls` lends it read-only).
/// Why:
///   A repeated escape must not list its policy twice:
///  the list is compared in tests
///       and printed in diagnostics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordControl(controls: Controls, meaning: ControlMeaning): void;
/// ```
pub fn record_control(controls: &mut Controls, meaning: ControlMeaning) {
    // `match` picks one arm per variant and unpacks the policy an `Escape` carries; the
    // compiler refuses the code if a variant is forgotten.
    match meaning {
        ControlMeaning::KeepGoing => controls.keep_going = true,
        ControlMeaning::SkipWorktreeCopy => controls.skip_worktree_copy = true,
        ControlMeaning::Escape(policy) => {
            // `.contains(&policy)` borrows the identity for the lookup.
            if !controls.escaped.contains(&policy) {
                controls.escaped.push(policy);
            }
        }
    }
}

/// What:
///  Whether a policy is skipped for this invocation.
/// Why:
///   The engine asks by typed identity instead of scanning the list itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escaped = controls.escaped.includes(policy);
/// ```
pub fn is_escaped(controls: &Controls, policy: PolicyId) -> bool {
    return controls.escaped.contains(&policy);
}

/// What:
///  Copy the arguments without the controls written before the subcommand,
///  adding
///       their effect to `controls`.
///  `&[OsString]` borrows the argument list;
///  `Vec<OsString>`
///       is the owned result.
/// Why:
///   Git 2.56.0's global-option reader stops at the first option it does not know.
///       When that option is exactly a control,
///  it sits where Git would have read an
///       option,
///  so it is removed by its position and the reader is asked again.
///  A value
///       of `-C`,
///  `-c` or another value-taking option is skipped by that reader and is
///       therefore never removed,
///  even when it spells a control.
/// Gotcha:
///  Each round removes one token,
///  so the loop runs at most once per argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stripGlobalControls(args: string[], controls: Controls): string[];
/// ```
pub fn strip_global_controls(arguments: &[OsString], controls: &mut Controls) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed list into an owned one that later rounds replace.
    let mut current: Vec<OsString> = arguments.to_vec();
    // `0..arguments.len()` counts the rounds; `_round` is unused because only the count matters.
    for _round in 0..arguments.len() {
        // `.as_slice()` lends the owned list as a borrowed view.
        let layout: GlobalLayout = global_layout(current.as_slice());
        if layout.outcome != GlobalOutcome::InvalidOption {
            break;
        }
        // `.as_encoded_bytes()` lends the raw bytes of the unknown option.
        let unknown: &[u8] = current[layout.prefix_len].as_encoded_bytes();
        // `let Some(x) = ... else { ... };` unwraps the meaning or leaves the loop.
        let Some(meaning) = control_meaning(unknown) else {
            break;
        };
        record_control(controls, meaning);
        // `&[layout.prefix_len]` is a one-item borrowed list of positions to drop.
        current = without_tokens(current.as_slice(), 0, &[layout.prefix_len]);
    }
    return current;
}

/// Spelling,
///  meaning and global-prefix controls stay out of the release executable.
#[cfg(test)]
#[path = "wrapper_controls_tests.rs"]
mod tests;
