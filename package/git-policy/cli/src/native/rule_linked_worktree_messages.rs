//! What:
//!  The rejection texts of the linked-worktree policy: outside any worktree, in the main worktree, and in a
//!  worktree that holds uncommitted state.
//! Why:
//!  Each text names the command, says what it could destroy, guesses what the person was trying to do, and names a
//!  sanctioned way to do that without losing work; the deliberate way to discard anyway comes last. Every
//!  alternative named here is run through the built executable by `binary_discard_tests.rs`, and the README lists
//!  each form with the same alternatives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // mainWorktreeMessage('restore-staged') === 'cli-git: git restore --staged is rejected in the main git worktree: ...'
//! ```

use super::rule_linked_worktree::GuardedCommand;

/// The sentence every text ends with: the one deliberate way to discard work anyway.
const HATCH_ADVICE: &str = "If you have decided to discard the work on purpose, save a copy first (git diff > backup.patch) and pass --no-enforce-worktree directly after the subcommand, for example git restore --no-enforce-worktree -- <path>.";

/// What:
///  How a text names the guarded form, what the form can destroy, and what the person most likely wanted.
///  A tuple is a fixed group of values, here (name, risk, alternatives).
/// Why:
///   One subcommand serves several intents, so the alternatives differ by form: unstaging is not reverting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(command: GuardedCommand): { name: string; risk: string; instead: string };
/// ```
fn describe(command: GuardedCommand) -> (&'static str, &'static str, &'static str) {
    match command {
        GuardedCommand::Stash => return (
            "git stash",
            "it moves your uncommitted changes out of the worktree, and a later pop or drop can lose them",
            "To park work in progress, commit it with explicit paths (git commit --message <message> -- <paths>) on a topic branch, ideally in its own linked worktree (git worktree add -b <branch> <path>). Reading stashes is allowed: git stash list and git stash show.",
        ),
        GuardedCommand::Clean => return (
            "git clean",
            "it deletes untracked files that Git cannot bring back",
            "To see what would be deleted, run git clean -n (allowed). To remove specific files, delete them by name with rm. To clear build output, use the project's own clean task. For a pristine copy of the code, create a linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::ResetHard => return (
            "git reset --hard (or --merge, --keep)",
            "it overwrites tracked files and staged entries with committed content",
            "To undo a commit and keep its changes, use git reset --soft <commit> (allowed). To compare with a commit, use git diff <commit> or git show <commit>:<path>. For a clean copy of a commit, create a linked worktree (git worktree add <path> <commit>).",
        ),
        GuardedCommand::ResetMixed => return (
            "git reset",
            "it replaces the index, which unstages everything it covers",
            "To unstage one path you rarely need to: git commit --message <message> -- <paths> commits only the paths you name and ignores other staged entries, and git diff --cached --name-only lists what is staged. To move the branch and keep your changes staged, use git reset --soft <commit> (allowed).",
        ),
        GuardedCommand::RestoreStaged => return (
            "git restore --staged",
            "it removes staged entries, and staged work such as a batch of renames is not recorded anywhere else",
            "To commit only some of what is staged, use git commit --message <message> -- <paths>, which leaves other staged entries out. To list what is staged, run git diff --cached --name-only. To rework the index freely, do it in a clean linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::RestoreWorktree => return (
            "git restore",
            "it overwrites files with committed content, so your edits to them are gone",
            "To compare with the committed version, use git diff -- <path> or git show HEAD:<path>. To revert a file you are sure about, save a patch first (git diff -- <path> > backup.patch) and then use the hatch below. For a fresh copy of the committed files, create a linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::CheckoutPaths => return (
            "git checkout with paths, --force, --patch, --ours or --theirs",
            "it overwrites files with other content, so your edits to them are gone",
            "To compare with the committed version, use git diff -- <path> or git show HEAD:<path>. To revert a file you are sure about, save a patch first (git diff -- <path> > backup.patch) and then use the hatch below. For another branch's files, open that branch in a linked worktree (git worktree add <path> <branch>).",
        ),
        GuardedCommand::CheckoutMove => return (
            "git checkout <branch>",
            "it switches branches while uncommitted changes exist, which can carry them over, conflict with them, or overwrite a file that shares the name",
            "To switch branches, first commit your work with explicit paths (git commit --message <message> -- <paths>), or leave this worktree alone and open the other branch in its own linked worktree (git worktree add <path> <branch>).",
        ),
        GuardedCommand::SwitchForce => return (
            "git switch --force (or --discard-changes)",
            "it throws local changes away to switch branches",
            "To switch branches, first commit your work with explicit paths (git commit --message <message> -- <paths>), or open the other branch in its own linked worktree (git worktree add <path> <branch>).",
        ),
        GuardedCommand::SwitchMove => return (
            "git switch <branch>",
            "it switches branches while uncommitted changes exist, which can carry them over or conflict with them",
            "To switch branches, first commit your work with explicit paths (git commit --message <message> -- <paths>), or leave this worktree alone and open the other branch in its own linked worktree (git worktree add <path> <branch>).",
        ),
        GuardedCommand::Remove => return (
            "git rm --force",
            "it deletes files even when they hold uncommitted changes",
            "git rm <path> without --force removes an unmodified file and refuses a modified one. To stop tracking a file and keep it on disk, use git rm --cached <path> (allowed). To delete a modified file on purpose, save a patch first and use the hatch below.",
        ),
        GuardedCommand::Move => return (
            "git mv --force",
            "it overwrites an existing destination",
            "Choose a destination that does not exist, or remove the old destination first with git rm <destination>, then run git mv without --force.",
        ),
        GuardedCommand::CheckoutIndex => return (
            "git checkout-index --force",
            "it overwrites worktree files with the staged content",
            "To get copies of staged files without touching the worktree, write them elsewhere: git checkout-index --prefix=<directory>/ --all (allowed).",
        ),
        GuardedCommand::ReadTree => return (
            "git read-tree --reset",
            "it replaces the index, and with -u the worktree files, discarding local entries",
            "To merge a tree into the index use git read-tree -m (allowed), or run the reset in a clean linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::AbortMerge => return (
            "git merge --abort",
            "it restores the state before the merge and discards the edits made since, including conflict resolutions",
            "To stop the merge and keep your index and files as they are, use git merge --quit (allowed). To finish it, resolve the conflicts and commit with explicit paths.",
        ),
        GuardedCommand::AbortRebase => return (
            "git rebase --abort",
            "it resets the branch to its state before the rebase and discards the edits made since, including conflict resolutions",
            "To stop the rebase and keep your index and files as they are, use git rebase --quit (allowed). To continue, resolve the conflicts and use git rebase --continue, or git rebase --skip to drop the current commit.",
        ),
        GuardedCommand::AbortPick => return (
            "git cherry-pick --abort (or git revert --abort)",
            "it restores the state before the sequence and discards the edits made since, including conflict resolutions",
            "To stop and keep your index and files as they are, use --quit with the same command (allowed). To continue, resolve the conflicts and use --continue, or --skip to drop the current commit.",
        ),
        GuardedCommand::AbortAm => return (
            "git am --abort",
            "it restores the state before the patch series and discards the edits made since",
            "To stop and keep your index and files as they are, use git am --quit (allowed). To continue, resolve the conflicts and use git am --continue, or git am --skip to drop the current patch.",
        ),
    }
}

/// What:
///  The rejection text for a command run where there is no worktree.
/// Why:
///   The text says what the command could destroy and how to run it safely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outsideWorktreeMessage({ command }): string;
/// ```
pub fn outside_worktree_message(command: GuardedCommand) -> String {
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    return format!(
        "cli-git: {name} requires the effective working directory to be inside a linked git worktree: {risk}. \
         cd to a linked worktree root or pass -C <linked-worktree-root> before the subcommand. {instead}"
    );
}

/// What:
///  The rejection text for a command run in the main worktree.
/// Why:
///   The text says what the command could destroy and what to do instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mainWorktreeMessage({ command }): string;
/// ```
pub fn main_worktree_message(command: GuardedCommand) -> String {
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    return format!(
        "cli-git: {name} is rejected in the main git worktree: {risk}, and nothing records which command did it. \
         {instead} {HATCH_ADVICE}"
    );
}

/// What:
///  The rejection text for a command run in a worktree that holds uncommitted state.
/// Why:
///   The state is what the command could destroy; committing it with explicit paths or moving to a clean linked
///  worktree are the ways forward that lose nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyWorktreeMessage({ command }): string;
/// ```
pub fn dirty_worktree_message(command: GuardedCommand) -> String {
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    let what: &str = if command == GuardedCommand::Clean {
        "changes to tracked files or untracked files"
    } else {
        "changes to tracked files"
    };
    return format!(
        "cli-git: {name} is rejected because this worktree has uncommitted {what}: {risk}. \
         {instead} {HATCH_ADVICE}"
    );
}
