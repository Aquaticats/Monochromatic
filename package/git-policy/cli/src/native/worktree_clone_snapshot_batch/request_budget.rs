//! What:
//!  The largest encoded request input one snapshot-destroy request may carry,
//!  as the caller passes it in.
//! Why:
//!  The design splits the verified set by the largest fitting prefix of the
//!  request's encoded input
//! (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//! step 3),
//! and the value is measured rather than picked:
//!  the plan's request budget is step 22's measurement
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 22).
//!  A named type keeps the unit beside the number and keeps this piece from
//!  shipping a limit of its own.
//!  The design's words are "Budget for encoded inputs and per-name results":
//!  the direct route's reply is sized for both by the binding,
//! which starts a
//!  write's reply at `max(256 KiB, twice the input)`
//!  (`requests/reply.rs`,
//! `initial_size`,
//! after `libzfs_core`'s
//!  `MAX(size * 2, 128 * 1024)`),
//! and a per-name answer is one int32
//!  per submitted name
//! (`module/zfs/dsl_destroy.c:683` to `:686`,
//! OpenZFS `zfs-2.4.4`),
//!  which is smaller than that name's own pair in the input list.
//!  A name list that
//!  fits therefore leaves a reply that can hold every name's failure,
//! so this
//!  budget bounds the direct route's input and its results together.
//!  The root
//!  route's program limits are a separate,
//! unmeasured question the design names
//!  and this route does not take.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const budget = RequestBudget.ofInputBytes(step22Bytes);
//! ```

/// Import the formatting traits the diagnostics implement.
use std::fmt;

/// What:
///  The bytes of packed request input one removal's snapshot-destroy request may carry.
/// Why:
///  The number is measured rather than picked,
///  and it is the only place the removal's own limit is written,
///  so a later measurement or the owner's decision changes one line.
///  The measurement is step 22's guest half,
///  `doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
///  whose one `ZFS_IOC_DESTROY_SNAPS` request took 64 names as 3,652 bytes of encoded input,
///  512 names as 28,740 bytes,
///  and 2,048 names as 114,756 bytes,
///  the largest the guest accepted,
///  at about 56 bytes of encoded input per name.
///  This budget is 2.3 times the 512-name request and under the largest accepted one,
///  which is the proposal those measurements carry.
///  It is not a number the owner has accepted:
///  the record says so,
///  and the value stands here as the removal's own with the owner's veto open.
pub const THE_REMOVALS_REQUEST_BUDGET_BYTES: usize = 65_536;

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
        return RequestBudget {
            input_bytes,
        };
    }

    /// The budget in bytes.
    pub fn input_bytes(&self) -> usize {
        return self.input_bytes;
    }
}

/// How the budget prints to a person.
impl fmt::Display for RequestBudget {
    /// Prints the number with its unit.
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(
            formatter,
            "{} bytes of encoded request input",
            self.input_bytes
        );
    }
}

/// What:
///  The budget one removal's snapshot requests are split by.
/// Why:
///  The removal takes one budget and every reader of the number takes it here,
///  so no caller writes a limit of its own and the constant above is the one line a
///  measurement or the owner's decision changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const budget = theRemovalsRequestBudget();
/// ```
pub fn the_removals_request_budget() -> RequestBudget {
    return RequestBudget::of_input_bytes(THE_REMOVALS_REQUEST_BUDGET_BYTES);
}

/// The controls of the budget.
#[cfg(test)]
#[path = "request_budget_tests.rs"]
mod tests;
