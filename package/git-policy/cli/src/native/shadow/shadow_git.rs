//! What:
//!  Git commands run against a transaction's shadow repository,
//!  and its location and
//!       removal.
//! Why:
//!  The shadow lives at `<git-common-dir>/monochromatic-git-policy-cli/shadow/<transaction-id>`,
//!  derived from the
//!      transaction ID so recovery finds it before `prepared.json` exists.
//!  monochromatic-git-policy-cli's own shadow
//!      commands remove every variable that would redirect Git elsewhere and point hooks at an
//!      absent directory,
//!  so no hook observes private shadow writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runShadowGit({ gitPath, shadowPath, args: ['rev-parse', 'HEAD'] });
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// Tree removal.
use crate::private_storage::remove_tree;
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// One Git start.
use crate::transaction_git::GitRequest;
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

/// Name of the absent directory inside a shadow,
/// named as the hooks path of monochromatic-git-policy-cli's own shadow commands.
pub const SHADOW_NO_HOOKS_NAME: &str = "no-hooks";

/// Name of the directory under `<git-common-dir>/monochromatic-git-policy-cli` that holds one shadow repository per transaction.
pub const SHADOW_ROOT_NAME: &str = "shadow";

/// What:
///  The shadow repository path of a transaction.
/// Why:
///   `shadowRepositoryPath`:
///  derived from the ID alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shadowRepositoryPath({ commonDir, transactionId }) // `${commonDir}/monochromatic-git-policy-cli/shadow/${transactionId}`
/// ```
pub fn shadow_repository_path(common_dir: &Path, transaction_id: &str) -> PathBuf {
    return common_dir
        .join("monochromatic-git-policy-cli")
        .join(SHADOW_ROOT_NAME)
        .join(transaction_id);
}

/// What:
///  The `--git-dir=` option naming `shadow`,
///  built from the path's own bytes.
/// Why:
///   A shadow lives under the repository's Git directory,
///  whose name may hold bytes that are not UTF-8;
///  a spelling of it names a directory that does not exist,
///  which Git reports as "not a git repository".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([Buffer.from('--git-dir='), Buffer.from(shadow)])
/// ```
pub fn git_dir_option(shadow: &Path) -> OsString {
    let mut option: OsString = OsString::from("--git-dir=");
    option.push(shadow.as_os_str());
    return option;
}

/// What:
///  A request running `arguments` against the shadow:
///  `--git-dir=<shadow>`,
///  hooks pointed
///       at an absent directory,
///  redirecting variables removed,
///  in `cwd`.
/// Why:
///   Every monochromatic-git-policy-cli shadow command looks the same;
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
    let git_dir: OsString = git_dir_option(shadow);
    let mut hooks: OsString = OsString::from("core.hooksPath=");
    hooks.push(shadow.join(SHADOW_NO_HOOKS_NAME).as_os_str());
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
