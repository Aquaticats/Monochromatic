//! What:
//!  The direct commands of the `monochromatic-git-policy-cli` namespace that run in this process (`git monochromatic-git-policy-cli <command> ...`):
//!  their refusals,
//! the configuration failure text,
//! and the plan that runs one.
//! Why:
//!  `management.rs` plans every management command;
//! the direct ones are a large part of it and depend on nothing in it,
//!  so they sit in a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // planDirect(...): Action; runDirectCommand(...): number; refusal(...): Action
//! ```

/// Import the sibling modules the decision combines.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure};
use crate::repository_link_answer::EXIT_GIT_FATAL;
/// A direct command asks for the worktree files its scope selects.
use crate::candidate_prediction::CandidateRequest;
use crate::child_environment::{child_environment_overlay, environment_value};
use crate::config_error::ConfigError;
use crate::config_file::LoadedConfig;
/// A scope that cannot be projected is a `transaction-failed` engine failure.
use crate::diagnostics::EngineFailureCode;
/// The tool-cache exemption entry the direct-check checks are built with.
use crate::effective_target::ToolCacheDirectory;
/// The direct fix:
///  converge corrections in memory,
///  then install them.
use crate::direct_fix::run_direct_fix;
use crate::invocation_config::{config_invalid_event, legacy_warning_events, load_identity_config};
use crate::management_arguments::{
    MANAGEMENT_USAGE, ManagementRefusal,
};
/// The refusal of a `--policy` selection that is off.
use crate::management_selection::selection_error;
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
use crate::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use crate::repository_location::RepositoryLocation;
/// The variable that names the forbidden-strings rules file.
use crate::scanner_selection::RULES_VARIABLE;
/// Startup recovery as a command step.
use crate::startup_gate::{Gate, recover_for_command};
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
/// function directCommand(fix: boolean): string { return `monochromatic-git-policy-cli ${directName(fix)}`; }
/// ```
fn direct_command(fix: bool) -> String {
    // `format!` builds owned text; `{}` interpolates the command word.
    return format!("monochromatic-git-policy-cli {}", direct_name(fix));
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
pub(crate) fn refusal(reason: ManagementRefusal) -> Action {
    // `match` picks the message for the refusal and binds its `fix` flag where present.
    let stderr: String = match reason {
        ManagementRefusal::Usage => String::from(MANAGEMENT_USAGE),
        ManagementRefusal::PathspecsBeforeSeparator { fix } => format!(
            "git monochromatic-git-policy-cli {} pathspecs must follow --.\n",
            direct_name(fix)
        ),
        ManagementRefusal::ScopeRequired { fix } => format!(
            "git monochromatic-git-policy-cli {} requires exactly one scope: --all or non-empty pathspecs after --.\n",
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
pub(crate) fn direct_config_failure(error: &ConfigError) -> Action {
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        stdout: config_invalid_event(error),
        stderr: String::new(),
    };
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
///  Plan a direct command (`git monochromatic-git-policy-cli check` and its kin) once the real Git is known.
/// Why:
///   `plan_management_with_section` adds the notice of a recorded fallback to whatever action this returns,
///  so the many early stops of the planning carry it without repeating it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planDirect(fix, selected, scope, controls, environment, gitGlobalArguments, realGit): Action;
/// ```
pub(crate) fn plan_direct(
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
        Vec::<ToolCacheDirectory>::new(),
    );
    // The forbidden-strings rules variable is read from this invocation's environment.
    checks.scanner_settings.rules_variable = environment_value(environment, RULES_VARIABLE);
    let location: RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(message) => return failure(message.as_str()),
    };
    // A direct command forwards nothing, so only an isolated merge whose process died matters here: it is settled first.
    match recover_for_command(&checks.facts.transaction_context(), &location.identity, environment) {
        Gate::Clear => {}
        Gate::Stop(stderr) => {
            return Action::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stdout: String::new(),
                stderr,
            };
        }
        Gate::RepositoryUnreachable(stderr) => {
            return Action::Exit {
                code: EXIT_GIT_FATAL,
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
