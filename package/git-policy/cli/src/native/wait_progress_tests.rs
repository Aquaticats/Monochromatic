//! Controls for the shared clock of wait notices: the cadence from its variable, the ticker's due moments,
//!  the place-in-line sentence, the child-wait line and the reporter that prints them.

use super::*;

/// A cadence of a one second grace and a ten second interval, the standard one.
const STANDARD: Cadence = Cadence {
    grace: Duration::from_secs(1),
    interval: Duration::from_secs(10),
};

/// The variable's value sets the interval and an unusable value falls back to the standard cadence.
#[test]
fn the_cadence_follows_the_variable_or_the_standard() {
    assert_eq!(Cadence::from_value(None), STANDARD);
    assert_eq!(DEFAULT_NOTICE_INTERVAL, Duration::from_secs(10));
    assert_eq!(
        Cadence::from_value(Some(OsStr::new("30"))),
        Cadence { grace: Duration::from_secs(1), interval: Duration::from_secs(30) }
    );
    for value in ["0", "", "-1", "1.5", "soon"] {
        assert_eq!(Cadence::from_value(Some(OsStr::new(value))), STANDARD, "{value:?}");
    }
}

/// The grace is never longer than the interval, so an interval of one second also starts after one second.
#[test]
fn the_grace_never_outlasts_the_interval() {
    let short: Cadence = Cadence::from_interval(Duration::from_millis(250));
    assert_eq!(short.grace, Duration::from_millis(250));
    assert_eq!(Cadence::from_interval(Duration::from_secs(1)).grace, Duration::from_secs(1));
    assert_eq!(Cadence::from_interval(Duration::from_secs(60)).grace, NOTICE_GRACE);
}

/// Nothing is due before the grace; one notice is due at the grace and one at every interval after it.
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

/// A poll that comes late answers once for all the notices it missed, never in a burst.
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

/// The place sentence says the number and how many are served first.
#[test]
fn the_place_sentence_counts_the_commits_ahead() {
    assert_eq!(
        queue_place_text(QueuePlace { position: 3, waiting: 4 }),
        "This is number 3 of 4 commits in line, so 2 will be served first."
    );
    assert_eq!(
        queue_place_text(QueuePlace { position: 1, waiting: 1 }),
        "This is number 1 of 1 commits in line, so 0 will be served first."
    );
}

/// The child line names the wait, the process, both durations and where the limit changes; the repeats drop the hint.
#[test]
fn the_child_line_names_the_wait_the_process_and_both_durations() {
    let facts: ChildWaitFacts<'_> = ChildWaitFacts {
        waiting_for: "git cat-file --batch",
        pid: 4242,
        waited: Duration::from_millis(12_900),
        limit: Duration::from_secs(1800),
        limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
        repeat: false,
    };
    let first: String = child_wait_line(&facts);
    assert_eq!(
        first,
        "cli-git: still waiting for git cat-file --batch (process 4242) after 12 seconds; stopping it after 1800 \
         seconds (CLI_GIT_CHILD_WAIT_SECONDS changes this). Hint: it is still running; if it looks stuck, check it \
         with `ps` and report it."
    );
    let repeated: String = child_wait_line(&ChildWaitFacts { repeat: true, limit_note: "", ..facts });
    assert_eq!(
        repeated,
        "cli-git: still waiting for git cat-file --batch (process 4242) after 12 seconds; stopping it after 1800 seconds."
    );
}

/// The reporter prints nothing before the grace, and the line builder does not run.
///  The grace is an hour, so no host is slow enough to make the early poll late.
#[test]
fn the_reporter_prints_nothing_inside_the_grace() {
    let mut sink: Vec<u8> = Vec::new();
    let mut built: u32 = 0;
    {
        let mut reporter: WaitReporter<'_> = WaitReporter::start(
            Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            &mut sink,
        );
        assert!(!reporter.poll(|_, _| { built += 1; return String::from("early"); }));
    }
    assert_eq!(built, 0, "the builder runs only when a line is printed");
    assert!(sink.is_empty());
}

/// The reporter prints a line at the grace and at every interval after it, and the line builder runs only for a printed
///  line and learns whether it is a repeat.
///  It polls until three lines are out, so the count does not depend on how fast the host runs the loop.
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
            if reporter.poll(|_, repeat| { built += 1; return format!("line repeat={repeat}"); }) {
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
