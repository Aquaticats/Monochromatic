//! What:
//!  Decide what one `git cli-git ...` invocation does.
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
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure, with_leading_notice};
/// A direct command asks for the worktree files its scope selects.
use crate::candidate_prediction::CandidateRequest;
use crate::child_environment::{child_environment_overlay, environment_value};
use crate::config_error::ConfigError;
use crate::config_file::LoadedConfig;
/// A scope that cannot be projected is a `transaction-failed` engine failure.
use crate::diagnostics::EngineFailureCode;
/// The direct fix:
///  converge corrections in memory,
///  then install them.
use crate::direct_fix::run_direct_fix;
/// The traditional-hook listing of `git cli-git doctor`.
use crate::doctor::doctor_action_with_section;
use crate::invocation_config::{config_invalid_event, legacy_warning_events, load_identity_config};
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout_after_success_for;
use crate::service_route::service_status;
use crate::management_arguments::{
    MANAGEMENT_HELP, MANAGEMENT_USAGE, ManagementAction, ManagementRefusal, RetiredCommand,
    parse_management_arguments,
};
/// The refusal of a `--policy` selection that is off.
use crate::management_selection::{selected_policies, selection_error};
/// The linter the executable gives the Markdown policy.
/// The shipped checks and their constructor.
use crate::policy_checks::{ShippedChecks, shipped_checks};
/// What the direct command offers its content policies.
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{StageEnd, StageRequest, pass_exit_code};
/// The events of a direct command and their rendering.
use crate::policy_events::{PolicyEvent, render_policy_events};
use crate::policy_pass::{PassResult, run_policy_pass};
use crate::policy_registry::PolicyId;
use crate::policy_trigger::Trigger;
use crate::real_git::{ResolutionInputs, ResolvedGit, resolve_real_git};
use crate::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use crate::repository_location::RepositoryLocation;
/// The variable that names the forbidden-strings rules file.
use crate::scanner_selection::RULES_VARIABLE;
/// Startup recovery as a command step.
use crate::startup_gate::{Gate, recover_for_command};
use crate::transaction_git::GitContext;
use crate::subvolume_arguments::MigrateArguments;
use crate::worktree_convert_arguments::ConvertArguments;
#[cfg(target_os = "linux")]
use crate::subvolume_command::migrate_action;
use crate::unported::{unported_from_unavailable, unported_notice};
/// The worktree's top level,
///  where a fix installs corrected files.
use crate::worktree_identity::worktree_root;
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

/// What:
///  The command word of a direct command.
///       `&'static str` borrows text compiled into the executable for its whole run.
/// Why:
///   Messages name the command the caller typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directName(fix: boolean): string { return fix ? 'fix' : 'check'; }
/// ```
fn direct_name(fix: bool) -> &'static str {
    if fix {
        return "fix";
    }
    return "check";
}

/// What:
///  The words of a direct command as the caller typed them after `git`.
///       `String` is owned UTF-8 text.
/// Why:
///   A refusal names the command that was not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directCommand(fix: boolean): string { return `cli-git ${directName(fix)}`; }
/// ```
fn direct_command(fix: bool) -> String {
    // `format!` builds owned text; `{}` interpolates the command word.
    return format!("cli-git {}", direct_name(fix));
}

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
        "git cli-git {} is retired. cli-git now reads cli-git.config.jsonc as data and runs no \
         repository-supplied code, so there is no code execution to approve, revoke or report. \
         Nothing was changed; existing trust records are left in place and are no longer read.\n",
        retired_name(command)
    );
}

/// What:
///  Turn a refusal into its usage action:
///  text on standard error,
///  exit status 2.
/// Why:
///   A malformed management invocation must never fall through to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusal(reason: ManagementRefusal): Action;
/// ```
fn refusal(reason: ManagementRefusal) -> Action {
    // `match` picks the message for the refusal and binds its `fix` flag where present.
    let stderr: String = match reason {
        ManagementRefusal::Usage => String::from(MANAGEMENT_USAGE),
        ManagementRefusal::PathspecsBeforeSeparator { fix } => format!(
            "git cli-git {} pathspecs must follow --.\n",
            direct_name(fix)
        ),
        ManagementRefusal::ScopeRequired { fix } => format!(
            "git cli-git {} requires exactly one scope: --all or non-empty pathspecs after --.\n",
            direct_name(fix)
        ),
    };
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        // `String::new()` is empty owned text.
        stdout: String::new(),
        stderr,
    };
}

/// What:
///  Stop a direct command with a `config-invalid` event on standard output.
/// Why:
///   Direct commands report policy events on standard output,
///  and an invalid
///       configuration or an unknown selected policy exits 2 before any policy runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directConfigFailure(error: ConfigError): Action;
/// ```
fn direct_config_failure(error: &ConfigError) -> Action {
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        stdout: config_invalid_event(error),
        stderr: String::new(),
    };
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
///  Run the policy pass of one direct command and build its action.
///       `&LoadedConfig` borrows the accepted configuration;
///  `&mut ShippedChecks<GitFacts>`
///       lends the shipped policies over real Git for writing.
/// Why:
///   Direct commands report every event on standard output.
///  `check` first reports a
///       legacy file left beside the JSONC file,
///  and policy events continue its numbering.
///       A policy that cannot be evaluated here stops the command with exit status 2 and
///       a notice on standard error,
///  after the events gathered so far.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runDirectCommand({ fix, selected, controls, loaded, checks }): Promise<Action>;
/// ```
fn run_direct_command(
    fix: bool,
    selected: Vec<PolicyId>,
    controls: &Controls,
    loaded: &LoadedConfig,
    checks: &mut ShippedChecks<GitFacts>,
    root: Option<&Path>,
) -> Action {
    // Only `check` reports a legacy file left beside the JSONC file; `fix` stays silent.
    let mut stdout: String = if fix {
        String::new()
    } else {
        legacy_warning_events(loaded)
    };
    // Each event is one line, so the lines written so far are the next event number.
    // `.matches('\n').count()` counts them; `as u64` widens the count to the number type.
    let first_sequence: u64 = stdout.matches('\n').count() as u64;
    // `.clone()` copies the scanner and Markdown options the configuration chose.
    checks.scanner_settings.options = loaded.config.policies.forbidden_strings.clone();
    checks.markdown.options = loaded.config.policies.markdown_autofix.clone();
    checks.discarded_limits = loaded.config.discarded;
    let trigger: Trigger = if fix {
        Trigger::DirectFix
    } else {
        Trigger::DirectCheck
    };
    // The selected files are staged on a private index before any policy runs, as the
    // installed wrapper does, so a scope Git refuses is reported whatever is selected.
    if let Err(error) = checks
        .content
        .prepare(&checks.candidates, &mut checks.facts)
    {
        stdout.push_str(
            render_policy_events(
                first_sequence,
                &[PolicyEvent::EngineFailure {
                    code: EngineFailureCode::TransactionFailed,
                    hint: crate::policy_events::EngineFailureHint::OfCode,
                    message: error.message,
                    // `Some(x)` is the "present" case of `Option`.
                    trigger: Some(trigger),
                    // `None` is the "absent" case: no policy has run.
                    policy: None,
                    path: None,
                }],
            )
            .as_str(),
        );
        return Action::Exit {
            code: ENGINE_FAILURE_EXIT_CODE,
            stdout,
            stderr: String::new(),
        };
    }
    let request: StageRequest = StageRequest {
        trigger,
        // `.clone()` copies the settings and controls into the request.
        config: loaded.config.policies.clone(),
        controls: controls.clone(),
        selected,
    };
    // A fix converges corrections and installs them; a check runs one pass.
    let pass: PassResult = if fix {
        run_direct_fix(&request, checks, root)
    } else {
        run_policy_pass(&request, checks)
    };
    // `.as_slice()` lends the owned events as a borrowed view.
    stdout.push_str(render_policy_events(first_sequence, pass.events.as_slice()).as_str());
    // `if let StageEnd::Unavailable(x) = ...` runs only for that ending and binds what it carries.
    let stderr: String = if let StageEnd::Unavailable(unavailable) = pass.end {
        unported_notice(
            &unported_from_unavailable(unavailable),
            direct_command(fix).as_str(),
        )
    } else {
        String::new()
    };
    return Action::Exit {
        code: pass_exit_code(pass.events.as_slice(), pass.end),
        stdout,
        stderr,
    };
}

/// What:
///  The global options of an invocation that choose the directory it runs in (`-C <path>`,
/// in order),
/// or nothing when
///  another option could choose the repository too.
/// Why:
///   `git -C /path/B cli-git service status` asks about B,
/// wherever it is run from;
/// `-c <name>=<value>` changes no directory,
///  while `--git-dir` and its kin name a repository the in-process reader does not resolve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoryOptions(globalArguments): string[] | undefined;
/// ```
fn directory_options(global_arguments: &[OsString]) -> Option<Vec<OsString>> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut options = global_arguments.iter();
    while let Some(option) = options.next() {
        let value: &OsString = options.next()?;
        if option == "-C" {
            kept.push(option.clone());
            kept.push(value.clone());
        } else if option != "-c" {
            return None;
        }
    }
    return Some(kept);
}

/// Which repository a `git cli-git service status` invocation selects.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Selection {
    /// The Git directory of the worktree it selects.
    Found(PathBuf),
    /// Git says the invocation is outside a repository.
    Outside,
    /// Neither the reader nor Git could say,
    /// with the reason.
    Unresolved(String),
}

/// What:
///  The repository a `git <global options> cli-git service status` selects:
/// the Git directory of its worktree,
/// outside a
///  repository,
/// or unresolved.
/// Why:
///   The command needs no valid configuration,
/// so the structure of the files decides and the configuration values do not.
///  The reader learns the system configuration file from the real Git `real_git` names,
/// and resolves `-C` options only.
///  Whatever it declines (`--git-dir`,
/// `--work-tree`,
/// `GIT_DIR`,
/// an unusual layout) is put to the real Git once,
/// which resolves the
///  selection as every other command does;
/// "outside a repository" is Git's own answer,
/// and anything Git cannot answer is
///  unresolved,
/// never an invitation to count unrelated services.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusGitDir(globalArguments, environment, realGit): string | undefined;
/// ```
fn status_git_dir(global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Selection {
    // The real Git is added to the environment only when one was found.
    let named: Vec<(OsString, OsString)> = match real_git {
        Some(found) => with_real_git(environment, found),
        None => environment.to_vec(),
    };
    if let Some(prefix) = directory_options(global_arguments)
        && let Some(layout) = read_layout_after_success_for(prefix.as_slice(), named.as_slice())
    {
        return Selection::Found(layout.git_dir);
    }
    let Some(real) = real_git else {
        return Selection::Unresolved(String::from("no real Git was found to ask"));
    };
    let mut asking = std::process::Command::new(real);
    asking.args(global_arguments).args(["rev-parse", "--path-format=absolute", "--absolute-git-dir"]);
    // One `env` call per entry, in order, as `envs` made; `LC_ALL` is set after them so it wins.
    for (name, value) in environment {
        asking.env(name, value);
    }
    asking.env("LC_ALL", "C").stdin(std::process::Stdio::null());
    // This start of the real Git says so too, like every other (`started_as_git.rs`).
    crate::started_as_git::mark_real_git_start(&mut asking, crate::started_as_git::StartMode::Child);
    let asked = asking.output();
    let output = match asked {
        Ok(found) => found,
        Err(error) => return Selection::Unresolved(format!("Git could not be started: {error}")),
    };
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    if output.status.success() && !said.is_empty() {
        return Selection::Found(PathBuf::from(said));
    }
    let complaint: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
    if output.status.code() == Some(NOT_A_REPOSITORY_EXIT_CODE) && complaint.contains("not a git repository") {
        return Selection::Outside;
    }
    return Selection::Unresolved(format!("Git said: {complaint}"));
}

/// The exit status Git gives for a fatal error such as running outside a repository.
const NOT_A_REPOSITORY_EXIT_CODE: i32 = 128;

/// What:
///  Decide the action of `git cli-git service status`.
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
///  Decide the action of `git cli-git service status` once the optional real Git is known.
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
///  Decide the action of `git cli-git migrate-to-subvolume`: a refused invocation prints its reason, anything else runs
///  from the working directory.
/// Why:
///   The command needs neither configuration nor Git: it reads `.git` itself and calls the filesystem.
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
                stderr: format!("git cli-git migrate-to-subvolume: {reason}\n"),
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
///  Decide the action of `git cli-git convert-worktree-to-subvolume`: a refused invocation prints its reason, anything else
///  runs from the working directory.
/// Why:
///   Like the migration, the command needs no valid configuration and starts no Git. It resolves the real Git the wrapper
///  forwards to, as every forwarding path does, because the wrapper's own environment does not name it (only its children's
///  does) and the content check needs to know where that Git reads its system files from. Where no real Git is found the
///  command still runs, and a worktree that needs the system files is refused with that reason instead of guessed.
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
                stderr: format!("git cli-git convert-worktree-to-subvolume: {reason}\n"),
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

/// Other platforms have no btrfs subvolumes, so the command says so.
#[cfg(not(target_os = "linux"))]
fn run_convert(_arguments: &ConvertArguments, _cwd: &Path, _environment: &[(OsString, OsString)], _real_git: Option<&Path>) -> Action {
    return failure("git cli-git convert-worktree-to-subvolume works on Linux with btrfs only");
}

/// The doctor's filesystem section read from the running system.
#[cfg(target_os = "linux")]
pub fn host_filesystem_section(common_directory: &Path) -> String {
    return crate::doctor_filesystem::host_section(common_directory);
}

/// Other platforms have no btrfs or overlayfs section.
#[cfg(not(target_os = "linux"))]
pub fn host_filesystem_section(_common_directory: &Path) -> String {
    return String::new();
}

/// Run the migration command where the platform has btrfs subvolumes.
#[cfg(target_os = "linux")]
fn run_migrate(arguments: &MigrateArguments, cwd: &Path) -> Action {
    return migrate_action(arguments, cwd);
}

/// Other platforms have no btrfs subvolumes, so the command says so.
#[cfg(not(target_os = "linux"))]
fn run_migrate(_arguments: &MigrateArguments, _cwd: &Path) -> Action {
    return failure("git cli-git migrate-to-subvolume works on Linux with btrfs only");
}

fn plan_doctor(
    git_global_arguments: &[OsString],
    environment: &[(OsString, OsString)],
    inputs: &ResolutionInputs,
    filesystem_section: &dyn Fn(&Path) -> String,
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
    let action: Action = doctor_action_with_section(&context, cwd.as_path(), filesystem_section);
    return with_leading_notice(action, resolved.notice.as_deref());
}

/// What:
///  Decide the action for the arguments after `git cli-git`.
///       `&[OsString]` borrows argument lists;
///  `&Controls` borrows what the wrapper
///       controls written before `cli-git` asked for;
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
///  The doctor's filesystem section for a plan that has none to show: the empty text.
/// Why:
///  `plan_management` hands it to `plan_management_with_section` where no caller supplies a section of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noFilesystemSection = (_cwd: string): string => '';
/// ```
fn no_filesystem_section(_cwd: &Path) -> String {
    return String::new();
}

/// What:
///  The same decision with the doctor's filesystem section supplied by the caller.
///  `&dyn Fn(&Path) -> String` is a borrowed function from the repository's common Git directory to the section text.
/// Why:
///   The executable passes the section read from the running system (`host_filesystem_section`); the controls of the
///  doctor and of the management command pass an empty one, so their exact output does not depend on the host.
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
    filesystem_section: &dyn Fn(&Path) -> String,
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

/// What:
///  Plan a direct command (`git cli-git check` and its kin) once the real Git is known.
/// Why:
///   `plan_management_with_section` adds the notice of a recorded fallback to whatever action this returns,
///  so the many early stops of the planning carry it without repeating it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planDirect(fix, selected, scope, controls, environment, gitGlobalArguments, realGit): Action;
/// ```
fn plan_direct(
    fix: bool,
    selected: Vec<PolicyId>,
    scope: Vec<OsString>,
    controls: &Controls,
    environment: &[(OsString, OsString)],
    git_global_arguments: &[OsString],
    real_git: PathBuf,
) -> Action {
    let overlay: Vec<(OsString, OsString)> =
        child_environment_overlay(environment, real_git.as_path());
    // `mut` lets the pass cache facts it asks Git for.
    let mut checks: ShippedChecks<GitFacts> = shipped_checks(
        git_facts(real_git.as_path(), git_global_arguments, overlay.as_slice()),
        // `.to_vec()` copies the borrowed global options for the rule cores.
        git_global_arguments.to_vec(),
        LifecycleContent::Requested(CandidateRequest::Direct(scope)),
        // `Vec::new()` is an empty owned list: no direct-command policy reads the tool caches.
        Vec::<PathBuf>::new(),
    );
    // The forbidden-strings rules variable is read from this invocation's environment.
    checks.scanner_settings.rules_variable = environment_value(environment, RULES_VARIABLE);
    let location: RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(message) => return failure(message.as_str()),
    };
    // A direct command forwards nothing, so only an isolated merge whose process died matters here: it is settled first.
    match recover_for_command(&checks.facts.transaction_context(), &location.identity) {
        Gate::Clear => {}
        Gate::Stop(stderr) => {
            return Action::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stdout: String::new(),
                stderr,
            };
        }
    }
    let action: Action = match load_identity_config(&location.identity) {
        Ok(loaded) => match selection_error(
            selected.as_slice(),
            &loaded.config.policies,
            if fix { Trigger::DirectFix } else { Trigger::DirectCheck },
        ) {
            // A selected policy that is off, or does not run for this command, would check nothing and look clean:
            // stop before any work.
            Some(error) => direct_config_failure(&error),
            None => run_direct_command(
                fix,
                selected,
                controls,
                &loaded,
                &mut checks,
                worktree_root(&location.identity),
            ),
        },
        Err(error) => direct_config_failure(&error),
    };
    return action;
}

/// Decision controls stay out of the release executable.
#[cfg(test)]
#[path = "management_tests.rs"]
mod tests;
