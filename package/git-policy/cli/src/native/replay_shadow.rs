//! What:
//!  Running real Git against the shadow repository for replay:
//!       the shadow selection, an optional private index, bytes on standard input,
//!       and the caller's kept global options when a signed rebuild needs them.
//! Why:
//!  Every replay step runs in `git --git-dir=<shadow>` so its objects stay in the shadow store
//!       until the next migration;
//!       one runner keeps the selection in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! runShadowGit({ gitPath, shadowPath, args, input, environment });
//! ```

/// Shadow selection and the transaction Git runner.
use super::shadow_git::shadow_request;
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// `OsString` names environment entries and arguments.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Runs one shadow Git command and requires success.
/// Why:
///   Replay cannot continue after a failed `merge-base`, `hash-object` or `read-tree`;
///       the caller maps the failure to `transaction-failed`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runShadowGit({ gitPath, shadowPath, args, input, environment });
/// ```
pub fn run_shadow_checked(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    arguments: &[&str],
    input: Option<&[u8]>,
    index: Option<&Path>,
    environment: &[(OsString, OsString)],
) -> Result<GitOutput, GitFailure> {
    let mut request: GitRequest = shadow_request(shadow, cwd, arguments);
    request.input = input.map(|bytes| { return bytes.to_vec(); });
    request.index = index.map(|path| { return path.to_path_buf(); });
    request.environment = environment.to_vec();
    return run_git_checked(context, &request);
}

/// What:
///  The private index file a replay step edits, named by the replay number.
/// Why:
///   Each replay keeps its own index under the transaction directory, so a later replay never reads an earlier one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(directory, `replay-base-${replay}.index`)
/// ```
pub fn replay_base_index(directory: &Path, replay: u64) -> PathBuf {
    return directory.join(format!("replay-base-{replay}.index"));
}

#[cfg(test)]
#[path = "replay_shadow_tests.rs"]
mod tests;
