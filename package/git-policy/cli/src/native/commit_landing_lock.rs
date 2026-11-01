//! What:
//!  The per-worktree landing lock a landing attempt takes, and the recovery every acquisition runs first.
//! Why:
//!  A crashed landing must be resolved before another landing moves the ref or the index, so the lock is acquired
//!  and then dead landings are recovered under it (`src/policy-engine/commit-landing-lock.ts`).
//!  An invocation nested under the holder, proven by an inherited landing lease, proceeds under its ancestor's lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using lock = await acquireLandingLock({ gitPath, cwd, registryRoot });
//! ```

use super::capture_invocation::InvocationLayout;
use super::diagnostic_log::debug;
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLock;
use super::recovery_error::RecoveryError;
use super::recovery_landing::RecoveryPlace;
use super::recovery_scan::recover_dead_landings;
use super::replay_reservation::{LandingReservation, ReservationError, acquire_reserved_landing_lock};
use super::transaction_git::GitContext;
use super::transaction_lease::{has_valid_landing_lease, landing_lease_token};
use super::transaction_registry::LANDING_LOCK_NAME;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "acquireLandingLock";

/// What:
///  The landing lock directory of one worktree's registry.
/// Why:
///   The reservation and the lease checks name the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(registryRoot, LANDING_LOCK_NAME)
/// ```
pub fn landing_lock_directory(layout: &InvocationLayout) -> PathBuf {
    return layout.registry_root.join(LANDING_LOCK_NAME);
}

/// What:
///  Whether this invocation lands under an ancestor's landing lock.
/// Why:
///   The ancestor's lock is the one the reservation holder waits for, so a nested landing neither waits nor reserves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hasValidLandingLease({ environment: process.env, lockDirectory })
/// ```
pub fn landing_is_nested(layout: &InvocationLayout, environment: &[(OsString, OsString)]) -> bool {
    return has_valid_landing_lease(environment, landing_lock_directory(layout).as_path());
}

/// What:
///  Take the landing lock and recover dead landings under it, or take the ancestor's view of it.
/// Why:
///   The lock is released again when recovery fails before the caller receives it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = await acquireLandingLock({ gitPath, cwd, registryRoot });
/// ```
pub fn acquire_landing_lock(
    context: &GitContext,
    cwd: &Path,
    layout: &InvocationLayout,
    environment: &[(OsString, OsString)],
) -> Result<OwnerLock, RecoveryError> {
    let directory: PathBuf = landing_lock_directory(layout);
    let lock: OwnerLock = if landing_is_nested(layout, environment) {
        let token: String = landing_lease_token(environment).unwrap_or_default();
        debug(TAG, format!("leaving {} to the ancestor that holds it", directory.display()).as_str());
        OwnerLock::inherited(directory.as_path(), token.as_str())
    } else {
        acquire_live_owner_lock(directory.as_path(), "the landing lock")
            .map_err(|error| { return RecoveryError(error.to_string()); })?
    };
    let place: RecoveryPlace = RecoveryPlace {
        git: context.clone(),
        cwd: cwd.to_path_buf(),
        common_dir: layout.common_dir.clone(),
    };
    // A failure drops `lock`, which releases it before the caller ever holds it.
    recover_dead_landings(&place, layout.registry_root.as_path())?;
    return Ok(lock);
}

/// What:
///  Take the landing lock once no other transaction's reservation stands in the way.
/// Why:
///   The reservation is checked again under the lock, so a reservation granted while this landing waited is honored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lock = await acquireReservedLandingLock({ gitPath, cwd, registryRoot, reservation });
/// ```
pub fn acquire_reserved_lock(
    context: &GitContext,
    cwd: &Path,
    layout: &InvocationLayout,
    environment: &[(OsString, OsString)],
    reservation: &LandingReservation,
) -> Result<OwnerLock, ReservationError> {
    return acquire_reserved_landing_lock(reservation, || {
        return acquire_landing_lock(context, cwd, layout, environment).map_err(ReservationError::Registry);
    });
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_landing_lock_tests.rs"]
mod tests;
