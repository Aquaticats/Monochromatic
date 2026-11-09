//! What:
//!  Decide what one `git monochromatic-git-policy-cli ...` invocation does.
//! Why:
//!  Help,
//!  refusals and retired commands need no repository;
//!  `check` and `fix`
//!      validate their policy selection and configuration,
//!  then run one policy pass over
//!      the selected repository and report its events on standard output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const action = planManagement(managementArgs, gitGlobalArgs, controls, environment, inputs);
//! ```

/// Import the sibling modules the decision combines.
use crate::management_direct_commands::{direct_config_failure, plan_direct, refusal};
use crate::management_service_repository_selection::{Selection, status_git_dir};
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure, with_leading_notice};
use crate::child_environment::{child_environment_overlay, environment_home, environment_value};
/// The traditional-hook listing of `git monochromatic-git-policy-cli doctor`.
use crate::doctor::{CheckoutViewpoint, doctor_action_with_section, doctor_failure};
/// The process's one runtime,
/// which the doctor's report is built on.
use crate::lazy_runtime::run_on_runtime;
use crate::service_route::service_status;
use crate::management_service_settle_earlier_batches::{plan_settle_earlier_batches};
use crate::management_service_stop_units_of_removed_worktrees::plan_stop_units_of_removed_worktrees;
use crate::management_arguments::{
    MANAGEMENT_HELP, ManagementAction, RetiredCommand,
    parse_management_arguments,
};
/// The refusal of a `--policy` selection that is off.
use crate::management_selection::selected_policies;
use crate::policy_registry::PolicyId;
use crate::real_git::{ResolutionInputs, ResolvedGit, resolve_real_git};
use crate::transaction_git::GitContext;
use crate::subvolume_arguments::MigrateArguments;
use crate::worktree_convert_arguments::ConvertArguments;
#[cfg(target_os = "linux")]
use crate::subvolume_command::migrate_action;
use crate::wrapper_controls::Controls;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use std::path::{Path, PathBuf};
/// The notes the doctor's action gains on standard error at its end.
pub use crate::management_action_notes::{with_earlier_batches_note, with_units_note};

/// What:
///  The command word of a retired command.
/// Why:
///   The explanation names the command the caller typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retiredName(command: RetiredCommand): string;
/// ```
fn retired_name(command: RetiredCommand) -> &'static str {
    // `match` must name every variant, so a new retired command cannot be forgotten.
    match command {
        RetiredCommand::Trust => return "trust",
        RetiredCommand::Untrust => return "untrust",
        RetiredCommand::Status => return "status",
    }
}

/// What:
///  Explain that a trust command is retired.
///       `String` is the owned,
///  line-terminated explanation.
/// Why:
///   Trust existed to approve executing a repository's configuration code.
///  JSONC
///       configuration is data,
///  so there is nothing to approve,
///  revoke or report.
///       Old trust records are neither read nor deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retiredExplanation(command: RetiredCommand): string;
/// ```
pub fn retired_explanation(command: RetiredCommand) -> String {
    // `format!` builds the owned text; `{}` interpolates the command word.
    return format!(
        "git monochromatic-git-policy-cli {} is retired. monochromatic-git-policy-cli now reads monochromatic-git-policy-cli.config.jsonc as data and runs no \
         repository-supplied code, so there is no code execution to approve, revoke or report. \
         Nothing was changed; existing trust records are left in place and are no longer read.\n",
        retired_name(command)
    );
}

/// What:
///  The pathspecs of a direct command's scope.
///  `all` is `--all`;
///  `Vec<OsString>`
///       is the owned list of pathspecs written after `--`.
/// Why:
///   `--all` selects the whole worktree from its top level,
///  which Git spells `:/`,
///       as the installed wrapper does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directPathspecs = all ? [':/'] : pathspecs;
/// ```
fn scope_pathspecs(all: bool, pathspecs: Vec<OsString>) -> Vec<OsString> {
    if all {
        // `vec![...]` builds the one-item list.
        return vec![OsString::from(":/")];
    }
    return pathspecs;
}

/// What:
///  Decide the action of `git monochromatic-git-policy-cli service status`.
/// Why:
///   The command is read-only:
/// it says whether the service of the worktree it runs in answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planService(globalArguments, environment, inputs): Action;
/// ```
fn plan_service(global_arguments: &[OsString], environment: &[(OsString, OsString)], inputs: &ResolutionInputs) -> Action {
    let (real_git, notice): (Option<PathBuf>, Option<String>) = optional_real_git(inputs);
    let action: Action = plan_service_with(global_arguments, environment, real_git.as_deref());
    return with_leading_notice(action, notice.as_deref());
}

/// What:
///  Decide the action of `git monochromatic-git-policy-cli service settle-earlier-batches`.
/// Why:
///   The command needs no valid configuration;
/// it works on the commit service's earlier journals of the worktree it runs in,
///  with the real Git the wrapper forwards to (a recorded fallback owes its notice,
/// as for the status).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planSettle(globalArguments, environment, inputs): Action;
/// ```
fn plan_settle(global_arguments: &[OsString], environment: &[(OsString, OsString)], inputs: &ResolutionInputs) -> Action {
    let (real_git, notice): (Option<PathBuf>, Option<String>) = optional_real_git(inputs);
    let action: Action = plan_settle_earlier_batches(global_arguments, environment, real_git.as_deref());
    return with_leading_notice(action, notice.as_deref());
}

/// What:
///  Resolve the real Git for a command that can run without one.
///  The pair is the path,
///  if any,
///  and the notice owed when it came from the commit service's record.
/// Why:
///   The status and conversion commands answer from what they can read when Git is missing,
///  but a recorded fallback is the same resolution every other start uses,
///  so its notice is given here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionalRealGit(inputs: ResolutionInputs): [string | undefined, string | undefined];
/// ```
fn optional_real_git(inputs: &ResolutionInputs) -> (Option<PathBuf>, Option<String>) {
    // `match` on the resolution: both parts when found, neither when not.
    match resolve_real_git(inputs) {
        Ok(found) => return (Some(found.path), found.notice),
        Err(_) => return (None, None),
    }
}

/// What:
///  Decide the action of `git monochromatic-git-policy-cli service status` once the optional real Git is known.
/// Why:
///   `plan_service` adds the notice of a recorded fallback to the action this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planServiceWith(globalArguments, environment, realGit?: string): Action;
/// ```
fn plan_service_with(global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    // An empty runtime directory is no directory.
    let runtime: Option<PathBuf> = match environment_value(environment, "XDG_RUNTIME_DIR") {
        Some(text) if !text.is_empty() => Some(PathBuf::from(text)),
        _ => None,
    };
    // The worktree the command runs in has its own service; outside a repository the answer counts any.
    let git_dir: Option<PathBuf> = match status_git_dir(global_arguments, environment, real_git) {
        Selection::Found(found) => Some(found),
        Selection::Outside => None,
        Selection::Unresolved(reason) => {
            return Action::Exit {
                code: 1,
                stdout: String::new(),
                stderr: format!("The commit service cannot be checked: {reason}, so it is not known which repository this invocation selects, and no service was counted.\n"),
            };
        }
    };
    let report = service_status(runtime.as_deref(), git_dir.as_deref());
    return Action::Exit { code: report.code, stdout: report.stdout, stderr: report.stderr };
}

/// What:
///  Decide the action of `git monochromatic-git-policy-cli migrate-to-subvolume`:
/// a refused invocation prints its reason,
/// anything else runs
///  from the working directory.
/// Why:
///   The command needs neither configuration nor Git:
/// it reads `.git` itself and calls the filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planMigrate(parsed: MigrateArguments | string): Action;
/// ```
fn plan_migrate(parsed: Result<MigrateArguments, String>) -> Action {
    let arguments: MigrateArguments = match parsed {
        Ok(found) => found,
        Err(reason) => {
            return Action::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stdout: String::new(),
                stderr: format!("git monochromatic-git-policy-cli migrate-to-subvolume: {reason}\n"),
            };
        }
    };
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => return failure(format!("cannot determine the current directory: {error}").as_str()),
    };
    return run_migrate(&arguments, cwd.as_path());
}

/// What:
///  Decide the action of `git monochromatic-git-policy-cli convert-worktree-to-subvolume`:
/// a refused invocation prints its reason,
/// anything else
///  runs from the working directory.
/// Why:
///   Like the migration,
/// the command needs no valid configuration and starts no Git.
/// It resolves the real Git the wrapper
///  forwards to,
/// as every forwarding path does,
/// because the wrapper's own environment does not name it (only its children's
///  does) and the content check needs to know where that Git reads its system files from.
/// Where no real Git is found the
///  command still runs,
/// and a worktree that needs the system files is refused with that reason instead of guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planConvert(parsed: ConvertArguments | string, globalArguments, environment, inputs): Action;
/// ```
fn plan_convert(parsed: Result<ConvertArguments, String>, global_arguments: &[OsString], environment: &[(OsString, OsString)], inputs: &ResolutionInputs) -> Action {
    let arguments: ConvertArguments = match parsed {
        Ok(found) => ConvertArguments { command_line_attributes: crate::worktree_convert_arguments::selects_attribute_source(global_arguments), ..found },
        Err(reason) => {
            return Action::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stdout: String::new(),
                stderr: format!("git monochromatic-git-policy-cli convert-worktree-to-subvolume: {reason}\n"),
            };
        }
    };
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => return failure(format!("cannot determine the current directory: {error}").as_str()),
    };
    let (real_git, notice): (Option<PathBuf>, Option<String>) = optional_real_git(inputs);
    let action: Action = run_convert(&arguments, cwd.as_path(), environment, real_git.as_deref());
    return with_leading_notice(action, notice.as_deref());
}

/// Run the conversion command where the platform has btrfs subvolumes.
#[cfg(target_os = "linux")]
fn run_convert(arguments: &ConvertArguments, cwd: &Path, environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    return crate::worktree_convert_command::convert_action(arguments, cwd, environment, real_git);
}

/// Other platforms have no btrfs subvolumes,
/// so the command says so.
#[cfg(not(target_os = "linux"))]
fn run_convert(_arguments: &ConvertArguments, _cwd: &Path, _environment: &[(OsString, OsString)], _real_git: Option<&Path>) -> Action {
    return failure("git monochromatic-git-policy-cli convert-worktree-to-subvolume works on Linux with btrfs only");
}

/// The doctor's filesystem section read from the running system,
/// followed by its line about the repository's link to the
/// 860 drive below the home the invocation names.
#[cfg(target_os = "linux")]
pub fn host_filesystem_section(viewpoint: &CheckoutViewpoint<'_>) -> String {
    return filesystem_section_with_link_line(viewpoint);
}

/// The doctor's filesystem section followed by its line about the repository's link to the 860 drive (plan item W15),
/// read from the identity file below the home the invocation names.
///  Nothing is read about a file system when Git named no checkout.
#[cfg(target_os = "linux")]
pub fn filesystem_section_with_link_line(viewpoint: &CheckoutViewpoint<'_>) -> String {
    let facts: String = match viewpoint.checkout {
        Some(checkout) => crate::doctor_filesystem::checkout_section(checkout, viewpoint.layout),
        None => String::new(),
    };
    return format!("{}{}", facts, crate::repository_link_doctor_line::doctor_link_line(viewpoint.home));
}

/// Other platforms have no btrfs or overlayfs section.
#[cfg(not(target_os = "linux"))]
pub fn host_filesystem_section(_viewpoint: &CheckoutViewpoint<'_>) -> String {
    return String::new();
}

/// Run the migration command where the platform has btrfs subvolumes.
#[cfg(target_os = "linux")]
fn run_migrate(arguments: &MigrateArguments, cwd: &Path) -> Action {
    return migrate_action(arguments, cwd);
}

/// Other platforms have no btrfs subvolumes,
/// so the command says so.
#[cfg(not(target_os = "linux"))]
fn run_migrate(_arguments: &MigrateArguments, _cwd: &Path) -> Action {
    return failure("git monochromatic-git-policy-cli migrate-to-subvolume works on Linux with btrfs only");
}

fn plan_doctor(
    git_global_arguments: &[OsString],
    environment: &[(OsString, OsString)],
    inputs: &ResolutionInputs,
    filesystem_section: &dyn Fn(&CheckoutViewpoint<'_>) -> String,
) -> Action {
    let resolved: ResolvedGit = match resolve_real_git(inputs) {
        Ok(found) => found,
        Err(error) => return failure(error.to_string().as_str()),
    };
    let real_git: PathBuf = resolved.path;
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => return failure(format!("cannot determine the current directory: {error}").as_str()),
    };
    let overlay: Vec<(OsString, OsString)> = child_environment_overlay(environment, real_git.as_path());
    let context: GitContext = GitContext {
        real_git,
        overlay,
        global_prefix: git_global_arguments.to_vec(),
        // The doctor sends no input, so no scratch directory is needed.
        common_dir: None,
    };
    let home: Option<PathBuf> = environment_home(environment);
    // The doctor's report is built on the runtime, so every Git question of it is awaited there instead of
    // entering the runtime per step; this command is still synchronous, so the enter happens once, here.
    let action: Action = match run_on_runtime(doctor_action_with_section(&context, cwd.as_path(), home.as_deref(), filesystem_section)) {
        Ok(report) => report,
        Err(refusal) => doctor_failure(format!("the doctor could not ask Git: {refusal}").as_str()),
    };
    let batches_noted: Action = with_earlier_batches_note(action, git_global_arguments, environment, context.real_git.as_path());
    let noted: Action = with_units_note(batches_noted, environment);
    return with_leading_notice(noted, resolved.notice.as_deref());
}

/// What:
///  Decide the action for the arguments after `git monochromatic-git-policy-cli`.
///       `&[OsString]` borrows argument lists;
///  `&Controls` borrows what the wrapper
///       controls written before `monochromatic-git-policy-cli` asked for;
///  `&ResolutionInputs` borrows process facts.
/// Why:
///   Help,
///  refusals and retired commands answer without resolving Git or reading
///       any repository.
///  A direct command validates its selection,
///  asks Git once where it
///       runs,
///  refuses beside commit transactions it cannot recover,
///  loads that
///       worktree's configuration and runs one policy pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planManagement(args, gitGlobalArgs, controls, environment, inputs): Action;
/// ```
pub fn plan_management(
    arguments: &[OsString],
    git_global_arguments: &[OsString],
    controls: &Controls,
    environment: &[(OsString, OsString)],
    inputs: &ResolutionInputs,
) -> Action {
    return plan_management_with_section(arguments, git_global_arguments, controls, environment, inputs, &no_filesystem_section);
}

/// What:
///  The doctor's filesystem section for a plan that has none to show:
/// the empty text.
/// Why:
///  `plan_management` hands it to `plan_management_with_section` where no caller supplies a section of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noFilesystemSection = (_viewpoint: CheckoutViewpoint): string => '';
/// ```
fn no_filesystem_section(_viewpoint: &CheckoutViewpoint<'_>) -> String {
    return String::new();
}

/// What:
///  The same decision with the doctor's filesystem section supplied by the caller.
///  `&dyn Fn(&CheckoutViewpoint) -> String` is a borrowed function from the checkout Git named,
/// how that checkout is laid
///  out and the home the invocation names to the section text.
/// Why:
///   The executable passes the section read from the running system (`host_filesystem_section`);
/// the controls of the
///  doctor and of the management command pass an empty one,
/// so their exact output does not depend on the host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planManagementWithSection(args, gitGlobalArgs, controls, environment, inputs, filesystemSection): Action;
/// ```
pub fn plan_management_with_section(
    arguments: &[OsString],
    git_global_arguments: &[OsString],
    controls: &Controls,
    environment: &[(OsString, OsString)],
    inputs: &ResolutionInputs,
    filesystem_section: &dyn Fn(&CheckoutViewpoint<'_>) -> String,
) -> Action {
    // What: `match` on the parse `Result`: keep the action or return the refusal.
    // Why:  Nothing else may run for an invocation the grammar rejects.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed = parseManagementArgs(args); if (parsed === MANAGEMENT_REFUSED) return usage();
    // ```
    let parsed: ManagementAction = match parse_management_arguments(arguments) {
        Ok(action) => action,
        Err(reason) => return refusal(reason),
    };
    // `match` picks by variant and binds the fields each one carries.
    let (fix, policies, scope): (bool, Vec<String>, Vec<OsString>) = match parsed {
        ManagementAction::Help => {
            return Action::Exit {
                code: 0,
                stdout: String::from(MANAGEMENT_HELP),
                stderr: String::new(),
            };
        }
        ManagementAction::Retired { command, help } => {
            // Help goes to standard output; an attempted action is explained on standard error.
            if help {
                return Action::Exit {
                    code: 0,
                    stdout: retired_explanation(command),
                    stderr: String::new(),
                };
            }
            return Action::Exit {
                code: 0,
                stdout: String::new(),
                stderr: retired_explanation(command),
            };
        }
        ManagementAction::Doctor => return plan_doctor(git_global_arguments, environment, inputs, filesystem_section),
        ManagementAction::ServiceStatus => return plan_service(git_global_arguments, environment, inputs),
        ManagementAction::SettleEarlierBatches => return plan_settle(git_global_arguments, environment, inputs),
        ManagementAction::StopUnitsOfRemovedWorktrees => return plan_stop_units_of_removed_worktrees(environment),
        ManagementAction::MigrateToSubvolume(migrate_parse) => return plan_migrate(migrate_parse),
        ManagementAction::ConvertWorktree(convert_parse) => return plan_convert(convert_parse, git_global_arguments, environment, inputs),
        ManagementAction::Direct {
            fix,
            all,
            policies,
            pathspecs,
        } => (fix, policies, scope_pathspecs(all, pathspecs)),
    };
    let selected: Vec<PolicyId> = match selected_policies(policies.as_slice()) {
        Ok(ids) => ids,
        // `&error` lends the error to the renderer.
        Err(error) => return direct_config_failure(&error),
    };
    let resolved: ResolvedGit = match resolve_real_git(inputs) {
        Ok(found) => found,
        Err(error) => return failure(error.to_string().as_str()),
    };
    let action: Action = plan_direct(
        fix,
        selected,
        scope,
        controls,
        environment,
        git_global_arguments,
        resolved.path,
    );
    // `.as_deref()` lends the optional owned notice as an optional borrowed one.
    return with_leading_notice(action, resolved.notice.as_deref());
}

/// Decision controls stay out of the release executable.
#[cfg(test)]
#[path = "management_tests.rs"]
mod tests;
