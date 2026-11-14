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
use crate::worktree_remove_plan::{RemovePlan, operand_and_force};
use crate::worktree_remove_registry::{Registration, git_worktree_for_operand, registration_of};
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
    let context: GitContext = checks.facts.transaction_context();
    let plan: RemovePlan = match plan_of_the_removal(command, common_dir.as_path(), &context) {
        Ok(Some(found)) => found,
        // Git's own resolution names no worktree, so Git itself refuses the command and nothing is removed.
        Ok(None) => return None,
        // Nothing here can say which worktree the operand names, and a guard that guessed would stand down on the one
        // case it exists for: the person is asked to say it again with the opt-out, or with a name only one worktree answers.
        Err(reason) => return Some(unprovable_operand_refusal(command, reason.as_str())),
    };
    if plan.force {
        return None;
    }
    let registration: Registration = registration_of(common_dir.as_path(), plan.target.as_path(), plan.operand.as_os_str()).ok()?;
    let mut reasons: Vec<String> = match git_work_a_removal_would_strand(&context, common_dir.as_path(), plan.target.as_path(), registration.admin.as_path()) {
        Ok(stranded) => stranded,
        Err(reason) => vec![format!("nothing proves no commit of this worktree is named by nothing: {reason}")],
    };
    match hidden_edits_beneath(&context, plan.target.as_path(), registration.admin.as_path(), common_dir.as_path()) {
        Ok(mut edits) => reasons.append(&mut edits),
        Err(reason) => reasons.push(format!("nothing proves no edit there is hidden from Git: {reason}")),
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
///  resolved exactly as Git's own `git worktree remove` resolves it,
///  nothing when Git names no worktree,
///  or the reason this cannot be told.
/// Why:
///  The target is relative to the directory Git starts in,
///  which the global `-C` options move,
///  and Git resolves the operand in two steps whose comparison folds ASCII case exactly when
///  `core.ignorecase` says so:
///  a wrapper that resolves bytes alone names nothing where Git names a worktree,
///  and standing down there let an unforced removal destroy a hidden edit and a stranded commit
///  (a review of `528279d099`,
///  finding 1).
///  `git_worktree_for_operand` is that resolution,
///  and the setting is Git's own answer,
///  read through the same invocation so a `-c core.ignorecase=...` the person typed is the one Git reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planOfTheRemoval(command, commonDir, context) // RemovePlan | undefined; throws the reason
/// ```
fn plan_of_the_removal(command: &ResolvedCommand, common_dir: &Path, context: &GitContext) -> Result<Option<RemovePlan>, String> {
    let Some((operand, force)) = operand_and_force(command.arguments.as_slice(), command.subcommand_index) else {
        return Ok(None);
    };
    let current: PathBuf = std::env::current_dir().map_err(|error| return format!("the directory this command runs in cannot be read ({error})"))?;
    let start: PathBuf = match start_directory(&command.arguments[..command.subcommand_index], current.as_path()) {
        Some(found) => found,
        None => return Err(String::from("the `-C` options this command carries name a directory this program cannot read")),
    };
    let fold_case: bool = case_folds(context, common_dir)?;
    let target: Option<PathBuf> = git_worktree_for_operand(common_dir, operand.as_os_str(), start.as_path(), fold_case)?;
    return Ok(target.map(|target| return RemovePlan { target, operand, force }));
}

/// What:
///  Whether Git folds ASCII case when it compares paths in this repository and this invocation,
///  or the reason that cannot be read.
/// Why:
///  Git's `fspathcmp` is `strcasecmp` exactly when `core.ignorecase` is set
///  (`dir.c`, `environment.c` in Git 2.56.0),
///  and that setting may come from any configuration scope the invocation reads,
///  or from a `-c` the person typed,
///  so the answer is Git's own rather than a file this program parses:
///  a wrong answer here stands the guard down on a name Git resolves and this program does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caseFolds(context, commonDir) // boolean; throws the reason
/// ```
fn case_folds(context: &GitContext, common_dir: &Path) -> Result<bool, String> {
    let arguments: Vec<OsString> = ["config", "--bool", "core.ignorecase"].map(OsString::from).to_vec();
    let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest::new(common_dir, arguments.as_slice());
    let output = match crate::transaction_git::run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git cannot say whether this repository folds case in paths ({error})")),
    };
    match output.code {
        // The key is set: Git's own reading of it is true or false.
        Some(0) => match String::from_utf8_lossy(output.stdout.as_slice()).trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(format!("Git answered `{other}` for whether this repository folds case in paths")),
        },
        // The key is unset, which is Git's own default of no folding.
        Some(1) => Ok(false),
        _ => Err(format!(
            "Git cannot say whether this repository folds case in paths ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )),
    }
}

/// What:
///  The report of a removal refused because nothing here can say which worktree the operand names.
/// Why:
///   Git resolves the name itself,
///  and this program must not stand down on a name it cannot resolve:
///  standing down is what let a case-differing name remove a worktree unguarded
///  (a review of `528279d099`,
///  finding 1).
///  The way out is the same opt-out every refusal of this guard names,
///  and the operand as the person typed it is the one Git resolves,
///  so the printed command is the command Git runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unprovableOperandRefusal(command, reason) // the report
/// ```
fn unprovable_operand_refusal(command: &ResolvedCommand, reason: &str) -> String {
    let operand: String = match command.arguments.get(command.subcommand_index + 2) {
        Some(word) => word.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let quoted: String = crate::shell_quote::shell_quote(operand.as_bytes());
    return format!(
        "{PREFIX}nothing here proves which worktree the name {quoted} a removal names, so nothing was removed: {reason}. If that worktree must go anyway, `git worktree --no-enforce-worktree remove {quoted}` runs as real Git runs it and deletes all of it."
    );
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
