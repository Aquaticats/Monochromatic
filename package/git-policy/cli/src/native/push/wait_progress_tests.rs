//! Controls for the shared clock of wait notices:
//! the cadence from its variable,
//! the ticker's due moments,
//!  the place-in-line sentence,
//! the child-wait line and the reporter that prints them.

use super::*;

/// How long a control waits for a notice that must come,
/// so a reporter that never prints fails it instead of holding it.
const PATIENCE: Duration = Duration::from_secs(20);

/// A cadence the ticker tests drive:
/// one second of grace and ten seconds between notices.
///  The values are the case's input,
/// chosen so the waits are short;
/// they are not the production defaults.
const STANDARD: Cadence = Cadence {
    grace: Duration::from_secs(1),
    interval: Duration::from_secs(10),
};

/// The variable's value sets the interval and an unusable value falls back to the standard cadence.
#[test]
fn the_cadence_follows_the_variable_or_the_standard() {
    assert_eq!(Cadence::from_value(None), Cadence::standard());
    // The default interval only has to be a positive wait: a zero one would print at every poll.
    assert!(DEFAULT_NOTICE_INTERVAL > Duration::ZERO);
    assert_eq!(
        Cadence::from_value(Some(OsStr::new("30"))),
        Cadence { grace: NOTICE_GRACE, interval: Duration::from_secs(30) }
    );
    for value in ["0", "", "-1", "1.5", "soon"] {
        assert_eq!(Cadence::from_value(Some(OsStr::new(value))), Cadence::standard(), "{value:?}");
    }
}

/// The grace is never longer than the interval,
/// so an interval of one second also starts after one second.
#[test]
fn the_grace_never_outlasts_the_interval() {
    let short: Cadence = Cadence::from_interval(Duration::from_millis(250));
    // Below the standard grace, the grace is the whole interval.
    assert_eq!(short.grace, short.interval);
    let one_second: Duration = Duration::from_secs(1);
    assert_eq!(Cadence::from_interval(one_second).grace, one_second);
    assert_eq!(Cadence::from_interval(Duration::from_secs(60)).grace, NOTICE_GRACE);
}

/// Nothing is due before the grace;
/// one notice is due at the grace and one at every interval after it.
#[test]
fn a_notice_is_due_after_the_grace_and_at_every_interval() {
    let mut ticker: Ticker = Ticker::new(STANDARD);
    assert!(!ticker.due(Duration::ZERO));
    assert!(!ticker.due(Duration::from_millis(999)));
    assert!(ticker.due(Duration::from_secs(1)));
    assert!(!ticker.due(Duration::from_secs(1)));
    assert!(!ticker.due(Duration::from_millis(10_999)));
    assert!(ticker.due(Duration::from_secs(11)));
    assert!(!ticker.due(Duration::from_secs(12)));
    assert!(ticker.due(Duration::from_secs(21)));
}

/// A poll that comes late answers once for all the notices it missed,
/// never in a burst.
#[test]
fn a_late_poll_answers_once() {
    let mut ticker: Ticker = Ticker::new(STANDARD);
    assert!(ticker.due(Duration::from_secs(95)));
    assert!(!ticker.due(Duration::from_secs(95)));
    assert!(!ticker.due(Duration::from_secs(100)));
    assert!(ticker.due(Duration::from_secs(101)));
}

/// A wait that printed its first line at the start is next told one interval later.
#[test]
fn a_wait_announced_at_its_start_repeats_one_interval_later() {
    let mut ticker: Ticker = Ticker::announced_at_start(STANDARD);
    assert!(!ticker.due(Duration::ZERO));
    assert!(!ticker.due(Duration::from_secs(1)));
    assert!(!ticker.due(Duration::from_millis(9_999)));
    assert!(ticker.due(Duration::from_secs(10)));
    assert!(!ticker.due(Duration::from_secs(19)));
    assert!(ticker.due(Duration::from_secs(20)));
}

/// A zero interval is raised to the shortest one instead of dividing by zero.
#[test]
fn a_zero_interval_does_not_divide_by_zero() {
    let mut ticker: Ticker = Ticker::new(Cadence { grace: Duration::ZERO, interval: Duration::ZERO });
    assert!(ticker.due(Duration::from_millis(5)));
    assert!(ticker.due(Duration::from_millis(7)));
}

/// The ticker says how long a watcher may sleep before the next notice.
#[test]
fn the_next_notice_is_due_in_the_remaining_time() {
    let mut ticker: Ticker = Ticker::new(STANDARD);
    assert_eq!(ticker.next_in(Duration::from_millis(400)), Duration::from_millis(600));
    assert!(ticker.due(Duration::from_secs(1)));
    assert_eq!(ticker.next_in(Duration::from_secs(1)), Duration::from_secs(10));
    assert_eq!(ticker.next_in(Duration::from_secs(4)), Duration::from_secs(7));
    assert_eq!(ticker.next_in(Duration::from_secs(40)), Duration::ZERO);
    let announced: Ticker = Ticker::announced_at_start(STANDARD);
    assert_eq!(announced.next_in(Duration::from_secs(3)), Duration::from_secs(7));
}

/// The child line names the wait,
/// the process,
/// both durations and where the limit changes;
/// the repeats drop the hint.
#[test]
fn the_child_line_names_the_wait_the_process_and_both_durations() {
    let facts: ChildWaitFacts<'_> = ChildWaitFacts {
        waiting_for: "git cat-file --batch",
        pid: 4242,
        waited: Duration::from_millis(12_900),
        limit: Duration::from_secs(1800),
        limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
        repeat: false,
        ends_when_quiet: true,
        left_running_after: None,
    };
    let first: String = child_wait_line(&facts);
    assert_eq!(
        first,
        "monochromatic-git-policy-cli: still waiting for git cat-file --batch (process 4242) after 12 seconds; it is ended only after 1800 \
         seconds with no sign of work (CLI_GIT_CHILD_WAIT_SECONDS changes this). Hint: it is still running; if it looks stuck, check it \
         with `ps` and report it."
    );
    let repeated: String = child_wait_line(&ChildWaitFacts { repeat: true, limit_note: "", ..facts });
    assert_eq!(
        repeated,
        "monochromatic-git-policy-cli: still waiting for git cat-file --batch (process 4242) after 12 seconds; it is ended only after 1800 seconds with no sign of work."
    );
    let writer: String = child_wait_line(&ChildWaitFacts { repeat: true, limit_note: "", ends_when_quiet: false, ..facts });
    assert_eq!(
        writer,
        "monochromatic-git-policy-cli: still waiting for git cat-file --batch (process 4242) after 12 seconds; it is a step that may not be run twice, so it is waited for as long as it lives."
    );
}

/// What:
///  The notice of a step the wrapper never ends,
///  that the command stops waiting for,
/// names that bound instead of saying it is waited for as long as the step lives.
/// Why:
///   A write of one of the wrapper's own side references is never ended,
///  and the command that started it does stop waiting,
///  at the step's class ceiling plus the short extra wait:
///  the release gate of 0.6.1-alpha.11 read a notice that said the write "is waited for as long as it lives"
///  of a write whose wait had ended at 127 seconds,
///  which names a wait with no bound where the wrapper has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(childWaitLine({ ...facts, endsWhenQuiet: false, leftRunningAfter: 127 })).toContain('stops waiting for it after 127 seconds and leaves it running')
/// ```
#[test]
fn a_notice_of_a_step_left_running_names_the_bound_the_wait_ends_at() {
    let facts: ChildWaitFacts<'_> = ChildWaitFacts {
        waiting_for: "git update-ref --no-deref refs/worktree/cli-git-service/1, a quick query step",
        pid: 4242,
        waited: Duration::from_secs(5),
        limit: Duration::from_secs(60),
        limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
        repeat: false,
        ends_when_quiet: false,
        left_running_after: Some(Duration::from_secs(127)),
    };
    let line: String = child_wait_line(&facts);
    assert!(
        line.contains("the wrapper never ends it, and this command stops waiting for it after 127 seconds and leaves it running"),
        "the notice names the bound the wait ends at: {line}"
    );
    assert!(!line.contains("as long as it lives"), "no step whose wait has a bound is called unbounded: {line}");
    assert!(!line.contains("CLI_GIT_CHILD_WAIT_SECONDS"), "the quiet window is not what ends this wait: {line}");
    let repeated: String = child_wait_line(&ChildWaitFacts { repeat: true, ..facts });
    assert!(!repeated.contains("Hint:"), "a repeat keeps the line short: {repeated}");
}

/// The reporter reports no notice due and prints nothing before the grace,
/// and its first line comes once the grace has passed:
///  the grace here is short enough to watch pass,
/// so neither half rests on a duration no control can outlive.
///  The polls inside the grace are timed from after the reporter started,
/// so every one of them is inside a grace of half of it on any host.
#[test]
fn the_reporter_prints_nothing_inside_the_grace() {
    let grace: Duration = Duration::from_millis(500);
    let mut sink: Vec<u8> = Vec::new();
    {
        let mut reporter: WaitReporter<'_> = WaitReporter::start(Cadence { grace, interval: grace }, &mut sink);
        let started: Instant = Instant::now();
        while started.elapsed() < grace / 2 {
            assert!(
                reporter.notice_due().is_none(),
                "a notice was due {:?} into a grace of {:?}",
                started.elapsed(),
                grace
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        // The other direction, so the polls above are a grace that had not passed rather than a reporter that never prints.
        let waited: Instant = Instant::now();
        let mut due: bool = false;
        while !due {
            assert!(waited.elapsed() < PATIENCE, "the reporter printed no first line once the grace of {grace:?} had passed");
            due = reporter.notice_due().is_some();
            if due {
                reporter.print_notice("line");
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    assert_eq!(String::from_utf8(sink).expect("text"), "line\n", "the first line comes once the grace has passed");
}

/// The reporter prints a line at the grace and at every interval after it,
/// and the line builder runs only for a printed
///  line and learns whether it is a repeat.
///  It polls until three lines are out,
/// so the count does not depend on how fast the host runs the loop.
#[test]
fn the_reporter_prints_at_the_pace_of_its_ticker() {
    let mut sink: Vec<u8> = Vec::new();
    let mut built: u32 = 0;
    {
        let mut reporter: WaitReporter<'_> = WaitReporter::start(
            Cadence { grace: Duration::from_millis(20), interval: Duration::from_millis(20) },
            &mut sink,
        );
        let mut printed: u32 = 0;
        while printed < 3 {
            if let Some(due) = reporter.notice_due() {
                built += 1;
                reporter.print_notice(format!("line repeat={}", due.repeat).as_str());
                printed += 1;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    assert_eq!(built, 3, "the builder runs only when a line is printed");
    assert_eq!(
        String::from_utf8(sink).expect("text"),
        "line repeat=false\nline repeat=true\nline repeat=true\n"
    );
}
