//! What:
//!  Why a request a commit sent is given up,
//!  and the failure that says what became of the commit.
//! Why:
//!   What is withdrawn decides what the person is told:
//!  a commit that reached the answer ceiling waited too long,
//!  and one the person interrupted is not made at all,
//!  so the two ends read differently
//!  (`service_messages.rs`).
//!  The client of the commit service keeps the wait,
//! the send and the answer
//!  (`service_client.rs`),
//!  and this file keeps what those answers are worded with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WhyWithdrawn = { kind: 'ceiling', minutes: number } | { kind: 'interrupted', signal: number }
//! ```

use crate::service_messages::ClientFailure;

/// Why a request is given up.
pub(crate) enum WhyWithdrawn {
    /// The wait passed the answer ceiling.
    Ceiling {
        /// The ceiling in whole minutes,
        /// as the person reads it.
        minutes: u64,
    },
    /// The person stopped the command with a signal.
    Interrupted {
        /// The signal the person sent.
        signal: i32,
        /// The process that sent it,
        /// where the kernel named one.
        sender: Option<i32>,
    },
}

impl WhyWithdrawn {
    /// What:
    ///  What the command ends with when the service confirmed that the request will never land.
    /// Why:
    ///  A commit that was given up is not made,
    ///  and the words name which of the two gave it up.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// why.itWillNotLand(blocker) // stillWithService(minutes) | interrupted(signal)
    /// ```
    pub(crate) fn it_will_not_land(&self, blocker: Option<String>) -> ClientFailure {
        return match self {
            Self::Ceiling { minutes } => ClientFailure::StillWithService { minutes: *minutes, blocker },
            Self::Interrupted { signal, sender } => ClientFailure::Interrupted { signal: *signal, sender: *sender, blocker },
        };
    }

    /// What:
    ///  What the command ends with when the service did not confirm the withdrawal,
    ///  so whether the commit was made is not known.
    /// Why:
    ///  Nothing was learned about the request,
    ///  and a commit that may be on the branch is never reported as one that was not made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// why.notKnown(blocker) // withdrawalUnconfirmed | interruptedNotKnown(signal)
    /// ```
    pub(crate) fn not_known(&self, blocker: Option<String>) -> ClientFailure {
        return match self {
            Self::Ceiling { .. } => ClientFailure::WithdrawalUnconfirmed { blocker },
            Self::Interrupted { signal, sender } => ClientFailure::InterruptedNotKnown { signal: *signal, sender: *sender, blocker },
        };
    }
}
