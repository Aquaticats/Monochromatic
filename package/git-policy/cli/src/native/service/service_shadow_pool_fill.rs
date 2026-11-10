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
//!  The fill runs on a thread of its own beside the landing loop,
//! never on a commit's path and never on the runtime's blocking pool:
//! a Git start needs the runtime that runs its waits,
//! and that runtime refuses a start asked for from inside itself.
//!  A fill that cannot run leaves the pool empty,
//! which is the case every commit already handles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! startPoolFill({ context, request }); // returns at once
//! ```

/// One thread per fill,
/// which the system may refuse.
use crate::scoped_start::{refused_thread_sentence, spawn_named};
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
use std::sync::atomic::{AtomicBool, Ordering};

/// Lock kept:
///  whether this service is already filling a pool,
///  which it sets and clears and nothing else touches:
/// one fill at a time is as much as a pool of two members ever needs,
/// and a landing never waits for this flag,
/// because a wake that finds it set simply leaves the pool as it is.
static FILLING: AtomicBool = AtomicBool::new(false);

/// What:
///  Starts one fill for the worktree a request names,
/// unless the pool already holds what it would make or a fill is already running.
/// Why:
///   Every answer the service gives is a commit the person waits for,
/// so the fill is started beside the loop and never inside it.
///  The two checks cost one `read_dir` and one atomic exchange,
/// and no Git start,
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
    if FILLING.swap(true, Ordering::AcqRel) {
        return;
    }
    let own: GitContext = context.clone();
    let named: String = request.common_dir.display().to_string();
    let common_dir: PathBuf = request.common_dir.clone();
    let git_dir: PathBuf = request.git_dir.clone();
    let repository: PathBuf = request.repository.clone();
    let started = spawn_named("shadow-pool-fill", move || {
        // The flag is cleared however the fill ends, so one refused Git start does not stop every later fill.
        let outcome: Result<Option<PathBuf>, GitFailure> = fill_one(&own, &common_dir, &git_dir, &repository);
        FILLING.store(false, Ordering::Release);
        match outcome {
            Ok(_made) => {}
            Err(error) => {
                crate::service_warn!("the shadow pool of {named} was not filled: {}", error.0);
            }
        }
    });
    if let Err(error) = started {
        FILLING.store(false, Ordering::Release);
        crate::service_warn!("{}", refused_thread_sentence("fill the shadow pool", &error));
    }
}

/// What:
///  One fill,
/// from the facts of the repository the request names.
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
///  The object directory,
/// the object format and the ref backend of a worktree,
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
