//! What:
//!  The rejection texts of the linked-worktree policy:
//! outside any worktree,
//! in the main worktree,
//! and in a
//!  worktree that holds uncommitted state.
//! Why:
//!  Each text names the command,
//! says what it could destroy,
//! guesses what the person was trying to do,
//! and names a
//!  sanctioned way to do that without losing work;
//! the deliberate way to discard anyway comes last.
//! Every
//!  alternative named here is run through the built executable by `binary_discard_tests.rs`,
//! and the README lists
//!  each form with the same alternatives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // mainWorktreeMessage('restore-staged') === 'monochromatic-git-policy-cli: git restore --staged is rejected in the main git worktree: ...'
//! ```

use crate::rule_linked_worktree::GuardedCommand;
/// The paths a message names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
/// The operation that is waiting for its conflicts,
/// whose own state `git status` reports.
use crate::conflict_sides::Operation;
use std::ffi::OsStr;
use std::path::Path;

/// What:
///  The sentence every text ends with:
///  the one deliberate way past the refusal.
/// Why:
///   The work is still where the person left it, because the refusal ran before Git did,
///  so the advice names only the option and where it goes.
///  It never advises a copy `git diff` makes:
///  a `git diff > backup.patch` holds no untracked file,
///  no ignored file and no edit the index hides behind `assume-unchanged`,
///  which are three of the entries these refusals name,
///  and advice that saves nothing is worse than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HATCH_ADVICE = '... pass --no-enforce-worktree directly after the subcommand, for example git restore --no-enforce-worktree -- <path>.';
/// ```
pub(crate) const HATCH_ADVICE: &str = "Nothing was run or changed, so your work is where you left it. If you have decided to discard it on purpose, pass --no-enforce-worktree directly after the subcommand, for example git restore --no-enforce-worktree -- <path>.";

/// What:
///  How a text names the guarded form,
/// what the form can destroy,
/// and what the person most likely wanted.
///  A tuple is a fixed group of values,
/// here (name,
/// risk,
/// alternatives).
/// Why:
///   One subcommand serves several intents,
/// so the alternatives differ by form:
/// unstaging is not reverting.
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
        GuardedCommand::StashDrop => return (
            "git stash drop (or git stash clear)",
            "it deletes entries of the one stash the whole repository shares, and git stash clear deletes every entry, so work parked in any worktree of this repository is gone",
            "git stash list and git stash show read the stash without changing it (allowed). To keep the work, move it into its own branch with git stash branch <branch> in a clean linked worktree, which applies the entry and keeps it in the branch; a stash form that changes state is refused in the main worktree, and in a linked worktree that holds uncommitted changes. To drop one entry on purpose, first run git stash branch <branch> in a clean linked worktree, which applies the entry and keeps its work, and then drop it with the hatch below.",
        ),
        GuardedCommand::Clean => return (
            "git clean",
            "it deletes untracked files that Git cannot bring back",
            "To see what would be deleted, run git clean -n (allowed). To remove specific files, delete them by name with rm. To clear build output, use the project's own clean task. For a pristine copy of the code, create a linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::CleanIgnored => return (
            "git clean -x",
            "it deletes ignored files as well as untracked ones, and an ignored file is where a person keeps their environment and their local secrets",
            "To see what would be deleted, run git clean -nx (allowed). To delete only untracked files and keep the ignored ones, pass --no-enforce-worktree directly after clean (`git clean --no-enforce-worktree -fd`), because a clean that deletes files is refused in the main worktree, and in a linked worktree while it holds uncommitted changes or untracked files. For a pristine copy of the code, create a linked worktree (git worktree add <path> HEAD), and to clear ignored build output, use the project's own clean task.",
        ),
        GuardedCommand::CleanIgnoredOnly => return (
            "git clean -X",
            "it deletes ignored files, which is where a person keeps their environment and their local secrets",
            "To see what would be deleted, run git clean -nX (allowed). To remove the ignored entries you no longer need, delete them by name with rm, or use the project's own clean task for its build output. For a pristine copy of the code, create a linked worktree (git worktree add <path> HEAD).",
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
            "To compare with the committed version, use git diff -- <path> or git show HEAD:<path>. To keep the file as it is now, copy it elsewhere first (cp -- <path> <a directory outside the worktree>), because a patch from git diff holds no untracked file, no ignored file and no edit the index marks assume-unchanged; then use the hatch below. For a fresh copy of the committed files, create a linked worktree (git worktree add <path> HEAD).",
        ),
        GuardedCommand::CheckoutPaths => return (
            "git checkout with paths, --force, --patch, --ours or --theirs",
            "it overwrites files with other content, so your edits to them are gone",
            "To compare with the committed version, use git diff -- <path> or git show HEAD:<path>. To keep the file as it is now, copy it elsewhere first (cp -- <path> <a directory outside the worktree>), because a patch from git diff holds no untracked file, no ignored file and no edit the index marks assume-unchanged; then use the hatch below. For another branch's files, open that branch in a linked worktree (git worktree add <path> <branch>).",
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
            "git rm <path> without --force removes an unmodified file and refuses a modified one. To stop tracking a file and keep it on disk, use git rm --cached <path> (allowed). To delete a modified file on purpose, copy it elsewhere first (cp -- <path> <a directory outside the worktree>) and use the hatch below.",
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
            "git read-tree",
            "it replaces the index and, with -u, the worktree files, so staged entries and the worktree content they carried are gone",
            "To see what a read would change without writing anything, run git read-tree --dry-run with the same arguments (allowed). To look at a tree without touching the index, use git ls-tree or git cat-file (allowed). To replace the index on purpose, run it in a clean linked worktree (git worktree add <path> HEAD).",
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
        GuardedCommand::SkipRebase => return (
            "git rebase --skip",
            "it drops the commit the rebase stopped at and discards the edits made since, including hand resolutions of its conflicts",
            "To keep your index and files as they are, use git rebase --quit (allowed). To finish the commit instead, resolve the conflicts and use git rebase --continue.",
        ),
        GuardedCommand::SkipPick => return (
            "git cherry-pick --skip (or git revert --skip)",
            "it drops the commit the sequence stopped at and discards the edits made since, including hand resolutions of its conflicts",
            "To stop and keep your index and files as they are, use --quit with the same command (allowed). To finish the commit instead, resolve the conflicts and use --continue.",
        ),
        GuardedCommand::SkipAm => return (
            "git am --skip",
            "it drops the patch the series stopped at and discards the edits made since",
            "To stop and keep your index and files as they are, use git am --quit (allowed). To finish the patch instead, resolve the conflicts and use git am --continue.",
        ),
    }
}

/// What:
///  The rejection text for a `git checkout` or `git restore` that would replace a path the index marks as unmerged
///  while a merge,
/// a rebase,
/// a cherry-pick or a revert waits for its conflicts.
///  `&[String]` borrows the repository-relative paths the command would replace;
///  `bool` is whether the command runs in the main worktree.
/// Why:
///   The file of a conflict is where the person's own resolution of it lives,
///  and nothing else holds it:
///  the index holds the three sides,
///  and this program keeps no copy.
///  So the text names every such file,
///  says that no copy is kept either way,
///  offers the way to settle the conflict that loses nothing,
///  and names the opt-out that runs the command as real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// conflictedFilesMessage({ command, operation, inMainWorktree, paths });
/// ```
pub fn conflicted_files_message(command: GuardedCommand, operation: Operation, in_main_worktree: bool, paths: &[String]) -> String {
    let (name, risk, _instead): (&str, &str, &str) = describe(command);
    let place: &str = if in_main_worktree {
        "in the main git worktree"
    } else {
        "because this worktree has uncommitted changes to tracked files"
    };
    let mut named: Vec<String> = Vec::<String>::with_capacity(SHARED_PATHS_NAMED.min(paths.len()) + 1);
    for path in paths.iter().take(SHARED_PATHS_NAMED) {
        named.push(format!("`{path}`"));
    }
    let mut listed: String = named.join(", ");
    if paths.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", paths.len() - SHARED_PATHS_NAMED).as_str());
    }
    let (subject, pronoun, them): (&str, &str, &str) = if paths.len() == 1 { ("it holds", "its", "it") } else { ("they hold", "their", "them") };
    return format!(
        "monochromatic-git-policy-cli: {name} is rejected {place}: {risk}. The {} in progress left {listed} with an \
         unmerged index entry, and {subject} the resolution you have written so far, which exists nowhere else: this \
         command replaces {pronoun} content, and it keeps no copy of {them}. To settle the conflict and keep what you \
         wrote, edit the file, stage it (`git add -- <paths>`) and commit the result with no paths (`git commit --message \
         <message>`); `git show :1:<path>`, `git show :2:<path>` and `git show :3:<path>` print the base \
         and the two sides, and `git diff -- <path>` prints the file as it stands. If you have decided to replace {them} on purpose, copy the file first \
         (`cp -- <path> <path>.before-the-opt-out`) and pass `--no-enforce-worktree` directly after the subcommand: the \
         command then runs exactly as real Git runs it, and no copy is kept; `cp -- <path>.before-the-opt-out <path>` puts \
         your file back afterwards, and you can delete the copy then.",
        operation_name(operation)
    );
}

/// What:
///  How a message names the operation that waits for its conflicts.
/// Why:
///   The person typed one of Git's own words,
/// so the sentence uses that word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// operationName(Operation.CherryPick) === 'cherry-pick'
/// ```
fn operation_name(operation: Operation) -> &'static str {
    return match operation {
        Operation::Merge => "merge",
        Operation::Rebase => "rebase",
        Operation::CherryPick => "cherry-pick",
        Operation::Revert => "revert",
    };
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
        "monochromatic-git-policy-cli: {name} requires the effective working directory to be inside a linked git worktree: {risk}. \
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
        "monochromatic-git-policy-cli: {name} is rejected in the main git worktree: {risk}, and nothing records which command did it. \
         {instead} {HATCH_ADVICE}"
    );
}

/// What:
///  The sentence that says which variable exempted a repository from the worktree guard.
///  `&str` borrows the two facts and `&Path` the directory the variable named.
/// Why:
///   The exemption is chosen by the invocation's own environment,
///  so a variable that happens to name the
///       repository exempts it and nothing measured or saved anything;
///  a person who is told that before Git runs can
///       stop the command or unset the variable.
///  The sentence names both paths the exemption asks for,
///  so a worktree added outside that directory reads as the person's own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exemption_notice(variable, directory: string): string;
/// ```
pub fn exemption_notice(variable: &str, directory: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: this repository is exempt from the worktree guard because {variable} names {}, which contains both its Git directory and its worktree. \
         Nothing was measured and nothing was saved, so the command runs without that guard. \
         Hint: if {variable} does not describe a tool cache on this machine, unset it for this command (env --unset={variable} git ...) or run the command from a shell where it does not name this directory.\n",
        directory.display()
    );
}

/// What:
///  The rejection text for a `git stash drop` or `git stash clear` that would delete an entry of the
///  repository's one stash.
/// Why:
///   The entry belongs to the repository,
/// not to the worktree the command runs in,
///  so the text says where
///       the stash lives and what keeps the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryStashMessage({ command }): string;
/// ```
pub fn repository_stash_message(command: GuardedCommand) -> String {
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    return format!(
        "monochromatic-git-policy-cli: {name} is rejected because this repository holds a stash entry: {risk}, and it is not this worktree's. \
         {instead} {HATCH_ADVICE}"
    );
}

/// What:
///  The rejection text for a command run in a worktree that holds uncommitted state.
/// Why:
///   The state is what the command could destroy;
/// committing it with explicit paths or moving to a clean linked
///  worktree are the ways forward that lose nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyWorktreeMessage({ command }): string;
/// ```
pub fn dirty_worktree_message(command: GuardedCommand) -> String {
    return dirty_worktree_message_naming(command, &[]);
}

/// What:
///  The same rejection with the paths the worktree holds that the command would destroy,
///  each named up to the bound and the rest counted.
///  `&[String]` borrows them;
///  an empty list says the reading did not answer, and the text names no path then.
/// Why:
///   The rule is that a refusal names what it would destroy:
///  a person acts on a path,
///  not on the kind of change,
///  and a path they recognise is what tells them whether the command was meant.
///  The reading runs only on the refusal path, so a command that runs pays nothing for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = (command, paths) => `... has uncommitted ${what}${names(paths)}: ${risk}. ${instead} ${HATCH_ADVICE}`;
/// ```
pub fn dirty_worktree_message_naming(command: GuardedCommand, paths: &[String]) -> String {
    let (name, risk, instead): (&str, &str, &str) = describe(command);
    let what: &str = match command {
        GuardedCommand::Clean => "changes to tracked files or untracked files",
        // An ignored file is not a reason to refuse this clean in a linked worktree, so the text names only what
        //  the worktree holds and this clean loses.
        GuardedCommand::CleanIgnored => "changes to tracked files or untracked files",
        // Only the spelling that carries its own `-e`/`--exclude` patterns is asked about, and what those patterns
        //  pull in is a path the worktree does not itself ignore.
        GuardedCommand::CleanIgnoredOnly => "work that only its own -e/--exclude patterns call ignored",
        _ => "changes to tracked files",
    };
    return format!(
        "monochromatic-git-policy-cli: {name} is rejected because this worktree has uncommitted {what}{}: {risk}. \
         {instead} {HATCH_ADVICE}",
        named_paths(paths)
    );
}

/// What:
///  The paths a rejection names between parentheses,
///  each in backticks,
///  with a count of the rest past the bound,
///  and nothing when the reading answered with no path.
/// Why:
///  A worktree holding thousands of changed paths would otherwise print a list nobody reads,
///  and a reading that did not answer must not claim the worktree holds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (paths) => paths.length === 0 ? '' : ` (${paths.slice(0, 5).map((p) => '`' + p + '`').join(', ')}${rest ? ` and ${rest} more` : ''})`;
/// ```
fn named_paths(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let mut named: Vec<String> = Vec::<String>::with_capacity(SHARED_PATHS_NAMED.min(paths.len()));
    for path in paths.iter().take(SHARED_PATHS_NAMED) {
        named.push(format!("`{path}`"));
    }
    let rest: usize = paths.len().saturating_sub(named.len());
    let and_more: String = match rest {
        0 => String::new(),
        _ => format!(" and {rest} more"),
    };
    return format!(" ({}{and_more})", named.join(", "));
}

/// What:
///  The rejection text for a forced checkout,
/// a forced switch or a hard reset whose target tree tracks a path the
///  worktree has ignored.
///  `&[String]` borrows the shared paths;
///  `Option<&OsStr>` is the revision the command named.
/// Why:
///   The tracked state of the worktree is clean,
/// so the usual text would say it is dirty;
///  this one names
///       the ignored files the command would replace with the tree's content,
///  which is where a person
///       keeps their environment and their local secrets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredFileMessage({ command, revision, paths }): string;
/// ```
pub fn ignored_file_message(command: GuardedCommand, revision: Option<&OsStr>, paths: &[String]) -> String {
    let (name, _risk, instead): (&str, &str, &str) = describe(command);
    let tree: &str = match revision {
        Some(named) => &named.to_string_lossy(),
        None => "HEAD",
    };
    let mut named: Vec<String> = Vec::<String>::with_capacity(SHARED_PATHS_NAMED.min(paths.len()) + 1);
    for path in paths.iter().take(SHARED_PATHS_NAMED) {
        named.push(path.clone());
    }
    let mut listed: String = named.join(", ");
    if paths.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", paths.len() - SHARED_PATHS_NAMED).as_str());
    }
    return format!(
        "monochromatic-git-policy-cli: {name} is rejected because this worktree ignores {listed}, which the tree of \
         {tree} tracks: Git would replace {} with the content that tree carries, and an ignored file is where a person \
         keeps their environment and their local secrets. Only the tracked state of the worktree is clean, and a forced \
         form overrides Git's own --no-overwrite-ignore. {instead} {HATCH_ADVICE}",
        if paths.len() == 1 { "it" } else { "them" }
    );
}
