//! What:
//! The verification of the lockfiles an update leaves,
//! the way an install verifies them.
//! Why:
//! `update.rs` holds the transaction;
//! this small judgment was split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-verify.ts
//! export function verifyUpdate(request: UpdateRequest, snapshots: LockSnapshot[]): CheckOutcome
//! ```

/// Imports the outcome type of a policy check.
use crate::checks::CheckOutcome;
/// Imports the snapshot type.
use crate::crates::LockSnapshot;
/// Imports the update request and error and the error constructor.
use crate::update::{UpdateError, UpdateRequest, other};
/// Imports the verification an install runs on the lockfiles it leaves.
use crate::verify::{verify_added_since_base_outcome, verify_locked_entries_outcome};

/// What:
/// `pub(crate) fn verify_update(request, snapshots) -> Result<CheckOutcome, UpdateError>` judges the lockfiles an update leaves.
/// Why:
/// Review finding (3c P1-2):
/// under `trustLockfile` every version the base revision did not hold is judged,
/// and otherwise every
/// locked version is,
/// exactly as an install judges them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifyUpdate(request: UpdateRequest, snapshots: LockSnapshot[]): CheckOutcome
/// ```
pub(crate) fn verify_update(request: &UpdateRequest, snapshots: &[LockSnapshot]) -> Result<CheckOutcome, UpdateError> {
    if request.settings.trust_lockfile {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        let Some(factory) = request.lock_base else {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err(other("trustLockfile is true in rust-catalog.toml, but no base-revision lockfile reader was given, so nothing can be trusted".to_string()));
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let reader = factory().map_err(other)?;
        // Borrow `&*reader`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        return verify_added_since_base_outcome(snapshots, &*reader, request.settings, request.registry, request.now).map_err(other);
    }
    // `.map_err(...)`: converts the callee's error into this function's error type.
    return verify_locked_entries_outcome(request.root, snapshots, request.settings, request.registry, request.now).map_err(other);
}
