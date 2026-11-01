//! What:
//!  Pure decision of the linked-worktree-only policy:
//!  commands that can discard uncommitted state never run in the main worktree, and run in a linked worktree
//!      only while it is clean.
//! Why:
//!  `git stash` that changes state,
//!  a deleting `git clean`,
//!  a `git reset` that is not `--soft`,
//!  `git restore`,
//!  `git checkout` and `git switch` forms that overwrite files,
//!  a forced `rm`,
//!  `mv`,
//!  or `checkout-index`,
//!  `read-tree --reset` and an `--abort` can throw away work, and the loss leaves no record.
//!  This file decides from the arguments alone whether the command is guarded and when,
//!  and from the worktree kind and dirtiness the caller measured whether to reject it.
//!  It starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideLinkedWorktree(['clean', '-fd']) => { kind: 'needs-effective-target', command: 'clean' }
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Whether a clean deletes and whether a reset rewrites files come from Git's own
///       option tables of those commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { cleanChangesWorktree, parseCleanRegion } from './command_clean.ts';
/// ```
use super::command_clean::{CleanRegion, clean_changes_worktree, parse_clean_region};
use super::command_discard::{
    Discard, abort_discard, checkout_discard, force_discard, read_tree_discard, restore_discard,
    restores_index_only, switch_discard,
};
use super::command_options::OptionError;
use super::command_reset::{ResetMode, ResetRegion, parse_reset_region};
use super::command_stash::{parse_stash_region, stash_changes_state};
use super::effective_target::EffectiveTarget;
use super::global_arguments::command_tokens;
use super::rule_worktree_dirty::DirtyScope;
pub use super::rule_linked_worktree_messages::{
    dirty_worktree_message, main_worktree_message, outside_worktree_message,
};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments are compared as bytes and never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The finding code of a linked-worktree rejection.
///  `&str` is borrowed text baked
///       into the program.
/// Why:
///   Callers identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINKED_WORKTREE_REQUIRED_CODE = 'linked-worktree-required';
/// ```
pub const LINKED_WORKTREE_REQUIRED_CODE: &str = "linked-worktree-required";

/// What:
///  The forms the policy guards.
///  An `enum` is a closed set of named alternatives.
///       `#[derive(...)]` asks the compiler to generate copying,
///  debug printing and `==`.
/// Why:
///   Each form has its own rejection text, because the same subcommand serves different intents:
///  `restore --staged` unstages while `restore <path>` reverts a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardedCommand = 'stash' | 'clean' | 'reset-hard' | 'reset-mixed' | 'restore-staged' | 'restore-worktree'
///   | 'checkout-paths' | 'checkout-move' | 'switch-force' | 'switch-move' | 'rm' | 'mv' | 'checkout-index'
///   | 'read-tree' | 'abort-merge' | 'abort-rebase' | 'abort-pick' | 'abort-am';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardedCommand {
    /// Every `git stash` form that changes state.
    Stash,
    /// A `git clean` that is not a dry run.
    Clean,
    /// A `git reset` with `--hard`,
    ///  `--merge` or `--keep`.
    ResetHard,
    /// A `git reset` in the default or `--mixed` mode.
    ResetMixed,
    /// A `git restore` that names the index and not the worktree.
    RestoreStaged,
    /// A `git restore` that writes worktree files.
    RestoreWorktree,
    /// A `git checkout` with paths, `--force`, `--patch`, `--ours` or `--theirs`.
    CheckoutPaths,
    /// A `git checkout` that moves `HEAD`.
    CheckoutMove,
    /// A `git switch` with `--force` or `--discard-changes`.
    SwitchForce,
    /// A `git switch` that moves `HEAD`.
    SwitchMove,
    /// A `git rm` with `--force`.
    Remove,
    /// A `git mv` with `--force`.
    Move,
    /// A `git checkout-index` with `--force`.
    CheckoutIndex,
    /// A `git read-tree` with `--reset`.
    ReadTree,
    /// `git merge --abort`.
    AbortMerge,
    /// `git rebase --abort`.
    AbortRebase,
    /// `git cherry-pick --abort` or `git revert --abort`.
    AbortPick,
    /// `git am --abort`.
    AbortAm,
}

/// What:
///  The outcome of looking at the argument list only.
/// Why:
///   Most commands are not guarded and need no repository query at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkedWorktreeDecision = { kind: 'pass' } | { kind: 'needs-effective-target'; command: GuardedCommand }
///   | { kind: 'when-dirty'; command: GuardedCommand };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkedWorktreeDecision {
    /// Not a guarded command,
    ///  a harmless form,
    ///  or a region Git refuses.
    Pass,
    /// The form overwrites or deletes by itself:
    ///  the caller classifies the worktree the command targets,
    ///  then calls `resolve_linked_worktree`;
    ///  a linked worktree is still judged by whether it is dirty.
    NeedsEffectiveTarget(GuardedCommand),
    /// The form moves `HEAD` and loses nothing unless local changes exist:
    ///  it is rejected in any worktree, the main one included, only while the worktree is dirty.
    WhenDirty(GuardedCommand),
}

/// What:
///  The files that count as uncommitted state for a guarded command.
/// Why:
///   A `clean` deletes untracked files, so they count for it; every other form loses tracked changes only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyScope(command: GuardedCommand): DirtyScope;
/// ```
pub fn dirty_scope(command: GuardedCommand) -> DirtyScope {
    if command == GuardedCommand::Clean {
        return DirtyScope::TrackedAndUntracked;
    }
    return DirtyScope::Tracked;
}

/// What:
///  Turn one classifier reading into a decision.
/// Why:
///   Every command shares this step: an overwrite needs the worktree kind, a `HEAD` move needs the dirtiness,
///  and a harmless region needs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decided(command: GuardedCommand, reading: Discard): LinkedWorktreeDecision;
/// ```
fn decided(command: GuardedCommand, reading: Discard) -> LinkedWorktreeDecision {
    match reading {
        Discard::Harmless => return LinkedWorktreeDecision::Pass,
        Discard::MovesHead => return LinkedWorktreeDecision::WhenDirty(command),
        Discard::Overwrites => return LinkedWorktreeDecision::NeedsEffectiveTarget(command),
    }
}

/// What:
///  Whether a guarded subcommand word and its region can discard, and in which way.
///  `&[u8]` borrows the subcommand word;
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   Each subcommand is read with its own Git grammar, so `rm -- -f` is a path and `rm -rf` is a force.
///  A region Git refuses is the refusal, and Git then discards nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function guardedDecision(word: string, region: string[]): LinkedWorktreeDecision; // throws OptionError
/// ```
fn guarded_decision(
    word: &[u8],
    region: &[OsString],
) -> Result<LinkedWorktreeDecision, OptionError> {
    if word == b"stash" {
        // A region the stash parser refuses stays guarded, as every stash form was before the read-only forms.
        let reads_only: bool = parse_stash_region(region, &[])
            .is_ok_and(|facts| { return !stash_changes_state(facts.subcommand); });
        if reads_only {
            return Ok(LinkedWorktreeDecision::Pass);
        }
        return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::Stash));
    }
    if word == b"clean" {
        // `&[]` is an empty flag list: wrapper controls were removed before this point.
        let facts: CleanRegion = parse_clean_region(region, &[])?;
        if clean_changes_worktree(&facts) {
            return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::Clean));
        }
        return Ok(LinkedWorktreeDecision::Pass);
    }
    if word == b"reset" {
        let facts: ResetRegion = parse_reset_region(region, &[])?;
        // Every mode but `--soft` replaces the index, and with it the staged entries.
        return Ok(match facts.mode {
            Some(ResetMode::Soft) => LinkedWorktreeDecision::Pass,
            Some(ResetMode::Hard | ResetMode::Merge | ResetMode::Keep) => {
                LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetHard)
            }
            Some(ResetMode::Mixed) | None => {
                LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetMixed)
            }
        });
    }
    if word == b"checkout" {
        return Ok(match checkout_discard(region)? {
            Discard::Overwrites => decided(GuardedCommand::CheckoutPaths, Discard::Overwrites),
            reading => decided(GuardedCommand::CheckoutMove, reading),
        });
    }
    if word == b"switch" {
        return Ok(match switch_discard(region)? {
            Discard::Overwrites => decided(GuardedCommand::SwitchForce, Discard::Overwrites),
            reading => decided(GuardedCommand::SwitchMove, reading),
        });
    }
    if word == b"restore" {
        let command: GuardedCommand = if restores_index_only(region) {
            GuardedCommand::RestoreStaged
        } else {
            GuardedCommand::RestoreWorktree
        };
        return Ok(decided(command, restore_discard(region)));
    }
    if word == b"rm" {
        return Ok(decided(GuardedCommand::Remove, force_discard(region)));
    }
    if word == b"mv" {
        return Ok(decided(GuardedCommand::Move, force_discard(region)));
    }
    if word == b"checkout-index" {
        return Ok(decided(GuardedCommand::CheckoutIndex, force_discard(region)));
    }
    if word == b"read-tree" {
        return Ok(decided(GuardedCommand::ReadTree, read_tree_discard(region)));
    }
    for (aborting, command) in [
        (&b"merge"[..], GuardedCommand::AbortMerge),
        (b"rebase", GuardedCommand::AbortRebase),
        (b"cherry-pick", GuardedCommand::AbortPick),
        (b"revert", GuardedCommand::AbortPick),
        (b"am", GuardedCommand::AbortAm),
    ] {
        if word == aborting {
            return Ok(decided(command, abort_discard(region)));
        }
    }
    return Ok(LinkedWorktreeDecision::Pass);
}

/// What:
///  Decide from the argument list.
///  `&[OsString]` borrows the arguments,
///  already free
///       of wrapper controls.
/// Why:
///   A command that is not guarded,
///  or that Git itself refuses,
///  needs no measurement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideLinkedWorktree(args: string[]): LinkedWorktreeDecision;
/// ```
pub fn decide_linked_worktree(arguments: &[OsString]) -> LinkedWorktreeDecision {
    // `let Some((a, b)) = ... else { ... };` unpacks the command word and the tokens after
    // it, or returns when Git runs no subcommand.
    let Some((word, region)) = command_tokens(arguments) else {
        return LinkedWorktreeDecision::Pass;
    };
    // `.as_encoded_bytes()` lends the raw bytes of the subcommand word.
    return guarded_decision(word.as_encoded_bytes(), region).unwrap_or(LinkedWorktreeDecision::Pass);
}

/// What:
///  Decide from the measured worktree kind:
///  the rejection text,
///  or nothing to let the command through for now.
/// Why:
///   The main worktree and a place without a worktree reject at once;
///  a linked worktree and an allowlisted tool cache pass this step,
///  and a linked worktree is then judged by its dirtiness.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveLinkedWorktree(command: GuardedCommand, target: EffectiveTarget): string | undefined;
/// ```
pub fn resolve_linked_worktree(command: GuardedCommand, target: EffectiveTarget) -> Option<String> {
    match target {
        // `Some(x)` is the "present" case of `Option`; `None` is the "absent" case.
        EffectiveTarget::OutsideWorktree => return Some(outside_worktree_message(command)),
        EffectiveTarget::MainWorktree => return Some(main_worktree_message(command)),
        EffectiveTarget::LinkedWorktree | EffectiveTarget::Allowlisted => return None,
    }
}

/// Guarded forms,
///  harmless forms and every rejection text.
#[cfg(test)]
#[path = "rule_linked_worktree_tests.rs"]
mod tests;
