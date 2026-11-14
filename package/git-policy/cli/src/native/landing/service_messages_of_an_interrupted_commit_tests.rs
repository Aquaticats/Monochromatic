//! Controls for the words of a commit the person interrupted:
//! it says that the commit was not made,
//! or that whether it was made is not known,
//! and it never uses an implementation term.

use crate::service_messages::{ClientFailure, NOTHING_CHANGED, render};

/// What:
///  A commit the service gave up after the person interrupted it says that it was not made,
///  and that nothing lands later by itself.
/// Why:
///   Native Git says nothing at all when its command is interrupted,
///  and the owner's words for a commit that is not made are that nothing lands later by itself.
#[test]
fn an_interrupted_commit_says_it_was_not_made() {
    let failure: ClientFailure = ClientFailure::Interrupted { signal: libc::SIGINT, sender: None, blocker: None };
    let words: String = render(&failure);
    assert!(words.contains("interrupted with signal 2"), "{words}");
    assert!(words.contains("it was not made"), "{words}");
    assert!(words.contains("nothing lands later by itself"), "{words}");
    assert!(words.contains(NOTHING_CHANGED), "{words}");
}

/// What:
///  The sentence about what the service waited for names it once,
///  with one full stop.
/// Why:
///  The notices the service sends end in a full stop,
///  and a second one after a quoted notice reads as a mistake.
#[test]
fn the_notice_it_waited_for_is_quoted_once() {
    let blocker: String = String::from("monochromatic-git-policy-cli: it waits for another program's index.lock.\n");
    let failure: ClientFailure = ClientFailure::Interrupted { signal: libc::SIGTERM, sender: Some(4242), blocker: Some(blocker) };
    let words: String = render(&failure);
    assert!(words.contains("It was waiting for: monochromatic-git-policy-cli: it waits for another program's index.lock."), "{words}");
    assert!(!words.contains(".."), "one full stop, not two: {words}");
}

/// What:
///  An interrupted commit whose withdrawal the service did not confirm says that it is not known,
///  and gives the commands that show whether it landed.
/// Why:
///   A commit that may be on the branch is never reported as one that was not made.
#[test]
fn an_unconfirmed_withdrawal_says_the_result_is_not_known() {
    let failure: ClientFailure = ClientFailure::InterruptedNotKnown { signal: libc::SIGINT, sender: None, blocker: Some(String::from("the service has been inside a write for 21 minutes")) };
    let words: String = render(&failure);
    assert!(words.contains("it is not known whether the commit was made"), "{words}");
    assert!(words.contains("git log -1") && words.contains("git status"), "{words}");
    assert!(words.contains("the service has been inside a write for 21 minutes"), "{words}");
}

/// What:
///  Every other failure is worded by the file that words them all,
///  so this one answers nothing for it.
/// Why:
///   The renderer asks this file first,
///  and a failure it does not know must fall through to the rest.
#[test]
fn every_other_failure_is_worded_by_the_renderer() {
    assert!(render(&ClientFailure::Refused).contains("was not made"));
    assert!(!render(&ClientFailure::Refused).contains("signal"));
}
