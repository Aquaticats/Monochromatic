//! What:
//!  Why a verified snapshot set is not submitted at all:
//! a name larger than the
//!  whole budget,
//! a name the set holds twice,
//! a request the binding refuses to
//!  pack,
//! and a request that would carry the `defer` key.
//! Why:
//!  Every one of these is decided before the kernel sees a request,
//! so none can
//!  be mistaken for a per-name answer,
//! and each names the input to correct and
//!  the way forward instead of inviting the same call again.
//!  A retry of a snapshot destroy is never safe:
//! the second call may meet a set the
//!  first one already changed
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery", step 3).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BatchRefusal = { kind: 'over-budget' | 'repeated' | 'unbuildable' | 'would-defer'; ... };
//! ```

/// The request failure a refused pack reports.
use monochromatic_zfs_request_to_openzfs_kernel_device::Error;
/// The formatting traits the diagnostics implement.
use std::fmt;

/// What:
///  Why one verified set is not submitted.
/// Why:
///  The caller keeps the removal record and the snapshot set,
///  and each variant says
///  which of the two it changes before another attempt.
#[derive(Debug)]
pub enum BatchRefusal {
    /// One name does not fit the budget even as the only name of a request.
    OneNameOverTheBudget {
        /// The snapshot name that does not fit.
        name: String,
        /// The bytes its own request would carry.
        request_bytes: usize,
        /// The caller's budget.
        budget_bytes: usize,
    },
    /// The verified set names one snapshot more than once.
    RepeatedName {
        /// The name that appears twice.
        name: String,
    },
    /// The binding could not pack the request from the pool and the names.
    Unbuildable {
        /// The pool the request names.
        pool: String,
        /// What the binding refused.
        source: Error,
    },
    /// The request carries a `defer` key,
    /// so it would mark what it cannot destroy.
    WouldDefer {
        /// The pool the request names.
        pool: String,
    },
}

/// How the refusal prints to a person.
impl fmt::Display for BatchRefusal {
    /// Prints the input to correct and the effect of refusing.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            BatchRefusal::OneNameOverTheBudget {
                name,
                request_bytes,
                budget_bytes,
            } => write!(
                formatter,
                "`{name}` needs {request_bytes} bytes of encoded request on its own, and the budget is {budget_bytes} bytes, so no request can carry it. Raise the budget, or leave this snapshot out of the set this removal destroys. No snapshot was destroyed."
            ),
            BatchRefusal::RepeatedName { name } => write!(
                formatter,
                "`{name}` appears more than once in the verified set, so one request would submit that name twice and no per-name outcome could be attributed to one object. Correct the set from a fresh inventory. No snapshot was destroyed."
            ),
            BatchRefusal::Unbuildable { pool, source } => write!(
                formatter,
                "the snapshot-destroy request for '{pool}' could not be built ({source}), so it was not sent. Correct the name the binding refused. No snapshot was destroyed."
            ),
            BatchRefusal::WouldDefer { pool } => write!(
                formatter,
                "the snapshot-destroy request for '{pool}' carries a `defer` key, so it would mark every snapshot it cannot destroy for deferred destruction instead of naming it. The request was not sent. Restore the binding's defer-free request before any removal runs. No snapshot was destroyed."
            ),
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for BatchRefusal {
    /// The binding's failure,
    /// when the request could not be packed.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return match self {
            BatchRefusal::Unbuildable { source, .. } => Some(source),
            BatchRefusal::OneNameOverTheBudget { .. }
            | BatchRefusal::RepeatedName { .. }
            | BatchRefusal::WouldDefer { .. } => None,
        };
    }
}

/// The controls of the refusal's words.
#[cfg(test)]
#[path = "request_refusal_tests.rs"]
mod tests;
