//! What:
//!  Git commands run against a transaction's shadow repository,
//!  and its location and
//!       removal.
//! Why:
//!  The shadow lives at `<git-common-dir>/cli-git/shadow/<transaction-id>`,
//!  derived from the
//!      transaction ID so recovery finds it before `prepared.json` exists.
//!  cli-git's own shadow
//!      commands remove every variable that would redirect Git elsewhere and point hooks at an
//!      absent directory,
//!  so no hook observes private shadow writes
//!      (`src/shadow-repository/shadow-refs.ts`,
//!  `runShadowGit`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runShadowGit({ gitPath, shadowPath, args: ['rev-parse', 'HEAD'] });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Tree removal.
use super::private_storage::remove_tree;
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// One Git start.
use super::transaction_git::GitRequest;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Variables that would redirect a shadow command to another repository,
///  index or store.
pub const REPOSITORY_REDIRECT_VARIABLES: [&str; 5] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_INDEX_FILE",
];

/// Absent shadow directory named as the hooks path of cli-git's own shadow commands.
pub const NO_HOOKS_DIRECTORY: &str = "no-hooks";

/// What:
///  The shadow repository path of a transaction.
/// Why:
///   `shadowRepositoryPath`:
///  derived from the ID alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shadowRepositoryPath({ commonDir, transactionId }) // `${commonDir}/cli-git/shadow/${transactionId}`
/// ```
pub fn shadow_repository_path(common_dir: &Path, transaction_id: &str) -> PathBuf {
    return common_dir
        .join("cli-git")
        .join("shadow")
        .join(transaction_id);
}

/// What:
///  A request running `arguments` against the shadow:
///  `--git-dir=<shadow>`,
///  hooks pointed
///       at an absent directory,
///  redirecting variables removed,
///  in `cwd`.
/// Why:
///   Every cli-git shadow command looks the same;
///  the caller adds input or variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runShadowGit({ gitPath, shadowPath, args, cwd })
/// ```
pub fn shadow_request<S: AsRef<std::ffi::OsStr>>(
    shadow: &Path,
    cwd: &Path,
    arguments: &[S],
) -> GitRequest {
    // `mut` allows prefixing the shadow selection.
    let mut git_dir: OsString = OsString::from("--git-dir=");
    git_dir.push(shadow.as_os_str());
    let mut hooks: OsString = OsString::from("core.hooksPath=");
    hooks.push(shadow.join(NO_HOOKS_DIRECTORY).as_os_str());
    let mut full: Vec<OsString> = vec![git_dir, OsString::from("-c"), hooks];
    for argument in arguments {
        full.push(argument.as_ref().to_os_string());
    }
    let mut request: GitRequest = GitRequest::new(cwd, full.as_slice());
    request.without_prefix = true;
    for name in REPOSITORY_REDIRECT_VARIABLES {
        request.unset.push(OsString::from(name));
    }
    return request;
}

/// What:
///  Remove a shadow repository,
///  treating an absent one as removed.
/// Why:
///   `removeShadowRepository`:
///  a shadow never outlives its transaction directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(shadowPath, { recursive: true, force: true });
/// ```
pub fn remove_shadow_repository(shadow: &Path) -> Result<(), RecoveryError> {
    if let Err(error) = remove_tree(shadow) {
        return Err(io_failure("removing the shadow repository", shadow, &error));
    }
    debug(
        "removeShadowRepository",
        format!("removed shadow repository {}", shadow.display()).as_str(),
    );
    return Ok(());
}

/// Shadow request controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_git_tests.rs"]
mod tests;
