//! What:
//!  `git monochromatic-git-policy-cli kept-references`:
//!  the command that lists the kept references of this repository with what the next pruning
//!  would take,
//!  and with `--prune`,
//!  the command that runs one pruning now.
//! Why:
//!  The pruning removes the only handle on a commit no branch reaches,
//!  so the one rule that needs a person able to look is answered by a command
//!  (`SPEC.md`,
//!  the section on the references a removal keeps).
//!  The listing is the pass with the removals left out,
//!  so it cannot say one thing and the pass do another,
//!  and `--prune` is the same call the commit service makes after a landed commit
//!  (`service_work.rs`,
//!  `maintenance_awaiting`),
//!  which lets a person give the space back before the next commit does.
//!  Both run their Git questions on the process's one runtime,
//!  as the doctor does,
//!  so no step enters a runtime from inside one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = planKeptReferences(management, globalArguments, environment, inputs);
//! ```

use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure};
use crate::child_environment::child_environment_overlay;
use crate::kept_reference_limits::limits_from_the_environment;
use crate::kept_reference_listing::{the_listing, the_pruning_report};
use crate::kept_reference_log::log_path;
use crate::kept_reference_pass::run_the_pass;
use crate::lazy_runtime::run_on_runtime;
use crate::management_service_repository_selection::{Selection, status_git_dir};
use crate::real_git::{ResolutionInputs, ResolvedGit, resolve_real_git};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The action of `git monochromatic-git-policy-cli kept-references [--prune]`.
/// Why:
///  A person's command has to say what is kept before anything goes,
///  and the pass has to answer what it took when it is asked to prune.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = planKeptReferences(prune, globalArguments, environment, inputs);
/// ```
pub fn plan_kept_references(prune: bool, global_arguments: &[OsString], environment: &[(OsString, OsString)], inputs: &ResolutionInputs) -> Action {
    let ResolvedGit { path: real_git, notice }: ResolvedGit = match resolve_real_git(inputs) {
        Ok(found) => found,
        Err(error) => return failure(error.to_string().as_str()),
    };
    let git_dir: PathBuf = match status_git_dir(global_arguments, environment, Some(real_git.as_path())) {
        Selection::Found(found) => found,
        Selection::Outside => return failure("there is no repository here, so this repository keeps no reference"),
        Selection::Unresolved(reason) => return failure(format!("this command cannot tell which repository it runs in: {reason}").as_str()),
    };
    let common_dir: PathBuf = crate::kept_reference_pass::common_directory_of_this_repository(git_dir.as_path());
    let context: GitContext = GitContext {
        overlay: child_environment_overlay(environment, real_git.as_path()),
        real_git,
        global_prefix: global_arguments.to_vec(),
        common_dir: Some(common_dir.clone()),
    };
    let limits = limits_from_the_environment();
    let outcome: Result<String, String> = match run_on_runtime(answered_with(&context, common_dir.as_path(), prune, limits)) {
        Ok(answered) => answered,
        Err(refusal) => Err(format!("this command has no runtime to ask Git on: {refusal}")),
    };
    let action: Action = match outcome {
        Ok(text) => Action::Exit { code: 0, stdout: text, stderr: String::new() },
        Err(reason) => Action::Exit {
            code: ENGINE_FAILURE_EXIT_CODE,
            stdout: String::new(),
            stderr: format!("monochromatic-git-policy-cli: the kept references could not be read: {reason}\n"),
        },
    };
    return match notice.as_deref() {
        Some(line) => crate::action::with_leading_notice(action, Some(line)),
        None => action,
    };
}

/// What:
///  The text one invocation prints:
///  the listing,
/// or what a pruning took.
/// Why:
///  Both are one reading of the same repository,
///  and neither a listing nor a report may fail the invocation on a Git question
///  whose words the caller shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answeredWith = async (context, commonDir, prune, limits) => prune ? report(await runThePass(context, commonDir, false), limits) : listing(await runThePass(context, commonDir, true), limits);
/// ```
async fn answered_with(context: &GitContext, common_dir: &Path, prune: bool, limits: crate::kept_reference_limits::KeptReferenceLimits) -> Result<String, String> {
    let pass = run_the_pass(context, common_dir, !prune).await?;
    if prune {
        return Ok(the_pruning_report(&pass, limits, log_path(common_dir).as_path()));
    }
    return Ok(the_listing(&pass, limits));
}
