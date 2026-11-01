//! What:
//!  The disposable, recoverable workspace of one commit transaction:
//!  a published transaction directory with its owner record, the shadow repository path, and disposal that leaves
//!  everything in place once a landing record handed it to recovery.
//! Why:
//!  The incumbent's `createCommitTransactionWorkspace` (`src/policy-engine/commit-transaction-workspace.ts`)
//!  publishes the directory before any Git mutation, so recovery can skip a live owner and retire a dead one,
//!  and disposal removes the shadow before the directory so no shadow outlives the journal that names it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using workspace = await createCommitTransactionWorkspace({ capture });
//! ```

use super::capture_invocation::InvocationLayout;
use super::diagnostic_log::{debug, warn};
use super::landed_disposal::{LandedTransactionPaths, dispose_landed_transaction};
use super::landing_attempt::LandingWorkspace;
use super::process_identity::ProcessIdentityError;
use super::random_id::random_uuid;
use super::recovery_error::RecoveryError;
use super::shadow_git::shadow_repository_path;
use super::transaction_owner::{TransactionOwner, current_transaction_owner, encode_transaction_owner};
use super::transaction_registry::{ensure_transaction_root, publish_transaction_directory};
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "commitWorkspace";

/// What:
///  Why the workspace could not be published.
/// Why:
///   The transaction fails closed before any Git mutation, naming the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(message);
/// ```
#[derive(Debug)]
pub enum WorkspaceError {
    /// The registry root or the directory could not be created or published.
    Registry(RecoveryError),
    /// The process identity could not be read.
    Identity(ProcessIdentityError),
    /// The random transaction ID could not be generated.
    Random(String),
}

impl std::fmt::Display for WorkspaceError {
    /// Writes the wording of the failing step.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Registry(error) => formatter.write_str(error.0.as_str()),
            Self::Identity(error) => write!(formatter, "{error:?}"),
            Self::Random(text) => formatter.write_str(text.as_str()),
        };
    }
}

/// What:
///  The owned state of one transaction.
/// Why:
///   The landing reads the `LandingWorkspace` part; the rest names the places disposal and preparation use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionWorkspace = { transactionId; directory; commitIndexPath; capturedIndexPath; shadowPath; objectDirectory; hooksDirectory };
/// ```
#[derive(Debug)]
pub struct CommitWorkspace {
    /// The part the landing attempt reads and marks.
    pub landing: LandingWorkspace,
    /// Shadow object store receiving every object the transaction writes before landing migrates them.
    pub object_directory: PathBuf,
    /// Hook dispatcher directory.
    pub hooks_directory: PathBuf,
    /// Git directory of the owning worktree, holding the capture store.
    git_dir: PathBuf,
    /// Transaction registry root.
    registry_root: PathBuf,
    /// Whether disposal already ran.
    disposed: bool,
}

/// What:
///  Publish a fresh transaction directory owned by this process.
/// Why:
///   The owner record is published with the directory, so recovery never sees an unowned directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workspace = await createCommitTransactionWorkspace({ capture });
/// ```
pub fn create_commit_workspace(layout: &InvocationLayout) -> Result<CommitWorkspace, WorkspaceError> {
    ensure_transaction_root(layout.registry_root.as_path()).map_err(WorkspaceError::Registry)?;
    let transaction_id: String = random_uuid().map_err(|error| {
        return WorkspaceError::Random(format!("a transaction ID could not be generated: {error:?}"));
    })?;
    let owner: TransactionOwner =
        current_transaction_owner(transaction_id.as_str(), layout.invoked_at.as_str()).map_err(WorkspaceError::Identity)?;
    let directory: PathBuf = publish_transaction_directory(
        layout.registry_root.as_path(),
        transaction_id.as_str(),
        encode_transaction_owner(&owner).as_bytes(),
    )
    .map_err(WorkspaceError::Registry)?;
    let shadow: PathBuf = shadow_repository_path(layout.common_dir.as_path(), transaction_id.as_str());
    debug(TAG, format!("published transaction {transaction_id}").as_str());
    return Ok(CommitWorkspace {
        landing: LandingWorkspace {
            transaction_id,
            commit_index: directory.join("commit.index"),
            captured_index: directory.join("captured.index"),
            shadow: shadow.clone(),
            directory: directory.clone(),
            preserved: false,
        },
        object_directory: shadow.join("objects"),
        hooks_directory: directory.join("hooks"),
        git_dir: layout.git_dir.clone(),
        registry_root: layout.registry_root.clone(),
        disposed: false,
    });
}

impl CommitWorkspace {
    /// What:
    ///  Mark durable completion, so disposal removes the recovery artifacts.
    /// Why:
    ///   After the landed commit's cleanup the landing's `preserved` mark no longer protects anything.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// workspace.finishTransaction();
    /// ```
    pub fn finish_transaction(&mut self) {
        self.landing.preserved = false;
    }

    /// What:
    ///  Remove the shadow repository, then the directory, then prune landed captures, unless recovery owns them.
    /// Why:
    ///   Later calls do nothing, and a failed removal is reported while the directory stays for recovery.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await workspace[Symbol.asyncDispose]();
    /// ```
    pub fn dispose(&mut self) -> Result<(), RecoveryError> {
        if self.disposed {
            return Ok(());
        }
        self.disposed = true;
        if self.landing.preserved {
            debug(
                TAG,
                format!("leaving transaction {} for recovery", self.landing.transaction_id).as_str(),
            );
            return Ok(());
        }
        dispose_landed_transaction(LandedTransactionPaths {
            shadow: self.landing.shadow.as_path(),
            directory: self.landing.directory.as_path(),
            git_dir: self.git_dir.as_path(),
            registry_root: self.registry_root.as_path(),
        })?;
        return Ok(());
    }

    /// What:
    ///  The Git directory of the owning worktree.
    /// Why:
    ///   Post-landing completion prunes the capture store there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// capture.gitDir
    /// ```
    pub fn git_dir(&self) -> &std::path::Path {
        return self.git_dir.as_path();
    }

    /// What:
    ///  The transaction registry root.
    /// Why:
    ///   Post-landing completion and the reservation read it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// capture.registryRoot
    /// ```
    pub fn registry_root(&self) -> &std::path::Path {
        return self.registry_root.as_path();
    }

    /// What:
    ///  Hand the disposal to a later step, which has already removed what the workspace owns.
    /// Why:
    ///   The post-landing completion removes the shadow and the directory itself, so the drop must not repeat it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// workspace.finishTransaction(); await workspace[Symbol.asyncDispose]();
    /// ```
    pub fn mark_disposed(&mut self) {
        self.disposed = true;
    }
}

impl Drop for CommitWorkspace {
    /// What:
    ///  Dispose of the workspace when the transaction ends by any route.
    /// Why:
    ///   A failure before the landing must not leave a shadow repository or a directory behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await using workspace = ...;
    /// ```
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            warn(TAG, format!("disposing of the workspace failed: {}", error.0).as_str());
        }
    }
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_workspace_tests.rs"]
mod tests;
