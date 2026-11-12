//! What:
//!  The words for a commit whose exchange with the user service manager ran out of its time,
//!  so the service was never asked to start.
//! Why:
//!  A bus that answered nothing in one exchange says nothing about whether a manager is there
//!  (`src/native/service/systemd_dbus_wire_format.rs`,
//! `CallError::TimedOut`),
//!  and the words this file replaces told the person that their computer cannot start the commit service on demand,
//!  with the remedy for an account that has no user services at all.
//!  Three of the seven stopped records of 2026-11-12 spent twenty seconds in exactly this exchange,
//!  so the person was sent after a manager that was there and busy,
//!  and the record said `no-activation`,
//! which sends its reader after the same thing.
//!  The other failures of a commit through the service are worded in
//!  `service_messages.rs`,
//!  which asks this file for this one,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAManagerThatDidNotAnswer.ts
//! export const didNotAnswer = (waited: Duration): string => ...;
//! ```

use crate::service_messages::NOTHING_CHANGED;

/// The command that says what this worktree's service is doing,
/// which is the one thing a person can look at while a manager is answering nothing.
const STATUS_COMMAND: &str = "git monochromatic-git-policy-cli service status";

/// What:
///  The words for a commit the user service manager never answered about.
/// Why:
///  The time the exchange was given is the fact the person can act on,
///  so the words name it,
///  say plainly that it says nothing about whether the manager is there,
///  and give the remedies that fit both:
///  look at what the service is doing,
/// and run the commit again.
///  The commit was not made and nothing lands later by itself,
///  because no request was ever written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The user service manager did not answer within the ${seconds} seconds one exchange with it is given, so this commit was not made. ...`
/// ```
pub(crate) fn did_not_answer(waited: std::time::Duration) -> String {
    let seconds: u64 = waited.as_secs().max(1);
    return format!(
        "The user service manager did not answer within the {seconds} seconds one exchange with it is given, so this \
         commit was not made. That says nothing about whether the manager is there: it may be busy, stopped, or waiting \
         on something else of its own. Run `{STATUS_COMMAND}` to see what the service of this worktree is doing, and run \
         the commit again. {NOTHING_CHANGED}"
    );
}
