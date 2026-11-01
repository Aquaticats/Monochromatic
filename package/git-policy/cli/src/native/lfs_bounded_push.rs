//! What:
//!  Run one bounded `git push` of the auto-push, with the Git LFS upload when the repository has one.
//! Why:
//!  The auto-push and its reconcile pushes publish refs, so they upload the LFS objects the pushed commits point to
//!  exactly as a typed `git push` does. The wrapper is the parent of that push and captures its output, so it also
//!  owns the explanation: a missing `git-lfs` is refused before Git starts, and a push the upload hook stopped gets
//!  the full explanation after Git's own text. No process other than Git is started for this, and none of them is
//!  the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const attempt = await runBoundedPush(pushCommand(git, args)); if (attempt.exitCode !== 0) attempt.output += trace.explanation();
//! ```

/// The attempt record, the coordinator's Git and its error type.
use super::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, push_command};
/// The deadline, the grace, the low-speed abort and the push itself.
use super::bounded_push::{STOP_GRACE, apply_low_speed_defaults, push_deadline, run_bounded_push, strip_one_line_break};
/// Git's trace of the hook.
use super::lfs_hook_trace::HookTrace;
/// The check that the program can start.
use super::lfs_push_precheck::lfs_refusal;
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// Exit status of a push the wrapper refused before Git started.
const REFUSED_EXIT_CODE: i64 = 1;

/// What:
///  Run `git <arguments>` (a `push`) as a bounded child and explain a failed LFS upload.
/// Why:
///   `arguments[0]` is `push`; what follows is its option region, which the pre-check reads for `--no-verify`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runLfsAwarePush(git: PushGit, args: string[]): Promise<PushAttempt>;
/// ```
pub fn run_push(git: &PushGit, arguments: &[OsString]) -> Result<PushAttempt, SingleFlightError> {
    let region: &[OsString] = arguments.get(1..).unwrap_or_default();
    if git.lfs
        && let Some(refusal) = lfs_refusal(&[], region, git.environment.as_slice())
    {
        return Ok(PushAttempt {
            exit_code: REFUSED_EXIT_CODE,
            output: strip_one_line_break(refusal.as_str()).to_owned(),
        });
    }
    let trace: Option<HookTrace> = if git.lfs { HookTrace::start(git.environment.as_slice()) } else { None };
    let mut command = push_command(git, arguments);
    apply_low_speed_defaults(&mut command, git.environment.as_slice());
    if let Some(found) = trace.as_ref() {
        let (name, value) = found.entry();
        command.env(name, value);
    }
    let mut attempt: PushAttempt =
        run_bounded_push(command, push_deadline(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice())?;
    if attempt.exit_code != 0
        && let Some(found) = trace.as_ref()
    {
        let explanation: String = found.explanation();
        if !explanation.is_empty() {
            let shown: &str = strip_one_line_break(explanation.as_str());
            attempt.output = if attempt.output.is_empty() { shown.to_owned() } else { format!("{}\n{shown}", attempt.output) };
        }
    }
    return Ok(attempt);
}
