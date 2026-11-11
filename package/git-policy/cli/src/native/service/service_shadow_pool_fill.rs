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
/// Answers of one `git rev-parse` report,
/// read as the bytes Git wrote.
use crate::git_report_answers::{absolute_reported, answers_of, object_format_of};
/// One answer without the line feed that ends it.
use crate::hooks_path_values::without_the_terminator;
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
    let (object_directory, object_format, ref_format): (PathBuf, String, RefFormat) = facts_of(context, repository)?;
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

/// How many answers the three facts make in one report.
const FACTS: usize = 3;

/// What:
///  The object directory,
/// the object format and the ref backend of a worktree,
///  as Git printed them.
/// Why:
///   One start answers all three,
///  and a pathname that holds a line feed puts one inside its own answer,
/// so a report of more answers than three sends each fact to a start of its own.
///  No answer is ever decoded as text:
/// the two that are text are read as text and the pathname keeps its bytes (`path_bytes.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = splitReport(await revParse(FACTS_ARGS)); answers.length === 3 ? facts(answers) : await eachFactAlone()
/// ```
fn facts_of(context: &GitContext, repository: &Path) -> Result<(PathBuf, String, RefFormat), GitFailure> {
    let report: Vec<u8> = rev_parse(
        context,
        repository,
        &["rev-parse", "--path-format=absolute", "--git-path", "objects", "--show-object-format", "--show-ref-format"],
    )?;
    let answers: Vec<&[u8]> = answers_of(report.as_slice());
    if answers.len() == FACTS {
        return Ok((
            absolute_reported(repository, answers[0])?,
            object_format_of(answers[1])?,
            ref_format_of(answers[2]),
        ));
    }
    let object_directory: Vec<u8> = rev_parse(context, repository, &["rev-parse", "--path-format=absolute", "--git-path", "objects"])?;
    let object_format: Vec<u8> = rev_parse(context, repository, &["rev-parse", "--show-object-format"])?;
    let ref_format: Vec<u8> = rev_parse(context, repository, &["rev-parse", "--show-ref-format"])?;
    return Ok((
        absolute_reported(repository, object_directory.as_slice())?,
        object_format_of(object_format.as_slice())?,
        ref_format_of(ref_format.as_slice()),
    ));
}

/// What:
///  The ref backend one answer names.
/// Why:
///   `reftable` is the only backend besides the files one this package knows,
///  and every other answer names the files backend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutTheTerminator(answer) === 'reftable' ? Reftable : Files
/// ```
fn ref_format_of(answer: &[u8]) -> RefFormat {
    return match without_the_terminator(answer) == b"reftable" {
        true => RefFormat::Reftable,
        false => RefFormat::Files,
    };
}

/// What:
///  The bytes of one `git rev-parse` report,
///  asked in the repository and with this package's own variables stripped.
/// Why:
///   The report is the bytes Git wrote,
///  because its answers include pathnames a decoding loses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = await runGit({ cwd: repository, args }); return report.stdout
/// ```
fn rev_parse(context: &GitContext, repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, GitFailure> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.without_prefix = true;
    request.unset = REPOSITORY_REDIRECT_VARIABLES.iter().copied().map(OsString::from).collect();
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(output.stdout);
}

/// Pool-fill controls stay out of the release executable.
#[cfg(test)]
#[path = "service_shadow_pool_fill_tests.rs"]
mod tests;
