//! Controls for the record of what the landing thread is stuck in.

use crate::service_busy_step_record_on_this_thread::{detail, enter_step, use_on_this_thread};
use crate::service_busy_answers_and_name_checks::LONGEST_ID;
use super::*;

/// A threshold below any step's age,
/// so a step counts as stuck at once.
const AT_ONCE: Duration = Duration::ZERO;

/// A threshold no control reaches,
/// so a step never counts as stuck.
const NEVER: Duration = Duration::from_secs(3600);

/// The age a control lets a step reach before it asks,
/// so the age is measurably above `AT_ONCE`.
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

/// A thread with no board records nothing;
/// one with a board records the step and forgets it when the guard ends.
#[test]
fn a_step_is_recorded_only_on_a_thread_with_a_board_and_only_while_it_runs() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
    });
}

/// The words of the write a client's own request is inside name the command and what the write does,
/// and no moment:
/// the client says them once the person's wait has mattered,
/// so a moment taken when the write began would be false by then.
#[test]
fn the_words_of_the_write_a_request_is_inside_carry_no_moment() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let record: Arc<LandingBoard> = board_stuck_after(NEVER);
        use_on_this_thread(Arc::clone(&record));
        assert_eq!(record.begin_group(ids(&["mine"]).as_slice()), GroupAdmission::default());
        let _detail = detail(String::from("it moves refs/heads/main to abc"));
        let _step = enter_step(String::from("git update-ref --stdin in /repo"));
        let (_key, words): (String, String) = record.inside_notice("mine").expect("the request is in the group and a write runs");
        assert!(words.contains("git update-ref --stdin") && words.contains("refs/heads/main"), "{words}");
        assert!(words.contains("never ends a write"), "the words say why it matters: {words}");
        assert!(!words.contains("ago"), "no moment is named: {words}");
    });
}

/// A step younger than the threshold is not described:
/// only a write that has outlasted what writes take is stuck.
#[test]
fn a_step_younger_than_the_threshold_is_not_described() {
    let record: Arc<LandingBoard> = board_stuck_after(NEVER);
    use_on_this_thread(Arc::clone(&record));
    let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
    assert_eq!(record.describe(), None);
}

/// The detail the landing gives the next write is part of its words,
/// and it is gone when the landing's guard ends.
#[test]
fn the_detail_of_a_write_names_the_commit_it_makes_and_ends_with_the_write() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
    });
}

/// A withdrawal is not answered here while the landing thread is not stuck:
/// it goes to the landing thread as before.
#[test]
fn a_withdrawal_is_queued_while_nothing_is_stuck() {
    let record: Arc<LandingBoard> = board();
    assert_eq!(record.withdraw_early("a"), EarlyAnswer::Queued);
}

/// A request of the group in flight is never withdrawn here,
/// because the landing may be making its commit;
/// one that has not begun
/// is withdrawn,
/// and when its group begins it is dropped and the others are marked.
#[test]
fn only_a_request_that_has_not_begun_is_withdrawn_while_the_landing_is_stuck() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
    });
}

/// A request already in flight when the landing thread's step ends is not withdrawn afterwards either:
/// the record of the step is
/// what makes a withdrawal early,
/// and without it every withdrawal is the landing thread's.
#[test]
fn a_withdrawal_after_the_stuck_write_returned_goes_to_the_landing_thread() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let record: Arc<LandingBoard> = board();
        use_on_this_thread(Arc::clone(&record));
        let guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        assert!(matches!(record.withdraw_early("a"), EarlyAnswer::Withdrawn(_)));
        drop(guard);
        assert_eq!(record.withdraw_early("b"), EarlyAnswer::Queued);
    });
}

/// An identity that is not a plain name gets no early answer and no marker,
/// so it cannot name a file outside the record's directory.
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

/// Fails closed (sol-1,
/// P1):
/// when the record cannot be written,
/// a withdrawal is never answered "withdrawn" and a request is never
/// taken as admitted on a marker that was not made.
/// The record's directory is made a plain file,
/// so every marker operation fails.
#[test]
fn a_record_that_cannot_be_written_never_answers_withdrawn() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
    });
}

/// A withdrawal is a tombstone with the identity of a normal one (sol-1,
/// P1):
/// every copy of the request that reaches a group after
/// it,
/// replays and duplicates included,
/// is dropped,
/// and not only the first.
#[test]
fn every_copy_of_a_withdrawn_request_is_dropped_not_only_the_first() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
    });
}

/// The landing and a withdrawal race for the same requests:
/// whichever way each interleaving falls,
/// a request answered "withdrawn" is
/// never admitted,
/// and one admitted is never answered "withdrawn" (sol-1,
/// P1).
/// The two sides start together behind a barrier,
/// repeatedly,
/// on fresh identities.
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

/// A journal entry that says request `id` landed as batch `seq`,
/// written the way the landing writes it.
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
        holders: vec![String::new()],
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: std::path::PathBuf::from("/repo/.git/index"),
        git_dir: std::path::PathBuf::from("/repo/.git"),
    };
    crate::service_journal::write_intent(journal, &intent).expect("the intent");
    crate::service_journal::mark_done(journal, seq).expect("the batch is done");
}

/// Withdrawer W1 wins the decision and is stopped before it asks the journal again;
/// withdrawer W2 meets the decision.
/// W2 must not
/// answer "withdrawn":
/// W1 has not confirmed it,
/// and the request lands in the meantime (sol-1 round 2,
/// P1).
/// W1 then finds the request in
/// the journal and answers "landed",
/// and W2's withdrawal is left to the landing thread.
#[test]
fn a_decision_another_withdrawer_has_not_confirmed_is_not_acknowledged() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
                        // The control releases this hook once the request has landed, which writes the journal: the wait
                        //  ends on the run's work, never on a stated span, because six such spans expired together once.
                        let _released: bool = crate::test_support::wait_until(&|| { return wait_for_release.try_recv().is_ok(); });
                    }));
                });
                return shared.withdraw_early("r");
            })
        };
        // W1 wins the decision by writing the journal, so the wait ends on the run's work and never on 30 s: a 1,034-test
        //  selection once failed this control at 42.9 s while the journal was still being written.
        assert!(
            crate::test_support::wait_until(&|| { return wait_for_won.try_recv().is_ok(); }),
            "W1 won the decision"
        );
        // W2 meets W1's decision, which W1 has not confirmed.
        assert_eq!(record.withdraw_early("r"), EarlyAnswer::Queued, "an unconfirmed decision is not acknowledged");
        // The request lands meanwhile, and the journal says so.
        record_landed(journal.as_path(), 1, "r");
        release.send(()).expect("let W1 go on");
        let answer: EarlyAnswer = first.join().expect("W1");
        assert_eq!(answer, EarlyAnswer::Landed("1".repeat(40)), "W1 asks the journal again and finds the request");
        assert!(!record.decision_path("r").exists(), "nobody acknowledged the decision, so W1 removes it");
    });
}

/// W1 wins and its second journal read fails;
/// W2 met the decision meanwhile.
/// W1 removes the decision and queues its withdrawal,
/// and
/// W2 was not told "withdrawn",
/// so the request can still be admitted and landed without any client having been told it never will
/// (sol-1 round 2,
/// P1:
/// the creator removed a decision another withdrawer had acknowledged).
#[test]
fn a_journal_that_fails_after_the_win_leaves_no_acknowledged_cancellation() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
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
                        // The control releases this hook once the request has landed, which writes the journal: the wait
                        //  ends on the run's work, never on a stated span, because six such spans expired together once.
                        let _released: bool = crate::test_support::wait_until(&|| { return wait_for_release.try_recv().is_ok(); });
                    }));
                });
                return shared.withdraw_early("r");
            })
        };
        // W1 wins the decision by writing the journal, so the wait ends on the run's work and never on 30 s: a 1,034-test
        //  selection once failed this control at 42.9 s while the journal was still being written.
        assert!(
            crate::test_support::wait_until(&|| { return wait_for_won.try_recv().is_ok(); }),
            "W1 won the decision"
        );
        let second: EarlyAnswer = record.withdraw_early("r");
        // The journal can no longer be read whole (the directory takes no listing), while the record's directory under it still works.
        std::fs::set_permissions(journal.as_path(), std::fs::Permissions::from_mode(0o300)).expect("make the journal unreadable");
        release.send(()).expect("let W1 go on");
        let first_answer: EarlyAnswer = first.join().expect("W1");
        std::fs::set_permissions(journal.as_path(), std::fs::Permissions::from_mode(0o700)).expect("restore");
        assert_eq!(first_answer, EarlyAnswer::Queued, "an unreadable journal confirms nothing");
        assert_eq!(second, EarlyAnswer::Queued, "and the other withdrawer was told nothing final");
        assert_eq!(record.begin_group(ids(&["r"]).as_slice()), GroupAdmission::default(), "the request is still admitted, not dropped");
    });
}

/// A staging file of a decision is removed whichever way the decision ends (sol-1 round 2,
/// P2:
/// a failed write returned before the
/// removal and left it).
/// The failure is injected inside the shared publish,
/// after the staging file was made and written,
/// so a
/// publish that stopped removing its staging file leaves one behind and fails this control (round 3:
/// the earlier control failed a
/// closure of its own).
#[test]
fn a_decision_whose_write_fails_leaves_no_temporary_file() {
    let directory: std::path::PathBuf = crate::test_support::fixture("service-busy-temporary");
    let record: Arc<LandingBoard> = LandingBoard::new(directory.join("journal"), AT_ONCE);
    crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = Some(Box::new(|| { return Err(std::io::Error::other("the write failed after the file was made")); })); });
    for round in 0..5 {
        assert!(record.decide("same", LANDING).is_err(), "round {round}");
    }
    crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = None; });
    let leftovers = |board: &LandingBoard| -> usize { return std::fs::read_dir(board.directory.as_path()).expect("listing").flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".partial"); }).count(); };
    assert_eq!(leftovers(&record), 0, "a failed write leaves no temporary file");
    assert!(!record.decision_path("same").exists(), "and decided nothing");
    for round in 0..20 {
        assert!(record.decide("same", LANDING).is_ok(), "round {round}");
    }
    assert_eq!(leftovers(&record), 0, "a decision that wins or loses leaves no temporary file");
}

/// Request `r` has a confirmed withdrawal,
/// and the retention removed its decision and paused before its confirmation (sol-1 round 3,
/// P1 A).
/// Withdrawer W1 wins a new decision and is stopped before its journal read;
/// W2 meets that decision beside the old
/// confirmation.
/// W2 must not answer "withdrawn":
/// the old confirmation belongs to the old decision.
/// W1 then finds the request landed.
#[test]
fn a_confirmation_that_outlived_its_decision_confirms_nothing_of_the_next_decision() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-stale-confirmation").join("journal");
        let record: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        assert!(matches!(record.withdraw_early("r"), EarlyAnswer::Withdrawn(_)), "the first withdrawal is confirmed");
        // The retention's first removal, and its pause.
        std::fs::remove_file(record.decision_path("r")).expect("the retention removed the decision");
        let seen_by_w2: Arc<std::sync::Mutex<Option<EarlyAnswer>>> = Arc::new(std::sync::Mutex::new(None));
        let sink: Arc<std::sync::Mutex<Option<EarlyAnswer>>> = Arc::clone(&seen_by_w2);
        let other: Arc<LandingBoard> = Arc::clone(&record);
        AFTER_THE_DECISION_IS_WON.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || {
                // W2, on another thread with a board of its own view, meets W1's decision beside the old confirmation.
                let board: Arc<LandingBoard> = Arc::clone(&other);
                let answer: EarlyAnswer = std::thread::spawn(move || { return board.withdraw_early("r"); }).join().expect("W2");
                *sink.lock().expect("lock") = Some(answer);
                // The request lands before W1 asks the journal again.
                record_landed(journal.as_path(), 1, "r");
            }));
        });
        let first: EarlyAnswer = record.withdraw_early("r");
        AFTER_THE_DECISION_IS_WON.with(|hook| { *hook.borrow_mut() = None; });
        assert_eq!(*seen_by_w2.lock().expect("lock"), Some(EarlyAnswer::Queued), "the old confirmation is not W1's");
        assert_eq!(first, EarlyAnswer::Landed("1".repeat(40)), "W1 finds the request in the journal");
    });
}

/// The retention works from a listing and removes by what it still finds:
/// a decision that replaced the one it listed (another token)
/// stays,
/// with its confirmation,
/// while the pair it listed is forgotten (sol-1 round 3,
/// P1 A:
/// concurrent pruners from a stale listing).
#[test]
fn the_retention_never_removes_a_decision_that_replaced_the_one_it_listed() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-stale-listing").join("journal");
        let record: Arc<LandingBoard> = LandingBoard::new(journal, AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        for round in 0..(WITHDRAWALS_RETAINED + 1) {
            assert!(matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)), "round {round}");
            std::thread::sleep(Duration::from_millis(5));
        }
        // Between the listing and the removals, the oldest request's pair is replaced by a decision of another withdrawal.
        let replaced: std::path::PathBuf = record.decision_path("r0");
        let replacement: String = format!("{WITHDRAWN}:another-token");
        let target = replaced.clone();
        AFTER_THE_LISTING.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || {
                std::fs::remove_file(target.as_path()).expect("the old decision goes");
                std::fs::write(target.as_path(), replacement.as_bytes()).expect("the new decision");
            }));
        });
        record.end_group(&[]);
        AFTER_THE_LISTING.with(|hook| { *hook.borrow_mut() = None; });
        assert_eq!(std::fs::read_to_string(replaced).expect("the replacement stays"), format!("{WITHDRAWN}:another-token"));
    });
}

/// The memory of confirmed withdrawals is bounded:
/// past the retention the oldest are forgotten,
/// decision and confirmation together.
#[test]
fn only_the_retained_number_of_confirmed_withdrawals_is_kept() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-retention").join("journal");
        let record: Arc<LandingBoard> = LandingBoard::new(journal, AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        for round in 0..(WITHDRAWALS_RETAINED + 6) {
            assert!(matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)), "round {round}");
        }
        // The retention runs on the landing thread, between two groups.
        record.end_group(&[]);
        let count = |extension: &str| -> usize { return std::fs::read_dir(record.directory.as_path()).expect("listing").flatten().filter(|entry| { return entry.path().extension().and_then(|text| { return text.to_str(); }) == Some(extension); }).count(); };
        assert_eq!(count(CANCELLED_EXTENSION), WITHDRAWALS_RETAINED);
        assert_eq!(count(DECISION_EXTENSION), WITHDRAWALS_RETAINED, "a decision never outlives its confirmation, nor the other way");
    });
}

/// The memory of confirmed withdrawals is bounded while the landing thread is stuck in a write and prunes nothing (sol-1 round 4,
/// P2):
/// past `WITHDRAWALS_CEILING` confirmed withdrawals the connection threads leave further withdrawals to the landing thread,
/// as
/// they do when nothing is stuck,
/// instead of adding a decision and a confirmation for each.
#[test]
fn the_memory_of_withdrawals_stays_bounded_while_the_landing_is_stuck() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-stuck-bound").join("journal");
        let record: Arc<LandingBoard> = LandingBoard::new(journal, AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let mut withdrawn: usize = 0;
        for round in 0..(WITHDRAWALS_CEILING + 10) {
            if matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)) {
                withdrawn += 1;
            }
        }
        let count = |extension: &str| -> usize { return std::fs::read_dir(record.directory.as_path()).expect("listing").flatten().filter(|entry| { return entry.path().extension().and_then(|text| { return text.to_str(); }) == Some(extension); }).count(); };
        assert_eq!(withdrawn, WITHDRAWALS_CEILING, "withdrawals past the bound are left to the landing thread");
        assert_eq!(count(CANCELLED_EXTENSION), WITHDRAWALS_CEILING);
        assert_eq!(count(DECISION_EXTENSION), WITHDRAWALS_CEILING);
        // The landing thread's retention brings the memory back to the retained number, and withdrawals are answered again.
        record.end_group(&[]);
        assert_eq!(count(CANCELLED_EXTENSION), WITHDRAWALS_RETAINED);
        assert!(matches!(record.withdraw_early("after"), EarlyAnswer::Withdrawn(_)));
    });
}

/// How long a control gives the board to answer about a record it must not read:
/// far above the read of a small file,
/// and as long as a client waits before it gives up on the service (`HEARTBEAT_SILENCE`).
#[cfg(unix)]
const TIME_TO_ANSWER_ABOUT_A_RECORD: Duration = Duration::from_secs(5);

/// What:
///  A record that is not a regular file is answered without being read.
/// Why:
///  The beat of every waiting client asks this board what the landing thread is inside,
/// and the question opens the step record,
///  so a named pipe at that name would make the open wait for a writer that never comes:
/// the client then hears nothing for the five seconds it allows (`HEARTBEAT_SILENCE`),
///  sends the commit again,
/// and may report it failed although it landed.
///  Nothing but this process writes these records,
/// so a pipe at one of their names is another program's,
///  and it names no write the landing thread is inside.
#[cfg(unix)]
#[test]
fn a_step_record_that_is_a_named_pipe_is_not_read() {
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let record: Arc<LandingBoard> = board();
    let step: std::path::PathBuf = record.directory.join(STEP_FILE);
    let named: std::ffi::CString = std::ffi::CString::new(step.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: `named` is a NUL-terminated path that outlives the call, and the mode is the record's own.
    assert_eq!(unsafe { libc::mkfifo(named.as_ptr(), 0o600) }, 0, "a named pipe where the step record belongs");
    let (answered, heard) = std::sync::mpsc::channel::<Option<String>>();
    let asking = std::thread::spawn(move || {
        let _sent = answered.send(record.describe());
    });
    let waited: Result<Option<String>, std::sync::mpsc::RecvTimeoutError> = heard.recv_timeout(TIME_TO_ANSWER_ABOUT_A_RECORD);
    // The pipe is let go of whatever the answer was, so a failing run leaves no thread behind: a write with no reader fails at
    //  once, and a reader that opened it reads the end of its file.
    {
        use std::io::Write as _;
        let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
        options.write(true).custom_flags(libc::O_NONBLOCK);
        if let Ok(mut half) = options.open(step.as_path()) {
            let _written = half.write_all(b"");
        }
    }
    let _removed = std::fs::remove_file(step.as_path());
    let said: Option<String> = waited.expect("the board answers about a record it does not read without waiting for a writer");
    asking.join().expect("the asking thread ends");
    assert_eq!(said, None, "a pipe names no write the landing thread is inside");
}

/// How long the landing thread,
/// paused between the check and the rename of a publish,
/// gives the connection thread to finish its own
///  publish of the same name:
/// far above the microseconds a publish takes,
/// so a connection thread that nothing holds back has
///  finished,
/// and a held one is still waiting.
#[cfg(unix)]
const TIME_FOR_THE_OTHER_PUBLISHER: Duration = Duration::from_millis(500);

/// How long a control waits for a decision that is certain to come once the landing thread has let go:
/// far above a publish,
/// so
///  only a decision that never comes ends the wait.
#[cfg(unix)]
const TIME_FOR_A_DECISION_TO_COME: Duration = Duration::from_secs(30);

/// On a filesystem that answers neither an exclusive rename nor a hard link,
/// the landing thread and a connection thread of one
///  service race for one decision with the checked rename (a check,
/// then a rename).
/// The landing thread is paused between its
///  check and its rename;
/// a connection thread decides the same request meanwhile.
/// Exactly one of them may see itself the winner,
///  and the other reads what the winner wrote (the plan,
/// "Safe file saving:
/// the publish replaced",
/// fact 1 and its settlement).
#[cfg(unix)]
#[test]
fn on_a_disk_without_exclusive_publish_the_landing_thread_and_a_connection_thread_never_both_win_a_decision() {
    use crate::private_storage::{BETWEEN_THE_CHECK_AND_THE_RENAME, THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH};
    use std::cell::RefCell;
    use std::rc::Rc;
    let record: Arc<LandingBoard> = board();
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<Decision>>();
    // What the connection thread decided, when it finished while the landing thread was paused.
    let decided_while_paused: Rc<RefCell<Option<std::io::Result<Decision>>>> = Rc::new(RefCell::new(None));
    THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH.with(|flag| { flag.set(true); });
    let pause_record: Arc<LandingBoard> = Arc::clone(&record);
    let pause_result: Rc<RefCell<Option<std::io::Result<Decision>>>> = Rc::clone(&decided_while_paused);
    let pause_receiver = Rc::new(receiver);
    let later_receiver = Rc::clone(&pause_receiver);
    BETWEEN_THE_CHECK_AND_THE_RENAME.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            let (shared_record, shared_sender) = (Arc::clone(&pause_record), sender.clone());
            std::thread::spawn(move || {
                THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH.with(|flag| { flag.set(true); });
                let _sent = shared_sender.send(shared_record.decide("request", "withdrawn:token"));
            });
            *pause_result.borrow_mut() = pause_receiver.recv_timeout(TIME_FOR_THE_OTHER_PUBLISHER).ok();
        }));
    });
    let landing: Decision = record.decide("request", LANDING).expect("the landing thread's decision");
    BETWEEN_THE_CHECK_AND_THE_RENAME.with(|hook| { *hook.borrow_mut() = None; });
    let withdrawing: Decision = match decided_while_paused.borrow_mut().take() {
        Some(early) => early,
        None => later_receiver.recv_timeout(TIME_FOR_A_DECISION_TO_COME).expect("the connection thread's decision comes once the landing thread has let go"),
    }
    .expect("the connection thread's decision");
    let content: String = std::fs::read_to_string(record.decision_path("request")).expect("the decision file");
    match (landing, withdrawing) {
        (Decision::Won, Decision::Lost(read)) => assert_eq!(read, LANDING, "the loser reads what the winner wrote"),
        (Decision::Lost(read), Decision::Won) => assert_eq!(read, content, "the loser reads what the winner wrote"),
        (Decision::Won, Decision::Won) => panic!("both threads saw themselves the winner; the file says `{content}`"),
        (Decision::Lost(_), Decision::Lost(_)) => panic!("neither thread won; the file says `{content}`"),
    }
}
