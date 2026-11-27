//! What:
//!  When the caller of one drain of a work directory must look at it again.
//! Why:
//!  A look owes a moment to come back for two reasons:
//! the housekeeping it left for a later look,
//! and the end of the earliest claim still held in the directory,
//! which its own words promise is tried again when the lease runs out
//! (`service_work_drain_pending_claims.rs`).
//! Both are one answer,
//! and the read of the claims it stands on has its own failure to answer for,
//! so they are one step of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await lookAgainAwaiting(workDir, lease, deferred) // Duration | undefined
//! ```

use crate::claim_time::ClaimTime;
use crate::housekeeping_of_one_round::LOOK_AGAIN_AFTER_DEFERRING;
use crate::service_work_storage_awaited::next_claim_expiry_awaiting;
use std::path::Path;
use std::time::Duration;

/// What:
///  What the caller must look at this directory again for:
///  the earliest of the housekeeping this look left for a later one and the end of the earliest claim still held here,
/// its own or another instance's.
/// Why:
///  A claim this look kept (a retryable answer,
/// a settling step that could not run) has nothing else to look at it again:
///  the service's own words promise it is tried again when its claim runs out,
/// so the look that kept it owes that moment
///  to the one loop that will come back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookAgain = async (workDir, lease, deferred) => earliest(deferred, claimExpiry(workDir, lease))
/// ```
pub(super) async fn look_again_awaiting(
    work_dir: &Path,
    lease: Duration,
    look_again: Option<Duration>,
) -> Option<Duration> {
    let claim_wait: Option<Duration> =
        match next_claim_expiry_awaiting(work_dir, &ClaimTime::read(), lease).await {
            Ok(wait) => wait.map(no_sooner_than_a_look_again),
            // A look that could not read the claims proves nothing about them, so this one asks to be looked at again after its own
            //  pause rather than answering nothing: a claim it may still hold is owed a wake, and the refusal is already said.
            Err(_unread) => Some(LOOK_AGAIN_AFTER_DEFERRING),
        };
    return match (look_again, claim_wait) {
        (Some(deferred), Some(claim)) => Some(deferred.min(claim)),
        (deferred, claim) => deferred.or(claim),
    };
}

/// What:
///  A wait that has already run out,
/// as the pause a look that left housekeeping for a later look takes.
/// Why:
///   A claim already past its lease that the taking back could not move answers a wait of no time at all,
/// and a look that came back at once would spin the service on the same claim.
/// The pause is the one such a look already waits,
/// so nothing is looked at later than that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noSoonerThanALookAgain = (left) => left.isZero() ? LOOK_AGAIN_AFTER_DEFERRING : left;
/// ```
fn no_sooner_than_a_look_again(left: Duration) -> Duration {
    return if left.is_zero() {
        LOOK_AGAIN_AFTER_DEFERRING
    } else {
        left
    };
}
