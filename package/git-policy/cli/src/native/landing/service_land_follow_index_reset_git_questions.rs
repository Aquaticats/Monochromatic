//! What:
//!  How one step of the index reset asks Git,
//! on a thread and on the runtime,
//! and what answer it stands for.
//! Why:
//!  The entries a landing installs in the index are asked of Git through one request
//! (`service_land_follow_index_reset.rs`),
//!  so neither face can run another command,
//! feed other bytes or read another index than the other.
//! The failure a refused step stands for belongs to that answer too,
//! which is why it is read here and not where the entries are derived.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await runAwaiting(git, repository, arguments, input, index); // or throw classify(output)
//! ```

use crate::service_protocol::FailureCode;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

/// What:
///  The question one step of this module asks Git,
/// with the input it feeds it and the index it reads.
/// Why:
///  Both faces ask one request,
/// so neither can run another command,
/// feed other bytes or read another index than
///  the other;
/// every pathspec this module writes is its own,
/// so no pathspec mode of the caller's reaches it
///  (`GitRequest::own_pathspecs`),
/// whether the caller set it as a variable or as a global option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { ...new GitRequest(repository, arguments), ownPathspecs: true, input, index };
/// ```
fn request_of(
    repository: &Path,
    arguments: &[&str],
    input: Option<Vec<u8>>,
    index: Option<&Path>,
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(repository, arguments).own_pathspecs();
    request.input = input;
    request.index = index.map(Path::to_path_buf);
    return request;
}

/// What:
///  Git's output of one step,
///  and the failure a Git that could not be started stands for.
/// Why:
///  Both faces read the run through this one function,
/// so a step Git could not start is the same failure for both of
///  them,
/// and every other answer is Git's own output whichever face asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runAnswered = (answered) => answered.error ? 'unavailable' : answered;
/// ```
fn run_answered(answered: std::io::Result<GitOutput>) -> Result<GitOutput, FailureCode> {
    return match answered {
        Ok(output) => Ok(output),
        Err(_unstartable) => Err(FailureCode::Unavailable),
    };
}

/// One step of the index reset,
/// asked of Git on this thread.
pub(super) fn run(
    git: &GitContext,
    repository: &Path,
    arguments: &[&str],
    input: Option<Vec<u8>>,
    index: Option<&Path>,
) -> Result<GitOutput, FailureCode> {
    return run_answered(run_git(
        git,
        &request_of(repository, arguments, input, index),
    ));
}

/// What:
///  `run` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and every step of this module runs
///  from the landing once that landing is a task.
/// The question and the reading of the answer are the synchronous face's,
/// so
///  the two cannot differ in what they run or in how they fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runAwaiting(git, repository, arguments, input, index);
/// ```
pub(super) async fn run_awaiting(
    git: &GitContext,
    repository: &Path,
    arguments: &[&str],
    input: Option<Vec<u8>>,
    index: Option<&Path>,
) -> Result<GitOutput, FailureCode> {
    return run_answered(
        run_git_awaiting(git, &request_of(repository, arguments, input, index)).await,
    );
}

/// The failure a Git step that did not succeed stands for.
pub(crate) fn classify(output: &GitOutput) -> FailureCode {
    let text: String = output.error_text().to_lowercase();
    if text.contains("no space left") || text.contains("quota exceeded") {
        return FailureCode::RepositoryFull;
    }
    if text.contains("read-only file system") {
        return FailureCode::RepositoryReadOnly;
    }
    if text.contains("permission denied") {
        return FailureCode::PermissionDenied;
    }
    if text.contains("index.lock") {
        return FailureCode::IndexInUse;
    }
    return FailureCode::Unavailable;
}
