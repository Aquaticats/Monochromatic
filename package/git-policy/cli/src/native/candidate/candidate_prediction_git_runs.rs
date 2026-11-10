//! What:
//!  The Git runs a candidate prediction makes:
//! one captured command with its failure messages,
//! and the path of the
//!  real index.
//! Why:
//!  The prediction replays `git add` on a private copy of the index and needs the real index path first;
//! both steps
//!  run Git the same way and report a failed or unstarted command the same way,
//! so they live in a file of their own
//!  that the prediction imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const output = await runGit(realGit, prefix, ['rev-parse', '--git-path', 'index'], overlay, purpose);
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// The captured output of a Git run,
/// the run itself,
/// and the helpers that read a printed path.
use crate::git_metadata::{MetadataOutput, path_from_git_bytes, read_stopped_text, run_metadata_git_without, strip_git_line};
/// The real Git executable is put first in the child environment.
use crate::git_system_config::with_real_git;
/// The in-process repository layout,
/// tried before the `rev-parse --git-path` query.
use crate::repo_read_layout::{git_path, read_layout};
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// What:
///  Build the failure of a Git command a prediction ran.
///       `&MetadataOutput` borrows the captured run;
///  `None` means Git could not be started.
/// Why:
///   A failed replay is reported with Git's own message,
///  so the person sees why the
///       command they typed could not be staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function predictionFailure(purpose: string, operation: string, output?: GitOutput, error?: Error): CandidateError;
/// ```
fn command_failure(purpose: &str, operation: &str, output: &MetadataOutput) -> CandidateError {
    // `String::from_utf8_lossy` decodes Git's message for display, replacing invalid bytes.
    return CandidateError::new(
        CandidateFailure::GitFailed,
        format!(
            "monochromatic-git-policy-cli could not {purpose}: git {operation} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )
        .as_str(),
    );
}

/// What:
///  Build the failure of a Git command that could not be started.
/// Why:
///   Without Git there is nothing to predict,
///  and the person needs the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notStarted(purpose: string, operation: string, error: Error): CandidateError;
/// ```
fn not_started(purpose: &str, operation: &str, error: &std::io::Error) -> CandidateError {
    return CandidateError::new(
        CandidateFailure::GitNotStarted,
        format!("monochromatic-git-policy-cli could not {purpose}: git {operation} could not be started: {error}.")
            .as_str(),
    );
}

/// What:
///  Run one Git command of a prediction and return its output when it succeeded.
///       `&[OsString]` borrows the global prefix and the command separately.
/// Why:
///   The prefix selects the same repository the caller's command selects;
///  the
///       command follows it.
///  A command whose pathspecs the wrapper wrote also names the mode variables to remove,
///  because
///  a mode the caller set would read those pathspecs as something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runGit(git, prefix, command, overlay, remove, purpose): Promise<GitOutput>;
/// ```
pub fn run_git(
    real_git: &Path,
    prefix: &[OsString],
    command: &[OsString],
    overlay: &[(OsString, OsString)],
    remove: &[&str],
    purpose: &str,
) -> Result<MetadataOutput, CandidateError> {
    // `.to_vec()` copies the borrowed prefix; `.extend_from_slice(..)` appends the command.
    let mut arguments: Vec<OsString> = prefix.to_vec();
    arguments.extend_from_slice(command);
    // The operation named in messages is the command's first word, `add` or `rev-parse`.
    let operation: String = match command.first() {
        Some(word) => word.to_string_lossy().into_owned(),
        None => String::from("(no command)"),
    };
    // `match` on the start result: `Err(error)` means Git could not be started at all.
    let output: MetadataOutput = match run_metadata_git_without(real_git, arguments.as_slice(), overlay, remove) {
        Ok(captured) => captured,
        // A read the wrapper ended at its ceiling says what blocked it and what stayed as it was.
        Err(error) => {
            return Err(match read_stopped_text(operation.as_str(), &error) {
                Some(stopped) => CandidateError::new(CandidateFailure::GitStopped, stopped.as_str()),
                None => not_started(purpose, operation.as_str(), &error),
            });
        }
    };
    if !output.success {
        return Err(command_failure(purpose, operation.as_str(), &output));
    }
    return Ok(output);
}

/// What:
///  The absolute path of the index the caller's command would use.
/// Why:
///   Git answers for `GIT_INDEX_FILE`,
///  a linked worktree's own index and
///       `--git-dir`,
///  so the copy is of exactly the index `git add` would change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function realIndexPath(git, prefix, overlay): Promise<string>;
/// ```
pub fn real_index_path(
    real_git: &Path,
    prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    purpose: &str,
) -> Result<PathBuf, CandidateError> {
    // A plain layout names the index without a Git process.
    let layout = read_layout(prefix, with_real_git(overlay, real_git).as_slice());
    if let Some(found) = layout.as_ref()
        && let Some(path) = git_path(found, "index")
    {
        return Ok(path);
    }
    let output: MetadataOutput = run_git(
        real_git,
        prefix,
        &[
            OsString::from("rev-parse"),
            OsString::from("--path-format=absolute"),
            OsString::from("--git-path"),
            OsString::from("index"),
        ],
        overlay,
        &[],
        purpose,
    )?;
    // `match` unpacks "a path or nothing": empty output names no index.
    match path_from_git_bytes(strip_git_line(output.stdout.as_slice())) {
        Some(path) => return Ok(path),
        None => {
            return Err(CandidateError::new(
                CandidateFailure::ListingMalformed,
                format!(
                    "monochromatic-git-policy-cli could not {purpose}: git rev-parse --git-path index printed no path."
                )
                .as_str(),
            ));
        }
    }
}
