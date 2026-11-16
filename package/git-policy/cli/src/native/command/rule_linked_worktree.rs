//! What:
//!  Pure decision of the linked-worktree-only policy:
//!  commands that can discard uncommitted state never run in the main worktree,
//! and in a linked worktree
//!      run only while it holds nothing that form would destroy,
//!      an ignored file being nothing a clean is refused over
//!      (`dirty_question` says when that is).
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
use crate::command_clean::{CleanRegion, clean_changes_worktree, parse_clean_region};
use crate::command_worktree::worktree_removal;
use crate::command_discard::{
    Discard, abort_discard, checkout_discard, checkout_index_reading, move_reading, read_tree_discard,
    remove_reading, restore_reading, skip_discard, switch_discard,
};
use crate::command_checkout_index_table::{CHECKOUT_INDEX_IGNORE_SKIP_WORKTREE_BITS, CHECKOUT_INDEX_TABLE};
use crate::command_checkout_table::{CHECKOUT_TABLE, IGNORE_SKIP_WORKTREE_BITS, SWITCH_TABLE};
use crate::command_options::{DEFAULT_MODE, OptionError, OptionErrorKind, parse_options};
use crate::command_options_query::is_enabled;
use crate::command_restore_table::{RESTORE_IGNORE_SKIP_WORKTREE_BITS, RESTORE_TABLE};
use crate::command_reset::{ResetMode, ResetRegion, parse_reset_region};
use crate::command_stash::{StashSubcommand, parse_stash_region, stash_changes_state};
use crate::effective_target::EffectiveTarget;
use crate::global_arguments::command_tokens;
use crate::rule_shared_remote::{SharedRemoteChange, shared_remote_write};
use crate::rule_worktree_dirty::DirtyScope;
pub use crate::rule_linked_worktree_messages::{
    dirty_worktree_message, dirty_worktree_message_naming, exemption_notice, ignored_file_message,
    main_worktree_message, outside_worktree_message, repository_stash_message,
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
///   | 'read-tree' | 'abort-merge' | 'abort-rebase' | 'abort-pick' | 'abort-am'
///   | 'skip-rebase' | 'skip-pick' | 'skip-am';
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
    /// `git rebase --skip`,
    /// which drops the commit the rebase stopped at and goes on.
    SkipRebase,
    /// `git cherry-pick --skip` or `git revert --skip`,
    /// which drop the commit a sequence stopped at and go on.
    SkipPick,
    /// `git am --skip`,
    /// which drops the patch it stopped at and goes on.
    SkipAm,
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
    pub const ALL: [GuardedCommand; 24] = [
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
        GuardedCommand::SkipRebase,
        GuardedCommand::SkipPick,
        GuardedCommand::SkipAm,
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
///   | { kind: 'when-dirty'; command: GuardedCommand } | { kind: 'shared-remote'; change: SharedRemoteChange };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// The form changes remote settings every worktree of the repository shares,
    ///  which only a linked
    ///  worktree is refused for;
    ///  the change names the file the command named for it,
    ///  when it named one.
    SharedRemote(SharedRemoteChange),
    /// What:
    ///  A word of the region is an option no table of this subcommand holds,
    ///  so the guard cannot tell what the command would write over or delete.
    /// Why:
    ///  The tables are Git 2.56.0's own, and a newer Git's option may discard:
    ///  reading such a word as harmless is the guess that let
    ///  `git rm -f --pathspec-from-file --no-force` delete an edit.
    ///  The command is refused,
    ///  the refusal names the word,
    ///  and the opt-out runs the command as real Git runs it.
    UnreadableOption {
        /// The guarded form the word belongs to.
        command: GuardedCommand,
        /// The word as the person typed it.
        written: OsString,
    },
}

/// What:
///  One reading the guard could not make,
///  with the form whose table the word was read from.
///  A `struct` is a record with named fields.
/// Why:
///  The caller has to tell a usage request (Git writes nothing) and Git's own refusal of a region
///  (whose options the table holds) from a word no table holds, and only the last one is the
///  guard's to refuse.
///  The form travels with the failure because the arm that read the table knows it, and the word
///  alone does not say which of its forms a HEAD-moving command is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unreadable = { command: GuardedCommand; error: OptionError };
/// ```
struct Unreadable {
    /// The form whose region was being read.
    command: GuardedCommand,
    /// What the tokenizer answered.
    error: OptionError,
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
/// type DirtyQuestion = { kind: 'worktree'; scope: DirtyScope } | { kind: 'clean-dry-run' }
///   | { kind: 'repository-stash' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirtyQuestion {
    /// Ask `git status` for the given scope.
    Worktree(DirtyScope),
    /// Ask Git what this exact clean would delete only because of the `-e`/`--exclude` patterns it carries,
    ///  because a status cannot see them.
    CleanDryRun,
    /// Ask whether the repository holds a stash entry.
    RepositoryStash,
}

/// What:
///  Which question a guarded form's answer needs,
///  or nothing when the form loses nothing this policy protects in a linked
///       worktree.
/// Why:
///   Each form reads the object it can destroy;
///  asking the wrong one rejects a harmless command or misses the loss.
///  The owner decided on 2026-11-10 that an ignored file is not a reason to refuse a clean in a linked worktree,
///  so a clean that deletes nothing else has nothing to ask and runs,
///  and a clean that also deletes untracked files is judged by tracked and untracked work only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyQuestion(command: GuardedCommand, region: string[]): DirtyQuestion | undefined;
/// ```
pub fn dirty_question(command: GuardedCommand, region: &[OsString]) -> Option<DirtyQuestion> {
    match command {
        GuardedCommand::StashDrop => {
            return Some(DirtyQuestion::RepositoryStash);
        }
        GuardedCommand::CleanIgnored => {
            // `-x` deletes untracked files as well, and an untracked file is still a reason to refuse it.
            return Some(DirtyQuestion::Worktree(DirtyScope::TrackedAndUntracked));
        }
        GuardedCommand::CleanIgnoredOnly => {
            // `-X` deletes ignored files and nothing else, so what it would destroy is nothing this policy asks about.
            // `-e`/`--exclude` adds patterns to the ignore rules this clean applies, so a path the worktree does not
            //  itself ignore is one this clean deletes, and Git's own dry run is the reading that shows the difference.
            match parse_clean_region(region, &[]) {
                Ok(facts) if !facts.exclusions.is_empty() => {
                    return Some(DirtyQuestion::CleanDryRun);
                }
                Ok(_) | Err(_) => {}
            }
            return None;
        }
        GuardedCommand::Clean => {
            return Some(DirtyQuestion::Worktree(DirtyScope::TrackedAndUntracked));
        }
        // What:
        //  Which edits a form that writes paths replaces, the one a `skip-worktree` mark hides included.
        // Why:
        //  Two words decide it, and either one is enough.
        //  The written paths are one: a tree written over named paths replaces the file the mark names
        //  (`checkout <tree> -- <path>`, `restore --source <tree> -- <path>`, and either of them with
        //  `--pathspec-from-file=<file>`), and a forced `mv` replaces the destination that way too,
        //  while a whole-tree write, an index write, a `read-tree` and a `checkout-index` keep it
        //  (each measured on this host with native Git 2.56.0, 2026-11-15).
        //  `--ignore-skip-worktree-bits` is the other: it tells Git to write the entries the marks hide,
        //  whichever form carries it, so `checkout-index -f --ignore-skip-worktree-bits -a`,
        //  `checkout --ignore-skip-worktree-bits -- <path>` and `restore --ignore-skip-worktree-bits -- <path>`
        //  each replace the edit the mark hides
        //  (measured on this host with native Git 2.56.0, 2026-11-16, and the axis of every option of every
        //  guarded table walks it).
        //  Asking the mark here is not going beyond Git: the command writes the file,
        //  and the person's edit in it is gone either way, so the guard refuses and the hatch runs it as typed.
        GuardedCommand::CheckoutPaths
        | GuardedCommand::RestoreWorktree
        | GuardedCommand::Move
        | GuardedCommand::CheckoutIndex => {
            let written: Option<crate::command_written_paths::WrittenPaths> =
                crate::command_written_paths::written_paths(command, region);
            let writes_a_marked_edit: bool = written
                .as_ref()
                .is_some_and(crate::command_written_paths::writes_over_an_edit_a_mark_hides)
                || ignores_skip_worktree_bits(command, region);
            if writes_a_marked_edit {
                return Some(DirtyQuestion::Worktree(DirtyScope::TrackedAndSkipWorktreeEdits));
            }
            return Some(DirtyQuestion::Worktree(DirtyScope::Tracked));
        }
        // What:
        //  A `git rm` that asks Git to ignore the sparse bits.
        // Why:
        //  `--sparse` is the person telling Git to reach a path a `skip-worktree` mark hides,
        //  and Git then deletes the file with the edit in it
        //  (`git rm -f --sparse s.txt` exits 0 and the edited `s.txt` is gone,
        //  while `git rm -f s.txt` refuses and keeps it, measured on this host with native Git
        //  2.56.0, 2026-11-14);
        //  `--cached` is the other half: it removes the index entry and leaves the worktree file
        //  alone, so a hidden edit there is no loss and the plain scope is asked.
        GuardedCommand::Remove => {
            return Some(DirtyQuestion::Worktree(if reaches_a_skipped_edit(region) {
                DirtyScope::TrackedAndSkipWorktreeEdits
            } else {
                DirtyScope::Tracked
            }));
        }
        // What:
        //  A `checkout` or a `switch` that carries `--ignore-skip-worktree-bits`.
        // Why:
        //  The option tells Git to write the entries the marks hide, whichever form carries it,
        //  and one such form is a `checkout` whose single word is a path rather than a revision:
        //  `git checkout --ignore-skip-worktree-bits a.txt` with one name and no `--` reads as a branch
        //  move to the guard, while Git reads the name as a path and writes the index's copy over the
        //  edit the mark hides
        //  (the eleventh review's fifth finding, measured on this host with native Git 2.56.0, 2026-11-16:
        //  `Updated 1 path from the index` and the edit gone, while the same words with `--`, with two
        //  names or with a tree were refused).
        //  Asking the marked edits for every such form costs a listing of the marked paths and nothing
        //  when the worktree holds none.
        _ => {
            return Some(DirtyQuestion::Worktree(if ignores_skip_worktree_bits(command, region) {
                DirtyScope::TrackedAndSkipWorktreeEdits
            } else {
                DirtyScope::Tracked
            }));
        }
    }
}

/// What:
///  Whether a `git rm` region asks Git to look past the `skip-worktree` mark and to remove the
///  worktree file.
/// Why:
///  `--sparse` (`--sp` and longer, the prefixes `rm`'s own option table leaves unambiguous) makes
///  Git reach a skipped path, and Git then deletes the file the mark hides;
///  `--cached` removes the index entry alone and keeps that file,
///  so the two together ask nothing about a hidden edit.
///  A token after `--` is a path, never an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reaches = (region) => region.some((w) => spellsLong(w, 'sparse', RM_LONG_OPTIONS)) && !region.some((w) => spellsLong(w, 'cached', RM_LONG_OPTIONS));
/// ```
fn reaches_a_skipped_edit(region: &[OsString]) -> bool {
    // Both flags are read from the removal's own table, so a value that looks like either one is neither:
    //  `git rm --sparse --sparse --no-sparse` is a plain removal and `--no-sparse --sparse` reaches the marked edit
    //  (measured on this host with native Git 2.56.0, 2026-11-15).
    return remove_reading(region).is_ok_and(|reading| return reading.reaches_a_skipped_edit);
}

/// What:
///  Whether a `checkout`, `restore` or `checkout-index` region tells Git to write the entries a
///  `skip-worktree` mark hides.
/// Why:
///  `--ignore-skip-worktree-bits` makes Git write those files,
///  so the edit the mark hides is one the form replaces and the guard has to refuse it;
///  a reading that treated the option as the person asking for the loss let three forms write over an
///  edit without a refusal
///  (the ninth review's first finding, `reproduced for all three forms`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignores = (region) => region.includes('--ignore-skip-worktree-bits');
/// ```
fn ignores_skip_worktree_bits(command: GuardedCommand, region: &[OsString]) -> bool {
    // The flag is a row of each command's own table, so a file named `--ignore-skip-worktree-bits` is that
    //  option's value and no switch:
    //  `git restore --source=other --pathspec-from-file --ignore-skip-worktree-bits` reads a file.
    let enabled: Option<bool> = match command {
        GuardedCommand::RestoreWorktree | GuardedCommand::RestoreStaged => {
            parse_options(region, RESTORE_TABLE, DEFAULT_MODE, &[]).ok().map(|parsed| return is_enabled(&parsed, RESTORE_IGNORE_SKIP_WORKTREE_BITS))
        }
        GuardedCommand::CheckoutIndex => {
            parse_options(region, CHECKOUT_INDEX_TABLE, DEFAULT_MODE, &[]).ok().map(|parsed| return is_enabled(&parsed, CHECKOUT_INDEX_IGNORE_SKIP_WORKTREE_BITS))
        }
        // `switch` has no such option of its own, and reading its region by `checkout`'s table could take a
        //  switch word for an option this command never reads.
        GuardedCommand::SwitchForce | GuardedCommand::SwitchMove => {
            parse_options(region, SWITCH_TABLE, DEFAULT_MODE, &[]).ok().map(|parsed| return is_enabled(&parsed, IGNORE_SKIP_WORKTREE_BITS))
        }
        _ => {
            parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok().map(|parsed| return is_enabled(&parsed, IGNORE_SKIP_WORKTREE_BITS))
        }
    };
    return enabled.unwrap_or(false);
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
///  The form `git merge --abort` and its siblings are,
///  or nothing for a word whose abort is not this policy's.
/// Why:
///   The abort reading and the skip reading are both decided from the same word,
///  and each names its own form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abortingForm(word: Buffer): GuardedCommand | undefined;
/// ```
fn aborting_form(word: &[u8]) -> Option<GuardedCommand> {
    return match word {
        b"merge" => Some(GuardedCommand::AbortMerge),
        b"rebase" => Some(GuardedCommand::AbortRebase),
        b"cherry-pick" | b"revert" => Some(GuardedCommand::AbortPick),
        b"am" => Some(GuardedCommand::AbortAm),
        _ => None,
    };
}

/// What:
///  The form `git rebase --skip` and its siblings are,
///  or nothing for a word that has no `--skip`.
/// Why:
///   `git merge` has no `--skip`,
///  so a region that spelled it is Git's to refuse and nothing is discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippingForm(word: Buffer): GuardedCommand | undefined;
/// ```
fn skipping_form(word: &[u8]) -> Option<GuardedCommand> {
    return match word {
        b"rebase" => Some(GuardedCommand::SkipRebase),
        b"cherry-pick" | b"revert" => Some(GuardedCommand::SkipPick),
        b"am" => Some(GuardedCommand::SkipAm),
        _ => None,
    };
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
) -> Result<LinkedWorktreeDecision, Unreadable> {
    // The settings every worktree shares are written by two command words only, and no guarded form is about them.
    if let Some(write) = shared_remote_write(word, region) {
        return Ok(LinkedWorktreeDecision::SharedRemote(write));
    }
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
        let facts: CleanRegion = read_by(GuardedCommand::Clean, parse_clean_region(region, &[]))?;
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
        let facts: ResetRegion = read_by(GuardedCommand::ResetHard, parse_reset_region(region, &[]))?;
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
        return Ok(match read_by(GuardedCommand::CheckoutPaths, checkout_discard(region))? {
            Discard::Overwrites => decided(GuardedCommand::CheckoutPaths, Discard::Overwrites),
            reading => decided(GuardedCommand::CheckoutMove, reading),
        });
    }
    if word == b"switch" {
        return Ok(match read_by(GuardedCommand::SwitchForce, switch_discard(region))? {
            Discard::Overwrites => decided(GuardedCommand::SwitchForce, Discard::Overwrites),
            reading => decided(GuardedCommand::SwitchMove, reading),
        });
    }
    if word == b"restore" {
        let reading = read_by(GuardedCommand::RestoreWorktree, restore_reading(region))?;
        let command: GuardedCommand = if reading.staged_only {
            GuardedCommand::RestoreStaged
        } else {
            GuardedCommand::RestoreWorktree
        };
        return Ok(decided(command, reading.discard));
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
        // An unforced `git rm` refuses a file whose worktree copy differs, but it does not look past a
        //  `skip-worktree` mark:
        //  `git rm --sparse s.txt` deletes the edit the mark hides at exit 0,
        //  while `git rm s.txt` without `--sparse` prints `The following paths and/or pathspecs matched paths that exist`
        //  and keeps the file
        //  (measured on this host with native Git 2.56.0, 2026-11-14),
        //  so a removal that reaches such an edit is guarded even without a force.
        //  Every other unforced removal is Git's own protection and stays out of this policy.
        let reading = read_by(GuardedCommand::Remove, remove_reading(region))?;
        if matches!(reading.discard, Discard::Harmless) && !reading.reaches_a_skipped_edit {
            return Ok(LinkedWorktreeDecision::Pass);
        }
        return Ok(decided(GuardedCommand::Remove, Discard::Overwrites));
    }
    if word == b"mv" {
        return Ok(decided(GuardedCommand::Move, read_by(GuardedCommand::Move, move_reading(region))?));
    }
    if word == b"checkout-index" {
        return Ok(decided(
            GuardedCommand::CheckoutIndex,
            read_by(GuardedCommand::CheckoutIndex, checkout_index_reading(region))?,
        ));
    }
    if word == b"read-tree" {
        return Ok(decided(GuardedCommand::ReadTree, read_by(GuardedCommand::ReadTree, read_tree_discard(region))?));
    }
    // The two forms that end the operation a conflict stopped are read before every other form of their word,
    //  because both drop the hand resolution the operation is waiting on.
    if let Some(aborting) = aborting_form(word) {
        if matches!(abort_discard(region), Discard::Overwrites) {
            return Ok(decided(aborting, Discard::Overwrites));
        }
        // A `--skip` goes on with the next commit or patch after dropping this one, and the reset it makes
        //  throws the hand resolution away just as the abort does (measured with Git 2.56.0), so it is guarded too.
        if let Some(skipping) = skipping_form(word)
            && matches!(skip_discard(region), Discard::Overwrites)
        {
            return Ok(decided(skipping, Discard::Overwrites));
        }
        return Ok(LinkedWorktreeDecision::Pass);
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
    return match guarded_decision(word, region) {
        Ok(decision) => decision,
        // What:
        //  A word no table of this subcommand holds.
        // Why:
        //  Its value and its meaning are unknown to the guard, so it may discard, and reading it as harmless is
        //  the guess that let a pathspec file named `--no-force` delete an edit.
        //  Git would refuse it itself at 2.56.0, so nothing is lost by refusing it here first, and the message
        //  names the word and the opt-out.
        Err(problem) if problem.error.kind == OptionErrorKind::UnknownOption => {
            LinkedWorktreeDecision::UnreadableOption {
                command: problem.command,
                written: region.get(problem.error.token).cloned().unwrap_or_default(),
            }
        }
        // A usage request, an ambiguous abbreviation, a missing or unexpected value and a long option behind one
        //  dash are all regions Git answers itself without writing anything, so the guard leaves them to Git.
        Err(_) => LinkedWorktreeDecision::Pass,
    };
}

/// What:
///  One table's reading,
///  with the form whose word was read set beside a failure.
/// Why:
///  Only an option no table holds is the guard's to refuse, and the caller cannot name the form from the
///  failure alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readBy = (command, reading) => reading.mapErr((error) => ({ command, error }));
/// ```
fn read_by<Reading>(command: GuardedCommand, reading: Result<Reading, OptionError>) -> Result<Reading, Unreadable> {
    return reading.map_err(|error| return Unreadable { command, error });
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
