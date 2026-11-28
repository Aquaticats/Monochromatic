//! What:
//! Why a route to a delegated control group was not had,
//! in words that name the missing capability,
//! and the two kinds of refusal the run tells apart.
//! Why:
//! Split out of `control_group_route_discovery.rs`,
//! which holds the discovery itself,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group_route_discovery`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_refusal.ts: GroupUnavailable, unavailable, refusesTheCall
//! ```

/// What:
/// `use std::fmt;` imports the standard library's formatting module.
/// It holds `Display` (a type that can be printed with `{}`),
/// `Formatter` (the printer that `Display` writes into),
/// and `Result` (the success-or-failure value of one write).
/// Why:
/// A refusal is printed as the reason the delegated scope is missing,
/// both in the one-line notice of the weaker path,
/// and in the words a stopped call carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // String(value) and template literals
/// ```
use std::fmt;

/// Why no control group could be had,
/// in words that name the missing capability.
/// Two kinds of refusal are told apart by `stops_the_call`:
/// one that means this host has no capability,
/// which the run stands in for with a process group of its own,
/// and one that is a moment no wait of this tool ends,
/// which stops the call instead.
#[derive(Clone, Debug)]
pub(crate) struct GroupUnavailable {
    /// The missing capability and what was tried.
    pub(crate) reason: String,
    /// Whether this refusal stops the call instead of taking the weaker path.
    pub(crate) stops_the_call: bool,
}

/// How the reason reads to a person.
impl fmt::Display for GroupUnavailable {
    /// What:
    /// `fn fmt(&self, formatter)` writes the reason.
    /// Why:
    /// The one-line notice of the weaker path is this text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(formatter, "{}", self.reason);
    }
}

/// What:
/// `fn unavailable(reason: String) -> GroupUnavailable` wraps a reason in the refusal that route discovery returns.
/// Why:
/// A function replaces the closure that `detect_route` called at every refusal.
pub(crate) fn unavailable(reason: String) -> GroupUnavailable {
    return GroupUnavailable {
        reason,
        stops_the_call: false,
    };
}

/// What:
/// `fn refuses_the_call(reason: String) -> GroupUnavailable` wraps a reason in the refusal that stops the call,
/// instead of the one the run stands in for with a process group of its own.
/// Why:
/// The reason it exists for is a route the manager reports beside a job a start could not wait out:
/// a scope planned there is refused while that job stands,
/// and waiting for another unit's stop has no bound this tool sets,
/// so a run that took the weaker path there would go on uncontained behind a stop of unknown length (rule "nothing waits forever")
/// while running the command it was asked to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusesTheCall(reason: string): GroupUnavailable  // the kind that throws instead of falling back
/// ```
pub(crate) fn refuses_the_call(reason: String) -> GroupUnavailable {
    return GroupUnavailable {
        reason,
        stops_the_call: true,
    };
}
