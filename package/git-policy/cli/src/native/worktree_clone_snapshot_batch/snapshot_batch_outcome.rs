//! What:
//!  What one snapshot-destroy request did, kept whole:
//!  every submitted name's
//!  outcome in request order,
//! the kernel's per-name list exactly as it answered,
//!  whether that list is complete,
//! and the request's whole error number.
//! Why:
//!  The per-name list is the kernel's own evidence and a resume reads the
//!  per-name outcomes,
//! so both travel together and a caller never reconstructs one
//!  from the other
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery", step 3;
//!  `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The states").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BatchOutcome = {
//!   pool: string;
//!   names: { name: string; outcome: NameOutcome }[];
//!   failures: { name: string; errno: number }[];
//!   failuresComplete: boolean;
//!   errno?: number;
//! };
//! ```

/// The kernel's per-name element type.
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::ElementError;

/// One submitted name with its outcome,
/// and the outcome vocabulary its accessor returns.
use super::snapshot_name_outcome::{NameOutcome, SubmittedName};

/// What one request did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOutcome {
    /// The pool the request named.
    pool: String,
    /// Every submitted name with its outcome,
    /// in the order the request carried them.
    names: Vec<SubmittedName>,
    /// The kernel's per-name failures,
    /// exactly as its reply listed them.
    failures: Vec<ElementError>,
    /// Whether [`BatchOutcome::failures`] is the kernel's complete list.
    failures_complete: bool,
    /// The request's whole error number,
    /// when the kernel answered a request with a failure.
    errno: Option<i32>,
}

/// The methods of `BatchOutcome`.
impl BatchOutcome {
    /// What:
    ///  Build one outcome from its parts.
    /// Why:
    ///  The reader in `snapshot_batch_answer.rs` is the only caller,
    ///  and the invariant that `names` holds one outcome per submitted name is
    ///  established there;
    /// this stays a data value a caller reads.
    pub(crate) fn new(
        pool: String,
        names: Vec<SubmittedName>,
        failures: Vec<ElementError>,
        failures_complete: bool,
        errno: Option<i32>,
    ) -> BatchOutcome {
        return BatchOutcome {
            pool,
            names,
            failures,
            failures_complete,
            errno,
        };
    }

    /// The pool the request named.
    pub fn pool(&self) -> &str {
        return &self.pool;
    }

    /// Every submitted name with its outcome,
    /// in request order.
    pub fn names(&self) -> &[SubmittedName] {
        return &self.names;
    }

    /// The kernel's per-name failures,
    /// empty when it supplied none.
    pub fn failures(&self) -> &[ElementError] {
        return &self.failures;
    }

    /// Whether the per-name list is the kernel's complete one.
    pub fn failures_complete(&self) -> bool {
        return self.failures_complete;
    }

    /// The request's whole error number,
    /// when the kernel answered with one.
    pub fn errno(&self) -> Option<i32> {
        return self.errno;
    }

    /// What the request did to [`name`](SubmittedName::name),
    /// which must be one it
    ///  submitted.
    pub fn outcome_of(&self, name: &str) -> Option<&NameOutcome> {
        return self
            .names
            .iter()
            .find(|submitted| {
                return submitted.name == name;
            })
            .map(|submitted| {
                return &submitted.outcome;
            });
    }
}
