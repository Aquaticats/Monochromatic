//! What:
//!  The Git LFS step before a push the caller typed:
//! the pre-check,
//! the plan and the upload,
//! with the caller's streams.
//! Why:
//!  The parent owns the whole step,
//! so a refusal,
//! a plan that cannot be worked out and a failed upload are all explained
//!  by the process that saw them,
//! and the push is not started after any of them.
//! The configuration the plan reads comes
//!  from the snapshot the wrapper already took when it planned the invocation,
//! so a push whose repository is known costs
//!  no Git start for it;
//! without a snapshot one `config --get-regexp` start reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stop = await uploadBeforePush({ prefix, region, realGit, environment, overlay, settings }); if (stop !== undefined) return stop;
//! ```

/// The invocation's directory option.
use crate::commit_command::effective_directory;
/// The builder of real-Git commands.
use crate::forwarding::git_command;
/// Who a real-Git start runs for: the wrapper's own work, not the person's command.
use crate::hooks_off::StartOwner;
/// The refusal texts.
use crate::lfs_pre_push::plan_failed_message;
/// The pre-check that the program can start.
use crate::lfs_push_precheck::lfs_refusal;
/// Narrows the plan to the objects the push introduces.
use crate::lfs_object_plan::narrow_plan;
/// The upload itself.
use crate::lfs_upload::{run_upload_interactive, submodule_notice};
/// The repository found from the files.
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout_after_success_for;
/// What to upload.
use crate::lfs_upload_plan::{GitQuery, PlanError, Settings, UploadPlan, plan_upload, read_settings};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// Standard error as a writable stream.
use std::io::Write;
/// The real Git.
use std::path::Path;

/// What the step needs from the invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InteractiveUpload = { prefix: string[]; region: string[]; realGit: string; environment; overlay; settings?: Settings };
/// ```
pub struct InteractiveUpload<'a> {
    /// The global options before `push`.
    pub prefix: &'a [OsString],
    /// The arguments after `push`.
    pub region: &'a [OsString],
    /// The real Git,
    /// whose queries fill in what the invocation does not hold.
    pub real_git: &'a Path,
    /// The invocation's environment.
    pub environment: &'a [(OsString, OsString)],
    /// Variables added to the environment of every Git the wrapper starts.
    pub overlay: &'a [(OsString, OsString)],
    /// The configuration entries the plan reads,
    /// when the invocation already holds them.
    pub settings: Option<&'a [(String, String)]>,
}

/// What:
///  The read-only Git questions of the upload plan,
/// asked by starting the real Git with the invocation's prefix and
///  overlay.
/// `&'a InteractiveUpload<'a>` lends the invocation's facts for as long as this value lives.
/// Why:
///   The plan asks through the `GitQuery` trait,
/// and the state a question needs is one borrowed value,
/// so a struct
///  stands where a closure that captured it stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InteractiveQuery implements GitQuery { constructor(readonly request: InteractiveUpload) {} ask(args: string[]): string {} }
/// ```
struct InteractiveQuery<'a> {
    /// The invocation whose prefix,
    /// real Git and overlay every question uses.
    request: &'a InteractiveUpload<'a>,
}

impl GitQuery for InteractiveQuery<'_> {
    fn ask(&mut self, query_arguments: &[&str]) -> Result<String, String> {
        let mut arguments: Vec<OsString> = self.request.prefix.to_vec();
        arguments.extend(query_arguments.iter().map(OsString::from));
        let output = match git_command(self.request.real_git, arguments.as_slice(), self.request.overlay, StartOwner::Wrapper).output() {
            Ok(done) => done,
            Err(error) => return Err(error.to_string()),
        };
        let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return match output.status.code() {
            Some(0) => Ok(text),
            Some(1) if text.is_empty() => Ok(text),
            code => Err(format!("exited with {code:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())),
        };
    }
}

/// What:
///  Run the step;
/// the status to stop with when the push must not start,
/// or nothing when it may.
/// Why:
///   A missing `git-lfs`,
/// a plan that cannot be worked out and a failed upload each end the push before it starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uploadFirst(request: InteractiveUpload): number | undefined;
/// ```
pub fn upload_first(request: &InteractiveUpload<'_>) -> Option<i32> {
    if let Some(refusal) = lfs_refusal(request.prefix, request.region, request.environment) {
        // Terminal output error: the exit status already carries the refusal.
        let _written = std::io::stderr().write_all(refusal.as_bytes());
        return Some(1);
    }
    let mut query: InteractiveQuery<'_> = InteractiveQuery { request };
    let directory: std::path::PathBuf =
        effective_directory(std::env::current_dir().unwrap_or_default().as_path(), request.prefix);
    let read: Result<Settings, PlanError> = match request.settings {
        Some(held) => Ok(held.to_vec()),
        None => read_settings(&mut query),
    };
    let planned: Result<Option<UploadPlan>, PlanError> = match read {
        Ok(settings) => plan_upload(request.prefix, request.region, request.environment, directory.as_path(), settings.as_slice(), &mut query),
        Err(error) => Err(error),
    };
    match planned {
        Ok(Some(plan)) => {
            // The layout is read from the files; a repository the readers decline keeps the plan, which `git-lfs` scans.
            // The plan came from Git queries that exited with status 0 (the settings may be held from an earlier query),
            // so Git had its say about the values it consults;
            // the files are read again,
            // and a plan that cannot be narrowed is still pushed by `git-lfs` and Git.
            let layout = read_layout_after_success_for(request.prefix, with_real_git(request.overlay, request.real_git).as_slice());
            let submodules: bool = plan.submodules;
            let Some(narrowed) = narrow_plan(plan, layout.as_ref()) else {
                // Nothing of this repository to upload, but Git still pushes the submodules, whose objects are not covered.
                if submodules {
                    // Terminal output error: the notice is advisory.
                    let _written = std::io::stderr().write_all(submodule_notice().as_bytes());
                }
                return None;
            };
            return run_upload_interactive(
                &narrowed,
                request.prefix,
                directory.as_path(),
                request.real_git,
                request.environment,
                request.overlay,
                &mut std::io::stderr(),
            );
        }
        Ok(None) => return None,
        Err(error) => {
            // Terminal output error: the exit status already carries the refusal.
            let _written = std::io::stderr().write_all(plan_failed_message(error.0.as_str()).as_bytes());
            return Some(1);
        }
    }
}
