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
    return LandingBoard::a_board_that_took_its_place(crate::test_support::fixture("service-busy-board").join("journal-that-does-not-exist"), slow_after);
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
    let first: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
    assert_eq!(first.begin_group(&ids(&["old"])), GroupAdmission::default());
    assert!(first.is_in_flight("old"));
    let second: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal, AT_ONCE);
    assert!(!second.is_in_flight("old"));
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
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
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
        let record: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
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
        assert!(record.withdrawal_of("r").is_none(), "nobody acknowledged the decision, so W1 removes it");
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
        let record: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
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

/// What:
///  An admission and a decision need no file of the record,
///  and leave none:
///  the directory the record used to keep is a plain file the filesystem refuses to open,
///  and both still work.
/// Why:
///  The record was a write and then a read of the service's state directory,
/// and both stood on the path a commit waits on
/// (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`);
/// this control fails if anything of the decision reaches for that directory again.
#[test]
fn an_admission_and_a_decision_need_no_file_of_the_record() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let record: Arc<LandingBoard> = board();
        use_on_this_thread(Arc::clone(&record));
        let _step = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let entries = |board: &LandingBoard| -> usize { return std::fs::read_dir(board.directory.as_path()).expect("the record's directory").count(); };
        let before: usize = entries(&record);
        // The directory takes no new file, as a full disk or a refused write leaves it, and the step record it already holds
        //  is still read: the early answer needs that read and nothing else.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(record.directory.as_path(), std::fs::Permissions::from_mode(0o500)).expect("make the directory read-only");
        let admitted: GroupAdmission = record.begin_group(ids(&["admitted"]).as_slice());
        let held: EarlyAnswer = record.withdraw_early("admitted");
        let decided: EarlyAnswer = record.withdraw_early("decided");
        let after: usize = entries(&record);
        std::fs::set_permissions(record.directory.as_path(), std::fs::Permissions::from_mode(0o700)).expect("restore");
        assert_eq!(admitted, GroupAdmission::default(), "an admission needs no file");
        assert_eq!(held, EarlyAnswer::Queued, "a request the landing thread holds is left to it");
        assert!(matches!(decided, EarlyAnswer::Withdrawn(_)), "a withdrawal of any other request is decided with no file at all");
        assert_eq!(before, after, "and neither of them leaves a file in the record's directory");
        record.end_group(ids(&["admitted"]).as_slice());
    });
}

/// The memory of confirmed withdrawals is bounded:
/// past the retention the oldest are forgotten.
#[test]
fn only_the_retained_number_of_confirmed_withdrawals_is_kept() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-retention").join("journal");
        let record: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal, AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        for round in 0..(WITHDRAWALS_RETAINED + 6) {
            assert!(matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)), "round {round}");
        }
        assert_eq!(record.confirmed_withdrawals(), WITHDRAWALS_RETAINED + 6, "nothing is forgotten while the landing goes on");
        // The retention runs on the landing thread, between two groups.
        record.end_group(&[]);
        assert_eq!(record.confirmed_withdrawals(), WITHDRAWALS_RETAINED);
        assert!(record.withdrawal_of("r0").is_none(), "the oldest decision goes");
        assert!(record.withdrawal_of(format!("r{}", WITHDRAWALS_RETAINED + 5).as_str()).is_some(), "the newest stays");
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
        let record: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal, AT_ONCE);
        use_on_this_thread(Arc::clone(&record));
        let _guard = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let mut withdrawn: usize = 0;
        for round in 0..(WITHDRAWALS_CEILING + 10) {
            if matches!(record.withdraw_early(format!("r{round}").as_str()), EarlyAnswer::Withdrawn(_)) {
                withdrawn += 1;
            }
        }
        assert_eq!(withdrawn, WITHDRAWALS_CEILING, "withdrawals past the bound are left to the landing thread");
        assert_eq!(record.confirmed_withdrawals(), WITHDRAWALS_CEILING);
        // The landing thread's retention brings the memory back to the retained number, and withdrawals are answered again.
        record.end_group(&[]);
        assert_eq!(record.confirmed_withdrawals(), WITHDRAWALS_RETAINED);
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

/// What:
///  Two connection threads decide the withdrawal of one request at the same moment,
///  and exactly one of them wins it.
/// Why:
///   The decision is one insert under the board's lock,
/// and such a decision is the whole of what the publish of a file gave before it
/// (`service_busy.rs`,
/// `decide`);
/// a control that saw both threads win would let two clients be told the withdrawal is theirs,
/// and one of them would then be told the commit never lands while it might.
#[cfg(unix)]
#[test]
fn two_connection_threads_never_both_win_a_decision() {
    let record: Arc<LandingBoard> = board();
    for round in 0..200 {
        let id: String = format!("decided-{round}");
        let barrier: Arc<std::sync::Barrier> = Arc::new(std::sync::Barrier::new(2));
        let mut deciding: Vec<std::thread::JoinHandle<Decision>> = Vec::new();
        for token in ["first", "second"] {
            let (shared_record, shared_barrier, shared_id, own) = (Arc::clone(&record), Arc::clone(&barrier), id.clone(), String::from(token));
            deciding.push(std::thread::spawn(move || {
                shared_barrier.wait();
                return shared_record.decide(shared_id.as_str(), own.as_str());
            }));
        }
        let mut won: usize = 0;
        let mut lost: Vec<String> = Vec::new();
        for thread in deciding {
            match thread.join().expect("a deciding thread") {
                Decision::Won => won += 1,
                Decision::Lost(theirs) => lost.push(theirs),
            }
        }
        assert_eq!(won, 1, "round {round}: exactly one thread decides a request");
        assert_eq!(lost.len(), 1, "round {round}: and the other reads what the winner decided");
        assert_eq!(record.withdrawal_of(id.as_str()).as_deref(), Some(lost[0].as_str()), "round {round}");
        record.end_group(&[]);
    }
}

/// What:
///  A wait shorter than the bound names nothing,
///  the wait that reaches it names the last write the landing thread was inside,
///  and one that follows no recorded write says so.
/// Why:
///   A wait over the bound must name the step it is in, in the log and to the person waiting,
/// and the service that cannot name one says that instead of saying nothing
/// (`wait_ceilings.rs`, `A_QUEUE_WAIT_NAMED_AFTER`).
#[test]
fn a_wait_over_the_bound_names_the_write_it_was_in_when_the_group_began() {
    let record: Arc<LandingBoard> = board();
    let bound: Duration = crate::wait_ceilings::A_QUEUE_WAIT_NAMED_AFTER;
    assert_eq!(record.words_of_a_queue_wait("a-request", bound - Duration::from_millis(1), Some(0)), None, "a wait inside the bound is named by the answered line alone");
    // The request is taken with nothing recorded, so the words say that.
    assert_eq!(record.begin_group(ids(&["a-request"]).as_slice()), GroupAdmission::default());
    let unrecorded: String = record.words_of_a_queue_wait("a-request", bound, Some(0)).expect("a wait at the bound is named");
    assert!(unrecorded.contains("had recorded no write"), "{unrecorded}");
    assert!(unrecorded.contains("nothing of its own batch was ahead of it"), "{unrecorded}");
    // What: The landing's own writes must not rename the wait's cause.
    // Why:  The cause is what the landing thread had left when the wait ended, at the take; read at the answer instead,
    //       the words name the landing's own write, which the queue never stood behind.
    record.note_the_write_it_left("git update-ref --stdin in /main-by-the-landing");
    let still_unrecorded: String = record.words_of_a_queue_wait("a-request", bound, Some(0)).expect("a wait at the bound is named");
    assert_eq!(still_unrecorded, unrecorded, "a write of the landing's own does not rename what its queue wait was in");
    record.end_group(ids(&["a-request"]).as_slice());
    // A request taken after a write of the previous group names that write, with how long it had been over.
    record.note_the_write_it_left("git update-ref --stdin in /repo");
    assert_eq!(record.begin_group(ids(&["another-request"]).as_slice()), GroupAdmission::default());
    let named: String = record.words_of_a_queue_wait("another-request", bound * 2, Some(3)).expect("a wait past the bound is named");
    assert!(named.contains("git update-ref --stdin in /repo"), "{named}");
    assert!(!named.contains("had recorded no write"), "{named}");
    assert!(named.contains("3 commits of the service were still waiting"), "{named}");
    let unknown: String = record.words_of_a_queue_wait("another-request", bound, None).expect("a wait at the bound is named");
    assert!(unknown.contains("is not known to the board"), "{unknown}");
    record.end_group(ids(&["another-request"]).as_slice());
}

/// What:
///  A withdrawal that reaches a second service of one journal, for a request the first service holds, is never
///  confirmed by the second.
/// Why:
///  A decision kept in one service's memory holds only where one service is the only one:
///  the journal's batch claims exist because two services over one journal is a case it carries
///  (`doc/planning/cli-git-resident-service.md`),
///  and a service that does not hold the request, with another live service over the same journal, cannot promise
///  that the request will never land.
///  A `Withdrawn` there would tell a person their commit never lands, and the other service could land it.
#[test]
fn a_second_service_of_one_journal_never_confirms_a_withdrawal_it_does_not_hold() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-two-services").join("journal");
        // Both services are alive before either takes the request, as during a service handover or a development run.
        let landing: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
        let withdrawing: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
        use_on_this_thread(Arc::clone(&landing));
        let _step = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        assert_eq!(landing.begin_group(ids(&["request-in-flight"]).as_slice()), GroupAdmission::default());
        assert!(landing.is_in_flight("request-in-flight"), "the first service holds the request");
        let answer: EarlyAnswer = withdrawing.withdraw_early("request-in-flight");
        assert_eq!(
            answer,
            EarlyAnswer::Queued,
            "the second service cannot promise that a request another live service of this journal holds will never land",
        );
        // What: With the other service gone, the same withdrawal is confirmed at once.
        // Why:  Nothing else can land the request then, so the promise is this service's to keep, and a client is not
        //       made to wait for the answer it can have.
        //       The step record holds its board, so it is let go before the board is, and the second service's own step
        //       is what its early answer needs.
        drop(_step);
        drop(landing);
        use_on_this_thread(Arc::clone(&withdrawing));
        let _mine = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let alone: EarlyAnswer = withdrawing.withdraw_early("request-in-flight");
        assert!(matches!(alone, EarlyAnswer::Withdrawn(_)), "with no other service of this journal, the withdrawal is confirmed: {alone:?}");
    });
}

/// What:
///  A service that could not take a place of its own in its journal never promises that a request it does not hold will
///  never land.
/// Why:
///  A place it could not take is "could not tell", and "could not tell" is not the best case:
///  with another service's one live place in the journal, a count alone would call the caller alone and let it answer
///  `Withdrawn` for a request the other service can still land
///  (the third review of this branch, finding 1).
#[test]
fn a_service_with_no_place_of_its_own_never_promises_a_request_will_not_land() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-no-place-of-its-own").join("journal");
        std::fs::create_dir_all(journal.as_path()).expect("the journal");
        // The first service takes the only place there can be for now.
        let landing: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
        let services: std::path::PathBuf = journal.join("services");
        // What: A services directory that can be read and not written, as a full or read-only journal leaves it.
        // Why:  This is the state in which a service that starts now cannot take a place of its own.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(services.as_path(), std::fs::Permissions::from_mode(0o500)).expect("refuse new places");
        // This one cannot take its place, which is what the control is about.
        let without_a_place: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
        assert!(!without_a_place.take_its_place_within(Duration::from_secs(5)), "the second service took no place");
        std::fs::set_permissions(services.as_path(), std::fs::Permissions::from_mode(0o700)).expect("restore the fixture");
        assert!(!without_a_place.may_promise_a_request_never_lands(), "a service with no place of its own cannot promise anything");
        // A request it does not hold, while the other service is inside a write, is left to the landing thread.
        use_on_this_thread(Arc::clone(&landing));
        let _step = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        let answer: EarlyAnswer = without_a_place.withdraw_early("a-request");
        assert_eq!(answer, EarlyAnswer::Queued, "it may not say that a request another service can land will never land");
        drop(_step);
    });
}

/// What:
///  A service that took no place of its own in its journal lands nothing, so a withdrawal another service confirmed is
///  never contradicted by it,
///  and the other service's promise is sound because the unregistered one serves nothing.
/// Why:
///  A place it could not take is "I cannot tell whether I am alone", and such a service cannot promise anything
///  (`may_promise_a_request_never_lands`) and cannot land on its own word either:
///  another service that took its place may have confirmed a withdrawal for the same request, and this one would land
///  it (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`,
///  and the fourth review of this branch, finding 1).
#[test]
fn a_service_that_took_no_place_of_its_own_lands_nothing() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let journal: std::path::PathBuf = crate::test_support::fixture("service-busy-no-place-landing").join("journal");
        std::fs::create_dir_all(journal.as_path()).expect("the journal");
        let services: std::path::PathBuf = journal.join("services");
        std::fs::create_dir_all(services.as_path()).expect("the places directory");
        // What: A places directory that can be read and not written, as a full or read-only journal leaves it.
        // Why:  This is the state in which a service that starts now cannot take its place.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(services.as_path(), std::fs::Permissions::from_mode(0o500)).expect("refuse new places");
        let unregistered: Arc<LandingBoard> = LandingBoard::new(journal.clone(), AT_ONCE);
        // The service takes its place after its acceptor starts (`take_its_place_within`), and this one cannot.
        let within: Duration = Duration::from_secs(5);
        assert!(!unregistered.take_its_place_within(within), "the first service took no place");
        std::fs::set_permissions(services.as_path(), std::fs::Permissions::from_mode(0o700)).expect("permit the next place");
        let withdrawing: Arc<LandingBoard> = LandingBoard::a_board_that_took_its_place(journal.clone(), AT_ONCE);
        assert!(withdrawing.take_its_place_within(within), "the second service took its own");
        assert!(!unregistered.holds_its_place(), "the first service took no place");
        assert!(withdrawing.holds_its_place(), "the second service took its own");
        use_on_this_thread(Arc::clone(&withdrawing));
        let _stuck = enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(A_MOMENT);
        // What: The second service may say the request will never land, and the first must then not land it.
        // Why:  A service that took no place serves nothing, so the journal's one live place is the whole of who can
        //       land the request; the promise is sound exactly because the unregistered service lands nothing.
        let answer: EarlyAnswer = withdrawing.withdraw_early("a-request");
        assert!(matches!(answer, EarlyAnswer::Withdrawn(_)), "the journal's only live service may say the request will never land: {answer:?}");
        drop(_stuck);
        // What: And the service that took no place lands nothing of that request.
        // Why:  Its own memory holds no withdrawal, so only its refusal keeps it from landing one another service may
        //       have been asked about.
        use_on_this_thread(Arc::clone(&unregistered));
        let admission: GroupAdmission = unregistered.begin_group(&[String::from("a-request")]);
        assert_eq!(admission.refused, ids(&["a-request"]), "a service with no place of its own refuses to land");
        assert!(!unregistered.is_in_flight("a-request"), "and holds nothing for its landing");
    });
}
