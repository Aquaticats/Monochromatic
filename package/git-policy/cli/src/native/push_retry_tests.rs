//! Controls for the push retry's schedule: the pauses grow and stop growing, a variable sets the first pause, a plan is
//! per branch, and only the conflict report is final.

use super::*;
use crate::work_item::WorkKind;

fn item(branch: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: String::from("/srv/repo"),
        git_dir: String::from("/srv/repo/.git"),
        branch_ref: branch.to_owned(),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
}

/// Each pause is twice the one before, from the first, up to the longest.
#[test]
fn the_pauses_double_and_stop_at_the_longest() {
    let first: Duration = Duration::from_secs(10);
    let pauses: Vec<Duration> = (1..=4).map(|attempt| { return pause_before(attempt, first); }).collect();
    assert_eq!(pauses, vec![first, first * 2, first * 4, first * 8]);
    assert_eq!(pause_before(1_000, first), LONGEST_PAUSE);
    assert_eq!(pause_before(u32::MAX, first), LONGEST_PAUSE);
    assert!(pause_before(0, first) == first, "a try number of zero is the first");
}

/// The variable sets the first pause in whole seconds; anything else is the default, and never longer than the longest.
#[test]
fn the_variable_sets_the_first_pause() {
    assert_eq!(first_pause_from(None), FIRST_PAUSE);
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("3"))), Duration::from_secs(3));
    for unusable in ["0", "-1", "1.5", "soon", ""] {
        assert_eq!(first_pause_from(Some(std::ffi::OsStr::new(unusable))), FIRST_PAUSE, "{unusable:?}");
    }
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("99999"))), LONGEST_PAUSE);
}

/// A plan is due after its pause and not before, replaces an earlier plan of its branch, and belongs to its Git directory.
#[test]
fn a_plan_is_per_branch_and_due_after_its_pause() {
    let mut retries: PushRetries = PushRetries::with_first_pause(Duration::from_secs(10));
    let now: Instant = Instant::now();
    let here: &Path = Path::new("/srv/repo/.git");
    let elsewhere: &Path = Path::new("/srv/other/.git");
    assert_eq!(retries.schedule(here, &item("refs/heads/main"), 1, now), Duration::from_secs(10));
    assert_eq!(retries.schedule(here, &item("refs/heads/main"), 2, now), Duration::from_secs(20), "the same branch has one plan");
    retries.schedule(here, &item("refs/heads/topic"), 1, now);
    assert_eq!(retries.next_wait(here, now), Some(Duration::from_secs(10)));
    assert_eq!(retries.next_wait(elsewhere, now), None);
    assert!(retries.take_due(here, now + Duration::from_secs(9)).is_empty());
    assert!(retries.take_due(elsewhere, now + Duration::from_secs(60)).is_empty());
    let due: Vec<Due> = retries.take_due(here, now + Duration::from_secs(10));
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].item.branch_ref, "refs/heads/topic");
    retries.forget(here, "refs/heads/main");
    assert_eq!(retries.next_wait(here, now), None);
}

/// Only a reported conflict needs the person; a stall, a ceiling and a refused connection do not.
#[test]
fn only_a_conflict_needs_the_person() {
    assert!(needs_the_person("cli-git: auto-push: not published: merging origin/main into main conflicts in: a.txt, b.txt. Nothing was pushed"));
    for output in [
        "cli-git: git push made no progress for 30 seconds and was stopped (limit=stall, bytes_moved=0, elapsed_seconds=30)",
        "cli-git: git push ran for 1800 seconds and was stopped (limit=ceiling)",
        "fatal: unable to access 'https://example.invalid/': Could not resolve host: example.invalid",
        "",
    ] {
        assert!(!needs_the_person(output), "{output}");
    }
}
