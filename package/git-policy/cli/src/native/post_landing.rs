//! What:
//!  The post-landing completion of one landed commit, in the incumbent's order:
//!  added-path worktree copies, shadow and transaction removal with capture pruning,
//!  automatic maintenance.
//! Why:
//!  The incumbent's `commit-transaction-conclusion.ts` runs these steps after a landing.
//!  Worktree copies come first, so an interruption before cleanup leaves the journal for recovery to finish them.
//!  Maintenance bounds the pack count each landing grows.
//!  A maintenance failure is logged and never fails the landed commit; a copy or disposal failure fails the command
//!  with its message, as the incumbent's thrown error does.
//!  No hook and no program named by a configuration runs after the landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await installAddedWorktreeFiles({ gitPath, cwd, repositoryRoot, records: [...addedPaths, ...worktreeRecords], objectDirectory });
//! await workspace[Symbol.asyncDispose]();
//! await runAutoMaintenance({ gitPath, cwd: repositoryRoot, globalArgs });
//! ```

use super::diagnostic_log::{debug, warn};
use super::landed_disposal::{LandedTransactionPaths, dispose_landed_transaction};
use super::config_snapshot::ConfigSnapshot;
use super::maintenance_run::{AutoMaintenanceOutcome, run_auto_maintenance_with};
use super::recovery_error::RecoveryError;
use super::transaction_git::GitContext;
use super::transaction_journal::AddedPath;
use super::worktree_completion::{InstallResult, install_added_worktree_files};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// What:
///  What a landing reports about the commit it made, and the transaction paths that landing left behind.
/// Why:
///   The landing lane fills this from its outcome; this lane only reads it.
///  `worktree_records` holds the selected corrections followed by the hook changes, as the incumbent orders them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedTransaction = { oid; branch; cwd; repositoryRoot; directory; shadowPath; gitDir; registryRoot; realIndexPath; addedPaths; worktreeRecords };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedTransaction {
    /// Landed commit object name.
    pub landed_oid: String,
    /// Branch the commit landed on, named in diagnostics.
    pub branch: String,
    /// Effective repository directory the Git commands of the transaction ran in.
    pub cwd: PathBuf,
    /// Canonical worktree root, where maintenance and `post-commit` run.
    pub repository_root: PathBuf,
    /// Durable transaction directory under the registry.
    pub directory: PathBuf,
    /// Shadow repository whose object store holds originals that never landed.
    pub shadow: PathBuf,
    /// Real Git directory holding the capture store.
    pub git_dir: PathBuf,
    /// Transaction registry root.
    pub registry_root: PathBuf,
    /// Paths a policy added, with their worktree completion records.
    pub added_paths: Vec<AddedPath>,
    /// Selected corrections whose worktree copies the landing completes.
    pub worktree_records: Vec<AddedPath>,
    /// The configuration read when the commit began, which supplies the maintenance keys when it models them.
    pub config_snapshot: Option<ConfigSnapshot>,
}

/// What:
///  The inputs of one post-landing completion.
/// Why:
///   The context carries the caller's global options for maintenance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { gitPath, globalArgs, landed }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PostLandingRequest<'a> {
    /// Real Git and the caller's global options.
    pub context: &'a GitContext,
    /// The landed commit and its transaction paths.
    pub landed: &'a LandedTransaction,
}

/// What:
///  What post-landing completion did, reported for the caller's diagnostics.
/// Why:
///   An action's ending is reported on standard error by the runner and never changes the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { landedOid, policyResult, committed: true }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostLandingOutcome {
    /// Landed commit object name.
    pub landed_oid: String,
    /// Worktree copies rewritten and kept because they changed concurrently.
    pub worktree: InstallResult,
    /// Landed-capture records pruning removed.
    pub pruned_captures: usize,
    /// Maintenance outcome, or `None` when maintenance could not start and the failure was logged.
    pub maintenance: Option<AutoMaintenanceOutcome>,
}

/// What:
///  A failure of one post-landing step, naming the step that failed.
/// Why:
///   The caller prints the message and exits 1; each variant keeps what the recovery may still need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw error; // from installAddedWorktreeFiles or workspace disposal
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostLandingError {
    /// Worktree copies could not be completed; the shadow and the directory remain for recovery.
    Install(RecoveryError),
    /// The shadow or the transaction directory could not be removed.
    Dispose(RecoveryError),
}

impl Display for PostLandingError {
    /// What:
    ///  The failure's message, without a prefix naming the step.
    /// Why:
    ///   The incumbent prints the thrown error's message alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.error(error.message);
    /// ```
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            PostLandingError::Install(error) | PostLandingError::Dispose(error) => {
                write!(formatter, "{}", error.0)
            }
        };
    }
}

/// What:
///  Complete a landed commit's transaction in the incumbent's order and report what each step did.
/// Why:
///   Each step runs only after the one before it succeeded, except maintenance, whose failure never stops the completion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await installAddedWorktreeFiles(...); await workspace[Symbol.asyncDispose](); await runAutoMaintenance(...);
/// ```
pub fn complete_landed_transaction(
    request: PostLandingRequest,
) -> Result<PostLandingOutcome, PostLandingError> {
    let landed: &LandedTransaction = request.landed;
    debug(
        "completeLandedTransaction",
        format!("completing {} on {}", landed.landed_oid, landed.branch).as_str(),
    );
    let mut records: Vec<AddedPath> = landed.added_paths.clone();
    records.extend(landed.worktree_records.iter().cloned());
    let objects: PathBuf = landed.shadow.join("objects");
    let worktree: InstallResult = install_added_worktree_files(
        request.context,
        landed.cwd.as_path(),
        landed.repository_root.as_path(),
        records.as_slice(),
        Some(objects.as_path()),
    )
    .map_err(PostLandingError::Install)?;
    let pruned_captures: usize = dispose_landed_transaction(LandedTransactionPaths {
        shadow: landed.shadow.as_path(),
        directory: landed.directory.as_path(),
        git_dir: landed.git_dir.as_path(),
        registry_root: landed.registry_root.as_path(),
    })
    .map_err(PostLandingError::Dispose)?;
    let maintenance: Option<AutoMaintenanceOutcome> = match run_auto_maintenance_with(
        request.context,
        landed.repository_root.as_path(),
        landed.config_snapshot.as_ref(),
    ) {
        Ok(outcome) => Some(outcome),
        Err(error) => {
            warn(
                "completeLandedTransaction",
                format!(
                    "automatic maintenance could not run, and the landed commit stands: {error:?}"
                )
                .as_str(),
            );
            None
        }
    };
    return Ok(PostLandingOutcome {
        landed_oid: landed.landed_oid.clone(),
        worktree,
        pruned_captures,
        maintenance,
    });
}

/// Post-landing controls stay out of the release executable.
#[cfg(test)]
#[path = "post_landing_tests.rs"]
mod tests;
