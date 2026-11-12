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
use crate::wrapped_command_notices_snapshots_and_manual_push::{prepare_manual_push, record_escape_use, show_held_commits, show_push_failures, snapshot_before_worktree_removal};
use crate::action::ENGINE_FAILURE_EXIT_CODE;
use crate::repository_link_answer::EXIT_GIT_FATAL;
/// `git add` asks for what it would stage.
use crate::candidate_prediction::CandidateRequest;
use crate::command_options::OptionError;
use crate::command_push::{PushRegion, parse_push_region};
use crate::config_loading::{ConfigLoading, classify_config_loading};
use crate::command_commit::{CommitRegion, parse_commit_region};
use crate::commit_command::{CommitCommand, run_commit_command};
use crate::config_schema::{CliGitConfig, PolicyConfig};
use crate::diagnostics::EngineFailureCode;
use crate::global_arguments::GlobalOutcome;
use crate::invocation_config::{config_invalid_event, load_identity_config};
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
use crate::policy_trigger::Trigger;
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
/// Startup recovery as a command step.
use crate::startup_gate::{Gate, recover_for_command};
/// A valid preparation lease marks a command started by a hook of a live transaction.
use crate::unported::{Unported, unported_from_unavailable, unported_notice};
use crate::worktree_identity::worktree_root;
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
    environment: &[(OsString, OsString)],
    checks: &mut ShippedChecks<F>,
) -> Result<CliGitConfig, WrappedOutcome> {
    // `match` unpacks the location answer; a failure to ask Git stops the command.
    let location: RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(message) => {
            return Err(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                // `format!` builds the owned, line-terminated text.
                stderr: format!("monochromatic-git-policy-cli: {message}\n"),
            });
        }
    };
    // An isolated merge whose process died is settled before configuration loads, so the checks that follow never read
    // its half-finished checkout.
    match recover_for_command(&checks.facts.transaction_context(), &location.identity, environment) {
        Gate::Clear => {}
        Gate::Stop(stderr) => {
            return Err(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stderr,
            });
        }
        Gate::RepositoryUnreachable(stderr) => {
            return Err(WrappedOutcome::Exit {
                code: EXIT_GIT_FATAL,
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
///  Run a real `git commit` through the commit transaction,
/// or nothing when the command is anything else.
///  `Option<WrappedOutcome>` is "the ending,
/// or nothing to say".
/// Why:
///   Every non-dry-run commit goes through the transaction instead of being forwarded,
/// because native `git commit`
///   keeps `index.lock` on disk through hooks and the editor.
/// A region Git's table refuses is not known to be a
///   dry run,
/// so it is treated as a real commit and Git itself rejects it.
///  A line Git refuses on its own terms is not carried out at all:
/// the whole line reaches Git,
/// which answers it in its own
///  words and status and makes no commit,
/// so a command line Git refuses can never become a commit here
///  (`refused_by_git`).
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
    if region.dry_run || crate::command_commit::refused_by_git(&region) {
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
    environment: &[(OsString, OsString)],
) -> Option<WrappedOutcome> {
    let Some(Ok(location)) = checks.facts.asked_location() else {
        return None;
    };
    match recover_for_command(&checks.facts.transaction_context(), &location.identity, environment) {
        Gate::Clear => return None,
        Gate::Stop(stderr) => {
            return Some(WrappedOutcome::Exit {
                code: ENGINE_FAILURE_EXIT_CODE,
                stderr,
            });
        }
        Gate::RepositoryUnreachable(stderr) => {
            return Some(WrappedOutcome::Exit {
                code: EXIT_GIT_FATAL,
                stderr,
            });
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
    // The guard reads the environment to know whether Git reaches an installed `git-<word>` before an alias of that
    //  word, so every check of a wrapped command carries the environment this invocation was started with.
    checks.environment = environment.to_vec();
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
        match prepare_guarded_command(stripped, environment, checks) {
            Ok(found) => Some(found),
            Err(ending) => return ending,
        }
    };
    // After the configuration step, so a command that had Git answer the repository location shows from that answer, and
    //  before the commit, so a failure shows ahead of the commit's own output.
    show_push_failures(stripped, checks);
    show_held_commits(stripped, checks);
    // A held commit that no reference of the repository can hold stops the command before Git removes the only
    //  references that name it (`worktree_removal_held_commits.rs`).
    if let Some(stderr) = snapshot_before_worktree_removal(stripped, checks) {
        return WrappedOutcome::Exit { code: ENGINE_FAILURE_EXIT_CODE, stderr };
    }
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
        && let Some(ending) = recover_before_read_only(checks, environment)
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

#[cfg(test)]
#[path = "wrapped_command_tests.rs"]
mod tests;
