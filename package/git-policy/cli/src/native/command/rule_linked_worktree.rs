//! What:
//!  Pure decision of the linked-worktree-only policy:
//!  commands that can discard uncommitted state never run in the main worktree,
//! and run in a linked worktree
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
//!  `read-tree`,
//!  and an `--abort` can throw away work,
//! and the loss leaves no record.
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
use crate::command_branch_delete::branch_deletions;
use crate::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
use crate::command_clean::{CleanRegion, clean_changes_worktree, parse_clean_region};
use crate::command_worktree::worktree_removal;
use crate::command_discard::{
    Discard, abort_discard, checkout_discard, force_discard, read_tree_discard, restore_discard,
    restores_index_only, revision_before_paths, switch_discard,
};
use crate::command_options::{OptionError, OptionSpec};
use crate::command_reset::{ResetMode, ResetRegion, RESET_TABLE, parse_reset_region};
use crate::command_stash::{StashSubcommand, parse_stash_region, stash_changes_state};
use crate::effective_target::EffectiveTarget;
use crate::global_arguments::command_tokens;
use crate::rule_worktree_dirty::DirtyScope;
pub use crate::rule_linked_worktree_messages::{
    dirty_worktree_message, exemption_notice, ignored_file_message, main_worktree_message,
    outside_worktree_message, repository_stash_message,
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
///   Each form has its own rejection text,
/// because the same subcommand serves different intents:
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
    /// Every `git stash` form that changes state except `drop` and `clear`.
    Stash,
    /// A `git stash drop` or `git stash clear`,
    /// which deletes an entry of the one stash the whole repository shares.
    StashDrop,
    /// A `git clean` that is not a dry run and deletes no ignored file.
    Clean,
    /// A `git clean` that is not a dry run and deletes ignored files (`-x` or `-X`).
    CleanIgnored,
    /// A `git clean` that is not a dry run and deletes ignored files and nothing else (`-X` alone).
    CleanIgnoredOnly,
    /// A `git reset` with `--hard`,
    ///  `--merge` or `--keep`.
    ResetHard,
    /// A `git reset` in the default or `--mixed` mode.
    ResetMixed,
    /// A `git restore` that names the index and not the worktree.
    RestoreStaged,
    /// A `git restore` that writes worktree files.
    RestoreWorktree,
    /// A `git checkout` with paths,
    /// `--force`,
    /// `--patch`,
    /// `--ours` or `--theirs`.
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

/// The enum's own lists.
impl GuardedCommand {
    /// What:
    ///  Every guarded form,
    /// so a control can walk all of them instead of naming some.
    ///  `[T; N]` is a fixed-size array.
    /// Why:
    ///   A control that lists the forms itself drifts from the enum;
    /// one that walks this list
    ///  reaches a new form as soon as it is added here,
    /// and the coverage test in `rule_linked_worktree_tests`
    ///  fails while it is missing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ALL_GUARDED_COMMANDS: readonly GuardedCommand[] = ['stash', 'clean', /* ... */ 'abort-am'];
    /// ```
    pub const ALL: [GuardedCommand; 21] = [
        GuardedCommand::Stash,
        GuardedCommand::StashDrop,
        GuardedCommand::Clean,
        GuardedCommand::CleanIgnored,
        GuardedCommand::CleanIgnoredOnly,
        GuardedCommand::ResetHard,
        GuardedCommand::ResetMixed,
        GuardedCommand::RestoreStaged,
        GuardedCommand::RestoreWorktree,
        GuardedCommand::CheckoutPaths,
        GuardedCommand::CheckoutMove,
        GuardedCommand::SwitchForce,
        GuardedCommand::SwitchMove,
        GuardedCommand::Remove,
        GuardedCommand::Move,
        GuardedCommand::CheckoutIndex,
        GuardedCommand::ReadTree,
        GuardedCommand::AbortMerge,
        GuardedCommand::AbortRebase,
        GuardedCommand::AbortPick,
        GuardedCommand::AbortAm,
    ];
}

/// What:
///  The forms the wrapper records and never refuses.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The owner decided these two run exactly as Git runs them and leave one audit line:
///  `git worktree remove`
///       ("This one is pretty intentional because agents are just as likely to run a rm rf,
///  so preventing
///       git worktree remove buys us nothing") and
///       `git branch -D` from the main worktree ("Allow,
///  add audit line").
///  A refusal here would only move the same loss to `rm -rf` or `update-ref -d`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditedCommand = 'worktree-remove' | 'branch-delete';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditedCommand {
    /// `git worktree remove`,
    ///  which deletes a whole worktree at once.
    WorktreeRemove,
    /// `git branch -d`,
    /// `git branch -D` or `git branch --delete`.
    BranchDelete,
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
    ///  it is rejected in any worktree,
    /// the main one included,
    /// only while the worktree is dirty.
    WhenDirty(GuardedCommand),
    /// The form runs whatever its worktree holds,
    /// and the caller records what it takes.
    Audited(AuditedCommand),
}

/// What:
///  The question that decides a guarded form in a linked worktree.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Almost every form destroys state of the worktree it runs in,
/// and a status answers that;
///  `git stash drop` and `git stash clear` delete an entry of `refs/stash`,
///  which every worktree of
///  the repository shares,
/// so the running worktree's cleanliness says nothing about them and the
///  repository's stash is asked instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtyQuestion = { kind: 'worktree'; scope: DirtyScope } | { kind: 'repository-stash' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirtyQuestion {
    /// Ask `git status` for the given scope.
    Worktree(DirtyScope),
    /// Ask Git what this exact clean would delete (`git clean --dry-run`),
    ///  because the clean carries
    ///       `-e`/`--exclude` patterns that the status cannot see.
    CleanDryRun,
    /// Ask whether the repository holds a stash entry.
    RepositoryStash,
}

/// What:
///  Which question a guarded form's answer needs.
/// Why:
///   Each form reads the object it can destroy;
///  asking the wrong one rejects a harmless command or misses the loss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyQuestion(command: GuardedCommand): DirtyQuestion;
/// ```
pub fn dirty_question(command: GuardedCommand, region: &[OsString]) -> DirtyQuestion {
    match command {
        GuardedCommand::StashDrop => {
            return DirtyQuestion::RepositoryStash;
        }
        GuardedCommand::CleanIgnored => {
            return DirtyQuestion::Worktree(DirtyScope::Everything);
        }
        GuardedCommand::CleanIgnoredOnly => {
            // `-e`/`--exclude` adds patterns to the ignore rules this clean applies, so a file the status calls
            //  untracked is one this clean deletes; nothing in the status answer shows that, and Git's own dry run
            //  does.
            match parse_clean_region(region, &[]) {
                Ok(facts) if !facts.exclusions.is_empty() => {
                    return DirtyQuestion::CleanDryRun;
                }
                Ok(_) | Err(_) => {}
            }
            return DirtyQuestion::Worktree(DirtyScope::IgnoredOnly);
        }
        GuardedCommand::Clean => {
            return DirtyQuestion::Worktree(DirtyScope::TrackedAndUntracked);
        }
        _ => return DirtyQuestion::Worktree(DirtyScope::Tracked),
    }
}

/// What:
///  The tree a guarded form writes into the worktree.
///  A `struct` is a record with named
///       fields.
/// Why:
///   Whether an ignored file the worktree holds is replaced depends on the tree the form writes,
///  so
///       the question has to carry the revision the command names
///  or nothing when it names none (every form
///       that names no tree writes `HEAD`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TargetTree = { revision?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetTree {
    /// The revision the command names,
    ///  or nothing for `HEAD`.
    pub revision: Option<OsString>,
}

/// What:
///  The tree a guarded form writes into the worktree,
///  when the form replaces an ignored file that
///       tree tracks whatever `--no-overwrite-ignore` says;
///  nothing for every other form.
///  `&[OsString]` borrows the tokens after the subcommand.
/// Why:
///   A forced `checkout`,
///  a forced `switch` and a hard `reset` overwrite an ignored file whose path their target
///       tree tracks,
///  and an ignored file is where a person keeps their environment and their local
///       secrets;
///  the token that may name another tree comes from that command's own option table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetTree(command: GuardedCommand, region: string[]): TargetTree | undefined;
/// ```
pub fn target_tree(command: GuardedCommand, region: &[OsString]) -> Option<TargetTree> {
    // `match` picks the one table the decided form belongs to; `other` is every form that writes no tree.
    let table: &[OptionSpec] = match command {
        GuardedCommand::ResetHard => RESET_TABLE,
        GuardedCommand::CheckoutPaths => CHECKOUT_TABLE,
        GuardedCommand::SwitchForce => SWITCH_TABLE,
        _ => return None,
    };
    return Some(TargetTree {
        revision: revision_before_paths(region, table),
    });
}

/// What:
///  Turn one classifier reading into a decision.
/// Why:
///   Every command shares this step:
/// an overwrite needs the worktree kind,
/// a `HEAD` move needs the dirtiness,
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
///  Whether a guarded subcommand word and its region can discard,
/// and in which way.
///  `&[u8]` borrows the subcommand word;
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   Each subcommand is read with its own Git grammar,
/// so `rm -- -f` is a path and `rm -rf` is a force.
///  A region Git refuses is the refusal,
/// and Git then discards nothing.
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
        let subcommand: Option<StashSubcommand> = match parse_stash_region(region, &[]) {
            Ok(facts) => Some(facts.subcommand),
            Err(_) => None,
        };
        if let Some(reading) = subcommand
            && !stash_changes_state(reading)
        {
            return Ok(LinkedWorktreeDecision::Pass);
        }
        // `drop` and `clear` delete entries of `refs/stash`, the one stash every worktree of the repository shares.
        if matches!(subcommand, Some(StashSubcommand::Drop | StashSubcommand::Clear)) {
            return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::StashDrop));
        }
        return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::Stash));
    }
    if word == b"clean" {
        // `&[]` is an empty flag list: wrapper controls were removed before this point.
        let facts: CleanRegion = parse_clean_region(region, &[])?;
        if !clean_changes_worktree(&facts) {
            return Ok(LinkedWorktreeDecision::Pass);
        }
        // An ignored file is not an untracked file to Git, and the person's environment file sits in exactly that
        // difference, so a clean that deletes ignored files is its own guarded form with its own measurement.
        if facts.ignored_only {
            return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnoredOnly));
        }
        if facts.deletes_ignored {
            return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnored));
        }
        return Ok(LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::Clean));
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
    if word == b"worktree" {
        if worktree_removal(region).is_some() {
            return Ok(LinkedWorktreeDecision::Audited(AuditedCommand::WorktreeRemove));
        }
        return Ok(LinkedWorktreeDecision::Pass);
    }
    if word == b"branch" {
        if branch_deletions(region).is_some() {
            return Ok(LinkedWorktreeDecision::Audited(AuditedCommand::BranchDelete));
        }
        return Ok(LinkedWorktreeDecision::Pass);
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
        return Ok(decided(GuardedCommand::ReadTree, read_tree_discard(region)?));
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
    return decide_guarded_word(word.as_encoded_bytes(), region);
}

/// What:
///  Decide from one subcommand word and the tokens after it.
///  `&[u8]` borrows the word and
///  `&[OsString]` its region.
/// Why:
///   The word on a command line is not always the word Git runs:
/// an alias expands to another command,
/// and
///  the caller that resolved it decides on the same tables this way.
///  A region Git refuses is the refusal,
///  and Git then
///       discards nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideGuardedWord(word: Buffer, region: string[]): LinkedWorktreeDecision;
/// ```
pub fn decide_guarded_word(word: &[u8], region: &[OsString]) -> LinkedWorktreeDecision {
    return guarded_decision(word, region).unwrap_or(LinkedWorktreeDecision::Pass);
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
