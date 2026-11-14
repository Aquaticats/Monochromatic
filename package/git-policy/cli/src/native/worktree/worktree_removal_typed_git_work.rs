//! What:
//!  The Git work a typed `git worktree remove` would destroy,
//!  asked once before the wrapper chooses how to send that command Git's way,
//!  and the words a removal refused for it prints.
//! Why:
//!  The guard of a removal is asked where the removal is made,
//!  and a typed removal reaches that place only by the copy route.
//!  A command that skips the copy (`--no-worktree-copy`),
//!  a form the removal plan does not serve,
//!  or a command that stands outside a worktree is handed to real Git as it stands,
//!  with no code of this program left to ask anything:
//!  Git then deletes a tracked edit a mark hides,
//!  and a commit only the worktree's `HEAD` names,
//!  without a word
//!  (a review of `bab700404d`,
//!  finding 1).
//!  So the questions are asked here,
//!  in the one place every forwarded command passes before a route is chosen,
//!  and a refusal stops the command with the same words the removal's own guard prints
//!  (`git_work_refusal`, below).
//!  Three states ask nothing:
//!  a forced removal, which is Git's own `--force` and deletes the same set;
//!  a command that carries the guard's opt-out,
//!  which runs as real Git runs it
//!  (`wrapper_controls::worktree_removal_escaped`);
//!  and a form that names no worktree this program can resolve under the repository's registrations,
//!  which Git refuses or reads as a force of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = refusalForATypedRemoval(command, escapedByThePerson, checks); // string | undefined
//! const words = gitWorkRefusal(target, reasons); // the report both guards print
//! ```

use crate::forwarded_command::ResolvedCommand;
use crate::policy_checks::ShippedChecks;
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::start_directory;
use crate::worktree_removal_hidden_edits::hidden_edits_beneath;
use crate::worktree_removal_keeps_git_work::git_work_a_removal_would_strand;
use crate::worktree_remove_plan::{RemovePlan, parse_remove_plan};
use crate::worktree_remove_registry::{Registration, registration_of};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The prefix every report of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  The refusal of a typed removal that would destroy Git work,
///  or nothing when this invocation is no removal,
///  when it was forced,
///  when the person escaped the guard,
///  or when nothing of the person's is at stake.
/// Why:
///   Every route a forwarded command takes passes this question before the route is chosen,
///  so no flag that skips another step can skip this one.
///  A repository whose common directory cannot be read asks nothing here,
///  because Git itself refuses a removal there,
///  and a target no registration names is no worktree a removal takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusalForATypedRemoval(command, escaped, checks) // string | undefined
/// ```
pub(crate) fn refusal_for_a_typed_removal<F: RepositoryFacts>(command: &ResolvedCommand, escaped_by_the_person: bool, checks: &mut ShippedChecks<F>) -> Option<String> {
    if escaped_by_the_person || !names_a_removal(command) {
        return None;
    }
    let common_dir: PathBuf = checks.facts.known_common_dir()?;
    let plan: RemovePlan = plan_of_the_removal(command, common_dir.as_path())?;
    if plan.force {
        return None;
    }
    let registration: Registration = registration_of(common_dir.as_path(), plan.target.as_path(), plan.operand.as_os_str()).ok()?;
    let context: GitContext = checks.facts.transaction_context();
    let mut reasons: Vec<String> = match git_work_a_removal_would_strand(&context, common_dir.as_path(), plan.target.as_path(), registration.admin.as_path()) {
        Ok(stranded) => stranded,
        Err(reason) => vec![format!("nothing proves no commit of this worktree is named by nothing, so nothing is removed: {reason}")],
    };
    match hidden_edits_beneath(&context, plan.target.as_path(), registration.admin.as_path(), common_dir.as_path()) {
        Ok(mut edits) => reasons.append(&mut edits),
        Err(reason) => reasons.push(format!("nothing proves no edit there is hidden from Git, so nothing is removed: {reason}")),
    }
    if reasons.is_empty() {
        return None;
    }
    return Some(git_work_refusal(plan.target.as_path(), reasons.as_slice()));
}

/// What:
///  Whether the resolved command is a `git worktree remove`,
///  read from the words Git will receive.
/// Why:
///   The two questions below cost a registration read and Git starts,
///  and every other command must pay none of that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesARemoval(command) // boolean
/// ```
fn names_a_removal(command: &ResolvedCommand) -> bool {
    let word = |at: usize| -> Option<&[u8]> { return command.arguments.get(at).map(|one| return one.as_encoded_bytes()) };
    return word(command.subcommand_index) == Some(b"worktree") && word(command.subcommand_index + 1) == Some(b"remove");
}

/// What:
///  The removal this command names,
///  resolved the way the copy route resolves it,
///  or nothing for a form the plan does not serve.
/// Why:
///  The target is relative to the directory Git starts in,
///  which the global `-C` options move,
///  and Git picks a registered worktree by the last components of its path before it reads the name as a path,
///  so the same reading the ladder uses is the one the guard needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planOfTheRemoval(command, commonDir) // RemovePlan | undefined
/// ```
fn plan_of_the_removal(command: &ResolvedCommand, common_dir: &Path) -> Option<RemovePlan> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    let arguments: &[OsString] = command.arguments.as_slice();
    return parse_remove_plan(arguments, command.subcommand_index, start.as_path(), Some(common_dir));
}

/// What:
///  The report of a removal refused because it would destroy Git work.
/// Why:
///   Two guards print it:
///  the one where the removal is made,
///  and the one a typed command meets before any route,
///  so one function words both.
///  The path is quoted,
///  because the report hands the person a command to copy and run,
///  and a path with a space,
///  a quote,
///  a dollar sign or a line break otherwise runs as something else
///  (a review of `bab700404d`,
///  finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitWorkRefusal(target, reasons) // the report
/// ```
pub(crate) fn git_work_refusal(target: &Path, reasons: &[String]) -> String {
    let quoted: String = crate::shell_quote::shell_quote(target.as_os_str().as_encoded_bytes());
    return format!(
        "{PREFIX}{} keeps Git work that a removal would destroy, or that nothing here proves it would not, so nothing was removed: {}. If this worktree must go anyway, `git worktree --no-enforce-worktree remove {quoted}` runs as real Git runs it and deletes all of it.",
        target.display(),
        reasons.join(". ")
    );
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_typed_git_work_tests.rs"]
mod tests;
