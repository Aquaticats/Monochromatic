//! What:
//!  The words for a commit whose exchange with the user service manager ran out of its time,
//!  before that exchange reached any unit of this worktree,
//!  so no service was asked to start and nothing of the commit was written.
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
//!  What to run is decided by what the answer can be in the state these words are printed in:
//!  `git monochromatic-git-policy-cli service status` tells the person what a running service is doing,
//!  and in the state of a worktree whose service nothing has started
//!  it answers that the service does not answer here and refers the reader back to this message,
//!  so it is named for the one state where its answer helps and not as a blind next step
//!  (measured in a fixture with no service up and in one with a service up,
//! 2026-11-14).
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

use crate::service_messages::{NOTHING_CHANGED, STATUS_COMMAND};

/// What:
///  The words for a commit the user service manager never answered about.
/// Why:
///  The time the exchange was given is the fact the person can act on,
///  so the words name it,
///  say plainly that it says nothing about whether the manager is there,
///  and give the one step that ends it:
///  running the commit again,
///  which is safe because no request of this one was written.
///  The commit was not made and nothing lands later by itself.
///  A service of this worktree that was already running is the one thing the reader can look at,
/// so its state is asked
///  about only for that reader:
///  the status command answers that question and names the unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The user service manager did not answer within the ${seconds} seconds one exchange with it is given, so this commit was not made. ...`
/// ```
pub(crate) fn did_not_answer(waited: std::time::Duration) -> String {
    let seconds: u64 = waited.as_secs().max(1);
    return format!(
        "The user service manager did not answer within the {seconds} seconds one exchange with it is given, so this \
         commit was not made, and nothing of it was written: no request reached the commit service, so there is nothing \
         that could land later by itself. That silence says nothing about whether the manager is there: it may be busy, \
         stopped, or waiting on something else of its own. Run the commit again, which asks the manager again and sets \
         this worktree's service up then. If a service of this worktree was already running before this commit, \
         `{STATUS_COMMAND}` says whether it still answers and names its unit, which is what the manager would be asked \
         about. {NOTHING_CHANGED}"
    );
}
