//! What:
//!  The forms the linked-worktree policy guards:
//!  every command form that can throw away uncommitted work,
//!  and the list of all of them that a control walks.
//! Why:
//!  Each form has its own rejection text and its own dirty question,
//!  so the decision and the texts both name this closed set.
//!  It sits in a file of its own because the decision and the texts
//!  named each other while one of them held it:
//!  the decision uses the texts,
//!  and the texts use this set only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type GuardedCommand = 'stash' | 'clean' | 'reset-hard' | 'restore-worktree' | 'abort-merge';
//! ```

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
    pub const ALL: &[GuardedCommand] = &[
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
