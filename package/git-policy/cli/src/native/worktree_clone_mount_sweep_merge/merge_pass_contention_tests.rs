//! What:
//!  Controls of the taker's contended paths:
//!  a held claim marks and runs nothing,
//!  a second claim attempt that wins runs the pass and drops the mark,
//!  and two takers started together run one pass between them and one extra.
//! Why:
//!  These are the races the design's tests 3 and 4 name,
//!  and each is forced by the device's own seams or by the test's own hold,
//!  never by a sleep or a delay
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does",
//!  "What the marks guarantee,
//! and what they do not",
//!  and veto entry V4).
//!  A pass that ran beside another must fail here rather than pass:
//!  the counts are the answer and no window is timed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = runAMergePass(folder, pass); expect(held.refused).toBe(true); expect(theMark(folder)).toBe('present');
//! ```

use crate::test_support::fixture;
use crate::worktree_clone_mount_sweep_merge::merge_pass::{
    MOST_PASSES,
    run_a_merge_pass,
    run_a_merge_pass_seamed,
};
use crate::worktree_clone_mount_sweep_merge::pass_mark::{
    MarkReading,
    the_mark_is_there,
};
use crate::worktree_clone_mount_sweep_merge::run_claim::{
    ClaimAnswer,
    HeldRunClaim,
    take_the_run_claim,
};
use crate::worktree_clone_mount_sweep_merge::sweeps_folder::SweepsFolder;
use std::path::PathBuf;
use std::sync::mpsc::{
    Receiver,
    Sender,
    channel,
};

/// This test's folder,
/// which the thread's fixtures remove however the control ends.
fn folder(name: &str) -> SweepsFolder {
    let root: PathBuf = fixture(name);
    return SweepsFolder::at(root.join("sweeps"), rustix::process::geteuid().as_raw());
}

/// The report one pass answers with.
const REPORT: &str = "attached one clone";

/// A trigger whose claim is held by this test runs no pass and leaves the mark.
#[test]
fn a_held_claim_marks_and_runs_no_pass() {
    let folder: SweepsFolder = folder("merge-held");
    let ClaimAnswer::Held(hold) = take_the_run_claim(&folder) else {
        panic!("a free claim is taken");
    };
    let mut runs: usize = 0;
    let answer: String = run_a_merge_pass(&folder, || {
        runs += 1;
        return Ok(String::from(REPORT));
    })
    .expect_err("a held trigger did no work");
    assert_eq!(runs, 0, "a held trigger runs no pass");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Present);
    assert!(
        answer.contains("another event-driven pass holds the run claim"),
        "the answer names the holder: {answer}"
    );
    assert!(
        answer.contains("another-pass-is-due stands"),
        "the answer says the mark covers the request: {answer}"
    );
    drop(hold);
}

/// A held claim whose second attempt wins runs the pass and drops the mark
/// that attempt wrote.
#[test]
fn a_held_claim_whose_second_attempt_wins_runs_the_pass() {
    let folder: SweepsFolder = folder("merge-second-attempt");
    let ClaimAnswer::Held(hold) = take_the_run_claim(&folder) else {
        panic!("a free claim is taken");
    };
    let mut held: Option<HeldRunClaim> = Some(hold);
    let mut let_go = || {
        let Some(claim) = held.take() else {
            return;
        };
        claim.release().expect("a claim this account may release");
    };
    let mut runs: usize = 0;
    let answer: String = run_a_merge_pass_seamed(
        &folder,
        || {
            runs += 1;
            return Ok(String::from(REPORT));
        },
        &mut let_go,
        &mut || {},
    )
    .expect("a pass the second attempt won");
    assert_eq!(runs, 1, "the second attempt is the one that runs the pass");
    assert!(answer.contains(REPORT), "the report is kept: {answer}");
    assert_eq!(
        the_mark_is_there(&folder),
        MarkReading::Absent,
        "the pass removed the mark its own attempt wrote"
    );
}

/// Two takers started together run one pass at a time:
/// the second finds the claim held and marks,
/// and the first answers that mark with one more pass.
#[test]
fn two_takers_started_together_run_one_pass_and_one_extra() {
    let folder: SweepsFolder = folder("merge-two-takers");
    let (entered, entered_rx): (Sender<()>, Receiver<()>) = channel();
    let (may_finish, may_finish_rx): (Sender<()>, Receiver<()>) = channel();
    let mut second_runs: usize = 0;
    let worker_folder: SweepsFolder = folder.clone();
    std::thread::scope(|scope| {
        let worker = scope.spawn(move || {
            let mut handshakes: usize = 0;
            let mut first_runs: usize = 0;
            let answer: Result<String, String> = run_a_merge_pass(&worker_folder, || {
                handshakes += 1;
                // The other taker runs while this pass is inside the claim, so the interleaving is forced by the
                // control and never by a delay.
                if handshakes == 1 {
                    entered.send(()).expect("the control waits for this pass");
                    may_finish_rx
                        .recv()
                        .expect("the control lets this pass finish");
                }
                first_runs += 1;
                return Ok(String::from(REPORT));
            });
            return (answer, first_runs);
        });
        entered_rx.recv().expect("the first taker entered its pass");
        let held: String = run_a_merge_pass(&folder, || {
            second_runs += 1;
            return Ok(String::from(REPORT));
        })
        .expect_err("the second taker ran no pass");
        assert!(
            held.contains("another event-driven pass holds the run claim"),
            "the second taker names the pass it found: {held}"
        );
        assert_eq!(
            the_mark_is_there(&folder),
            MarkReading::Present,
            "the second taker left its request as the mark"
        );
        may_finish.send(()).expect("the first pass may finish");
        let (answer, first_runs): (Result<String, String>, usize) =
            worker.join().expect("the first taker's thread");
        let answer: String = answer.expect("a pass that answered 0");
        assert!(
            answer.contains("one more pass"),
            "the first taker answered the mark with one more pass: {answer}"
        );
        assert_eq!(
            first_runs, MOST_PASSES,
            "one pass and one more, and no third"
        );
    });
    assert_eq!(second_runs, 0, "the held taker ran no pass");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Absent);
}
