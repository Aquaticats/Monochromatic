//! Controls for the schedule of repositories to look at again.

use super::*;

fn path(name: &str) -> PathBuf {
    return PathBuf::from(format!("/repositories/{name}/.git"));
}

/// An empty schedule waits for nothing;
/// one entry is waited for until it is due,
/// then taken once.
#[test]
fn an_entry_is_waited_for_until_it_is_due_and_then_taken_once() {
    let start: Instant = Instant::now();
    let mut schedule: RetrySchedule = RetrySchedule::default();
    assert!(schedule.is_empty());
    assert_eq!(schedule.next_wait(start), None);
    schedule.replace(&[], vec![(path("one"), Duration::from_secs(30))], start);
    assert_eq!(schedule.next_wait(start), Some(Duration::from_secs(30) + RETRY_SLACK));
    assert!(schedule.take_due(start + Duration::from_secs(30)).is_empty(), "the slack is part of the wait");
    assert_eq!(schedule.take_due(start + Duration::from_secs(30) + RETRY_SLACK), vec![path("one")]);
    assert!(schedule.is_empty());
    assert_eq!(schedule.take_due(start + Duration::from_secs(3600)), Vec::<PathBuf>::new(), "taken once");
}

/// The wait is for the first of several entries,
/// and an entry that is already due waits no time.
#[test]
fn the_wait_is_for_the_first_entry() {
    let start: Instant = Instant::now();
    let mut schedule: RetrySchedule = RetrySchedule::default();
    schedule.replace(&[], vec![(path("late"), Duration::from_secs(100)), (path("soon"), Duration::from_secs(10))], start);
    assert_eq!(schedule.next_wait(start), Some(Duration::from_secs(10) + RETRY_SLACK));
    assert_eq!(schedule.next_wait(start + Duration::from_secs(500)), Some(Duration::ZERO));
}

/// A round that ran a repository replaces its entry with what the round asked for,
/// or drops it when the round asked for
/// nothing,
/// and leaves the entries of repositories it did not run.
#[test]
fn a_round_replaces_the_entries_of_the_repositories_it_ran() {
    let start: Instant = Instant::now();
    let mut schedule: RetrySchedule = RetrySchedule::default();
    schedule.replace(&[], vec![(path("one"), Duration::from_secs(10)), (path("two"), Duration::from_secs(20))], start);
    schedule.replace(&[path("one")], Vec::new(), start);
    assert_eq!(schedule.take_due(start + Duration::from_secs(3600)), vec![path("two")], "one was run and asked for nothing");
    schedule.replace(&[], vec![(path("two"), Duration::from_secs(10))], start);
    schedule.replace(&[path("two")], vec![(path("two"), Duration::from_secs(50))], start);
    assert_eq!(schedule.next_wait(start), Some(Duration::from_secs(50) + RETRY_SLACK), "the newest answer wins");
}
