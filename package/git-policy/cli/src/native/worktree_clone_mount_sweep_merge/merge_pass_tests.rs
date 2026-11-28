//! What:
//!  Controls of one invocation's taker:
//!  a free claim runs one pass and gives the claim back,
//!  a mark buys exactly one more pass,
//!  a mark that survives the second pass is left standing,
//!  a pass that refuses answers two,
//!  and a device that could not be read is reported and the pass runs.
//! Why:
//!  These are the counts the design adopts with the project's veto open:
//!  one pass and one more,
//!  never a third,
//!  and never a refusal from a cost record
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does";
//!  veto entries V4,
//!  V7 and V12).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runAMergePass(folder, () => { runs += 1; return ['a pass']; });
//! ```

use super::{
    MOST_PASSES,
    SweepsFolder,
    run_a_merge_pass,
    run_a_merge_pass_seamed,
    take_the_run_claim,
};
use crate::test_support::fixture;
use crate::worktree_clone_mount_sweep_merge::pass_mark::{
    MarkReading,
    the_mark_is_there,
    write_the_mark,
};
use std::path::PathBuf;

/// This test's folder,
/// which the thread's fixtures remove however the control ends.
fn folder(name: &str) -> SweepsFolder {
    let root: PathBuf = fixture(name);
    return SweepsFolder::at(root.join("sweeps"), rustix::process::geteuid().as_raw());
}

/// The report one pass answers with.
const REPORT: &str = "attached one clone";

/// What one control's pass did,
/// and what it answers.
struct CountingPass {
    /// How many times the pass ran.
    runs: usize,
    /// Whether a mark was written from inside the pass's own first run.
    mark_from_the_run: bool,
    /// The report the pass answers with.
    report: Result<String, String>,
}

impl CountingPass {
    /// A pass that answers `REPORT` and marks nothing.
    fn answering() -> Self {
        return CountingPass {
            runs: 0,
            mark_from_the_run: false,
            report: Ok(String::from(REPORT)),
        };
    }

    /// One run of the pass:
    /// the count,
    /// and a mark when this control asked for one.
    fn run(
        &mut self,
        folder: &SweepsFolder,
    ) -> Result<String, String> {
        self.runs += 1;
        if self.mark_from_the_run {
            write_the_mark(folder).expect("a mark this account may write");
        }
        return self.report.clone();
    }
}

/// A free claim runs one pass and gives the claim back,
/// and the answer carries the pass's own report.
#[test]
fn a_free_claim_runs_one_pass_and_gives_it_back() {
    let folder: SweepsFolder = folder("merge-one-pass");
    let mut pass: CountingPass = CountingPass::answering();
    let answer: String =
        run_a_merge_pass(&folder, || return pass.run(&folder)).expect("a pass that answered 0");
    assert_eq!(pass.runs, 1, "one trigger is one pass");
    assert!(
        answer.contains(REPORT),
        "the answer keeps the report: {answer}"
    );
    let taken = take_the_run_claim(&folder);
    let super::ClaimAnswer::Held(claim) = taken else {
        panic!("the claim the pass gave back is free again");
    };
    assert_eq!(claim.path(), folder.run_claim_file(2).as_path());
}

/// A mark written after the first pass's release buys one more pass,
/// and the second pass removes the mark it found.
#[test]
fn a_mark_after_the_last_look_buys_one_more_pass() {
    let folder: SweepsFolder = folder("merge-extra-pass");
    let mut pass: CountingPass = CountingPass::answering();
    let mut write_it = || {
        write_the_mark(&folder).expect("a mark this account may write");
    };
    let answer: String = run_a_merge_pass_seamed(
        &folder,
        || return pass.run(&folder),
        &mut || {},
        &mut write_it,
    )
    .expect("two passes that answered 0");
    assert_eq!(pass.runs, MOST_PASSES, "a mark buys exactly one more pass");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Absent);
    assert!(
        answer.contains("one more pass"),
        "the answer says why it ran twice: {answer}"
    );
}

/// A mark that survives the second pass is left standing for the next trigger,
/// and the answer says so.
#[test]
fn a_mark_that_survives_the_second_pass_is_left_standing() {
    let folder: SweepsFolder = folder("merge-mark-survives");
    let mut pass: CountingPass = CountingPass::answering();
    pass.mark_from_the_run = true;
    let answer: String =
        run_a_merge_pass(&folder, || return pass.run(&folder)).expect("a pass that answered 0");
    assert_eq!(
        pass.runs, MOST_PASSES,
        "a mark written by every pass buys no third pass"
    );
    assert_eq!(the_mark_is_there(&folder), MarkReading::Present);
    assert!(
        answer.contains("next trigger"),
        "the answer says who covers the request: {answer}"
    );
}

/// A pass whose own answer is a refusal answers two,
/// with the device's lines before it.
#[test]
fn a_pass_that_refuses_answers_two() {
    let folder: SweepsFolder = folder("merge-refused");
    let mut pass: CountingPass = CountingPass::answering();
    pass.report = Err(String::from("one clone refused"));
    let answer: String =
        run_a_merge_pass(&folder, || return pass.run(&folder)).expect_err("a pass that refused");
    assert_eq!(pass.runs, 1);
    assert!(
        answer.contains("one clone refused"),
        "the pass's own refusal is kept: {answer}"
    );
}

/// A device that could not be read is reported and the pass runs anyway.
#[test]
fn a_device_that_could_not_be_read_is_reported_and_the_pass_runs() {
    let root: PathBuf = fixture("merge-unreadable");
    let blocker: PathBuf = root.join("sweeps");
    std::fs::write(blocker.as_path(), b"not a folder").expect("a file where the folder goes");
    let folder: SweepsFolder = SweepsFolder::at(blocker, rustix::process::geteuid().as_raw());
    let mut pass: CountingPass = CountingPass::answering();
    let answer: String =
        run_a_merge_pass(&folder, || return pass.run(&folder)).expect("a pass that answered 0");
    assert_eq!(pass.runs, 1, "the pass runs without the device");
    assert!(
        answer.contains("could not be read"),
        "the answer names the record it could not read: {answer}"
    );
    assert!(
        answer.contains("the per-slot claim is safety"),
        "the answer says what keeps the work safe: {answer}"
    );
    assert!(answer.contains(REPORT), "the report is kept: {answer}");
}
