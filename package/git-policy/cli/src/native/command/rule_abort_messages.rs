//! What:
//!  The plain-language texts of the abort and skip guard:
//!  which command is refused,
//!  each path it would discard and what kind of work stands there,
//!  and the two ways forward a person has.
//! Why:
//!  An abort is what a person reaches for when an operation has them stuck,
//!  so the refusal has to leave them less stuck than the loss would:
//!  it names the work it would throw away,
//!  the command that stops the operation and keeps that work,
//!  and the opt-out that runs the command exactly as real Git runs it.
//!  Nothing is saved by the wrapper,
//!  so no sentence here may promise a copy.
//!  The wording names the command and the paths,
//!  never the wrapper's internals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! abortCommandText('merge', 'abort') === 'git merge --abort';
//! hatchCommandText('merge', 'abort') === 'git merge --no-enforce-worktree --abort';
//! discardRefusalText({ command: 'git merge --abort', paths, baseline }) // 'monochromatic-git-policy-cli: ...'
//! ```

use crate::repository_facts_abort_discard::DiscardBaseline;
use crate::rule_abort_operation::{AbortOperation, OperationEnd, operation_name};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

/// What:
///  The command an operation's abort or skip is spelled with.
/// Why:
///   The refusal names the command the person typed,
///  in the spelling they typed (the bare form),
///  so a reader can match the sentence to what they ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abortCommandText(operation: AbortOperation, ends: OperationEnd): string;
/// ```
pub fn abort_command_text(operation: AbortOperation, ends: OperationEnd) -> &'static str {
    return match (operation, ends) {
        (AbortOperation::Merge, _) => "git merge --abort",
        (AbortOperation::Rebase, OperationEnd::Abort) => "git rebase --abort",
        (AbortOperation::Pick, OperationEnd::Abort) => "git cherry-pick --abort or git revert --abort",
        (AbortOperation::Am, OperationEnd::Abort) => "git am --abort",
        (AbortOperation::Rebase, OperationEnd::Skip) => "git rebase --skip",
        (AbortOperation::Pick, OperationEnd::Skip) => "git cherry-pick --skip or git revert --skip",
        (AbortOperation::Am, OperationEnd::Skip) => "git am --skip",
    };
}

/// What:
///  The same command with the guard's opt-out in the one place that strips it,
///  directly after the subcommand.
/// Why:
///   A refusal that named no opt-out would leave the person with nothing to do,
///  and one that spelled the opt-out in the wrong place would not be stripped at all:
///  the wrapper reads the controls only in option position of the subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hatchCommandText(operation: AbortOperation, ends: OperationEnd): string;
/// ```
pub fn hatch_command_text(operation: AbortOperation, ends: OperationEnd) -> &'static str {
    return match (operation, ends) {
        (AbortOperation::Merge, _) => "git merge --no-enforce-worktree --abort",
        (AbortOperation::Rebase, OperationEnd::Abort) => "git rebase --no-enforce-worktree --abort",
        (AbortOperation::Pick, OperationEnd::Abort) => "git cherry-pick --no-enforce-worktree --abort",
        (AbortOperation::Am, OperationEnd::Abort) => "git am --no-enforce-worktree --abort",
        (AbortOperation::Rebase, OperationEnd::Skip) => "git rebase --no-enforce-worktree --skip",
        (AbortOperation::Pick, OperationEnd::Skip) => "git cherry-pick --no-enforce-worktree --skip",
        (AbortOperation::Am, OperationEnd::Skip) => "git am --no-enforce-worktree --skip",
    };
}

/// What:
///  The command that stops the operation and keeps the index and the files,
///  which is the way out that loses nothing.
/// Why:
///   Every operation has a quit form and none of them is a guarded form,
///  so a person who wants their resolution and their edits more than they want the operation gone
///  has one command to type and nothing to lose
///  (measured for every operation with Git 2.56.0:
/// the hand resolution stays in the index and the file).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// quitCommandText('merge') === 'git merge --quit';
/// ```
pub fn quit_command_text(operation: AbortOperation) -> &'static str {
    return match operation {
        AbortOperation::Merge => "git merge --quit",
        AbortOperation::Rebase => "git rebase --quit",
        AbortOperation::Pick => "git cherry-pick --quit or git revert --quit",
        AbortOperation::Am => "git am --quit",
    };
}

/// What:
///  The paths a sentence names,
/// the first few and how many more there are.
///  `&[String]` borrows the paths Git printed.
/// Why:
///   A sentence that named a thousand paths would bury its remedy,
///  and one that named none would leave the person guessing which file stands in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedPaths(paths: string[]): string;
/// ```
fn named_paths(paths: &[String]) -> String {
    return named_items(paths.iter().map(String::clone).collect::<Vec<String>>().as_slice());
}

/// What:
///  The first few of a list as one sentence fragment,
///  with how many the sentence leaves out.
/// Why:
///   Every refusal here lists paths under one bound,
///  and a reader who meets two bounds would not know which count is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedItems(items: string[]): string;
/// ```
fn named_items(items: &[String]) -> String {
    let mut listed: String = items.iter().take(SHARED_PATHS_NAMED).cloned().collect::<Vec<String>>().join(", ");
    if items.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more paths", items.len() - SHARED_PATHS_NAMED).as_str());
    }
    return listed;
}

/// What:
///  Whether this listing named the path.
/// Why:
///   A path's kind comes from which of the two listings named it,
///  and comparing the bytes Git printed keeps a name with a tab,
/// a newline or a backslash intact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (listing: string[], path: string): boolean => listing.some((each) => each === path);
/// ```
fn named(listing: &[Vec<u8>], path: &[u8]) -> bool {
    for each in listing {
        if each.as_slice() == path {
            return true;
        }
    }
    return false;
}

/// What:
///  What stands at one path that the command would discard,
///  in words that are true of the reading that named it.
///  `bool` says which listings named the path.
/// Why:
///   The person has to know which of the things they made is at risk:
///  an edit in the worktree,
///  a staged edit,
///  and,
///  where Git recorded the result of the conflict it stopped at,
///  a staged resolution whose content stands nowhere else.
///  A path the operation is still stopped at,
/// in a state Git recorded no result for,
///  is neither:
/// it holds Git's own conflict text or a resolution over it,
///  and nothing the wrapper can read tells the two apart,
///  so it is named as a conflicted path rather than as an edit somebody made.
///  Where no result is recorded the listings also compare the worktree with the index,
///  so a path only the index listing names is content the worktree holds too:
///  a sentence claiming it stood only in the index would be false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describePath('a.txt', { baseline: 'recorded-result', inWorktree: true, staged: true, conflicted: false });
/// ```
fn describe_path(baseline: DiscardBaseline, in_worktree: bool, staged: bool, conflicted: bool) -> &'static str {
    // The order is the order of certainty: a path the operation is still stopped at is named as that first,
    //  because what stands there may be Git's own text.
    if conflicted {
        return "a conflicted path";
    }
    if staged && in_worktree {
        return match baseline {
            DiscardBaseline::RecordedResult => "a resolution or edit you made since Git stopped at the conflict; a staged resolution",
            DiscardBaseline::OperationStart => "an edit you made since the operation began; a staged edit",
        };
    }
    if staged {
        return match baseline {
            DiscardBaseline::RecordedResult => "content only the index holds",
            DiscardBaseline::OperationStart => "content the index holds",
        };
    }
    return match baseline {
        DiscardBaseline::RecordedResult => "a resolution or edit you made since Git stopped at the conflict",
        DiscardBaseline::OperationStart => "an edit you made since the operation began",
    };
}

/// What:
///  Each path the command would discard,
///  with what stands there,
///  as one sentence fragment,
///  bounded to the paths a sentence names.
///  `&[Vec<u8>]` borrows the two listings and the paths the operation is still stopped at.
/// Why:
///   Paths that differ in both the worktree and the index are one item,
///  and a path that differs in either is what the command would discard,
///  so the items are the union of the two listings and each carries its own facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// discardedList({ baseline, worktree, staged, conflicted }) // 'a.txt (an edit you made since ...), b.txt (content the index holds)'
/// ```
pub fn discarded_list(baseline: DiscardBaseline, worktree: &[Vec<u8>], staged: &[Vec<u8>], conflicted: &[Vec<u8>]) -> String {
    let union: Vec<Vec<u8>> = crate::repository_facts_abort_discard::both_listings(worktree.to_vec(), staged.to_vec());
    let mut items: Vec<String> = Vec::<String>::with_capacity(union.len());
    for path in &union {
        let facts: &str = describe_path(
            baseline,
            named(worktree, path.as_slice()),
            named(staged, path.as_slice()),
            named(conflicted, path.as_slice()),
        );
        items.push(format!("{} ({facts})", String::from_utf8_lossy(path.as_slice())));
    }
    return named_items(items.as_slice());
}

/// What:
///  Whether the paths a refusal names include one the operation is still stopped at,
///  which is the case that has to say why the wrapper could not judge it.
///  `&[Vec<u8>]` borrows the two listings and the conflicted paths.
/// Why:
///   The sentence explains a refusal that looks over-careful otherwise:
///  a person who left a conflicted file exactly as Git wrote it is told why the wrapper still counted it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesAConflictedPath = ({ worktree, staged, conflicted }) => conflicted.some((path) => worktree.concat(staged).includes(path));
/// ```
fn names_a_conflicted_path(worktree: &[Vec<u8>], staged: &[Vec<u8>], conflicted: &[Vec<u8>]) -> bool {
    for path in conflicted {
        if named(worktree, path.as_slice()) || named(staged, path.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  The refusal of an abort or a skip that would discard work standing nowhere else,
///  naming each path,
///  what stands there,
///  the way out that keeps the work and the opt-out that discards it.
/// Why:
///   The command is the one a person reaches for when stuck,
///  so the refusal is written as two routes out and not as a wall:
///  the quit form stops the operation and keeps the index and the files,
///  and the opt-out runs the command exactly as real Git runs it.
///  Nothing is copied,
/// so nothing here says a copy was made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function discardRefusalText({ operation, ends, inProgress, baseline, worktree, staged }): string;
/// ```
pub fn discard_refusal_text(
    operation: AbortOperation,
    ends: OperationEnd,
    in_progress: AbortOperation,
    baseline: DiscardBaseline,
    worktree: &[Vec<u8>],
    staged: &[Vec<u8>],
    conflicted: &[Vec<u8>],
) -> String {
    let command: &str = abort_command_text(operation, ends);
    let word: &str = operation_name(in_progress);
    let halt: &str = match ends {
        OperationEnd::Abort => "restores the state before it",
        OperationEnd::Skip => "goes on without the commit or patch it stopped at",
    };
    // A refusal that counted a conflicted path says why, because the file may hold exactly what Git wrote there.
    let unexplained: &str = match names_a_conflicted_path(worktree, staged, conflicted) {
        true => " Git recorded no result for the conflict it stopped at, so nothing proves such a path holds the text Git wrote rather than a resolution of yours.",
        false => "",
    };
    return format!(
        "monochromatic-git-policy-cli: {command} is refused: it would discard work that stands nowhere else, and the {word} in progress still waits. \
         Nothing was changed, and nothing was copied. It would discard {}.{unexplained} To keep that work, stop the operation and keep your index and \
         files with {quit} (that command is allowed), then commit or save what you want to keep; {command} {halt}. To discard the work on purpose, run \
         {hatch}: the guard steps aside and Git does exactly what it does on its own.",
        discarded_list(baseline, worktree, staged, conflicted),
        quit = quit_command_text(in_progress),
        hatch = hatch_command_text(operation, ends)
    );
}

/// What:
///  A reading's reason as a sentence that ends in a full stop,
///  so the sentence after it starts where a reader expects.
///  `&str` borrows the reason.
/// Why:
///   A reason that came from Git's own words ends without a full stop,
///  and running a new sentence into it reads as if the two were one clause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = asSentence('could not read the operation state: fatal: not a git repository');
/// ```
fn as_a_sentence(message: &str) -> String {
    let trimmed: &str = message.trim_end();
    return match trimmed.ends_with('.') {
        true => trimmed.to_string(),
        false => format!("{trimmed}."),
    };
}

/// What:
///  The refusal of an abort or a skip whose own reading failed,
///  naming what could not be read and the two ways out that hold there.
/// Why:
///   A measurement that failed cannot say the command is safe,
///  so the command is refused;
///  the reason is the one the reading gave,
///  which names the path and the defect.
///  The quit form is the advice because it needs nothing of what could not be read,
///  while the command itself and its opt-out both send Git to that same path
///  (measured with Git 2.56.0:
/// with `rebase-merge/orig-head` gone,
///  `git rebase --abort` fails and `git rebase --quit` stops the rebase),
///  so a person is told that,
/// rather than sent into a read that cannot end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreadableRefusalText({ command, operation, message }): string;
/// ```
pub fn unreadable_refusal_text(command: AbortOperation, ends: OperationEnd, in_progress: AbortOperation, message: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} is refused because the wrapper could not read what it would discard, and it changes nothing it cannot \
         see: {} The {} in progress still waits. To stop the operation and keep your index and files, use {} (that command is allowed). The guard \
         steps aside with {} and Git then reads that path itself, so the command can wait there.",
        abort_command_text(command, ends),
        as_a_sentence(message),
        operation_name(in_progress),
        quit_command_text(in_progress),
        hatch_command_text(command, ends)
    );
}

/// What:
///  The refusal of an abort or a skip whose restored tree tracks a path where the worktree holds an untracked file.
///  `&str` is the revision whose tree the command writes back.
/// Why:
///   Git refuses the reset itself and names the files,
///  for an abort and for a skip alike
///  (measured with Git 2.56.0 for `merge --abort`,
/// `rebase --abort`,
/// `rebase --skip`,
///  `cherry-pick --skip` and `am --skip`:
///  `Untracked working tree file 'victim.txt' would be overwritten by merge`),
///  so nothing is lost and nothing is owed.
///  The wrapper's words have to say that it is Git that refuses,
///  because a person who read them as the wrapper's own decision would look for a second reason.
///  A plain `git reset --hard` writes such a file over silently,
///  which is why the measurement is each command's own and not a reset's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untrackedFileText({ operation, ends, inProgress, restored, paths }): string;
/// ```
pub fn untracked_file_text(operation: AbortOperation, ends: OperationEnd, in_progress: AbortOperation, restored: &str, paths: &[String]) -> String {
    let command: &str = abort_command_text(operation, ends);
    let listed: String = named_paths(paths);
    let (holding, tracks, there, them, whose): (&str, &str, &str, &str, &str) = if paths.len() == 1 {
        ("the untracked file", "it", "it is", "it", "its")
    } else {
        ("untracked files", "them", "they are", "them", "their")
    };
    return format!(
        "monochromatic-git-policy-cli: {command} is refused because Git itself refuses it here: the worktree holds {holding} {listed}, and this \
         command writes the tree of {restored}, which tracks {tracks}, so Git will not write that tree while {there} there. The {word} in progress \
         still waits, and nothing was changed. Move {them} out of the way (mv <path> <somewhere else>), or copy {them} somewhere else to keep {whose} \
         content, and run {command} again: Git then writes the tree and the command goes on as Git runs it.",
        word = operation_name(in_progress)
    );
}

/// What:
///  The refusal of an abort or a skip whose restored tree tracks a path where the worktree holds an ignored file.
///  `&[String]` borrows the paths Git printed and `&str` the message the shared ignored-file reading gives.
/// Why:
///   An ignored file is where a person keeps their environment and their local secrets,
///  and the command writes that tree back over it;
///  the shared sentence already names the paths and the way through,
///  and what is added here is that the operation still waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredFileRefusalText({ message, inProgress }): string;
/// ```
pub fn ignored_file_refusal_text(message: String, in_progress: AbortOperation) -> String {
    return format!(
        "{} The {} in progress still waits, and nothing was changed.",
        as_a_sentence(message.as_str()),
        operation_name(in_progress)
    );
}

/// The refusals and the advice they carry.
#[cfg(test)]
#[path = "rule_abort_messages_tests.rs"]
mod tests;
