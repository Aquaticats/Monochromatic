//! What:
//!  What a wake came to:
//!  whether the commit service read the one line that told it work waits for it.
//! Why:
//!   The record a wake names is on disk before the line is written,
//!  so nothing of the work is lost by a line nothing read.
//!  What the line changes is when the work runs,
//!  and what a command may tell a person it has handed over:
//!  a wake the service read is work it will look at,
//!  and a wake no byte of which went in is work that waits for another command of the repository.
//!  A command that skipped its own work because its wake answered
//!  `Ok` promised the person a relink nothing had heard of
//!  (the review of 2026-11-13,
//!  "Out-of-scope observation"),
//!  so the answer names the case rather than saying only that the write returned.
//!   The bounds are the ones a write and a wait already have
//!  (`wait_ceilings.rs`,
//!  `REQUEST_WRITE_SILENCE`,
//!  `REQUEST_WRITE_CEILING`)
//!  and the client's own silence bound for the close that confirms the read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Woken = { kind: 'taken' } | { kind: 'refused'; code: FailureCode } | { kind: 'nothing-taken' } | { kind: 'part-taken' } | { kind: 'unwitnessed' }
//! ```

use crate::service_protocol::FailureCode;

/// What:
///  What became of the one line a wake carries.
/// Why:
///   A wake of a few hundred bytes fits one connection's buffers,
///  so the write going in is no evidence that the service read it;
///  the close is the one thing the service does after it has read a work line,
///  and the failures of the write tell the other cases apart.
///  A line the service answered is one it refused:
///  no work line is answered any other way
///  (`service_server_connection_handling.rs`, `answer_request`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // 'taken' | { refused: FailureCode } | 'nothing-taken' | 'part-taken' | 'unwitnessed'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Woken {
    /// The whole line went into the connection and the service closed it without answering,
    /// which is what it does once it has read a work line:
    /// it holds what the line named.
    Taken,
    /// The service read the line and answered that it will not act on it.
    Refused {
        /// What its answer said.
        code: FailureCode,
    },
    /// The service took no byte of the line for the whole silence window of a write:
    /// it stopped reading.
    NothingTaken,
    /// The service took part of the line, and the whole of it did not go in within the ceiling of a write.
    PartTaken,
    /// Every byte of the line went into the connection, and the service closed nothing within the time this command
    /// waits for a service's silence:
    /// nothing witnessed it read the line.
    Unwitnessed,
}

impl Woken {
    /// What:
    ///  Whether the service answered that it holds the work the line named.
    /// Why:
    ///   This is the one answer a command may hand work over on,
    ///  and every other one means the record stays for a later command or for the service's own look.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// woken.holdsTheWork() // kind === 'taken'
    /// ```
    pub fn holds_the_work(&self) -> bool {
        return matches!(self, Self::Taken);
    }

    /// What:
    ///  What became of the wake,
    ///  in the words a report about the record it named uses as its reason.
    /// Why:
    ///  Every command that hands work over says why the work waits when it does,
    ///  and the person reads the state of the wake rather than the name of a constant:
    ///  a service that stopped reading,
    ///  one that took part of the line,
    ///  one that read it and refused it,
    ///  and one that closed nothing are four different next steps.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// woken.whyNothingTookIt() // 'the commit service took no byte of the wake ...'
    /// ```
    pub fn why_nothing_took_it(&self) -> String {
        return match self {
            Self::Taken => String::from("the commit service read the wake"),
            Self::Refused { code } => format!("the commit service read the wake and refused it ({})", refusal_words(*code)),
            Self::NothingTaken => String::from("the commit service took no byte of the wake, so it has stopped reading what is sent to it"),
            Self::PartTaken => String::from("the commit service took part of the wake and not the whole of it within the time a write is given"),
            Self::Unwitnessed => String::from(
                "the commit service did not close the connection after the wake, so nothing witnessed it read the line and no one may say it holds the record",
            ),
        };
    }
}

/// What:
///  What a service's refusal of a work line was about.
/// Why:
///  The service refuses a wake it will not act on for one of two reasons,
///  and they send a person to two different places:
///  another instance serves the repository,
///  or the service could not tell whose repository the line names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusalWords = (code) => code === 'rejected' ? 'it serves another instance' : 'it could not tell whose repository this is'
/// ```
fn refusal_words(code: FailureCode) -> String {
    return match code {
        FailureCode::Rejected => String::from("it serves another instance of this repository"),
        FailureCode::Unavailable => String::from("it could not tell whether this repository is the one it serves"),
        other => String::from(other.word()),
    };
}

/// Controls of the wake's answer stay out of the release executable.
#[cfg(test)]
#[path = "service_client_what_became_of_a_wake_tests.rs"]
mod tests;
