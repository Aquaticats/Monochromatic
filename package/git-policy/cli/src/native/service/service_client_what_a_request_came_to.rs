//! What:
//!  What a commit request came to, as the wrapper's own side of the service reads it:
//!  the commit is on the branch,
//!  the commit is written and kept behind a reference lock,
//!  the commit must be prepared again from a base the branch moved to,
//!  or the commit was not made at all.
//! Why:
//!  Each of these is a different thing the command does next,
//!  and the landing path and the waiting path name them by this one type,
//!  so neither spells the branches of another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Submitted =
//!   | { kind: 'landed'; commit: string; already: boolean }
//!   | { kind: 'held'; commit: string; words: string }
//!   | { kind: 'prepareAgain'; base: string; landed: LandedCapture[] }
//!   | { kind: 'moved' };
//! ```

use crate::service_protocol::LandedCapture;
/// What a request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submitted {
    /// The commit is on the branch.
    Landed { commit: String, already: bool },
    /// The commit is written and kept,
    /// and a reference lock keeps its branch where it is:
    /// the command exits `0`,
    /// and the next command that finds the lock gone moves the branch.
    Held {
        /// The commit kept on the batch's side reference.
        commit: String,
        /// What the command says about it.
        words: String,
    },
    /// A named path changed on the branch:
    /// prepare again from this base and submit again with the same identity.
    ///  `landed` tells when the bytes of the commits the service landed since were captured,
    /// for the ones it knows.
    PrepareAgain {
        base: String,
        landed: Vec<LandedCapture>,
    },
    /// The branch is no longer at the base of a commit that cannot be moved to another tip (an amend,
    /// or a commit Git built).
    ///  The caller decides whether to build it again on the new tip or to stop.
    Moved,
}
