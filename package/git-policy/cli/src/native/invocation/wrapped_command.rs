//! What:
//!  Decide what one wrapped Git command does once real Git is known:
//!  forward it,
//!       possibly rewritten,
//!  or stop with policy events or a refusal.
//! Why:
//!  This is the whole lifecycle of a forwarded command in one place and in the
//!      installed wrapper's order:
//!  unported work is refused,
//!  configuration is loaded only
//!      for commands that need it,
//!  the policy pass runs,
//!  and a real push is stopped until
//!      its own lifecycle exists.
//!  A read-only command never reads configuration and asks
//!      Git at most where it runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = await runWrappedCommand(stripped, environment, checks);
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The lifecycle joins control stripping,
///  the frontier,
///  configuration and the pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runPolicyPass } from './policy_pass.ts';
/// ```
use crate::action::ENGINE_FAILURE_EXIT_CODE;
use crate::auto_push_single_flight::PushGit;
use crate::child_environment::environment_with_overlay;
/// `git add` asks for what it would stage.
use crate::candidate_prediction::CandidateRequest;
use crate::command_options::OptionError;
use crate::command_push::{PushRegion, parse_push_region};
use crate::config_loading::{ConfigLoading, classify_config_loading};
use crate::command_commit::{CommitRegion, parse_commit_region};
use crate::commit_command::{CommitCommand, run_commit_command};
use crate::config_schema::{CliGitConfig, PolicyConfig};
use crate::diagnostics::EngineFailureCode;
use crate::discard_audit::{AuditOutcome, record_audit};
use crate::global_arguments::GlobalOutcome;
use crate::invocation_config::{config_invalid_event, load_identity_config};
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_lifecycle::manual_push_scope;
use crate::policy_checks::ShippedChecks;
/// What the command offers its content policies.
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{
    StageEnd, StageRequest, StageResult, any_policy_applies, pass_exit_code, run_policy_stage,
};
use crate::event_hints::manual_push_failure_hint;
use crate::policy_events::{EngineFailureHint, PolicyEvent, render_policy_events};
use crate::policy_pass::{PassResult, policies_of_kind, run_policy_pass};
use crate::policy_registry::PolicyId;
use crate::landed_summary::short_name;
use crate::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, is_machine_read, notice_directory, take_orphaned_failures, take_push_failures_for};
use crate::worktree_removal_rescue::{settle_removals, snapshot_linked_worktrees};
use crate::policy_trigger::Trigger;
use crate::rule_linked_worktree::{LinkedWorktreeDecision, decide_linked_worktree};
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
/// Startup recovery as a command step.
use crate::startup_gate::{Gate, recover_for_command};
/// A valid preparation lease marks a command started by a hook of a live transaction.
use crate::unported::{Unported, unported_from_unavailable, unported_notice};
use crate::worktree_identity::worktree_root;
use crate::wrapper_controls::is_escaped;
use crate::wrapper_invocation::{StrippedInvocation, command_region, command_word};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;
use std::io::Write;
/// Paths of the probe's working directory.
use std::path::{Path, PathBuf};

/// What:
///  How the lifecycle of one wrapped command ended.
///  An `enum` is a closed set of named
///       alternatives;
///  each carries the values its ending needs.
///  `#[derive(...)]` asks the
///       compiler to generate copying,
///  debug printing and `==`.
/// Why:
///   The caller either becomes Git with these arguments or exits with this code;
///       both may first print event lines on standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrappedOutcome = { kind: 'forward'; args: string[]; stderr: string } | { kind: 'exit'; code: number; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WrappedOutcome {
    /// Run real Git with these arguments.
    Forward {
        /// The arguments for Git:
        ///  no wrapper control,
        ///  fixed transforms applied.
        arguments: Vec<OsString>,
        /// Complete event lines for standard error,
        ///  written before Git starts.
        stderr: String,
    },
    /// Stop without running Git.
    Exit {
        /// Process exit code:
        ///  1 for a policy rejection,
        ///  2 for a failure or a refusal.
        code: i32,
        /// Complete text for standard error.
        stderr: String,
    },
}

/// What:
///  The ending for a command that needs unported work:
///  events so far,
///  then the notice.
///       `String` is the owned event text;
///  `&Unported` borrows the reason;
///  `&str` borrows
///       the command word.
/// Why:
///   Every refusal exits 2,
///  whether it was found before or inside the policy pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refused(events: string, what: Unported, command: string): WrappedOutcome;
/// ```
fn refused(events: String, what: &Unported, command: &str) -> WrappedOutcome {
    // `mut` allows appending the notice after the events.
    let mut stderr: String = events;
    // `.as_str()` lends the owned notice as borrowed text.
    stderr.push_str(unported_notice(what, command).as_str());
    return WrappedOutcome::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        stderr,
    };
}

/// What:
///  The ending of a stage that did not let the command proceed,
///  or nothing when it
///       did.
///  `&[PolicyEvent]` borrows every event so far;
///  `Option<WrappedOutcome>` is "an
///       ending or nothing".
/// Why:
///   The pre-forward pass and the push gate end the same way:
///  an unavailable stage is
///       a refusal,
///  a failed one exits 2,
///  and an error finding exits 1,
///  each after the
///       events gathered so far.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stageEnding(events: PolicyEvent[], end: StageEnd, command: string): WrappedOutcome | undefined;
/// ```
fn stage_ending(events: &[PolicyEvent], end: StageEnd, command: &str) -> Option<WrappedOutcome> {
    let rendered: String = render_policy_events(0, events);
    // `if let StageEnd::Unavailable(x) = end` runs only for that ending and binds what it carries.
    if let StageEnd::Unavailable(unavailable) = end {
        // `Some(x)` is the "present" case of `Option`.
        return Some(refused(
            rendered,
            &unported_from_unavailable(unavailable),
            command,
        ));
    }
    let code: i32 = pass_exit_code(events, end);
    if code == 0 {
        // `None` is the "absent" case: the command may proceed.
        return None;
    }
    return Some(WrappedOutcome::Exit {
        code,
        stderr: rendered,
    });
}

/// What:
///  Whether Git's own option table reads the tokens after `push` as a dry run.
///       `&[OsString]` borrows those tokens,
///  already free of wrapper controls.
/// Why:
///   Only a dry run publishes nothing.
///  A region the table refuses is not known to be
///       a dry run,
///  so it is treated as a real push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushIsDryRun(region: string[]): boolean { try { return parsePushRegion(region).isDryRun; } catch { return false; } }
/// ```
fn push_is_dry_run(region: &[OsString]) -> bool {
    // `&[]` is an empty list of wrapper flags: they were removed before this point.
    let parsed: Result<PushRegion, OptionError> = parse_push_region(region, &[]);
    // `match` unpacks the `Result`: `Ok` carries the facts, `Err(_)` ignores the refusal.
    match parsed {
        Ok(found) => return found.dry_run,
        Err(_) => return false,
    }
}

/// What:
///  Every shipped policy in registry order:
///  the built-in ones,
///  then the optional ones.
/// Why:
///   The push gate asks all of them at once,
///  as the installed wrapper does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const everyPolicy = [...builtInPolicies, ...optionalPolicies];
/// ```
fn every_policy() -> Vec<PolicyId> {
    // `mut` allows appending the second kind.
    let mut policies: Vec<PolicyId> = policies_of_kind(false);
    policies.extend(policies_of_kind(true));
    return policies;
}

/// What:
///  Prepare a command that loads configuration:
///  recover dead transactions,
///  read the
///       configuration of the worktree Git reports,
///  and say what a content policy may
///       read.
///  Returns the effective policy settings,
///  or the ending that stops the command.
///       `Result<PolicyConfig, WrappedOutcome>` is "settings,
///  or an ending".
/// Why:
///   The order is the installed wrapper's:
///  dead transactions are recovered before
///       configuration,
///  and a commit is refused only after its configuration was accepted,
///       so an invalid file is always reported as such.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareGuardedCommand(stripped, environment, checks): Promise<PolicyConfig>; // throws the ending
/// ```
fn prepare_guarded_command<F: RepositoryFacts>(
    stripped: &StrippedInvocation,
    checks: &mut ShippedChecks<F>,
) -> Result<CliGitConfig, WrappedOutcome> {
    // `match` unpacks the location answer; a failure to ask Git stops the command.
    let location: RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(message) => {
            return Err(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                // `format!` builds the owned, line-terminated text.
                stderr: format!("cli-git: {message}\n"),
            });
        }
    };
    // An isolated merge whose process died is settled before configuration loads, so the checks that follow never read
    // its half-finished checkout.
    match recover_for_command(&checks.facts.transaction_context(), &location.identity) {
        Gate::Clear => {}
        Gate::Stop(stderr) => {
            return Err(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stderr,
            });
        }
    }
    let config: CliGitConfig = match load_identity_config(&location.identity) {
        Ok(loaded) => loaded.config,
        Err(error) => {
            return Err(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stderr: config_invalid_event(&error),
            });
        }
    };
    // `.clone()` copies the scanner and Markdown options the configuration chose.
    checks.scanner_settings.options = config.policies.forbidden_strings.clone();
    checks.markdown.options = config.policies.markdown_autofix.clone();
    checks.discarded_limits = config.discarded;
    // Only `git add` inside a worktree has content for a policy to read before Git runs:
    // what it would stage, predicted when a content policy first reads it.
    if command_word(stripped) == b"add" && worktree_root(&location.identity).is_some() {
        checks.candidates = LifecycleContent::Requested(CandidateRequest::Add(
            // `.to_vec()` copies the arguments after `add` for the prediction.
            command_region(stripped).to_vec(),
        ));
    }
    // `Ok(x)` is the success case.
    return Ok(config);
}

/// What:
///  Run a real `git commit` through the commit transaction, or nothing when the command is anything else.
///  `Option<WrappedOutcome>` is "the ending, or nothing to say".
/// Why:
///   Every non-dry-run commit goes through the transaction instead of being forwarded, because native `git commit`
///   keeps `index.lock` on disk through hooks and the editor. A region Git's table refuses is not known to be a
///   dry run, so it is treated as a real commit and Git itself rejects it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const transaction = willShortCircuit ? NOT_APPLICABLE : await runCommitTransactionBoundary({ args, gitPath, policyOptions, concurrency });
/// ```
fn run_commit_if_real<F: RepositoryFacts>(
    stripped: &StrippedInvocation,
    environment: &[(OsString, OsString)],
    checks: &mut ShippedChecks<F>,
    config: &CliGitConfig,
) -> Option<WrappedOutcome> {
    if command_word(stripped) != b"commit" {
        return None;
    }
    let region: CommitRegion = match parse_commit_region(command_region(stripped), &[]) {
        Ok(found) => found,
        Err(_) => return None,
    };
    if region.dry_run {
        return None;
    }
    let context = checks.facts.transaction_context();
    let ending = run_commit_command(
        &CommitCommand {
            context: &context,
            environment,
            arguments: stripped.arguments.as_slice(),
            subcommand_index: stripped.layout.prefix_len,
            region: &region,
            config,
            controls: &stripped.controls,
        },
        checks,
    );
    return Some(WrappedOutcome::Exit {
        code: ending.code,
        stderr: String::new(),
    });
}

/// What:
///  Record a guarded command form that runs because the invocation carries the escape for the linked-worktree-only
///  policy.
/// Why:
///   The policy never runs under its escape, so nothing else would say that a discarding form was deliberately let
///  through; a later loss must be attributable to the escape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordEscapeUse(stripped, checks);
/// ```
fn record_escape_use<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if !is_escaped(&stripped.controls, PolicyId::LinkedWorktreeOnly) {
        return;
    }
    let command = match decide_linked_worktree(stripped.arguments.as_slice()) {
        LinkedWorktreeDecision::Pass => return,
        LinkedWorktreeDecision::NeedsEffectiveTarget(command) | LinkedWorktreeDecision::WhenDirty(command) => command,
    };
    if let Ok(location) = checks.facts.location() {
        record_audit(&location.identity, command, AuditOutcome::Hatch, stripped.arguments.as_slice(), None);
    }
}

/// What:
///  Recover before a read-only command that already asked Git for the repository location;
///       a live transaction does not stop it,
///  a legacy directory or a recovery failure does.
///       `Option<WrappedOutcome>` is "the ending,
///  or nothing to say".
/// Why:
///   The incumbent recovers before read-only commands too,
///  reusing the identity it already
///       resolved;
///  a command that never asked for the location starts no extra work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (configFreeIdentity !== undefined) await recoverCommitTransaction({ args, gitPath, identity: configFreeIdentity });
/// ```
fn recover_before_read_only<F: RepositoryFacts>(
    checks: &ShippedChecks<F>,
) -> Option<WrappedOutcome> {
    let Some(Ok(location)) = checks.facts.asked_location() else {
        return None;
    };
    match recover_for_command(&checks.facts.transaction_context(), &location.identity) {
        Gate::Clear => return None,
        Gate::Stop(stderr) => {
            return Some(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stderr,
            });
        }
    }
}

/// What:
///  Shows, on standard error, each backup push failure the service recorded since one was last shown and that belongs to this
///  worktree, before the command runs.
/// Why:
///   The service pushes after a commit returned, so the command that made the commit cannot print the failure; the next
///  command in the worktree whose commit it was does, because the records live under that worktree's own Git directory.
///  A command whose output a program reads shows nothing and takes
///  nothing, and neither does a command run where the repository is not found without starting Git. When nothing
///  failed the check is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isMachineRead(command)) for (const text of takePushFailures(gitDir)) stderr.write(text + '\n');
/// ```
fn show_push_failures<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if is_machine_read(command_word(stripped), command_region(stripped)) {
        return;
    }
    // The records are the worktree's own, so a notice shows in the worktree whose push failed.
    let Some(git_dir) = checks.facts.known_git_dir() else { return };
    // Without a known common directory, the worktree's own Git directory stands in.
    let common_dir: PathBuf = match checks.facts.known_common_dir() {
        Some(known) => known,
        None => git_dir.clone(),
    };
    // A worktree Git removed took its Git directory along; what it held is shown to whoever asks next. What could not be kept
    //  for the person is said here, each time, until it can be.
    for problem in settle_removals(common_dir.as_path()) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{problem}");
    }
    // Nothing failed: two directory checks, and no question to Git.
    let own: bool = notice_directory(git_dir.as_path()).is_dir();
    let orphaned: bool = common_dir.join(ORPHAN_NOTICE_DIRECTORY).is_dir();
    if !own && !orphaned {
        return;
    }
    let context = checks.facts.transaction_context();
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_unreadable) => git_dir.clone(),
    };
    let objects: PathBuf = common_dir.join("objects");
    // The name Git gives the commit in this repository as it is now, asked of Git once per shown notice.
    let abbreviate = |commit: &str| -> String { return short_name(&context, cwd.as_path(), objects.as_path(), None, commit); };
    let mut texts: Vec<String> = take_push_failures_for(git_dir.as_path(), None, &abbreviate);
    texts.extend(take_orphaned_failures(common_dir.as_path(), &abbreviate));
    for text in texts {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{text}");
    }
}

/// What:
///  Before `git worktree remove` or `git worktree prune`, copies the records of the linked worktrees into snapshots that
///  survive the removal.
/// Why:
///   Git deletes a removed worktree's Git directory, and with it the failed pushes nobody was told about and the pushes the
///  service had not run yet; the process is handed to Git, so only a copy made first can keep them. Which worktrees go is
///  decided later, from whether each still exists (`worktree_removal_rescue.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (command === 'worktree' && ['remove', 'prune'].includes(region[0])) await snapshotLinkedWorktrees(commonDir);
/// ```
fn snapshot_before_worktree_removal<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if command_word(stripped) != b"worktree" {
        return;
    }
    let removes: bool = match command_region(stripped).first() {
        Some(word) => matches!(word.as_encoded_bytes(), b"remove" | b"prune"),
        None => false,
    };
    if !removes {
        return;
    }
    if let Some(common_dir) = checks.facts.known_common_dir() {
        // A record that could not be saved is lost with the worktree, so the person is told before Git runs.
        for problem in snapshot_linked_worktrees(common_dir.as_path()) {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _written = writeln!(std::io::stderr(), "{problem}");
        }
    }
}

/// What:
///  Run the lifecycle of one wrapped command.
///  `&StrippedInvocation` borrows the
///       invocation without its wrapper controls;
///  `&mut ShippedChecks<F>` lends the shipped
///       policies and their facts provider `F` for writing.
/// Why:
///   A command Git does not run as a subcommand is forwarded as it is.
///  A read-only
///       command skips configuration and every refusal check,
///  and uses the default
///       settings.
///  Every other command is prepared first.
///  Then one pre-forward pass runs,
///       and a real push must also clear the manual-push gate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runWrappedCommand(stripped, environment, checks): Promise<WrappedOutcome>;
/// ```
pub fn run_wrapped_command<F: RepositoryFacts>(
    stripped: &StrippedInvocation,
    environment: &[(OsString, OsString)],
    checks: &mut ShippedChecks<F>,
) -> WrappedOutcome {
    if stripped.layout.outcome != GlobalOutcome::Command {
        return WrappedOutcome::Forward {
            // `.clone()` copies the arguments for the caller.
            arguments: stripped.arguments.clone(),
            stderr: String::new(),
        };
    }
    // `String::from_utf8_lossy` renders the command word for messages, replacing bytes that
    // are not UTF-8; `.into_owned()` makes it owned text.
    let command: String = String::from_utf8_lossy(command_word(stripped)).into_owned();
    // `.as_slice()` lends an owned list as a borrowed view.
    let skips_config: bool =
        classify_config_loading(stripped.arguments.as_slice()) == ConfigLoading::Skip;
    let loaded: Option<CliGitConfig> = if skips_config {
        None
    } else {
        match prepare_guarded_command(stripped, checks) {
            Ok(found) => Some(found),
            Err(ending) => return ending,
        }
    };
    // After the configuration step, so a command that had Git answer the repository location shows from that answer, and
    //  before the commit, so a failure shows ahead of the commit's own output.
    show_push_failures(stripped, checks);
    snapshot_before_worktree_removal(stripped, checks);
    if let Some(config) = &loaded
        && let Some(ending) = run_commit_if_real(stripped, environment, checks, config)
    {
        return ending;
    }
    let policies: PolicyConfig = match loaded {
        Some(config) => config.policies,
        None => PolicyConfig::defaults(),
    };
    record_escape_use(stripped, checks);
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: policies,
        controls: stripped.controls.clone(),
        // `Vec::<PolicyId>::new()` is an empty owned list: a wrapped command selects every policy.
        selected: Vec::<PolicyId>::new(),
    };
    let pass: PassResult = run_policy_pass(&request, checks);
    if let Some(ending) = stage_ending(pass.events.as_slice(), pass.end, command.as_str()) {
        return ending;
    }
    if skips_config
        && let Some(ending) = recover_before_read_only(checks)
    {
        return ending;
    }
    // `mut` allows the push gate to append its events.
    let mut events: Vec<PolicyEvent> = pass.events;
    if command_word(stripped) == b"push" && !push_is_dry_run(command_region(stripped)) {
        let gate_request: StageRequest = StageRequest {
            trigger: Trigger::ManualPush,
            config: request.config,
            controls: request.controls,
            selected: request.selected,
        };
        let gated: Vec<PolicyId> = every_policy();
        if any_policy_applies(&gate_request, gated.as_slice()) {
            // The push's scope is discovered before any policy reads content; a discovery failure refuses the push.
            if let Err(failure) = prepare_manual_push(checks, environment, command_region(stripped)) {
                // Git refused the push itself: nothing could be published, and Git's own words and status are the answer.
                if let Some(refusal) = failure.refusal {
                    return WrappedOutcome::Exit {
                        code: refusal.code,
                        stderr: refusal.stderr,
                    };
                }
                events.push(PolicyEvent::EngineFailure {
                    code: EngineFailureCode::ContentUnavailable,
                    hint: EngineFailureHint::Chosen(manual_push_failure_hint(failure.cause)),
                    message: failure.message,
                    trigger: Some(Trigger::ManualPush),
                    policy: None,
                    path: None,
                });
                return WrappedOutcome::Exit {
                    code: ENGINE_FAILURE_EXIT_CODE,
                    stderr: render_policy_events(0, events.as_slice()),
                };
            }
            let gate: StageResult = run_policy_stage(&gate_request, gated.as_slice(), checks);
            // `.extend(list)` moves every item of `list` onto the end.
            events.extend(gate.events);
            if let Some(ending) = stage_ending(events.as_slice(), gate.end, command.as_str()) {
                return ending;
            }
        }
    }
    // The notices come first: they say what was saved before Git changes anything.
    return WrappedOutcome::Forward {
        arguments: pass.arguments,
        stderr: format!("{}{}", checks.notices.concat(), render_policy_events(0, events.as_slice())),
    };
}

/// Lifecycle order,
///  fact use and every ending stay out of the release executable.
/// What:
///  Discover the scope of a manual push and hand it to the manual-push stage as its candidates.
/// Why:
///   The stage reads only what the push publishes, and the discovery runs against the repository the
///  command targets, with the invocation's global options and environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scope = await manualPushScope({ gitPath, cwd, args }); // then the manual-push stage reads it
/// ```
fn prepare_manual_push<F: RepositoryFacts>(
    checks: &mut ShippedChecks<F>,
    environment: &[(OsString, OsString)],
    post_subcommand: &[OsString],
) -> Result<(), crate::manual_push_lifecycle::ManualPushFailure> {
    let worktree: Option<PathBuf> = match checks.facts.location() {
        Ok(location) => worktree_root(&location.identity).map(Path::to_path_buf),
        Err(message) => return Err(crate::manual_push_lifecycle::ManualPushFailure::step(ManualPushCause::SelfExplained, message)),
    };
    let context = checks.facts.transaction_context();
    let cwd = match worktree {
        Some(root) => root,
        None => match std::env::current_dir() {
            Ok(directory) => directory,
            Err(error) => {
                return Err(crate::manual_push_lifecycle::ManualPushFailure::step(ManualPushCause::SelfExplained, format!(
                    "the working directory could not be read: {error}"
                )));
            }
        },
    };
    let git = PushGit {
        real_git: context.real_git,
        cwd,
        environment: environment_with_overlay(environment, context.overlay.as_slice()),
        // The scope probe is a dry-run push; an upload during it would run before any policy has decided.
        lfs: false,
    };
    let scope = manual_push_scope(&git, context.global_prefix.as_slice(), post_subcommand)?;
    checks.candidates = LifecycleContent::Requested(CandidateRequest::Pushed(scope));
    return Ok(());
}

#[cfg(test)]
#[path = "wrapped_command_tests.rs"]
mod tests;
