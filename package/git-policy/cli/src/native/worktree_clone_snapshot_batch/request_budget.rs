//! What:
//!  The largest encoded request input one snapshot-destroy request may carry,
//!  as the caller passes it in.
//! Why:
//!  The design splits the verified set by the largest fitting prefix of the
//!  request's encoded input
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery", step 3),
//! and the value is measured rather than picked:
//!  the plan's request budget is step 22's measurement
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 22).
//!  A named type keeps the unit beside the number and keeps this piece from
//!  shipping a limit of its own.
//!  The design's words are "Budget for encoded inputs and per-name results":
//!  the direct route's reply is sized for both by the binding,
//! which starts a
//!  write's reply at `max(256 KiB, twice the input)`
//!  (`requests/reply.rs`, `initial_size`, after `libzfs_core`'s
//!  `MAX(size * 2, 128 * 1024)`),
//! and a per-name answer is one int32
//!  per submitted name
//! (`module/zfs/dsl_destroy.c:683` to `:686`, OpenZFS `zfs-2.4.4`),
//!  which is smaller than that name's own pair in the input list.
//!  A name list that
//!  fits therefore leaves a reply that can hold every name's failure,
//! so this
//!  budget bounds the direct route's input and its results together.
//!  The root
//!  route's program limits are a separate, unmeasured question the design names
//!  and this route does not take.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const budget = RequestBudget.ofInputBytes(step22Bytes);
//! ```

/// Import the formatting traits the diagnostics implement.
use std::fmt;

/// What:
///  The bytes of packed request input one `ZFS_IOC_DESTROY_SNAPS` request may
///  carry.
/// Why:
///  The comparison is against `Request::input_bytes().len()`,
///  which is the exact list the
///  kernel receives,
/// so alignment,
/// the list header and every pair count and nothing is
///  estimated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RequestBudget {
    /// The largest encoded input list a request may carry.
    input_bytes: usize,
}

/// The methods of `RequestBudget`.
impl RequestBudget {
    /// A budget of `input_bytes` bytes of encoded request input.
    pub fn of_input_bytes(input_bytes: usize) -> RequestBudget {
        return RequestBudget { input_bytes };
    }

    /// The budget in bytes.
    pub fn input_bytes(&self) -> usize {
        return self.input_bytes;
    }
}

/// How the budget prints to a person.
impl fmt::Display for RequestBudget {
    /// Prints the number with its unit.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{} bytes of encoded request input", self.input_bytes);
    }
}

/// The controls of the budget.
#[cfg(test)]
#[path = "request_budget_tests.rs"]
mod tests;
