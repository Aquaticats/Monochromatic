//! What:
//!  The dirty refusal of the linked-worktree-only policy:
//!  the two places it names,
//!  and the text that names them.
//! Why:
//!  The refusal said "this worktree" and named no place,
//!  though a `!` alias's shell hands the nested Git one worktree's Git directory while the caller's own `-C`
//!  moves the files to another worktree,
//!  so a person could not tell which of them to clean (the alpha.52 read's L2).
//!  The text sits beside its siblings in a file of its own,
//!  because the module that holds every rejection text would otherwise pass the linter's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { describe, HATCH_ADVICE } from './rule_linked_worktree_messages.ts';
//! ```

/// The form's name,
///  what it can destroy,
///  and the alternative that fits it,
///  which every text of this policy shares.
use crate::rule_linked_worktree_messages::{
    HATCH_ADVICE,
    describe,
};
/// The guarded forms.
use crate::the_guarded_commands::GuardedCommand;
use std::path::Path;

/// What:
///  The state a dirty refusal names:
///  the guarded form,
///  the Git directory whose index and `HEAD` that form rewrites,
///  the worktree whose files it writes,
///  and whether that Git directory is the one that worktree uses.
/// Why:
///  The two are one place in every ordinary worktree,
///  and two when the index and the files come from different ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtyWorktree = { command: GuardedCommand; gitDirectory: string; worktreeRoot: string; oneWorktree: boolean };
/// ```
pub(crate) struct DirtyWorktree<'a> {
    /// The guarded form the text names.
    pub(crate) command: GuardedCommand,
    /// The Git directory whose index and `HEAD` the form rewrites.
    pub(crate) git_directory: &'a Path,
    /// The worktree whose files the form writes.
    pub(crate) worktree_root: &'a Path,
    /// Whether `git_directory` is the Git directory `worktree_root` uses.
    pub(crate) one_worktree: bool,
}

/// What:
///  The rejection text for a command run in a worktree that holds uncommitted state.
/// Why:
///  The state is what the command could destroy;
/// committing it with explicit paths or moving to a clean linked
///  worktree are the ways forward that lose nothing.
///  Each place the judgement read is named,
///  because the index and the files can be two worktrees' and only a text that names both
///  tells a person which one to clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyWorktreeMessage({ command, gitDirectory, worktreeRoot, oneWorktree }): string;
/// ```
pub(crate) fn dirty_worktree_message(dirty: DirtyWorktree<'_>) -> String {
    let DirtyWorktree {
        command,
        git_directory,
        worktree_root,
        one_worktree,
    } = dirty;
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    let what: &str = match command {
        GuardedCommand::Clean => "changes to tracked files or untracked files",
        // An ignored file is not a reason to refuse this clean in a linked worktree, so the text names only what
        //  the worktree holds and this clean loses.
        GuardedCommand::CleanIgnored => "changes to tracked files or untracked files",
        // Only the spelling that carries its own `-e`/`--exclude` patterns is asked about, and what those patterns
        //  pull in is a path the worktree does not itself ignore.
        GuardedCommand::CleanIgnoredOnly => {
            "work that only its own -e/--exclude patterns call ignored"
        }
        _ => "changes to tracked files",
    };
    let places: String = if one_worktree {
        format!(
            "the worktree `{}` has uncommitted {what}: {risk}. That worktree's index and HEAD are the ones in `{}`",
            worktree_root.display(),
            git_directory.display()
        )
    } else {
        format!(
            "the place it acts in has uncommitted {what}: {risk}. That place is two: the files of the worktree \
             `{}`, which this command writes, and the index and HEAD in `{}`, which it rewrites and which belong to \
             another worktree, so the work that would be lost is that index's staged entries and those files' own \
             edits",
            worktree_root.display(),
            git_directory.display()
        )
    };
    return format!(
        "monochromatic-git-policy-cli: {name} is rejected because {places}. {instead} {HATCH_ADVICE}"
    );
}
