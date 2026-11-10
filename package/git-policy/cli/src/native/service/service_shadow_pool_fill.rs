//! What:
//!  The creation of a finished shadow the resident service runs ahead of a commit,
//!  from the facts of a request it already holds.
//! Why:
//!  The owner chose the pool ahead of the alternatives
//! (`doc/handover/questions-for-the-owner.md`,
//! "May a commit take its private repository from a pool made ahead?",
//! "Pool made ahead (Recommended),
//! We don't need to build a shadow for every commit.
//! Be smarter."),
//! and the option's own words name who fills it:
//! "The resident service fills a pool of finished shadows,
//! a commit renames one into place,
//! and it builds inline as today when the pool is empty."
//!  The fill runs beside the landing loop,
//! never on a commit's path,
//! and a fill that cannot run leaves the pool empty,
//! which is the case every commit already handles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! startPoolFill({ context, request }); // returns at once
//! ```

/// One step of blocking work,
/// off the runtime's threads.
use crate::blocking_job::run_blocking_job;
/// The service's own runtime.
use crate::lazy_runtime::spawn_on_runtime;
/// What a commit request names,
/// and the facts a shadow is made from.
use crate::service_protocol::CommitRequest;
use crate::shadow_create::ShadowTemplate;
use crate::shadow_config::{RepositoryFormat, read_repository_format};
use crate::shadow_pool::{POOL_LIMIT, fill, members, pool_root_for};
use crate::shadow_git::REPOSITORY_REDIRECT_VARIABLES;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use crate::transaction_journal::RefFormat;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  Starts one fill for the worktree a request names,
/// unless the pool already holds what it would make.
/// Why:
///   Every answer the service gives is a commit the person waits for,
/// so the fill is started beside the loop and never inside it.
///  The check that the pool is full costs one `read_dir` and no Git start,
/// which is what a busy repository does on every landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start({ context, request }) // void
/// ```
pub fn start(context: &GitContext, request: &CommitRequest) {
    let root: PathBuf = pool_root_for(request.common_dir.as_path(), request.git_dir.as_path());
    if members(root.as_path()).len() >= POOL_LIMIT {
        return;
    }
    let context: GitContext = context.clone();
    let common_dir: PathBuf = request.common_dir.clone();
    let named: String = common_dir.display().to_string();
    let told: String = named.clone();
    let git_dir: PathBuf = request.git_dir.clone();
    let repository: PathBuf = request.repository.clone();
    let started = spawn_on_runtime(async move {
        match run_blocking_job(move || fill_one(&context, &common_dir, &git_dir, &repository)).await {
            Ok(Ok(_made)) => {}
            Ok(Err(error)) => {
                crate::service_warn!("the shadow pool of {told} was not filled: {}", error.0);
            }
            Err(error) => {
                crate::service_warn!("the shadow pool of {told} was not filled: the blocking pool gave no thread ({error:?})");
            }
        }
    });
    if started.is_none() {
        crate::diagnostic_log::debug(
            "poolFill",
            format!("the shadow pool of {named} is left to a later fill: this service has no worker to run one on").as_str(),
        );
    }
}

/// What:
///  One fill, from the facts of the repository the request names.
/// Why:
///   The facts are read here rather than in the request,
/// because a request carries what a landing needs and not what a creation does;
///  a fact this derivation gets wrong makes a member the claim refuses,
/// never a wrong commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fillOne({ context, commonDir, gitDir, repository })
/// ```
fn fill_one(
    context: &GitContext,
    common_dir: &Path,
    git_dir: &Path,
    repository: &Path,
) -> Result<Option<PathBuf>, GitFailure> {
    let template: ShadowTemplate = template_of(context, common_dir, git_dir, repository)?;
    return fill(context, &template);
}

/// What:
///  The facts a creation would be made from in this repository,
///  read the way the invocation reads them.
/// Why:
///   A member is refused unless its config and alternates are what the claiming invocation would write,
/// so the fill asks Git the same questions the client's capture asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await templateOf({ context, commonDir, gitDir, repository })
/// ```
fn template_of(
    context: &GitContext,
    common_dir: &Path,
    git_dir: &Path,
    repository: &Path,
) -> Result<ShadowTemplate, GitFailure> {
    let format: RepositoryFormat = read_repository_format(context, common_dir)?;
    let reported: String = rev_parse_of(context, repository)?;
    let mut lines = reported.lines();
    let object_directory: PathBuf = absolute(repository, lines.next().unwrap_or_default());
    let object_format: String = String::from(lines.next().unwrap_or_default());
    let ref_format: RefFormat = match lines.next() {
        Some("reftable") => RefFormat::Reftable,
        _ => RefFormat::Files,
    };
    return Ok(ShadowTemplate {
        common_dir: common_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        repository_root: repository.to_path_buf(),
        object_directory,
        object_format,
        ref_format,
        format,
    });
}

/// What:
///  The object directory, the object format and the ref backend of a worktree,
///  as one Git start answers them.
/// Why:
///   The three facts come from the same repository,
/// and the client's own capture takes them in one start too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await revParseOf({ context, repository }) // 'path\nformat\nrefbackend\n'
/// ```
fn rev_parse_of(context: &GitContext, repository: &Path) -> Result<String, GitFailure> {
    let mut request: GitRequest = GitRequest::new(
        repository,
        &["rev-parse", "--path-format=absolute", "--git-path", "objects", "--show-object-format", "--show-ref-format"],
    );
    request.without_prefix = true;
    request.unset = REPOSITORY_REDIRECT_VARIABLES.iter().copied().map(OsString::from).collect();
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim_end().to_string());
}

/// What:
///  A path Git reported,
///  made absolute against the directory it was asked in.
/// Why:
///   `--path-format=absolute` answers an absolute path,
/// and a report of another shape must still name one path rather than a relative one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// absolute(repository, reported)
/// ```
fn absolute(repository: &Path, reported: &str) -> PathBuf {
    let path: PathBuf = PathBuf::from(reported.trim());
    return if path.is_absolute() { path } else { repository.join(path) };
}

/// Pool-fill controls stay out of the release executable.
#[cfg(test)]
#[path = "service_shadow_pool_fill_tests.rs"]
mod tests;
