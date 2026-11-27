//! What:
//!  The two readings that say what work a form would take from the person:
//!  whether the form ends the operation it finds waiting,
//!  and whether a `git clean` carrying its own `-e`/`--exclude` patterns would delete a path those patterns
//!  alone call ignored.
//! Why:
//!  The check asks both before it lets a form run,
//!  and each answer is one whole question about work that exists nowhere else:
//!  the operation's own state is what an abort or a skip may end,
//!  while a path the worktree itself ignores is not a reason to refuse a clean in a linked worktree,
//!  and an untracked path only those patterns call ignored still is.
//!  They sit beside the check that asks them,
//!  `policy_linked_worktree.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const ended = endsAnOperation(command);
//! // const adds = cleanDeletesAPathOnlyItsExclusionsIgnore(facts, region);
//! ```

/// The builder of the clean's own dry run,
///  and the comparison of what each answer names.
use crate::command_clean::{
    clean_dry_run_arguments,
    clean_dry_run_arguments_without_exclusions,
    paths_only_the_exclusions_add,
};
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// What a guarded form does to the operation it finds waiting.
use crate::rule_abort_operation::ended_operation;
/// The linked-worktree rule core.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Whether the form ends the operation it finds waiting,
///  which is every abort form and every skip form.
/// Why:
///   Such a form is decided by its own reading wherever it runs,
///  so the refusal the linked-worktree rule gives an overwriting form in a main worktree
///  is not asked of it:
///  the owner decided on 2026-11-13 that the rule is the same in the main worktree and in a linked one,
///  an abort that would discard nothing runs and one that would discard work is refused
///  (`doc/handover/cli-git-rust-implementation.md`,
///  "An abort in the main worktree:
/// Same rule everywhere").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endsAnOperation = (command: GuardedCommand): boolean => endedOperation(command) !== undefined;
/// ```
pub(crate) fn ends_an_operation(command: GuardedCommand) -> bool {
    return ended_operation(command).is_some();
}

/// What:
///  Whether a `git clean -X` that carries its own `-e`/`--exclude` patterns would delete a path those patterns
///  alone call ignored.
///  `&[OsString]` borrows the clean's own tokens after the subcommand.
/// Why:
///   A pattern can pull two different things into a clean:
///  a path the worktree itself ignores is an ignored file,
///  which the owner decided is not a reason to refuse this clean in a linked worktree,
///  while a path only the patterns call ignored is an untracked file this clean would lose,
///  which still is.
///  Git answers both sets itself:
///  the dry run of the clean as typed and the dry run of the same clean without the patterns,
///  and what only the first answer names is what the patterns add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await cleanDeletesAPathOnlyItsExclusionsIgnore(facts, ['-fX', '-e', 'notes.txt'])
/// ```
pub(crate) fn clean_deletes_a_path_only_its_exclusions_ignore(
    facts: &mut dyn RepositoryFacts,
    region: &[OsString],
) -> Result<bool, String> {
    // The global options come before every start of Git, so they are read once for both dry runs.
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    // A region Git would refuse deletes nothing, and Git refuses it in its own words when the command runs.
    let Some(with_patterns) = clean_dry_run_arguments(prefix.as_slice(), region) else {
        return Ok(false);
    };
    let deleted: Vec<Vec<u8>> = facts.clean_deleted_paths(with_patterns)?;
    if deleted.is_empty() {
        return Ok(false);
    }
    // The same clean without them deletes what the worktree itself ignores; both answers name a path the same way,
    //  so they are compared as Git printed them.
    let Some(without_patterns) =
        clean_dry_run_arguments_without_exclusions(prefix.as_slice(), region)
    else {
        return Ok(true);
    };
    let otherwise_deleted: Vec<Vec<u8>> = facts.clean_deleted_paths(without_patterns)?;
    return Ok(
        !paths_only_the_exclusions_add(deleted.as_slice(), otherwise_deleted.as_slice()).is_empty(),
    );
}
