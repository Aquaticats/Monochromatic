//! What:
//!  An explicit `git push` that lost a race:
//! the commits that landed on the remote since the local base are merged in
//!       the object store,
//! and the merge is published by the wrapper pushing it again through its own gates.
//! Why:
//!  Several sessions push to one branch all day;
//! a push rejected because the remote moved cannot succeed again,
//!  and every caller would fetch,
//! merge or rebase and retry by hand while racing the others.
//!  The wrapper runs an eligible push as a child with the caller's streams,
//! so Git's own output is unchanged.
//!  Only a failed push is looked at again,
//! and only a divergence of the destination is acted on;
//!  an authentication,
//! offline,
//! hook or rule refusal keeps the exit status and the text Git gave.
//!  The merge is pushed through the same push planning,
//! run inside this process (`stacked_merge_push`) with
//!  `STACKED_PUSH_MARKER` set in the planned environment,
//! so the policies that gate any push read the integrated result
//!  and the marker stops that push from stacking again.
//! The wrapper never starts itself for this.
//!  A push that carries the Git LFS upload also runs here,
//! as a child,
//! so this parent can explain a failed upload.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await runPush(args); if (code !== 0) code = (await stackFailedPush({ args })) ?? code;
//! ```

/// The reconcile loop that auto-push shares,
/// and what it needs.
use crate::auto_push_reconcile::{Destination, Integrated, MergePusher, Reconcile, integrate_and_push};
/// The coordinator's Git,
/// its read-only query runner and its error type.
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query};
/// The action data of a stacked push.
use crate::action::PushForward;
/// The environment of a Git child.
use crate::child_environment::environment_with_overlay;
/// The table-driven read of a push's options.
use crate::command_push::stackable_option_tokens;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// The check that the upload can start,
/// what the upload sends,
/// and its run.
use crate::lfs_interactive::{InteractiveUpload, upload_first};
/// The merge push,
/// planned and run inside this process.
use crate::stacked_merge_push::gated_merge_push;
/// The destination and object names the push would send.
use crate::manual_push_probe::probe_manual_push_updates;
/// The settled or plain run of the forwarded command.
use crate::worktree_copy_forward::{ForwardedRun, run_forwarded_git};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// Standard error as a writable stream.
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The environment variable the stacked push carries so that it is never stacked again.
pub const STACKED_PUSH_MARKER: &str = "CLI_GIT_STACKED_PUSH";

/// How the merge of an integrated push is published,
/// returning the push's exit status.
pub type MergePush<'a> =
    &'a dyn Fn(&PushGit, &[OsString], &[OsString], &[String]) -> Result<i32, SingleFlightError>;

/// What a failed explicit push needs to be integrated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StackRequest = { git: PushGit; ownExecutable: string; globalPrefix: string[]; region: string[] };
/// ```
pub struct StackRequest<'a> {
    /// The repository's Git with the invocation's environment.
    pub git: &'a PushGit,
    /// The global options before `push`.
    pub global_prefix: &'a [OsString],
    /// Pushes the merge:
    /// the coordinator's Git,
    /// the global options,
    /// the option tokens,
    /// then the remote and refspec.
    ///  Production passes the in-process planner (`gated_merge_push`);
    /// controls pass plain Git.
    pub merge_push: MergePush<'a>,
    /// The arguments after `push`.
    pub region: &'a [OsString],
}

/// What:
///  The branch HEAD names,
/// or nothing for a detached HEAD.
/// Why:
///   The branch follows the published merge when it can;
/// a detached HEAD stays where it is.
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
///  Integrate the remote's new commits after an explicit push failed,
/// or say nothing when the failure was no divergence.
///  `Option<i32>` is "the new exit status,
/// or nothing to change".
/// Why:
///   The status is `0` when the merge was published,
/// and the failing status when the run says why it was not
///  (a conflict,
/// a gate that refused the merge,
/// a remote that kept moving);
/// every other failure returns nothing
///  and prints nothing,
/// so the person sees only Git's own report.
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
    // The loop takes its push as a value so controls can stand in for it; this one carries the request, the Git context and the tokens.
    let mut push_merge: StackedMergePush<'_> = StackedMergePush { request, git, tokens: tokens.as_slice() };
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
///  The push of a merge commit for a failed explicit push:
/// the request's merge planner run with the person's push options.
/// Why:
///   The loop that integrates and pushes takes its push as a value;
/// this one carries the request,
/// the Git context and the option
///  tokens of the push the person typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StackedMergePush implements MergePusher { pushMerge(destination, mergeOid) { return { exitCode: request.mergePush(git, prefix, tokens, [remote, `${mergeOid}:${mergeRef}`]), output: '' }; } }
/// ```
struct StackedMergePush<'a> {
    /// The failed push's request,
    /// which carries the merge planner.
    request: &'a StackRequest<'a>,
    /// The coordinator's Git.
    git: &'a PushGit,
    /// The option tokens of the push the person typed.
    tokens: &'a [OsString],
}

impl MergePusher for StackedMergePush<'_> {
    fn push_merge(&mut self, destination: &Destination, merge_oid: &str) -> Result<PushAttempt, SingleFlightError> {
        let code: i32 = (self.request.merge_push)(
            self.git,
            self.request.global_prefix,
            self.tokens,
            &[destination.remote.clone(), format!("{merge_oid}:{}", destination.merge_ref)],
        )?;
        return Ok(PushAttempt {
            exit_code: i64::from(code),
            output: String::new(),
        });
    }
}

/// What:
///  Run an eligible explicit push as a child with the caller's streams,
/// then integrate it when it failed on a divergence.
/// Why:
///   The wrapper must outlive Git to look at its status;
/// nothing is buffered,
/// so Git's own output is what the person sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = await runStackedPush(forward);
/// ```
pub fn run_stacked_push(forward: &PushForward) -> i32 {
    let (Some(prefix), Some(region)) = (
        forward.arguments.get(..forward.subcommand_index),
        forward.arguments.get(forward.subcommand_index + 1..),
    ) else {
        return 1;
    };
    // The parent uploads the LFS objects first, as one child of its own; the push, the probe and the merge push below
    //  carry no hook and start from the unchanged arguments.
    if forward.lfs
        && let Some(code) = upload_first(&InteractiveUpload {
            prefix,
            region,
            real_git: forward.real_git.as_path(),
            environment: forward.environment.as_slice(),
            overlay: forward.overlay.as_slice(),
            settings: forward.lfs_settings.as_deref(),
        })
    {
        return code;
    }
    // What: The death by a signal is kept beside the status the stacking decision reads.
    // Why:  A push the person's Git was killed in failed, and the stacking decision takes a status, so a shell's 128 +
    //       the signal is what it reads; a push stacking does not take over must still end the way native Git's caller
    //       sees it, so the signal itself is kept for that end
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 4 of the round-4 review).
    let mut signalled: Option<i32> = None;
    let code: i32 = match run_forwarded_git(
        forward.real_git.as_path(),
        forward.arguments.as_slice(),
        forward.overlay.as_slice(),
        forward.copy.as_deref(),
    ) {
        ForwardedRun::Ran(0) => return 0,
        ForwardedRun::Ran(code) => code,
        ForwardedRun::Killed(signal) => {
            signalled = Some(signal);
            crate::forwarding::SIGNAL_EXIT_BASE + signal
        }
        ForwardedRun::Failed { stderr, code } => {
            // Terminal output error: there is nowhere left to report a failure to write a diagnostic.
            let _written = std::io::stderr().write_all(stderr.as_bytes());
            return code;
        }
    };
    // A push stacking does not integrate keeps the death of its Git: the caller's `wait` sees the signal and a shell
    //  reports 128 + the signal, as the plain forward's caller does.
    let ended: i32 = match signalled {
        Some(signal) => crate::forwarding::end_as_git_ended(signal),
        None => code,
    };
    if !forward.stack {
        return ended;
    }
    let Ok(cwd) = std::env::current_dir() else {
        return code;
    };
    let git = PushGit {
        real_git: forward.real_git.clone(),
        cwd,
        environment: environment_with_overlay(forward.environment.as_slice(), forward.overlay.as_slice()),
        lfs: false,
    };
    let request = StackRequest {
        git: &git,
        global_prefix: prefix,
        merge_push: &gated_merge_push,
        region,
    };
    return stack_failed_push(&request, i64::from(code), &mut std::io::stderr()).unwrap_or(ended);
}

/// Stacking is controlled in the test module.
#[cfg(test)]
#[path = "push_stacking_tests.rs"]
mod tests;
