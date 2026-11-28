//! What:
//!  The controls of the line a bound slot's removal says before it runs:
//!  the order of the line and the removal,
//!  and a terminal that refuses the line.
//! Why:
//!  The order is the whole of what this file owes a person,
//!  and a closed terminal is the case that would otherwise keep a removal from
//!  running,
//!  so each is held here rather than argued from the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sink = new SharedSink(); const answer = sayThenRunTheRemoval({ said, saidTo: sink, theRemoval });
//! expect(seenAtTheRemoval).toBe(said);
//! ```

use super::{
    TheOpeningLinesArm,
    TheRemovalSaidBeforeItRuns,
    TheRemovalsOpeningLine,
    say_then_run_the_removal,
    the_opening_line,
};
use crate::test_support::AStreamThatRefusesEveryByte;
use crate::test_support::SharedSink;
use crate::worktree_clone_removal_proof_and_plan::removal_command_path::{
    REMOVED_EXIT_CODE,
    TheRemovalAnswer,
};
use std::cell::RefCell;
use std::path::Path;

/// The line a control says,
/// which is the release's own opening words,
/// so a reader of the control reads the form a person meets.
const THE_LINE: &str =
    "the removal of `fix-flaky-test` runs here: a whole managed clone is bound to this slot\n";

/// The report the removal a control runs answers with.
const THE_REPORT: &str = "the clone, its snapshots and its records are destroyed\n";

/// What:
///  The line is out before the removal runs,
///  and the removal's own answer is what comes back.
/// Why:
///  A person waiting at a terminal reads the line only if it is written before
///  the work starts,
///  so the control reads what the stream held at the moment the removal was
///  entered.
#[test]
fn the_line_is_out_before_the_removal_runs() {
    let sink: SharedSink = SharedSink::default();
    let mut said_to: SharedSink = sink.clone();
    let seen_at_the_removal: RefCell<String> = RefCell::new(String::new());
    let answer: TheRemovalAnswer = say_then_run_the_removal(TheRemovalSaidBeforeItRuns {
        said: THE_LINE,
        said_to: &mut said_to,
        the_removal: &|| {
            *seen_at_the_removal.borrow_mut() = sink.text();
            return TheRemovalAnswer {
                lines: String::from(THE_REPORT),
                code: REMOVED_EXIT_CODE,
            };
        },
    });
    assert_eq!(
        seen_at_the_removal.into_inner().as_str(),
        THE_LINE,
        "the line the person reads stood in the stream before the removal was entered"
    );
    assert_eq!(answer.lines, THE_REPORT, "the removal's own report travels");
    assert_eq!(
        answer.code, REMOVED_EXIT_CODE,
        "the removal's own status travels"
    );
    assert_eq!(
        sink.text().as_str(),
        THE_LINE,
        "the line is written once, and nothing else was written to the stream"
    );
}

/// What:
///  A terminal that refuses the line keeps no removal from running,
///  and the refusal itself is not reported as the removal's answer.
/// Why:
///  A closed terminal is not a reason to keep a person's removal from running,
///  which is the rule every notice of this crate follows.
#[test]
fn a_terminal_that_refuses_the_line_still_lets_the_removal_run() {
    let mut refusing: AStreamThatRefusesEveryByte = AStreamThatRefusesEveryByte;
    let ran: RefCell<bool> = RefCell::new(false);
    let answer: TheRemovalAnswer = say_then_run_the_removal(TheRemovalSaidBeforeItRuns {
        said: THE_LINE,
        said_to: &mut refusing,
        the_removal: &|| {
            *ran.borrow_mut() = true;
            return TheRemovalAnswer {
                lines: String::from(THE_REPORT),
                code: REMOVED_EXIT_CODE,
            };
        },
    });
    assert!(
        ran.into_inner(),
        "the removal runs although the terminal refused the line"
    );
    assert_eq!(answer.lines, THE_REPORT, "the removal's own report travels");
}

/// What:
///  The live line says that the removal runs,
///  that Git does not,
///  that it ends no holder,
///  and how many forces this command carried,
///  and the refused line says the pool is the caller's own answer and removes nothing.
/// Why:
///  The line is what tells a person which of the two arms they met,
///  and it is the only place the force count a person typed is said back to
///  them,
///  so a control holds what each arm says rather than the code holding it alone.
#[test]
fn each_arm_says_what_it_promises() {
    let target: &Path = Path::new("/repo/.claude/worktrees/fix-flaky-test");
    let live: String = the_opening_line(&TheRemovalsOpeningLine {
        target,
        plan: None,
        forces: 2,
        arm: TheOpeningLinesArm::Live,
    });
    for said in [
        "runs here: a whole managed clone is bound to this slot",
        "Git does not run",
        "ends no holder and forces nothing beyond what the command's own force count passes",
        "`2` force(s) were counted in the words, where two pass every refusal the removal has",
    ] {
        assert!(live.contains(said), "the live line says {said}: {live}");
    }
    let plan: &Path =
        Path::new("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/1234");
    let live_again: String = the_opening_line(&TheRemovalsOpeningLine {
        target,
        plan: Some(plan),
        forces: 0,
        arm: TheOpeningLinesArm::Live,
    });
    assert!(
        live_again.contains(plan.display().to_string().as_str()),
        "the live line names the plan this run resumes: {live_again}"
    );
    assert!(
        live_again.contains("so this run resumes it"),
        "the live line says the run resumes that plan: {live_again}"
    );
    let refused: String = the_opening_line(&TheRemovalsOpeningLine {
        target,
        plan: None,
        forces: 0,
        arm: TheOpeningLinesArm::Refused,
    });
    for said in [
        "is refused: a whole managed clone is bound to this slot",
        "The pool here is the caller's own answer and not this machine's device",
        "nothing was removed and Git did not run",
    ] {
        assert!(
            refused.contains(said),
            "the refused line says {said}: {refused}"
        );
    }
}
