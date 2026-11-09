//! What:
//!  The push of a merge that integrated a lost race,
//! planned and run inside the running wrapper.
//! Why:
//!  The merge push must pass every gate a typed `git push` passes,
//! so the policies read the integrated result.
//!  An earlier version started the wrapper's own executable with a marker variable to get those gates;
//! the wrapper
//!  never starts itself,
//! so the same planning (`plan_invocation`) now runs in this process,
//! with the marker set only
//!  in the environment handed to the planner,
//! never in the process environment,
//! and the planned action is run here.
//!  The marker stops that planned push from stacking again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = planInvocation(['push', ...tokens, remote, `${mergeOid}:${ref}`], { ...env, CLI_GIT_STACKED_PUSH: '1' });
//! return await runAction(action);
//! ```

/// The coordinator's Git and its error type.
use crate::auto_push_single_flight::{PushGit, SingleFlightError};
/// The planned action and the engine failure status.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// The planner of an invocation.
use crate::entry::plan_invocation;
/// The pushes run as children.
use crate::push_stacking::{STACKED_PUSH_MARKER, run_stacked_push};
/// The facts a planner needs about this process.
use crate::real_git::process_resolution_inputs;
/// The settled or plain run of the planned Git.
use crate::worktree_copy_forward::{ForwardedRun, run_forwarded_git};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// Standard streams as writable values.
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What:
///  The environment handed to the planner:
/// the coordinator's complete environment with the stacking marker set once.
/// Why:
///   A marker the caller already had must not appear twice,
/// because the first entry would win a lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ ...environment, CLI_GIT_STACKED_PUSH: '1' })
/// ```
fn marked_environment(environment: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let mut marked: Vec<(OsString, OsString)> = Vec::new();
    for pair in environment {
        if pair.0.as_os_str() != STACKED_PUSH_MARKER {
            marked.push(pair.clone());
        }
    }
    marked.push((OsString::from(STACKED_PUSH_MARKER), OsString::from("1")));
    return marked;
}

/// Write `text` to standard error;
/// a failed write is ignored because nowhere is left to report it.
fn write_to_standard_error(text: &str) {
    // Terminal output error: nowhere is left to report a failed diagnostic write.
    let _written = std::io::stderr().write_all(text.as_bytes());
}

/// The status of a forwarded run:
/// the child's own code,
/// or the code of the failure after its message is written.
fn status_of_forwarded(run: ForwardedRun) -> i32 {
    return match run {
        ForwardedRun::Ran(code) => code,
        // A push the person's Git was killed in keeps a status of its own here: what the stacked run needs is whether the
        //  push failed, and a shell's report of the death is 128 + the signal.
        ForwardedRun::Killed(signal) => crate::forwarding::SIGNAL_EXIT_BASE + signal,
        ForwardedRun::Failed { stderr, code } => {
            write_to_standard_error(stderr.as_str());
            code
        }
    };
}

/// What:
///  Run a planned action to its end as a child of this process and return its status.
/// Why:
///   Only the actions a marked push can plan are handled:
/// a plain run,
/// a settled run,
/// a push child,
/// or a refusal;
///  anything else is a planning bug and ends with the engine failure status and a plain message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runAction(action: Action): Promise<number>;
/// ```
fn run_planned(action: Action) -> i32 {
    // Git and its hooks see the marker, as they did when the merge push was a separate process.
    let marker = (OsString::from(STACKED_PUSH_MARKER), OsString::from("1"));

    match action {
        Action::Forward { real_git, arguments, mut overlay, stderr } => {
            write_to_standard_error(stderr.as_str());
            overlay.push(marker);
            return status_of_forwarded(run_forwarded_git(real_git.as_path(), arguments.as_slice(), overlay.as_slice(), None));
        }
        Action::ForwardSettled { real_git, arguments, mut overlay, copy, stderr } => {
            write_to_standard_error(stderr.as_str());
            overlay.push(marker);
            return status_of_forwarded(run_forwarded_git(real_git.as_path(), arguments.as_slice(), overlay.as_slice(), Some(&*copy)));
        }
        Action::ForwardPush { mut forward, stderr } => {
            write_to_standard_error(stderr.as_str());
            forward.overlay.push(marker);
            return run_stacked_push(&forward);
        }
        Action::Exit { code, stdout, stderr } => {
            // Terminal output error: the exit status carries the result.
            let _written = std::io::stdout().write_all(stdout.as_bytes());
            write_to_standard_error(stderr.as_str());
            return code;
        }
        Action::ExitBytes { code, stdout, stderr } => {
            // The report is pathnames as bytes: the exit status carries the result.
            let _written = std::io::stdout().write_all(stdout.as_slice());
            write_to_standard_error(stderr.as_str());
            return code;
        }
        other => {
            debug(TAG, format!("a merge push planned an action it cannot run: {other:?}").as_str());
            write_to_standard_error("monochromatic-git-policy-cli: the merge push could not be planned; nothing was pushed.\n");
            return ENGINE_FAILURE_EXIT_CODE;
        }
    }
}

/// What:
///  Push a merge through the push gates of this process and report Git's status.
/// Why:
///   The status is what the stacking loop reads;
/// Git's own output went to the caller's streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function gatedMergePush(git, globalPrefix, tokens, [remote, refspec]): Promise<number>; // throws SingleFlightError
/// ```
///
/// # Errors
/// The facts of this process could not be read.
pub fn gated_merge_push(
    git: &PushGit,
    global_prefix: &[OsString],
    tokens: &[OsString],
    destination: &[String],
) -> Result<i32, SingleFlightError> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("push"));
    arguments.extend(tokens.iter().cloned());
    arguments.extend(destination.iter().map(OsString::from));
    let environment: Vec<(OsString, OsString)> = marked_environment(git.environment.as_slice());
    let inputs = process_resolution_inputs(environment.as_slice()).map_err(SingleFlightError::Io)?;
    let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);
    return Ok(run_planned(action));
}
