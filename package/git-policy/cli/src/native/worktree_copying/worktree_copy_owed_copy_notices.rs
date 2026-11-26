//! What:
//!  Every line the pass that finishes an owed worktree copy prints:
//!  the one that names a copy owed into another worktree,
//!  the three that say what this command's own copy is doing,
//!  and the one that says it could not be made.
//! Why:
//!  The lines share their shape and their promise,
//! "the person is never left guessing what stands in the way",
//!  so they are read in one place,
//! away from the checks that decide which of them a command prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = owedInAnotherWorktreeNotice(destination);
//! stderr.write(finishingNotice(destination, source));
//! ```

use crate::json_record::quote;
use crate::shell_quote::shell_quote;
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

/// What:
///  The line a command prints when it meets a copy owed into another worktree,
/// and the worktree whose own commands fill it.
/// Why:
///   The work is the worktree's own,
/// so the person learns which worktree is short of files,
/// that no command pays for it here,
/// and that nothing is lost while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the ignored-state copy an interrupted \`git worktree add\` owed ${shown} is still owed; the first command run in that worktree makes it, and this command does not wait for it.`
/// ```
pub fn owed_in_another_worktree_notice(destination: &Path) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    return format!(
        "monochromatic-git-policy-cli: the ignored-state copy an interrupted `git worktree add` owed {shown} is still owed; the first command run in that worktree makes it, and this command does not wait for it."
    );
}

/// What:
///  The line that says this worktree's own command is about to make the copy it owes,
/// naming the worktree and the source.
/// Why:
///   The copy costs what the source's ignored tree costs,
/// which can be minutes;
/// a command that says what it is doing before it starts leaves nothing silent while it lasts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: finishing the ignored-state copy an interrupted \`git worktree add\` owed ${shown} from ${source}; this command waits for it, and it stops at the ceiling one command holds for a copy it did not start.`
/// ```
pub(crate) fn finishing_notice(
    destination: &Path,
    source: &Path,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: finishing the ignored-state copy an interrupted `git worktree add` owed {} from {}, because this command runs in that worktree and its files are missing there; it stops at the ceiling one command holds for a copy it did not start.",
        quote(destination.to_string_lossy().as_ref()),
        quote(source.to_string_lossy().as_ref())
    );
}

/// What:
///  The line that says the copy an interrupted add owed is still owed,
/// and why the command in that worktree did not make it.
/// Why:
///   A command that runs in the worktree its files are missing from must never leave the person guessing:
/// it either makes the copy or says what stands in the way,
/// and the record stays so the next command tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the ignored-state copy an interrupted \`git worktree add\` owed ${shown} is still owed, and this command did not make it: ${reason}`
/// ```
pub(crate) fn still_owed_notice(
    destination: &Path,
    reason: &str,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: the ignored-state copy an interrupted `git worktree add` owed {} is still owed, and this command did not make it: {reason}. Nothing was changed; the next command run in that worktree tries again.",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// What:
///  The line that says the copy an interrupted add owed was made,
/// and how much it copied.
/// Why:
///   The person reads one line naming the worktree,
/// the source and the count,
/// and knows nothing is left to do there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `finished the ignored-state copy an interrupted git worktree add owed ${destination}: copied ${copied} entries from ${source}.`
/// ```
pub(crate) fn finished_notice(
    destination: &Path,
    source: &Path,
    copied: usize,
) -> String {
    let entry_noun: &str = if copied == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: finished the ignored-state copy an interrupted `git worktree add` owed {}: copied {copied} ignored filesystem {entry_noun} from {}.",
        quote(destination.to_string_lossy().as_ref()),
        quote(source.to_string_lossy().as_ref())
    );
}

/// What:
///  The line that says the copy could not be made,
/// names the worktree,
/// what failed,
/// and the two commands that settle it.
/// Why:
///   The same words come again from the record of the unfilled worktree on every later command,
/// so they must say everything the person needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the ignored-state copy an interrupted git worktree add owed ${destination} could not be made (${reason}); ...`
/// ```
pub(crate) fn not_finished_notice(
    destination: &Path,
    reason: &str,
) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let commands: String = shell_quote(destination.as_os_str().as_bytes());
    return format!(
        "monochromatic-git-policy-cli: the ignored-state copy an interrupted `git worktree add` owed {shown} could not be made ({reason}), and nothing in that worktree was changed. `git worktree remove --force {commands}` removes that worktree, and running the same `git worktree add` line again copies the ignored state; this line comes again from every command in this repository until that worktree is removed or replaced."
    );
}
