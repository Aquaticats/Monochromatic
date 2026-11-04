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

/// A journal entry that says request `id` landed as batch `seq`, written the way the landing writes it.
fn record_landed(journal: &std::path::Path, seq: u64, id: &str) {
    let intent: crate::service_journal::Intent = crate::service_journal::Intent {
        seq,
        repository: std::path::PathBuf::from("/repo"),
        common_dir: std::path::PathBuf::from("/repo/.git"),
        reference: String::from("refs/heads/main"),
        old: "0".repeat(40),
        new: "1".repeat(40),
        ids: vec![String::from(id)],
        commits: vec!["1".repeat(40)],
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: std::path::PathBuf::from("/repo/.git/index"),
        git_dir: std::path::PathBuf::from("/repo/.git"),
    };
    crate::service_journal::write_intent(journal, &intent).expect("the intent");
    crate::service_journal::mark_done(journal, seq).expect("the batch is done");
}

/// Withdrawer W1 wins the decision and is stopped before it asks the journal again; withdrawer W2 meets the decision. W2 must not
/// answer "withdrawn": W1 has not confirmed it, and the request lands in the meantime (sol-1 round 2, P1). W1 then finds the request in
/// the journal and answers "landed", and W2's withdrawal is left to the landing thread.
#[test]
fn a_decision_another_withdrawer_has_not_confirmed_is_not_acknowledged() {
    let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-two-withdrawers").join("journal");
    let record: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    let (won, wait_for_won) = std::sync::mpsc::channel::<()>();
    let (release, wait_for_release) = std::sync::mpsc::channel::<()>();
    let first = {
        let shared = Arc::clone(&record);
        std::thread::spawn(move || {
            AFTER_THE_DECISION_IS_WON.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    let _told = won.send(());
                    let _released = wait_for_release.recv_timeout(Duration::from_secs(30));
                }));
            });
            return shared.withdraw_early("r");
        })
    };
    wait_for_won.recv_timeout(Duration::from_secs(30)).expect("W1 won the decision");
    // W2 meets W1's decision, which W1 has not confirmed.
    assert_eq!(record.withdraw_early("r"), EarlyAnswer::Queued, "an unconfirmed decision is not acknowledged");
    // The request lands meanwhile, and the journal says so.
    record_landed(journal.as_path(), 1, "r");
    release.send(()).expect("let W1 go on");
    let answer: EarlyAnswer = first.join().expect("W1");
    assert_eq!(answer, EarlyAnswer::Landed("1".repeat(40)), "W1 asks the journal again and finds the request");
    assert!(!record.decision_path("r").exists(), "nobody acknowledged the decision, so W1 removes it");
}

/// W1 wins and its second journal read fails; W2 met the decision meanwhile. W1 removes the decision and queues its withdrawal, and
/// W2 was not told "withdrawn", so the request can still be admitted and landed without any client having been told it never will
/// (sol-1 round 2, P1: the creator removed a decision another withdrawer had acknowledged).
#[test]
fn a_journal_that_fails_after_the_win_leaves_no_acknowledged_cancellation() {
    use std::os::unix::fs::PermissionsExt;
    let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-journal-fails").join("journal");
    let record: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    let (won, wait_for_won) = std::sync::mpsc::channel::<()>();
    let (release, wait_for_release) = std::sync::mpsc::channel::<()>();
    let first = {
        let shared = Arc::clone(&record);
        std::thread::spawn(move || {
            AFTER_THE_DECISION_IS_WON.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    let _told = won.send(());
                    let _released = wait_for_release.recv_timeout(Duration::from_secs(30));
                }));
            });
            return shared.withdraw_early("r");
        })
    };
    wait_for_won.recv_timeout(Duration::from_secs(30)).expect("W1 won the decision");
    let second: EarlyAnswer = record.withdraw_early("r");
    // The journal can no longer be read whole (the directory takes no listing), while the record's directory under it still works.
    std::fs::set_permissions(journal.as_path(), std::fs::Permissions::from_mode(0o300)).expect("make the journal unreadable");
    release.send(()).expect("let W1 go on");
    let first_answer: EarlyAnswer = first.join().expect("W1");
    std::fs::set_permissions(journal.as_path(), std::fs::Permissions::from_mode(0o700)).expect("restore");
    assert_eq!(first_answer, EarlyAnswer::Queued, "an unreadable journal confirms nothing");
    assert_eq!(second, EarlyAnswer::Queued, "and the other withdrawer was told nothing final");
    assert_eq!(record.begin_group(ids(&["r"]).as_slice()), GroupAdmission::default(), "the request is still admitted, not dropped");
}

/// A temporary file of a decision is removed whichever way the decision ends (sol-1 round 2, P2: a failed write returned before the
/// removal and left it): the guard removes its file when it goes out of scope, by an early return as by the end of the function.
#[test]
fn a_temporary_file_is_removed_by_an_early_return() {
    let directory: std::path::PathBuf = crate::test_support::fixture("service-busy-temporary");
    let leaking = |path: std::path::PathBuf| -> std::io::Result<()> {
        let guard: TemporaryFile = TemporaryFile(path);
        std::fs::write(guard.0.as_path(), b"partial")?;
        return Err(std::io::Error::other("the write failed after the file was made"));
    };
    let file: std::path::PathBuf = directory.join("r.1.1.tmp");
    assert!(leaking(file.clone()).is_err());
    assert!(!file.exists(), "the file is removed although the function returned early");
    let record: Arc<LandingBoard> = LandingBoard::new(directory.join("journal"), AT_ONCE);
    for round in 0..20 {
        assert!(record.decide("same", LANDING).is_ok(), "round {round}");
    }
    let leftovers: usize = std::fs::read_dir(record.directory.as_path()).expect("listing").flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".tmp"); }).count();
    assert_eq!(leftovers, 0, "a decision that wins or loses leaves no temporary file");
}

/// The memory of confirmed withdrawals is bounded: past the retention the oldest are forgotten, decision and confirmation together.
#[test]
fn only_the_retained_number_of_confirmed_withdrawals_is_kept() {
    let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-retention").join("journal");
    let record: Arc<LandingBoard> = LandingBoard::new(journal, AT_ONCE);
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    std::thread::sleep(A_MOMENT);
    for round in 0..(WITHDRAWALS_RETAINED + 6) {
        assert!(matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)), "round {round}");
    }
    let count = |extension: &str| -> usize { return std::fs::read_dir(record.directory.as_path()).expect("listing").flatten().filter(|entry| { return entry.path().extension().and_then(|text| { return text.to_str(); }) == Some(extension); }).count(); };
    assert_eq!(count(CANCELLED_EXTENSION), WITHDRAWALS_RETAINED);
    assert_eq!(count(DECISION_EXTENSION), WITHDRAWALS_RETAINED, "a decision never outlives its confirmation, nor the other way");
}
