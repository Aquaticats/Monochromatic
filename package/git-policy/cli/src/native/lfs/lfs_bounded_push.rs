//! What:
//!  Run one bounded `git push` of the auto-push,
//! preceded by the Git LFS upload when the repository has one.
//! Why:
//!  The auto-push and its reconcile pushes publish refs,
//! so they upload the LFS objects the pushed commits point to
//!  exactly as a typed `git push` does.
//! The wrapper is the parent of both children and captures their output,
//! so it
//!  owns the explanation:
//! a missing `git-lfs` is refused before anything starts,
//! a failed upload ends the attempt with
//!  the full explanation after the program's own output,
//! and the push is not started.
//! The push carries no hook,
//! and no
//!  process other than Git and `git-lfs` is started for this,
//! none of them the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failed = plan && await uploadBounded(plan); if (failed) return failed; return await runBoundedPush(pushCommand(git, args));
//! ```

/// The attempt record,
/// the coordinator's Git and its error type.
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, git_command, run_query, run_query_awaiting};
/// The patience,
/// the grace and the two faces of the push itself.
use crate::bounded_push::{STOP_GRACE, push_patience, run_bounded_push, run_bounded_push_awaiting, strip_one_line_break};
/// The check that the program can start.
use crate::lfs_push_precheck::lfs_refusal;
/// Narrows the plan to the objects the push introduces.
use crate::lfs_object_plan::narrow_plan;
/// What to upload and the bounded run of it.
use crate::lfs_upload::{run_upload_bounded, run_upload_bounded_awaiting};
/// The repository found from the files.
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout_after_success;
use crate::lfs_pre_push::plan_failed_message;
use crate::lfs_upload_plan::{
    GitQuery, GitQueryAwaiting, PlanError, Settings, UploadPlan, plan_upload, plan_upload_awaiting, read_settings, read_settings_awaiting,
};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// Exit status of a push the wrapper refused before anything started.
const REFUSED_EXIT_CODE: i64 = 1;

/// What:
///  The read-only Git questions of the upload plan,
/// asked through the coordinator's own bounded queries.
///  `&'a PushGit` lends the coordinator's Git for as long as this value lives.
/// Why:
///   The plan asks through the `GitQuery` trait,
/// and the state a question needs (the real Git,
/// the directory,
/// the
///  environment) is one borrowed value,
/// so a struct stands where a closure that captured it stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushQuery implements GitQuery { constructor(readonly git: PushGit) {} ask(args: string[]): string {} }
/// ```
struct PushQuery<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    git: &'a PushGit,
}

impl GitQuery for PushQuery<'_> {
    fn ask(&mut self, query_arguments: &[&str]) -> Result<String, String> {
        let owned: Vec<OsString> = query_arguments.iter().map(OsString::from).collect();
        let output = match run_query(self.git, owned.as_slice()) {
            Ok(done) => done,
            Err(error) => return Err(error.to_string()),
        };
        let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return match output.code {
            Some(0) => Ok(text),
            Some(1) if text.is_empty() => Ok(text),
            code => Err(format!("exited with {code:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())),
        };
    }
}

/// What:
///  The same read-only Git questions,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push plans its upload as a task,
/// and the synchronous `PushQuery` enters the runtime for each question and is
///  refused from there.
/// The questions and the reading of their answers are the synchronous struct's,
/// so the two plan from the same
///  configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushQueryAwaiting implements GitQueryAwaiting { constructor(readonly git: PushGit) {} async ask(args: string[]): string {} }
/// ```
struct PushQueryAwaiting<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    git: &'a PushGit,
}

impl GitQueryAwaiting for PushQueryAwaiting<'_> {
    fn ask<'a>(&'a mut self, query_arguments: &'a [&'a str]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>> {
        return Box::pin(async move {
            let owned: Vec<OsString> = query_arguments.iter().map(OsString::from).collect();
            let output = match run_query_awaiting(self.git, owned.as_slice()).await {
                Ok(done) => done,
                Err(error) => return Err(error.to_string()),
            };
            let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
            return match output.code {
                Some(0) => Ok(text),
                Some(1) if text.is_empty() => Ok(text),
                code => Err(format!("exited with {code:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())),
            };
        });
    }
}

/// What:
///  Run `git <arguments>` (a `push`) as a bounded child,
/// after the upload when the repository has LFS keys.
/// Why:
///   `arguments[0]` is `push`;
/// what follows is its option region,
/// which the pre-check and the plan read.
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
        let mut query: PushQuery<'_> = PushQuery { git };
        let read: Result<Settings, PlanError> = read_settings(&mut query);
        let planned: Result<Option<UploadPlan>, PlanError> = match read {
            Ok(settings) => plan_upload(&[], region, git.environment.as_slice(), git.cwd.as_path(), settings.as_slice(), &mut query),
            Err(error) => Err(error),
        };
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

/// What:
///  The same push,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push runs as a task,
/// and every Git question of the upload plan and the wait for the push itself enter the
///  runtime on the synchronous face and are refused from there.
/// The pre-check,
/// the plan and what a plan that cannot be worked out says are the synchronous face's,
/// so the two cannot
///  upload different objects.
/// A plan that cannot be narrowed is still pushed by `git-lfs` and Git,
/// as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = plan && await uploadBoundedAwaiting(plan); if (failed) return failed; return await runBoundedPushAwaiting(gitCommand(git, args));
/// ```
pub async fn run_push_awaiting(git: &PushGit, arguments: &[OsString]) -> Result<PushAttempt, SingleFlightError> {
    let region: &[OsString] = arguments.get(1..).unwrap_or_default();
    if git.lfs {
        if let Some(refusal) = lfs_refusal(&[], region, git.environment.as_slice()) {
            return Ok(PushAttempt { exit_code: REFUSED_EXIT_CODE, output: strip_one_line_break(refusal.as_str()).to_owned() });
        }
        let mut query: PushQueryAwaiting<'_> = PushQueryAwaiting { git };
        let read: Result<Settings, PlanError> = read_settings_awaiting(&mut query).await;
        let planned: Result<Option<UploadPlan>, PlanError> = match read {
            Ok(settings) => plan_upload_awaiting(&[], region, git.environment.as_slice(), git.cwd.as_path(), settings.as_slice(), &mut query).await,
            Err(error) => Err(error),
        };
        match planned {
            Ok(Some(plan)) => {
                // The plan came from Git queries that exited with status 0, so Git had its say about the values it consults;
                //  the files are read again, and a plan that cannot be narrowed is still pushed by `git-lfs` and Git.
                let layout = read_layout_after_success(git.cwd.as_path(), with_real_git(git.environment.as_slice(), git.real_git.as_path()).as_slice());
                if let Some(narrowed) = narrow_plan(plan, layout.as_ref())
                    && let Some(failed) = run_upload_bounded_awaiting(git, &narrowed).await? {
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
    return run_bounded_push_awaiting(command, push_patience(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice()).await;
}
