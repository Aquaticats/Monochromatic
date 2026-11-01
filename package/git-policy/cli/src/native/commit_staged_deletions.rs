//! What:
//!  The commit-only finding for a staged deletion that an explicit-path commit cannot record because the path is
//!  still on disk.
//! Why:
//!  An explicit pathspec selects worktree content, never index state, so a deletion staged with `git rm --cached`
//!  for a path that stays on disk (an ignored editor file, for example) is not the content the pathspec selects.
//!  Without this finding Git answers "nothing to commit", or a private `git add` fails with a hint to force-add the
//!  ignored path, which would re-track the file instead of recording its removal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const present = stagedDeletions.filter((path) => existsSync(join(repositoryRoot, path)));
//! ```

use super::capture_order::indexed_request;
use super::path_bytes::{os_from_path_bytes, path_message_text};
use super::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
use std::ffi::OsString;
use std::path::Path;

/// The core of the finding; the same core as the other commit-only findings.
pub const STAGED_DELETION_CORE_ID: &str = "commit-only";

/// The code of the finding, reported as `commit-only/staged-deletion-worktree-present`.
pub const STAGED_DELETION_CODE: &str = "staged-deletion-worktree-present";

/// What:
///  The selected paths whose deletion is staged in the real index while an entry still exists at that worktree path.
/// Why:
///   Only these paths make an explicit-path commit disagree with the staged state; the read touches no index,
///  object store or worktree file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git diff-index --cached --name-only --diff-filter=D -z <base> -- <pathspecs>
/// ```
pub fn staged_deletions_present_on_disk(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    real_index: &Path,
    base_revision: &str,
    pathspecs: &[OsString],
) -> Result<Vec<Vec<u8>>, GitFailure> {
    if pathspecs.is_empty() {
        return Ok(Vec::new());
    }
    let mut arguments: Vec<OsString> = ["diff-index", "--cached", "--name-only", "--diff-filter=D", "-z", base_revision, "--"]
        .iter()
        .map(OsString::from)
        .collect();
    arguments.extend(pathspecs.iter().cloned());
    let output: GitOutput = run_git_checked(context, &indexed_request(cwd, &arguments, Some(real_index), None))?;
    return Ok(output
        .stdout
        .split(|byte| { return *byte == 0; })
        .filter(|path| { return !path.is_empty(); })
        .filter(|path| {
            return repository_root.join(os_from_path_bytes(path)).symlink_metadata().is_ok();
        })
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The message of the finding: the paths, why a pathspec cannot record them, and each valid way forward.
/// Why:
///   Both intents are legitimate (commit the removal, or re-track the file), and the diagnostic must not pick one;
///  forcing an add of an ignored path belongs only to the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: git commit with explicit paths selects worktree content, so it cannot record the staged deletion of ...`
/// ```
pub fn staged_deletion_message(paths: &[Vec<u8>]) -> String {
    let names: Vec<String> = paths.iter().map(|path| { return path_message_text(path.as_slice()); }).collect();
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "cli-git: git commit with explicit paths selects worktree content, so it cannot record the staged deletion of \
{listed}: the path stays on disk. Choose explicitly: to commit the deletion and keep the file, pass --no-only without \
paths (this commits every staged change, not only these paths); to commit the file's current content instead, stage it \
again with git add first (an ignored path also needs git add --force, which tracks the file again)."
    );
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_staged_deletions_tests.rs"]
mod tests;
