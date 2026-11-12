//! What:
//!  The notices a commit prints while it waits for the commit service:
//!  where it stands,
//!  what the wait is waiting for and on whom,
//!  that the service is asked to restart,
//!  and that the commit is built again because its branch moved,
//!  and the sentence about what the service last said it waited for.
//! Why:
//!  Every one of them is a wait the person would otherwise not see,
//! and the owner's rules are that nothing waits without saying so and
//!  that every wait ends.
//! They live beside the file that renders every failure rather than inside it,
//! because that file is at its line
//!  bound,
//! and a notice and a failure are read at different moments:
//! a notice says the command is going on,
//! and a failure
//!  says it stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.stderr.write(unplacedNotice(elapsed));
//! ```

use std::time::Duration;

/// What:
///  The sentence about what the service last said it waited for,
///  and nothing when it said nothing.
/// Why:
///  The notices the service sends end in a full stop,
///  and a second one after a quoted notice reads as a mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitingFor(blocker) // ' It was waiting for: <text>' | ''
/// ```
pub(crate) fn waiting_for(blocker: Option<&str>) -> String {
    return match blocker {
        Some(text) => format!(" It was waiting for: {}", ending_in_a_full_stop(text.trim())),
        None => String::new(),
    };
}

/// What:
///  `text` with one full stop at its end,
///  its own or the one this adds.
/// Why:
///  A notice the service sent already ends in a full stop,
///  and a second one after a quoted notice reads as a mistake,
///  while a quote with none would run into the sentence that follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.endsWith('.') ? text : text + '.'
/// ```
fn ending_in_a_full_stop(text: &str) -> String {
    return if text.ends_with('.') { text.to_string() } else { format!("{text}.") };
}

/// What:
///  The sentence about what the service last said while it kept the request,
///  and nothing when it said nothing.
/// Why:
///  The same reading as `waiting_for`,
/// for the words the service said on the connection it was asked on,
/// and quoted as the service sent them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it_had_said(blocker)
/// ```
pub(crate) fn it_had_said(blocker: Option<&str>) -> String {
    // The notice is quoted as the service sent it: this sentence ends with no full stop of its own, so the quoted one stands.
    return match blocker {
        Some(text) => format!(" The service had said: {}", text.trim()),
        None => String::new(),
    };
}

/// Printed once when the service takes longer than the grace to answer.
pub const STARTING_NOTICE: &str = "Waiting for the commit service of this worktree...";

/// What:
///  The line that says what a wait for the service is waiting for,
/// on whom,
/// and how long it has waited.
/// Why:
///  A wait that says nothing is a wait a person cannot tell from a hang:
/// a commit with a service that never answered printed
///  nothing for ten seconds and then named a cause that was not true,
/// and an add printed nothing for 613 s.
/// The line names the address the service is asked at and the state the service manager reports,
/// which is
///  the one fact that tells a service that is starting from one that is stopped inside its own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startingNotice('/run/user/1000/monochromatic-git-policy-cli/abc.sock', 'the service manager says monochromatic-git-policy-cli@abc.service is active');
/// ```
pub fn starting_notice(address: &str, who: &str) -> String {
    return format!("monochromatic-git-policy-cli: {STARTING_NOTICE} nothing has answered at {address} yet ({who}); this wait ends as soon as it answers.");
}

/// What:
///  The line that says a running service did not answer and is being restarted,
/// and why that is the repair.
/// Why:
///   The command ends the service,
/// which is not a thing to do without saying so:
/// a unit the service manager reports as running that
///  answers nothing is one whose program stopped inside its own process,
/// and the manager starts a unit it already
///  believes runs for no connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restartingNotice('monochromatic-git-policy-cli@abc.service', 3)
/// ```
pub fn restarting_notice(unit: &str, waited: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit service of this worktree has answered nothing for {} seconds, so it is stopped inside its own process; \
         it is asked to restart ({unit}), and this commit goes on once it answers.",
        waited.as_secs()
    );
}

/// Printed each time the service goes silent while a commit waits,
/// before the same request is sent to it again.
pub const SILENT_NOTICE: &str = "The commit service stopped answering; asking it again with the same commit, which cannot land twice.";

/// Printed once when the service is being replaced by a newer one.
pub const UPDATING_NOTICE: &str = "Updating the commit service...";

/// What:
///  The line that says the service of another release has not left within the stated bound,
/// that it is ended once,
/// and that the commit goes on.
/// Why:
///   The wait for a service of another release is the commit's,
/// not a start's:
/// a service that finishes a batch,
/// or a push it planned to try again,
/// can outlast the bound,
/// and the commit that
///  ended there left the person's change uncommitted.
/// A service that is still there at the bound is ended once,
/// which lets the manager start the program this
///  release installs,
/// and the line says so once because ending a service is not a thing to do without saying it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replacingNotice('monochromatic-git-policy-cli@abc.service', 10)
/// ```
pub fn replacing_notice(unit: &str, waited: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit service of this worktree was started from another release and has not stopped after {} seconds, so it is ended and started again from the program this release installs ({unit}); this commit goes on once it answers.",
        waited.as_secs()
    );
}

/// What:
///  The notice for a commit that waits behind others.
/// Why:
///   The position is the one fact the person needs:
/// how many commits go first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positionNotice(number, of)
/// ```
pub fn position_notice(number: usize, of: usize) -> String {
    let ahead: usize = number.saturating_sub(1);
    let _ = of;
    return match ahead {
        0 => String::from("This commit is next."),
        1 => String::from("Waiting for 1 earlier commit."),
        many => format!("Waiting for {many} earlier commits."),
    };
}

/// What:
///  The notice for a commit the service has named no place for,
/// given how long it has waited.
/// Why:
///   The client knows only that no answer has come and that the service named no place:
/// the landing may not have read the
///  request yet,
/// or it may be waiting behind another program's `index.lock`,
/// so the words may say nothing about whether the landing has
///  begun.
///  The wait so far is the one figure it does know,
/// and every other waiting line gives it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unplacedNotice(elapsed) // 'The commit service has not said where this commit stands yet; it has waited 12 seconds.'
/// ```
pub fn unplaced_notice(waited: Duration) -> String {
    // A part second of waiting is a second waited, and a long wait counts in whole seconds.
    let seconds: u64 = waited.as_secs().max(1);
    let unit: &str = if seconds == 1 { "second" } else { "seconds" };
    return format!("The commit service has not said where this commit stands yet; it has waited {seconds} {unit}.");
}

/// What:
///  The line that says a commit is built again because its branch moved:
/// the branch,
/// how many builds were made,
/// how long it has been since the first,
/// and the time at which the rebuilding stops.
/// Why:
///   A rebuild is a wait the person would otherwise not see,
/// and the owner's rule is that every wait says what it is for and when it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the branch ${branch} moved ...`
/// ```
pub fn rebuild_notice(branch: &str, builds: usize, waited: std::time::Duration, ceiling: std::time::Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the branch {branch} moved while this commit was being built ({builds} builds so far, {} seconds), so it is built again on the new tip; \
         nothing conflicted, and the building stops after {} seconds in all.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}
