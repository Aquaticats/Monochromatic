//! What:
//!  Run one bounded `git push` of the auto-push, preceded by the Git LFS upload when the repository has one.
//! Why:
//!  The auto-push and its reconcile pushes publish refs, so they upload the LFS objects the pushed commits point to
//!  exactly as a typed `git push` does. The wrapper is the parent of both children and captures their output, so it
//!  owns the explanation: a missing `git-lfs` is refused before anything starts, a failed upload ends the attempt with
//!  the full explanation after the program's own output, and the push is not started. The push carries no hook, and no
//!  process other than Git and `git-lfs` is started for this, none of them the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failed = plan && await uploadBounded(plan); if (failed) return failed; return await runBoundedPush(pushCommand(git, args));
//! ```

/// The attempt record, the coordinator's Git and its error type.
use super::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, git_command, run_query};
/// The patience, the grace and the push itself.
use super::bounded_push::{STOP_GRACE, push_patience, run_bounded_push, strip_one_line_break};
/// The check that the program can start.
use super::lfs_push_precheck::lfs_refusal;
/// Narrows the plan to the objects the push introduces.
use super::lfs_object_plan::narrow_plan;
/// What to upload and the bounded run of it.
use super::lfs_upload::run_upload_bounded;
/// The repository found from the files.
use super::git_system_config::with_real_git;
use super::repo_read_layout::read_layout_after_success;
use super::lfs_pre_push::plan_failed_message;
use super::lfs_upload_plan::{Query, plan_upload, read_settings};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// Exit status of a push the wrapper refused before anything started.
const REFUSED_EXIT_CODE: i64 = 1;

/// What:
///  Run `git <arguments>` (a `push`) as a bounded child, after the upload when the repository has LFS keys.
/// Why:
///   `arguments[0]` is `push`; what follows is its option region, which the pre-check and the plan read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runLfsAwarePush(git: PushGit, args: string[]): Promise<PushAttempt>;
/// ```
pub fn run_push(git: &PushGit, arguments: &[OsString]) -> Result<PushAttempt, SingleFlightError> {
    let region: &[OsString] = arguments.get(1..).unwrap_or_default();
    if git.lfs {
        if let Some(refusal) = lfs_refusal(&[], region, git.environment.as_slice()) {
            return Ok(PushAttempt {
                exit_code: REFUSED_EXIT_CODE,
                output: strip_one_line_break(refusal.as_str()).to_owned(),
            });
        }
        let mut query = |query_arguments: &[&str]| -> Result<String, String> {
            let owned: Vec<OsString> = query_arguments.iter().map(OsString::from).collect();
            let output = run_query(git, owned.as_slice()).map_err(|error| {
                return error.to_string();
            })?;
            let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
            return match output.code {
                Some(0) => Ok(text),
                Some(1) if text.is_empty() => Ok(text),
                code => Err(format!("exited with {code:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())),
            };
        };
        let planned = read_settings(&mut query as Query<'_>).and_then(|settings| {
            return plan_upload(&[], region, git.environment.as_slice(), git.cwd.as_path(), settings.as_slice(), &mut query);
        });
        match planned {
            Ok(Some(plan)) => {
                // The plan came from Git queries that exited with status 0,
                // so Git had its say about the values it consults;
                // the files are read again,
                // and a plan that cannot be narrowed is still pushed by `git-lfs` and Git.
                let layout = read_layout_after_success(git.cwd.as_path(), with_real_git(git.environment.as_slice(), git.real_git.as_path()).as_slice());
                if let Some(narrowed) = narrow_plan(plan, layout.as_ref())
                    && let Some(failed) = run_upload_bounded(git, &narrowed)? {
                    return Ok(failed);
                }
            }
            Ok(None) => {}
            Err(error) => {
                return Ok(PushAttempt {
                    exit_code: REFUSED_EXIT_CODE,
                    output: strip_one_line_break(plan_failed_message(error.0.as_str()).as_str()).to_owned(),
                });
            }
        }
    }
    // No low-speed abort of the wrapper's own is added to a push (`bounded_push.rs`): it waits while
    //  its connection is open. A limit and time the person set in the environment or in `http.lowSpeedLimit` and
    //  `http.lowSpeedTime` still reach Git (`http.c`, lines 453 to 460 and 1477 to 1478) and still apply.
    let command = git_command(git, arguments);
    return run_bounded_push(command, push_patience(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice());
}
