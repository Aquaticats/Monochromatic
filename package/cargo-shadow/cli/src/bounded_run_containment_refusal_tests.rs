//! What:
//! What a refusal says when a stop of a bare process group,
//! or a stop of a scope whose direct child did not end,
//! could not settle the run:
//! which wait was waited,
//! or that none was,
//! and how a reason kept from an earlier failed stop is not lost.
//! Why:
//! Issue 6528's review (Low 2,
//! Low 3 and Low 9):
//! the process-group reasons named a bound that was not the wait
//! (a group already shown gone,
//! and a group that was never named),
//! a `try_wait` the kernel refused was worded as a five-second bound,
//! and `lose` replaced the reason of an earlier failed stop.
//! The weaker path is a process group of the test's own,
//! so none of these controls needs a delegated scope.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('what a refusal says when a stop could not settle the run', () => { ... })
//! ```

/// Borrows the state one failed stop writes its reason into.
use std::cell::{
    Cell,
    RefCell,
};
/// Starts a program in a process group of its own.
use std::os::unix::process::CommandExt;
/// Owns the journal path a stand-in scope state holds.
use std::path::PathBuf;
/// Starts the program the weaker path stops.
use std::process::Command;
/// The bound the direct child is given to be reaped,
/// and the bound the process group is given to be shown gone.
use std::time::Duration;

/// The open scope of one run as this process remembers it.
use crate::bounded_run::ScopeState;
/// The one stop that ends a run whatever its containment is.
use crate::bounded_run::stop_everything;
/// The bounds the refusals must state.
use crate::bounded_run::{
    QUIESCE_BOUND,
    REAP_BOUND,
};
/// The state of a scope,
/// and the two stops whose reasons these controls judge.
use crate::bounded_run_containment::{
    Containment,
    GroupStop,
    ReapShortfall,
    Reaped,
    lose,
    refusal_of_scope,
};
/// What a stop of a scope found,
/// and the listing it found it from.
use crate::control_group_stop::report::{
    Listing,
    StopReport,
};

/// The fence of a test build that makes the reaping of a direct child report that its bound passed.
use crate::failpoints::{
    Point,
    arm,
};
/// The signal the test sends the program it reaps itself,
/// the call that reaps it,
/// and the wait options of that call.
use rustix::process::{
    Pid,
    Signal,
    WaitOptions,
    kill_process_group,
    waitpid,
};

/// What:
/// `fn a_program_in_a_group_of_its_own() -> std::process::Child` starts a program that lives until it is killed,
/// in a process group of its own.
/// Why:
/// A stop of the weaker path signals the group,
/// so the group must be the program's own and not the test binary's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aProgramInAGroupOfItsOwn(): ChildProcess  // spawn('sleep', ['600'], { detached: true })
/// ```
fn a_program_in_a_group_of_its_own() -> std::process::Child {
    let mut command = Command::new("sleep");
    command.arg("600");
    command.process_group(0);
    return command
        .spawn()
        .expect("starts a program in a group of its own");
}

/// Issue 6528's review,
/// Low 2:
/// a bound that passed on the reaping of a program that is still a zombie leaves the group it led present,
/// so the refusal names that bound as the one it waited,
/// and does not claim the program's end was waited for.
#[test]
fn a_process_group_whose_program_was_not_reaped_is_not_shown_gone() {
    let mut child = a_program_in_a_group_of_its_own();
    arm(Point::ReapStuck);

    let refusal: String = stop_everything(&Containment::ProcessGroup, &mut child)
        .expect_err("a program that was not reaped is not a settled run");
    let _ = child.wait();

    assert!(
        !crate::failpoints::fires(Point::ReapStuck),
        "the reaping fault was never reached, so the refusal has another cause"
    );
    assert!(
        refusal.contains(&format!(
            "the members of the process group could not be shown gone within {QUIESCE_BOUND:?}"
        )),
        "the refusal does not state the bound it waited on the group: {refusal}"
    );
    assert!(
        !refusal.contains("did not end within"),
        "the refusal states a wait that was not the wait: {refusal}"
    );
}

/// Issue 6528's review,
/// Low 2 and Low 3:
/// the program was reaped by someone else,
/// so the group it led is gone and its exit can no longer be read.
/// The refusal must say the group is gone and that the exit could not be read,
/// and must not state a five-second wait that never happened.
/// No test-only switch is needed:
/// the test reaps the program itself,
/// as another waiter would.
#[test]
fn a_process_group_that_is_gone_and_whose_exit_could_not_be_read_says_so() {
    let mut child = a_program_in_a_group_of_its_own();
    let group: Pid =
        Pid::from_raw(i32::try_from(child.id()).expect("an id that fits")).expect("a group id");
    let _ = kill_process_group(group, Signal::KILL);
    let reaped = waitpid(Some(group), WaitOptions::empty());
    assert!(
        reaped.is_ok_and(|found| return found.is_some()),
        "the program was not killed, so the group is not gone: {reaped:?}"
    );

    let refusal: String = stop_everything(&Containment::ProcessGroup, &mut child)
        .expect_err("a program whose exit could not be read is not a settled run");

    assert!(
        refusal.contains("no member of the process group is left"),
        "the refusal does not say the group itself is gone: {refusal}"
    );
    assert!(
        refusal.contains("the exit of the program it led could not be read"),
        "the refusal does not say the exit could not be read: {refusal}"
    );
    assert!(
        !refusal.contains("did not end within")
            && !refusal.contains(&format!("could not be shown gone within {QUIESCE_BOUND:?}")),
        "the refusal states a wait that was not the wait: {refusal}"
    );
}

/// Issue 6528's review,
/// Low 2:
/// the two process-group reasons that cannot be produced from outside name only what was found:
/// a group whose members were still there at its bound says that bound,
/// and a group that could not be named says that no stop was tried.
#[test]
fn the_process_group_reasons_state_the_wait_that_was_waited_or_none() {
    assert!(
        GroupStop::Gone.settled(),
        "a group shown gone is a settled run"
    );
    assert!(
        !GroupStop::MembersLeft.settled(),
        "a group that still has members is a settled run"
    );
    let left: String = GroupStop::MembersLeft.refusal();
    assert!(
        left.contains(&format!("could not be shown gone within {QUIESCE_BOUND:?}")),
        "a group with members left does not state the bound it waited: {left}"
    );
    let unnamed: String = GroupStop::Unnamed.refusal();
    assert!(
        unnamed.contains("could not be named, so nothing of it was stopped"),
        "a group that could not be named does not say that nothing was stopped: {unnamed}"
    );
    assert!(
        !unnamed.contains("within"),
        "a group that could not be named states a wait that never started: {unnamed}"
    );
}

/// The state a failed stop writes its reason into,
/// for the control below.
fn a_scope_state() -> std::rc::Rc<ScopeState> {
    return std::rc::Rc::new(ScopeState {
        log: PathBuf::from("/dev/null"),
        lost: Cell::new(false),
        lost_reason: RefCell::new(None),
        process_group_only: Cell::new(false),
    });
}

/// Issue 6528's review,
/// Low 9:
/// two failed stops under one transaction must both be named,
/// because a refusal that names only the last would hide the first writer that may still remain.
#[test]
fn a_second_failed_stop_keeps_the_reason_of_the_first() {
    let state = a_scope_state();
    lose(
        &Some(std::rc::Rc::clone(&state)),
        "the first stop".to_string(),
    );
    lose(
        &Some(std::rc::Rc::clone(&state)),
        "the second stop".to_string(),
    );

    let kept: String = state
        .lost_reason
        .borrow()
        .clone()
        .expect("a reason is kept");
    assert!(
        kept.contains("the first stop") && kept.contains("the second stop"),
        "a second failed stop replaced the reason of the first: {kept}"
    );
}

/// Issue 6528's review,
/// Low 3:
/// a program that was not reaped is named before the scope's own sentence,
/// the absent scope is worded as absent,
/// and a run whose scope was shown empty has no refusal at all.
#[test]
fn a_child_that_was_not_reaped_is_named_before_the_scope_and_an_absent_scope_is_worded_as_gone() {
    let absent: StopReport = StopReport::absent(Duration::from_millis(1));
    let bound_spent: String = refusal_of_scope(
        &absent,
        &Reaped::NotReaped(ReapShortfall::Bound),
        "the scope /sys/fs/cgroup/g.scope was gone when the stop looked, and its absence was verified",
    )
    .expect("a program that was not reaped is a refusal");
    assert!(
        bound_spent.contains(&format!(
            "the program did not end within {REAP_BOUND:?} after it was killed;"
        )),
        "the refusal does not name the program that did not end: {bound_spent}"
    );
    assert!(
        bound_spent.contains("was gone when the stop looked"),
        "the refusal does not keep the absent scope's own words: {bound_spent}"
    );

    let unreadable: String = refusal_of_scope(
        &absent,
        &Reaped::NotReaped(ReapShortfall::Unreadable("boom".to_string())),
        "the scope /sys/fs/cgroup/h.scope was gone when the stop looked",
    )
    .expect("a program whose exit could not be read is a refusal");
    assert!(
        unreadable.contains("could not be read after it was killed (boom)")
            && !unreadable.contains("did not end within"),
        "a failed read of the exit is worded as a wait: {unreadable}"
    );

    let settled: Option<String> = refusal_of_scope(
        &StopReport::new(
            Duration::from_millis(1),
            true,
            Some(false),
            Listing::Read(Vec::new()),
        ),
        &Reaped::Ended,
        "the stop of /sys/fs/cgroup/i.scope waited 1ms and the kernel reported the scope empty",
    );
    assert!(
        settled.is_none(),
        "a run whose scope was shown empty still carries a refusal: {settled:?}"
    );
}

/// The control of a refusal of a bare process group:
/// it names the group and the wait,
/// and the sentence about the journal it kept names no scope,
/// so a person whose host can give no scope reads a message that is true of their run
/// (the read of alpha.15,
/// Low 9).
#[test]
fn the_refusal_of_a_process_group_carries_a_journal_sentence_that_names_no_scope() {
    // What:     `GroupStop::MembersLeft.refusal()` is the reason a store of a process group gives when members are left.
    // Why:      It is the reason an install or update refusal carries for this containment, so the message is assembled from the same
    //           pieces a run assembles.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reason = GroupStop.MembersLeft.refusal();
    // ```
    let reason: String = GroupStop::MembersLeft.refusal();
    // What:     `crate::install::refusal_of_writers` is the one place both refusals build their message.
    // Why:      Judging it here judges what an install and an update print, and the shim adds only its own line after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refusal = refusalOfWriters('lockfiles', reason);
    // ```
    let refusal: String = crate::install::refusal_of_writers("lockfiles", &reason);

    assert!(
        refusal.contains("the members of the process group could not be shown gone"),
        "the refusal does not carry the reason of the process group: {refusal}"
    );
    assert!(
        refusal.contains(crate::journal_recover::KEPT_JOURNAL),
        "the refusal does not carry the one sentence about the kept journal: {refusal}"
    );
    assert!(
        refusal.contains("nothing this run started is still running"),
        "the refusal does not say when a later run settles the journal, in words a process group fits: {refusal}"
    );
    assert!(
        !refusal.contains("scope"),
        "the refusal of a process group names a scope, which such a run never had: {refusal}"
    );
}

/// The control of the error a contained call prints when its stop could not settle it:
/// it says what may still write and what the tool did not do,
/// and it says nothing about a journal,
/// because the refusal that keeps the journal says that once
/// (`journal_recover::KEPT_JOURNAL`),
/// and a call that ran outside any transaction has no journal to speak of.
#[test]
fn the_not_quiescent_error_says_nothing_about_a_journal() {
    // What:     `RunError::NotQuiescent` is the error a contained call answers when its stop could not show every writer gone.
    // Why:      Its text is printed by the callers that have no refusal around it, and the install and update refusals add the
    //           sentence about the journal themselves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const error = new RunError.NotQuiescent({ what: 'cargo update', shortfall: 'the stop waited 5s' });
    // ```
    let error: crate::bounded_run::RunError = crate::bounded_run::RunError::NotQuiescent {
        what: "cargo update".to_string(),
        shortfall: "the stop of the scope waited 5s for the kernel to report it empty".to_string(),
    };

    // What:     `error.to_string()` is what a caller prints for the error.
    // Why:      The assertions judge the message a person reads, not the fields.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const said = String(error);
    // ```
    let said: String = error.to_string();

    assert!(
        said.contains("could not be stopped or shown to be gone"),
        "the error does not say what the stop could not show: {said}"
    );
    assert!(
        said.contains("the stop of the scope waited 5s for the kernel to report it empty"),
        "the error does not carry the stop's own words: {said}"
    );
    assert!(
        said.contains("this tool changed no file because of that"),
        "the error does not say what the tool did not do: {said}"
    );
    assert!(
        !said.contains("journal"),
        "the error says what became of a journal, which the refusal that keeps one says once: {said}"
    );
}
