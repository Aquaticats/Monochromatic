//! What:
//!  The sentences the word-level second pass ends a stopped operation with:
//! what it staged,
//! what is left,
//! and the command that ends the operation.
//! Why:
//!  A person reads these lines last,
//! so each one names the state the index and the Git directory hold at the moment it is printed,
//! and a continuation that stopped on the
//!  next change says so rather than that the conflict is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = unfinishedWords(operation, staged, conflicted, endingCommand(gitDir, operation));
//! ```

use crate::merge_second_pass::REPORT_MARK;
use std::path::Path;

/// The line prefix of every line this module writes.
pub(super) const LINE_PREFIX: &str = "monochromatic-git-policy-cli: ";

/// The sentence after the list of staged files.
pub(super) fn closing_words(operation: &str, remaining: usize) -> String {
    let rest: String = if remaining == 0 {
        format!("no conflicted file is left; finish the {operation} the way Git said above")
    } else {
        format!("resolve the files Git still lists as conflicted, then finish the {operation} the way Git said above")
    };
    return format!("Review a merged file with `git diff --cached -- <file>`; {rest}.");
}

/// What:
///  The command that ends an operation the wrapper did not finish.
/// Why:
///   The line after a failed finish must name a way on that works for the operation at hand;
/// Git ends a merge with `git commit`,
/// and a
///  rebase,
/// a cherry-pick and a revert with their own `--continue`.
/// Which operation is in progress is read from the state files Git writes,
/// so a `pull` that runs a rebase
///  is told the rebase's command,
/// and the operation's own word is only the fallback for a state file that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endingCommand = (gitDir, operation) => endingCommandOf(gitDir) ?? ({rebase: 'git rebase --continue', 'cherry-pick': 'git cherry-pick --continue', revert: 'git revert --continue'}[operation] ?? 'git commit');
/// ```
pub(super) fn ending_command(git_dir: &Path, operation: &str) -> &'static str {
    if let Some(in_progress) = crate::merge_second_pass_finish::ending_command_of(git_dir) {
        return in_progress;
    }
    return match operation {
        "rebase" => "`git rebase --continue`",
        "cherry-pick" => "`git cherry-pick --continue`",
        "revert" => "`git revert --continue`",
        _ => "`git commit`",
    };
}

/// What:
///  The line that says what is left after a finish that did not end the operation.
/// Why:
///   The ending step's own text is about that step alone:
/// a commit that could not be made ends with
///  "Nothing in the repository changed.",
/// which is true of the commit and false of the merge,
/// whose conflicts the pass
///  replaced with staged files before the commit started.
/// The last word a person reads must name the state they will find,
/// and a continuation that stopped on
///  the next change leaves files conflicted again,
/// so the conflict is not gone and saying it is describes a state nobody has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (operation, staged, conflicted, ending) => `${LINE_PREFIX}${REPORT_MARK}: the ${operation} did not finish: the files the second merge pass staged as resolved are ${staged}, and ${conflicted.length === 0 ? 'no file is conflicted' : `${conflicted.join(', ')} ${conflicted.length === 1 ? 'is' : 'are'} conflicted again where the ending step stopped`}; run ${ending} to end the ${operation}.`;
/// ```
pub(super) fn unfinished_words(operation: &str, staged: &str, conflicted: Option<&[String]>, ending: &str) -> String {
    let kept: String = format!("the files the second merge pass staged as resolved are {staged}");
    let Some(known) = conflicted else {
        // An index that cannot be read again is not a clean one, and the line must not say it is.
        return format!("{LINE_PREFIX}{REPORT_MARK}: the {operation} did not finish: {kept}, and the index could not be read again, so which files are conflicted now cannot be said; run {ending} to end the {operation}.");
    };
    if known.is_empty() {
        return format!("{LINE_PREFIX}{REPORT_MARK}: the {operation} did not finish: {kept}, and no file is conflicted; run {ending} to end the {operation}.");
    }
    let one: bool = known.len() == 1;
    return format!(
        "{LINE_PREFIX}{REPORT_MARK}: the {operation} did not finish: {kept}, and {} {} conflicted again where the ending step stopped; resolve {} and run {ending} to end the {operation}.",
        known.join(", "),
        if one { "is" } else { "are" },
        if one { "it" } else { "them" },
    );
}

/// The sentence that names the paths the pass left because they moved.
pub(super) fn moved_words(moved: &[String]) -> String {
    return format!(
        "{LINE_PREFIX}{REPORT_MARK}: changed while the second merge pass computed its merge, so it is left as it is now: {}. ",
        moved.join(", ")
    );
}

/// What:
///  The sentence that names the paths the pass left because it could not compare them.
/// Why:
///   Two failed reads are not two equal ones:
/// a path that could not be read says nothing about whether it still holds
///  what the merge was computed from;
/// the sentence must not claim the file changed when nobody read it,
/// and it must not
///  leave the person wondering why the file still conflicts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadableWords = (paths: string[]) => `${LINE_PREFIX}${REPORT_MARK}: could not be read to compare with what the merge was computed from, so it is left as it is now: ${paths.join(', ')}. `;
/// ```
pub(super) fn unreadable_words(unreadable: &[String]) -> String {
    return format!(
        "{LINE_PREFIX}{REPORT_MARK}: could not be read to compare with what the merge was computed from, so it is left as it is now: {}. ",
        unreadable.join(", ")
    );
}
