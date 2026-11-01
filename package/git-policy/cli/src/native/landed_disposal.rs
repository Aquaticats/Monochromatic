//! What:
//!  Disposal of a landed transaction: its shadow repository, then its transaction directory,
//!  then the landed-capture records no published transaction still needs.
//! Why:
//!  The incumbent's `commit-transaction-workspace.ts` runs these steps in this order.
//!  A shadow never outlives the directory that names it, and pruning runs last, so the last
//!  transaction to finish sees no other transaction pinning a landed record.
//!  A failed removal stops the sequence, so recovery still finds the directory and its records.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeShadowRepository(shadowPath);
//! await removeTransactionDirectory(directory);
//! await pruneLandedCaptures({ gitDir, registryRoot });
//! ```

use super::capture_records::prune_landed_captures;
use super::recovery_error::RecoveryError;
use super::shadow_git::remove_shadow_repository;
use super::transaction_registry::remove_transaction_directory;
use std::path::Path;

/// What:
///  The paths of one landed transaction that disposal removes or reads.
/// Why:
///   Named fields keep the four paths apart at the call site, and the struct is `Copy` so one value serves every step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedTransactionPaths = { shadowPath: string; directory: string; gitDir: string; registryRoot: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LandedTransactionPaths<'a> {
    /// Shadow repository derived from the transaction ID.
    pub shadow: &'a Path,
    /// Durable transaction directory under the registry.
    pub directory: &'a Path,
    /// Real Git directory holding the capture store.
    pub git_dir: &'a Path,
    /// Transaction registry root whose published entries pin landed records.
    pub registry_root: &'a Path,
}

/// What:
///  Remove the shadow repository and the transaction directory, then prune landed-capture records.
/// Why:
///   Each removal is idempotent, so a recovery that finds the work half done can run it again.
///  The returned count is the number of landed-capture records pruning removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeShadowRepository(shadowPath); await removeTransactionDirectory(directory); return pruneLandedCaptures({ gitDir, registryRoot });
/// ```
pub fn dispose_landed_transaction(paths: LandedTransactionPaths) -> Result<usize, RecoveryError> {
    remove_shadow_repository(paths.shadow)?;
    remove_transaction_directory(paths.directory)?;
    return Ok(prune_landed_captures(paths.git_dir, paths.registry_root));
}

/// Disposal controls stay out of the release executable.
#[cfg(test)]
#[path = "landed_disposal_tests.rs"]
mod tests;
