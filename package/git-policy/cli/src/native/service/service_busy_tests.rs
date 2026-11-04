//! Controls for the record of what the landing thread is stuck in.

use super::*;

/// A threshold below any step's age, so a step counts as stuck at once.
const AT_ONCE: Duration = Duration::ZERO;

/// A threshold no control reaches, so a step never counts as stuck.
const NEVER: Duration = Duration::from_secs(3600);

/// The age a control lets a step reach before it asks, so the age is measurably above `AT_ONCE`.
const A_MOMENT: Duration = Duration::from_millis(20);

fn board_stuck_after(slow_after: Duration) -> Arc<LandingBoard> {
    return LandingBoard::new(crate::test_support::fixture("service-busy-board").join("journal-that-does-not-exist"), slow_after);
}

fn board() -> Arc<LandingBoard> {
    return board_stuck_after(AT_ONCE);
}

fn ids(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| { return String::from(*name); }).collect();
}

/// A thread with no board records nothing; one with a board records the step and forgets it when the guard ends.
#[test]
fn a_step_is_recorded_only_on_a_thread_with_a_board_and_only_while_it_runs() {
    assert!(enter_step(String::from("git update-ref --stdin in /repo")).is_none(), "no board on this thread yet");
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    assert_eq!(record.describe(), None, "nothing runs");
    let guard = enter_step(String::from("git update-ref --stdin in /repo")).expect("this thread has a board now");
    std::thread::sleep(A_MOMENT);
    let words: String = record.describe().expect("the step is stuck at once");
    assert!(words.contains("git update-ref --stdin in /repo") && words.contains("never ends a write"), "{words}");
    drop(guard);
    assert_eq!(record.describe(), None, "the step ended");
}

/// A step younger than the threshold is not described: only a write that has outlasted what writes take is stuck.
#[test]
fn a_step_younger_than_the_threshold_is_not_described() {
    let record: Arc<LandingBoard> = board_stuck_after(NEVER);
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    assert_eq!(record.describe(), None);
}

/// The detail the landing gives the next write is part of its words, and it is gone when the landing's guard ends.
#[test]
fn the_detail_of_a_write_names_the_commit_it_makes_and_ends_with_the_write() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    {
        let _detail = detail(String::from("it moves refs/heads/main to abc; the commit is also at refs/worktree/cli-git-service/7"));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let words: String = record.describe().expect("stuck");
        assert!(words.contains("refs/worktree/cli-git-service/7"), "{words}");
    }
    let _guard = enter_step(String::from("git update-ref -d scratch in /repo"));
    std::thread::sleep(A_MOMENT);
    let later: String = record.describe().expect("stuck");
    assert!(!later.contains("refs/worktree/cli-git-service/7"), "{later}");
}

/// A withdrawal is not answered here while the landing thread is not stuck: it goes to the landing thread as before.
#[test]
fn a_withdrawal_is_queued_while_nothing_is_stuck() {
    let record: Arc<LandingBoard> = board();
    assert_eq!(record.withdraw_early("a"), EarlyAnswer::Queued);
}

/// A request of the group in flight is never withdrawn here, because the landing may be making its commit; one that has not begun
/// is withdrawn, and when its group begins it is dropped and the others are marked.
#[test]
fn only_a_request_that_has_not_begun_is_withdrawn_while_the_landing_is_stuck() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    assert_eq!(record.begin_group(ids(&["running"]).as_slice()), GroupAdmission::default());
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    assert_eq!(record.withdraw_early("running"), EarlyAnswer::Queued, "the request in flight waits for the landing");
    let EarlyAnswer::Withdrawn(words) = record.withdraw_early("later") else { panic!("a request that has not begun is withdrawn") };
    assert!(words.contains("git update-ref --stdin"), "the answer carries the words that name the stuck write: {words}");
    record.end_group(ids(&["running"]).as_slice());
    let dropped: Vec<String> = record.begin_group(ids(&["later", "other"]).as_slice()).dropped;
    assert_eq!(dropped, ids(&["later"]), "the withdrawn request is dropped when its group would begin");
    assert_eq!(record.withdraw_early("other"), EarlyAnswer::Queued, "the request that begins is in flight");
}

/// A request already in flight when the landing thread's step ends is not withdrawn afterwards either: the record of the step is
/// what makes a withdrawal early, and without it every withdrawal is the landing thread's.
#[test]
fn a_withdrawal_after_the_stuck_write_returned_goes_to_the_landing_thread() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    let guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    assert!(matches!(record.withdraw_early("a"), EarlyAnswer::Withdrawn(_)));
    drop(guard);
    assert_eq!(record.withdraw_early("b"), EarlyAnswer::Queued);
}

/// An identity that is not a plain name gets no early answer and no marker, so it cannot name a file outside the record's directory.
#[test]
fn an_identity_that_is_not_a_plain_name_is_never_answered_early() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    for hostile in ["../escape", "a/b", "", "with space", &"x".repeat(LONGEST_ID + 1)] {
        assert_eq!(record.withdraw_early(hostile), EarlyAnswer::Queued, "{hostile:?}");
        assert_eq!(record.begin_group(&[String::from(hostile)]), GroupAdmission::default(), "{hostile:?}");
    }
    assert!(!record.directory.parent().expect("parent").join("escape.decision").exists());
}

/// A service that starts finds no record of a thread that ran before it.
#[test]
fn a_new_board_removes_what_an_earlier_service_left() {
    let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-restart").join("journal");
    let first: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
    assert_eq!(first.begin_group(&ids(&["old"])), GroupAdmission::default());
    assert!(first.is_in_flight("old"));
    let second: Arc<LandingBoard> = LandingBoard::new(journal, AT_ONCE);
    assert!(!second.is_in_flight("old"));
}

/// Fails closed (sol-1, P1): when the record cannot be written, a withdrawal is never answered "withdrawn" and a request is never
/// taken as admitted on a marker that was not made. The record's directory is made a plain file, so every marker operation fails.
#[test]
fn a_record_that_cannot_be_written_never_answers_withdrawn() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    assert!(matches!(record.withdraw_early("works"), EarlyAnswer::Withdrawn(_)), "the control that must move");
    // The directory stays readable (the step file can still be read) but takes no new file, as a full disk or a refused write leaves it.
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(record.directory.as_path(), std::fs::Permissions::from_mode(0o500)).expect("make the directory read-only");
    let answer: EarlyAnswer = record.withdraw_early("later");
    let admission: GroupAdmission = record.begin_group(ids(&["later"]).as_slice());
    std::fs::set_permissions(record.directory.as_path(), std::fs::Permissions::from_mode(0o700)).expect("restore");
    assert_eq!(answer, EarlyAnswer::Queued, "no cancellation was established, so none is confirmed");
    assert!(admission.dropped.is_empty() && admission.unavailable == ids(&["later"]), "a request is not landed on a decision that was not made, and is not dropped either");
}

/// A withdrawal is a tombstone with the identity of a normal one (sol-1, P1): every copy of the request that reaches a group after
/// it, replays and duplicates included, is dropped, and not only the first.
#[test]
fn every_copy_of_a_withdrawn_request_is_dropped_not_only_the_first() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    assert!(matches!(record.withdraw_early("r"), EarlyAnswer::Withdrawn(_)));
    for copy in 0..3 {
        assert_eq!(record.begin_group(ids(&["r"]).as_slice()).dropped, ids(&["r"]), "copy {copy} is dropped");
        record.end_group(ids(&["r"]).as_slice());
    }
    assert!(matches!(record.withdraw_early("r"), EarlyAnswer::Withdrawn(_)), "and the same answer to a repeated withdrawal");
}

/// The landing and a withdrawal race for the same requests: whichever way each interleaving falls, a request answered "withdrawn" is
/// never admitted, and one admitted is never answered "withdrawn" (sol-1, P1). The two sides start together behind a barrier,
/// repeatedly, on fresh identities.
#[test]
fn admission_and_early_withdrawal_never_both_succeed() {
    let record: Arc<LandingBoard> = board();
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    for round in 0..200 {
        let id: String = format!("race-{round}");
        let barrier: Arc<std::sync::Barrier> = Arc::new(std::sync::Barrier::new(2));
        let withdrawer = {
            let (shared_record, shared_barrier, shared_id) = (Arc::clone(&record), Arc::clone(&barrier), id.clone());
            std::thread::spawn(move || { shared_barrier.wait(); return shared_record.withdraw_early(shared_id.as_str()); })
        };
        barrier.wait();
        let dropped: Vec<String> = record.begin_group(std::slice::from_ref(&id)).dropped;
        let answer: EarlyAnswer = withdrawer.join().expect("withdrawer");
        match answer {
            EarlyAnswer::Withdrawn(_) => assert_eq!(dropped, vec![id.clone()], "round {round}: answered withdrawn, so it must not land"),
            EarlyAnswer::Queued | EarlyAnswer::Landed(_) => assert!(dropped.is_empty() || answer == EarlyAnswer::Queued, "round {round}"),
        }
        record.end_group(&[id]);
    }
}
