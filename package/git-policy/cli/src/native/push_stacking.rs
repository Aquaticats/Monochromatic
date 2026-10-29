//! What:
//!  An explicit `git push` that lost a race: the commits that landed on the remote since the local base are merged in
//!       the object store, and the merge is published by the wrapper pushing it again through its own gates.
//! Why:
//!  Several sessions push to one branch all day; a push rejected because the remote moved cannot succeed again,
//!  and every caller would fetch, merge or rebase and retry by hand while racing the others.
//!  The wrapper runs an eligible push as a child with the caller's streams, so Git's own output is unchanged.
//!  Only a failed push is looked at again, and only a divergence of the destination is acted on;
//!  an authentication, offline, hook or rule refusal keeps the exit status and the text Git gave.
//!  The merge is pushed by this executable with `STACKED_PUSH_MARKER` set, so the policies that gate any push read the
//!  integrated result, and the marker stops that push from stacking again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await runPush(args); if (code !== 0) code = (await stackFailedPush({ args })) ?? code;
//! ```

/// The reconcile loop that auto-push shares, and what it needs.
use super::auto_push_reconcile::{Destination, Integrated, Reconcile, integrate_and_push};
/// The coordinator's Git, its read-only query runner and its error type.
use super::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query};
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// The action data of a stacked push.
use super::action::PushForward;
/// The environment of a Git child.
use super::child_environment::environment_with_overlay;
/// The table-driven read of a push's options.
use super::command_push::stackable_option_tokens;
/// The diagnostic log of the wrapper.
use super::diagnostic_log::debug;
/// The Git LFS upload that rides on the push itself.
use super::lfs_pre_push::with_lfs_hook;
/// The destination and object names the push would send.
use super::manual_push_probe::probe_manual_push_updates;
/// The settled or plain run of the forwarded command.
use super::worktree_copy_forward::{ForwardedRun, run_forwarded_git};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// Standard error as a writable stream.
use std::io::Write;
/// File locations.
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// The environment variable the stacked push carries so that it is never stacked again.
pub const STACKED_PUSH_MARKER: &str = "CLI_GIT_STACKED_PUSH";

/// What a failed explicit push needs to be integrated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StackRequest = { git: PushGit; ownExecutable: string; globalPrefix: string[]; region: string[] };
/// ```
pub struct StackRequest<'a> {
    /// The repository's Git with the invocation's environment.
    pub git: &'a PushGit,
    /// This executable, which publishes the merge so every push gate reads it.
    pub own_executable: &'a Path,
    /// The global options before `push`.
    pub global_prefix: &'a [OsString],
    /// The arguments after `push`.
    pub region: &'a [OsString],
}

/// What:
///  The branch HEAD names, or nothing for a detached HEAD.
/// Why:
///   The branch follows the published merge when it can; a detached HEAD stays where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await git(['symbolic-ref', '--quiet', 'HEAD'])).stdout.trim() || undefined
/// ```
fn checked_out_branch(git: &PushGit) -> Option<String> {
    let arguments: Vec<OsString> = ["symbolic-ref", "--quiet", "HEAD"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = run_query(git, &arguments).ok()?;
    if output.code != Some(0) {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
///  Integrate the remote's new commits after an explicit push failed, or say nothing when the failure was no divergence.
///  `Option<i32>` is "the new exit status, or nothing to change".
/// Why:
///   The status is `0` when the merge was published, and the failing status when the run says why it was not
///  (a conflict, a gate that refused the merge, a remote that kept moving); every other failure returns nothing
///  and prints nothing, so the person sees only Git's own report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stackFailedPush(request, failedCode, err): Promise<number | undefined>;
/// ```
pub fn stack_failed_push(
    request: &StackRequest<'_>,
    failed_code: i64,
    err: &mut dyn Write,
) -> Option<i32> {
    let tokens: Vec<OsString> = stackable_option_tokens(request.region)?;
    let git = request.git;
    let updates = match probe_manual_push_updates(git, request.global_prefix, request.region) {
        Ok(found) => found,
        Err(error) => {
            debug(
                TAG,
                &format!("the failed push's updates could not be read: {}", error.message),
            );
            return None;
        }
    };
    let [update] = updates.as_slice() else {
        return None;
    };
    let local = update.local_oid.clone()?;
    update.remote_oid.as_ref()?;
    if !update.remote_ref.starts_with("refs/heads/") {
        return None;
    }
    let branch_ref: Option<String> = checked_out_branch(git);
    let plan = Reconcile {
        destination: Destination {
            remote: update.remote_name.clone(),
            merge_ref: update.remote_ref.clone(),
        },
        local,
        branch_ref: branch_ref.as_deref(),
        label: "push",
        explicit: true,
    };
    let mut push_merge = |destination: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        let status = std::process::Command::new(request.own_executable)
            .current_dir(&git.cwd)
            .args(request.global_prefix)
            .arg("push")
            .args(&tokens)
            .arg(&destination.remote)
            .arg(format!("{merge_oid}:{}", destination.merge_ref))
            .env_clear()
            .envs(git.environment.iter().map(|(name, value)| {
                return (name, value);
            }))
            .env(STACKED_PUSH_MARKER, "1")
            .status_retrying()
            .map_err(SingleFlightError::Io)?;
        return Ok(PushAttempt {
            exit_code: i64::from(status.code().unwrap_or(1)),
            output: String::new(),
        });
    };
    let rejected = PushAttempt {
        exit_code: failed_code,
        output: String::new(),
    };
    match integrate_and_push(git, &plan, rejected, &mut push_merge) {
        Ok(Integrated::NotDivergent(_)) => return None,
        Ok(Integrated::Finished(attempt)) => {
            let text = attempt.output.trim();
            if !text.is_empty() {
                // Terminal output error: the report is advisory; the exit status already carries the result.
                let _written = writeln!(err, "{text}");
            }
            return Some(i32::try_from(attempt.exit_code).unwrap_or(1));
        }
        Err(error) => {
            debug(TAG, &format!("stacking the failed push stopped: {error}"));
            return None;
        }
    }
}

/// What:
///  Run an eligible explicit push as a child with the caller's streams, then integrate it when it failed on a divergence.
/// Why:
///   The wrapper must outlive Git to look at its status; nothing is buffered, so Git's own output is what the person sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = await runStackedPush(forward);
/// ```
pub fn run_stacked_push(forward: &PushForward) -> i32 {
    // Only the push itself uploads LFS objects; the probe and the merge push below start from the unchanged arguments.
    let first_arguments: Vec<OsString> = match forward.lfs_pre_push.as_deref() {
        Some(executable) => with_lfs_hook(forward.arguments.as_slice(), executable),
        None => forward.arguments.clone(),
    };
    let code: i32 = match run_forwarded_git(
        forward.real_git.as_path(),
        first_arguments.as_slice(),
        forward.overlay.as_slice(),
        forward.copy.as_deref(),
    ) {
        ForwardedRun::Ran(0) => return 0,
        ForwardedRun::Ran(code) => code,
        ForwardedRun::Failed { stderr, code } => {
            // Terminal output error: there is nowhere left to report a failure to write a diagnostic.
            let _written = std::io::stderr().write_all(stderr.as_bytes());
            return code;
        }
    };
    let (Ok(cwd), Ok(own_executable)) = (std::env::current_dir(), std::env::current_exe()) else {
        return code;
    };
    let git = PushGit {
        real_git: forward.real_git.clone(),
        cwd,
        environment: environment_with_overlay(forward.environment.as_slice(), forward.overlay.as_slice()),
        lfs_pre_push: None,
    };
    let Some(prefix) = forward.arguments.get(..forward.subcommand_index) else {
        return code;
    };
    let Some(region) = forward.arguments.get(forward.subcommand_index + 1..) else {
        return code;
    };
    let request = StackRequest {
        git: &git,
        own_executable: own_executable.as_path(),
        global_prefix: prefix,
        region,
    };
    return stack_failed_push(&request, i64::from(code), &mut std::io::stderr()).unwrap_or(code);
}

/// Stacking is controlled in the test module.
#[cfg(test)]
#[path = "push_stacking_tests.rs"]
mod tests;
