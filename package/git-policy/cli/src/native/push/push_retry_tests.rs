//! Controls for the push retry's schedule:
//! the pauses grow and stop growing,
//! a variable sets the first pause,
//! a plan is
//! per branch,
//! and the plans of the branches waiting for one.

use super::*;
use crate::push_retry_store::{StoredPlan, load, remove, save};
use crate::work_item::WorkKind;

fn item(branch: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::from("/srv/repo"),
        git_dir: PathBuf::from("/srv/repo/.git"),
        branch_ref: branch.to_owned(),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
}

/// Each pause is twice the one before,
/// from the first,
/// up to the longest.
#[test]
fn the_pauses_double_and_stop_at_the_longest() {
    let first: Duration = Duration::from_secs(10);
    let pauses: Vec<Duration> = (1..=4).map(|attempt| { return pause_before(attempt, first); }).collect();
    assert_eq!(pauses, vec![first, first * 2, first * 4, first * 8]);
    assert_eq!(pause_before(1_000, first), LONGEST_PAUSE);
    assert_eq!(pause_before(u32::MAX, first), LONGEST_PAUSE);
    assert!(pause_before(0, first) == first, "a try number of zero is the first");
}

/// The variable sets the first pause in whole seconds;
/// anything else is the default,
/// and never longer than the longest.
#[test]
fn the_variable_sets_the_first_pause() {
    assert_eq!(first_pause_from(None), FIRST_PAUSE);
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("3"))), Duration::from_secs(3));
    for unusable in ["0", "-1", "1.5", "soon", ""] {
        assert_eq!(first_pause_from(Some(std::ffi::OsStr::new(unusable))), FIRST_PAUSE, "{unusable:?}");
    }
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("99999"))), LONGEST_PAUSE);
}

/// A plan is due after its pause and not before,
/// replaces an earlier plan of its branch,
/// and belongs to its Git directory.
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

/// What:
///  A forget ends the plan a branch had in the list,
/// while the plans of other branches stay,
/// and the branch's file goes with the worker's own removal.
/// Why:
///   A landed push makes an earlier failure of its branch untrue,
/// and the other branches' retries are still wanted.
/// The list writes nothing (`PushRetries::forget`):
/// the worker that decided the branch needs no new try removes its file
/// (`Replanning::forget`),
/// so a round's replay of that forget cannot reach the journal after a later round planned the branch again.
#[test]
fn a_forget_drops_the_branch_from_the_list_and_the_worker_takes_its_file() {
    let root: PathBuf = crate::test_support::fixture("retry-forget-older");
    let here: &Path = Path::new("/srv/repo/.git");
    let main: StoredPlan = StoredPlan { git_dir: here.to_path_buf(), item: item("refs/heads/main"), attempt: 1 };
    let other: StoredPlan = StoredPlan { git_dir: here.to_path_buf(), item: item("refs/heads/topic"), attempt: 1 };
    save(root.as_path(), &main).expect("saved");
    save(root.as_path(), &other).expect("saved");
    let first: Duration = Duration::from_secs(10);
    let mut retries: PushRetries = PushRetries::restored(first, root.as_path());
    retries.forget(here, "refs/heads/main");
    let mut due: Vec<String> = retries.take_due(here, Instant::now() + Duration::from_secs(60)).into_iter().map(|one| { return one.item.branch_ref; }).collect();
    due.sort();
    assert_eq!(due, vec![String::from("refs/heads/topic")], "the list kept the other branch's retry and dropped the forgotten one");
    assert_eq!(load(root.as_path()).len(), 2, "the list writes nothing, so the file is the worker's to remove");
    remove(root.as_path(), here, "refs/heads/main");
    assert_eq!(load(root.as_path()), vec![other], "only the forgotten branch's plan is gone with the worker's removal");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A forget a round applies late leaves the plan a later round wrote for the same branch.
/// Why:
///  Finding 4 of the review of `bc3d54f9cd`:
/// the job that applies a round's changes can be left behind at its ceiling,
/// so a forget of an older round reaches the journal while a later round is planning the same branch again;
/// the record in the file names that branch,
/// so a removal that takes every file of the branch takes the newer plan too,
/// and a service killed before that try is due finds nothing where its notice promised a retry.
/// The list is the round's own statement about the branch,
/// and it is the worker that decided the change that writes or removes the plan's file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(dir, planB); retries.apply([forget(gitDir, branch)]); expect(await load(dir)).toEqual([planB]);
/// ```
#[test]
fn a_forget_applied_after_a_later_plan_leaves_that_plan_on_disk() {
    let root: PathBuf = crate::test_support::fixture("retry-forget-after-later-plan");
    let here: &Path = Path::new("/srv/repo/.git");
    let branch: &str = "refs/heads/main";
    let later: StoredPlan = StoredPlan { git_dir: here.to_path_buf(), item: item(branch), attempt: 4 };
    // The later round's worker wrote the plan it promised before its notice reached the person.
    save(root.as_path(), &later).expect("the later round's plan");
    let mut retries: PushRetries = PushRetries::restored(Duration::from_secs(10), root.as_path());
    assert!(retries.next_wait(here, Instant::now()).is_some(), "the plan the later round wrote is what this service is trying");
    // The older round's forget reaches the journal now, after the later plan was published.
    retries.apply(vec![RetryChange::Forget { git_dir: here.to_path_buf(), branch_ref: branch.to_owned(), only_while_unreachable: false, taken: None }]);
    assert!(retries.next_wait(here, Instant::now()).is_none(), "the list of this process forgot the branch, which is the older round's statement");
    assert_eq!(load(root.as_path()), vec![later], "a plan a later round wrote is not taken away by an earlier round's forget");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A read of the journal that came to nothing is owed again after a longer pause,
///  and the list keeps the journal's directory while it is owed.
/// Why:
///  A read that gave no answer is made again rather than given up on (`PushRetries::needs_restore`),
/// and each attempt leaves its read behind in a job that holds a place of the blocking bound
///  until the journal answers,
/// so a journal that stopped answering may not be read at the shortest pause for ever:
/// the pause doubles with each attempt that came to nothing.
#[test]
fn a_journal_read_that_came_to_nothing_is_owed_again_after_a_longer_pause() {
    // The first pause the control sets, so every later pause is that one doubled and nothing else.
    let first: Duration = Duration::from_secs(2);
    let root: PathBuf = crate::test_support::fixture("retry-restore-pause");
    let mut retries: PushRetries = PushRetries::unread(first, root.as_path());
    assert_eq!(retries.plans(), Some(root.as_path()), "the journal's directory is kept while the read is owed");
    assert!(retries.needs_restore(), "a list that has not read its journal is owed the read");
    assert_eq!(retries.restore_pause(), first, "the first attempt is made after the first pause");
    retries.restore_came_to_nothing();
    assert!(retries.needs_restore(), "a read that gave no answer is still owed");
    assert_eq!(retries.restore_pause(), first * 2, "the second attempt is made after the pause doubled");
    retries.restore_came_to_nothing();
    assert_eq!(retries.restore_pause(), first * 4, "the third attempt is made after the pause doubled again");
    // The read that answers ends what the list owes, and the record of what came to nothing goes with it.
    let _found: Vec<PathBuf> = retries.restore();
    assert!(!retries.needs_restore(), "a read that answered is not owed again");
    assert_eq!(retries.restore_pause(), first, "a list that has read its journal starts the pauses again");
    crate::test_support::remove(root.as_path());
}
