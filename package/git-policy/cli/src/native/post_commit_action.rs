//! What:
//!  The declared post-commit actions, run by the wrapper itself after a commit lands.
//! Why:
//!  The wrapper supports no external hook program, so what a `post-commit` hook did is an action declared in
//!  `cli-git.config.jsonc` (`postCommit.actions`): a direct child with an argument array, never a shell, bounded in
//!  time and output, started once per landing after the landed commit is final.
//!  An action never fails the commit: a start failure, a bad exit status, a missing environment variable or a timeout
//!  is reported on standard error and the commit stays landed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const action of config.postCommit.actions) await runAction(action, { head, topLevel, environment });
//! ```

/// The declared actions.
use super::config_post_commit::PostCommitAction;
/// Bounded child runs and how they end.
use super::bounded_child::{ChildExit, ChildFailure, ChildLimits, ChildRequest, FinishedChild, run_bounded_child_noticing};
use super::diagnostic_log::debug;
use super::post_commit_template::{Template, Values, render};
use super::wait_progress::Cadence;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Module tag for diagnostics.
const TAG: &str = "postCommitAction";

/// The most bytes of one stream an action may write before the wrapper stops it.
const OUTPUT_LIMIT: usize = 1 << 20;

/// What:
///  Everything the actions of one landing need.
/// Why:
///   The values are the wrapper's own facts, so no Git start is needed to fill the fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { actions, landedOid, topLevel, environment, scratch }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PostCommitActionsRequest<'a> {
    /// The declared actions, in order.
    pub actions: &'a [PostCommitAction],
    /// The landed commit's object name.
    pub landed_oid: &'a str,
    /// The worktree's top level, where each action runs.
    pub top_level: &'a Path,
    /// The wrapper's environment, which each action inherits.
    pub environment: &'a [(OsString, OsString)],
    /// A directory for the anonymous stream files of a child.
    pub scratch: &'a Path,
}

/// What:
///  How one action ended.
/// Why:
///   The caller reports anything but success and the tests assert the exact variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActionOutcome = 'succeeded' | { exitCode } | { signal } | { notRun: string } | 'timed-out';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    /// Exit status 0.
    Succeeded,
    /// Another exit status.
    Failed(i32),
    /// A signal ended it.
    Signaled(i32),
    /// It never started or was stopped; the sentence says why.
    NotRun(String),
}

/// What:
///  One action's name and ending.
/// Why:
///   The caller (and the tests) see what happened per action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActionReport = { name: string; outcome: ActionOutcome };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionReport {
    /// The action's declared name.
    pub name: String,
    /// How it ended.
    pub outcome: ActionOutcome,
}

/// What:
///  Render every argument of an action.
/// Why:
///   A field with no value stops the action before anything starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// action.argv.map((template) => render(template, values));
/// ```
fn render_argv(argv: &[Template], values: &Values<'_>) -> Result<Vec<OsString>, String> {
    let mut rendered: Vec<OsString> = Vec::with_capacity(argv.len());
    for template in argv {
        rendered.push(render(template, values)?);
    }
    return Ok(rendered);
}

/// What:
///  Say what became of one action in words.
/// Why:
///   The person who landed the commit sees which action, what happened and that the commit stays landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: post-commit action ${name} ${what}; the commit stays landed.`
/// ```
fn notice(name: &str, outcome: &ActionOutcome) -> Option<String> {
    let what: String = match outcome {
        ActionOutcome::Succeeded => return None,
        ActionOutcome::Failed(code) => format!("exited with status {code}"),
        ActionOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
        ActionOutcome::NotRun(reason) => format!("did not run: {reason}"),
    };
    return Some(format!(
        "cli-git: post-commit action {name} {what}; the commit stays landed. Hint: the action is declared under \
         postCommit.actions in cli-git.config.jsonc.\n"
    ));
}

/// What:
///  Run one action as a bounded direct child.
/// Why:
///   The child's output is forwarded to standard error, as a hook's output was, after the child ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runAction(action, request); // ActionOutcome
/// ```
fn run_action(action: &PostCommitAction, request: &PostCommitActionsRequest<'_>) -> ActionOutcome {
    let values: Values<'_> = Values {
        head: request.landed_oid,
        top_level: request.top_level.as_os_str(),
        environment: request.environment,
    };
    let rendered: Vec<OsString> = match render_argv(action.argv.as_slice(), &values) {
        Ok(found) => found,
        Err(reason) => return ActionOutcome::NotRun(reason),
    };
    let Some((program, arguments)) = rendered.split_first() else {
        return ActionOutcome::NotRun(String::from("it has no program"));
    };
    let program_path: PathBuf = PathBuf::from(program);
    let waiting_for: String = format!("the post-commit action {}", action.name);
    let child: ChildRequest<'_> = ChildRequest {
        executable: program_path.as_path(),
        arguments,
        directory: request.top_level,
        environment: request.environment,
        input: &[],
        scratch: request.scratch,
        limits: ChildLimits {
            timeout: Duration::from_secs(action.timeout_seconds),
            max_stdout: OUTPUT_LIMIT,
            max_stderr: OUTPUT_LIMIT,
        },
        waiting_for: waiting_for.as_str(),
        cadence: Cadence::from_environment(),
    };
    let finished: FinishedChild = match run_bounded_child_noticing(&child, &mut std::io::stderr()) {
        Ok(found) => found,
        Err(ChildFailure::Start(reason)) => {
            return ActionOutcome::NotRun(format!("{} could not start: {reason}", program_path.display()));
        }
        Err(ChildFailure::TimedOut(limit)) => {
            return ActionOutcome::NotRun(format!("it was still running after {} seconds and was stopped", limit.as_secs()));
        }
        Err(ChildFailure::TooLarge { stream, limit }) => {
            return ActionOutcome::NotRun(format!("it wrote more than {limit} bytes to {stream} and was stopped"));
        }
        Err(ChildFailure::Files(reason) | ChildFailure::Wait(reason)) => return ActionOutcome::NotRun(reason),
    };
    forward(finished.stdout.as_slice());
    forward(finished.stderr.as_slice());
    return match finished.exit {
        ChildExit::Code(0) => ActionOutcome::Succeeded,
        ChildExit::Code(code) => ActionOutcome::Failed(code),
        ChildExit::Signal(signal) => ActionOutcome::Signaled(signal),
        ChildExit::Unknown => ActionOutcome::NotRun(String::from("its ending is unknown")),
    };
}

/// What:
///  Write a child's captured output to standard error, ignoring a stream that refuses it.
/// Why:
///   A closed terminal or broken pipe is normal and must not fail a landed commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { process.stderr.write(bytes); } catch {}
/// ```
fn forward(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    if let Err(error) = std::io::stderr().write_all(bytes) {
        debug(TAG, format!("the action output could not be forwarded: {error}").as_str());
    }
}

/// What:
///  Run every declared action in order and report each.
/// Why:
///   Actions are independent; one that fails or times out does not stop the next, and none fails the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reports = []; for (const action of actions) reports.push(await runAndReport(action));
/// ```
pub fn run_post_commit_actions(request: &PostCommitActionsRequest<'_>) -> Vec<ActionReport> {
    let mut reports: Vec<ActionReport> = Vec::with_capacity(request.actions.len());
    for action in request.actions {
        let outcome: ActionOutcome = run_action(action, request);
        if let Some(line) = notice(action.name.as_str(), &outcome)
            && let Err(error) = std::io::stderr().write_all(line.as_bytes())
        {
            debug(TAG, format!("the notice could not be written: {error}").as_str());
        }
        reports.push(ActionReport { name: action.name.clone(), outcome });
    }
    return reports;
}

/// Running, bounding and reporting are controlled in the test module.
#[cfg(test)]
#[path = "post_commit_action_tests.rs"]
mod tests;
