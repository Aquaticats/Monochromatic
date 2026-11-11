//! What:
//!  Controls of the line a long-running `git worktree add` prints:
//! its words,
//! when it is printed,
//! that it stops,
//! and that the add's own steps are what it names.
//! Why:
//!  The notice is output only,
//! so what it must not do is as important as what it must:
//! it never ends the add,
//! never names a step the add is not in,
//! and never prints after the add answered.

use super::*;
use crate::add_timing;
use crate::wait_ceilings::ADD_STANDING_NOTICE_AFTER;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

/// What:
///  Everything a notice printed during one control,
/// read until `stop` says how many lines are wanted or a bound passes.
/// Why:
///   The line is printed on the notice's own thread,
/// so a control reads what arrived rather than what it hopes arrived,
/// and never waits for the thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (lines: mpsc.Receiver<string>, wanted: number, bound: Duration) => { const said = []; while (said.length < wanted && !timedOut) said.push(lines.recvTimeout(bound)); return said; }
/// ```
fn read_lines(lines: &Receiver<String>, wanted: usize, bound: Duration) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    while said.len() < wanted {
        let Ok(line) = lines.recv_timeout(bound) else { break };
        said.push(line);
    }
    return said;
}

/// What:
///  What stands published for the notice now,
/// or nothing.
/// Why:
///   The add's own step recording is what a control of the publication reads,
/// so the one fact it needs is read through the same door the notice uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// insideAnAdd() // [step, began] | undefined
/// ```
fn published() -> Option<(Option<String>, Instant)> {
    return inside_an_add();
}

/// A step the add entered and left is published as that step,
/// then as the step that encloses it,
/// and a step entered with no recording publishes nothing.
///
/// This is the control that the line names what the add is doing:
/// before the add's timing published its steps,
/// a waiting add had nothing to name.
#[test]
fn the_add_s_own_steps_are_what_the_notice_names() {
    let root: PathBuf = crate::test_support::fixture("add-standing-published");
    add_timing::begin(root.join("common"));
    the_add_ended();
    assert_eq!(published(), None, "no step is published before one is entered");
    let outer: bool = add_timing::begin_step("copyIgnored");
    let inner: bool = add_timing::begin_step("git ls-files");
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(Some(String::from("git ls-files"))), "the innermost step is the one named");
    add_timing::end_step(inner);
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(Some(String::from("copyIgnored"))), "leaving a step names the one that encloses it");
    add_timing::end_step(outer);
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(None), "between two steps nothing is named");
    add_timing::finish(0);
    assert_eq!(published(), None, "an add that answered publishes no step");
    crate::test_support::remove(root.as_path());
}

/// A notice is printed once the add has been in a step longer than the wait,
/// and it stops when the add's notice goes out of scope.
///
/// The wait here is a few milliseconds instead of the shipped seconds,
/// so the control's bound is the machine's speed
/// rather than a person's patience.
#[test]
fn a_notice_is_printed_once_the_wait_is_over_and_stops_with_its_value() {
    let root: PathBuf = crate::test_support::fixture("add-standing-printed");
    add_timing::begin(root.join("common"));
    let standing: bool = add_timing::begin_step("gitAdd");
    let (announced, heard): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let notice: StandingNotice = begin_after(
        Box::new(move |line: String| {
            let _sent = announced.send(line);
        }),
        Duration::from_millis(1),
    );
    let said: Vec<String> = read_lines(&heard, 1, Duration::from_secs(10));
    assert_eq!(said.len(), 1, "one line arrives once the wait is over");
    assert!(said[0].starts_with("monochromatic-git-policy-cli: `git worktree add` is still running after "), "{}", said[0]);
    assert!(said[0].contains("it is in Git's own `git worktree add`"), "the line names what the add is doing: {}", said[0]);
    drop(notice);
    add_timing::end_step(standing);
    add_timing::finish(0);
    crate::test_support::remove(root.as_path());
}

/// The line says how long the add has run and what it is doing,
/// in words a person reads rather than the timing record's step names.
#[test]
fn the_line_counts_the_seconds_and_says_the_step_in_words() {
    let line: String = notice_line(Some("snapshotIoctl"), Duration::from_secs(7));
    assert!(line.starts_with("monochromatic-git-policy-cli: `git worktree add` is still running after 7 seconds; "), "{line}");
    assert!(line.contains("it is taking the snapshot of the main checkout"), "the step is said in the person's words: {line}");
    assert!(!line.contains("snapshotIoctl"), "the timing record's name for the step is not the person's: {line}");
    assert!(line.contains("Nothing here waits for another command to finish"), "the line says what the wait is not: {line}");
    assert!(notice_line(None, Duration::from_secs(9)).contains("it is between two of its steps"), "a step that has just ended is named as such");
    assert_eq!(step_sentence(Some("a step of a later release")), "the add is working in a step its timing record names", "a step this release does not know is still named as work");
}

/// The wait a control sets is the shipped one unless a positive number of seconds is set.
#[test]
fn the_notice_takes_a_positive_setting_and_the_shipped_wait_otherwise() {
    assert_eq!(crate::wait_ceilings::notice_after_from(Some("30"), ADD_STANDING_NOTICE_AFTER), Duration::from_secs(30));
    for meaningless in [None, Some(""), Some("abc"), Some("0"), Some("-5"), Some("1.5")] {
        assert_eq!(crate::wait_ceilings::notice_after_from(meaningless, ADD_STANDING_NOTICE_AFTER), ADD_STANDING_NOTICE_AFTER, "{meaningless:?}");
    }
    assert_eq!(crate::wait_ceilings::ADD_STANDING_NOTICE_VARIABLE, "CLI_GIT_ADD_NOTICE_SECONDS");
}
