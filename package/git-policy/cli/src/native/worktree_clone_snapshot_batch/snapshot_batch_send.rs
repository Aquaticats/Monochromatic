//! What:
//!  One planned chunk sent as its own `ZFS_IOC_DESTROY_SNAPS` request,
//! through the
//!  wrapper's existing `/dev/zfs` layer,
//! with the empty-set skip and the `defer`
//!  guard standing between a caller and the kernel.
//! Why:
//!  The design skips the request for an empty set,
//! and a deferred destroy would
//!  mark what it cannot destroy instead of naming it,
//! so both are refused here
//!  rather than at the caller's discretion
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery", step 3).
//!  The device seam is the binding's own `destroy_snaps`,
//! so the request that is sent
//!  is built by the constructor whose bytes the guard and the plan read,
//! and no
//!  second send path exists.
//!  A request is never replayed:
//! a refusal is a value the caller
//!  keeps,
//! and a failure that cannot be attributed to names is `unknown` and stops
//!  the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = sendRequest(device, pool, chunk);
//! if (result.kind === 'skipped') return; // the empty-set skip made no call
//! result.outcome.names(); // per-name outcomes, kept by the caller
//! ```

/// The binding's live failure,
/// reply list and open handle.
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    NvList,
    Request,
    ZfsDevice,
};

/// The refusal a set this route will not submit reports.
use super::request_refusal::BatchRefusal;
/// The per-name outcome of one answer.
use super::snapshot_batch_answer::outcome_of_answer;
/// One request's outcome as a value.
use super::snapshot_batch_outcome::BatchOutcome;
/// The one request builder,
/// its encoded bytes and its `defer` absence.
use super::snapshot_destroy_request::{
    carries_no_defer,
    destroy_request,
};

/// What:
///  The one device call this route makes,
/// as the binding offers it.
/// Why:
///  A test needs to answer without a pool,
/// and the production answer is the binding's
///  own method,
/// so the seam is this trait and its one implementation and no
///  cmdlet-shaped indirection.
pub trait SnapshotDestroyDevice {
    /// `ZFS_IOC_DESTROY_SNAPS` of `names` in `pool`,
    /// with no `defer` key.
    ///
    /// # Throws
    ///
    /// What the binding's own `destroy_snaps` throws.
    fn destroy_snaps(
        &self,
        pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError>;
}

/// The open device is the production answer.
impl SnapshotDestroyDevice for ZfsDevice {
    /// Sends the binding's own request,
    /// exactly as [`destroy_request`] builds it.
    fn destroy_snaps(
        &self,
        pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        return ZfsDevice::destroy_snaps(self, pool, names);
    }
}

/// What one call to [`send_request`] did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestResult {
    /// The set was empty,
    /// so no request was made and the device was not called.
    Skipped,
    /// One request was made,
    /// and this is what it did to every name it submitted.
    Answered(BatchOutcome),
}

/// What:
///  Submit `names` to the device as one request,
/// and read its answer per name.
/// Why:
///  The empty set is skipped here as well as in the plan,
/// because the skip is
///  the route's rule and not a caller's choice,
/// and a request that would carry
///  `defer` is refused before the device sees it.
///  The bytes the guard reads are the
///  constructor's own,
/// which is the constructor the device's method calls.
///
/// # Throws
///
/// [`BatchRefusal::Unbuildable`] when the binding refuses a name,
/// and
/// [`BatchRefusal::WouldDefer`] when the built request carries a `defer` key.
pub fn send_request(
    device: &dyn SnapshotDestroyDevice,
    pool: &str,
    names: &[String],
) -> Result<RequestResult, BatchRefusal> {
    if names.is_empty() {
        return Ok(RequestResult::Skipped);
    }
    let request: Request = destroy_request(pool, names)?;
    if !carries_no_defer(&request) {
        return Err(BatchRefusal::WouldDefer {
            pool: pool.to_string(),
        });
    }
    let borrowed: Vec<&str> = names
        .iter()
        .map(|name| {
            return name.as_str();
        })
        .collect();
    let answer: Result<Option<NvList>, DeviceError> =
        device.destroy_snaps(pool, borrowed.as_slice());
    return Ok(RequestResult::Answered(outcome_of_answer(
        pool, names, answer,
    )));
}

/// The controls of the skip,
/// the guard and the per-name answer.
#[cfg(test)]
#[path = "snapshot_batch_send_tests.rs"]
mod tests;
