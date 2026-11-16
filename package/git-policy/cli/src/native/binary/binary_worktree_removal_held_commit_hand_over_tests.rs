//! What:
//!  The two states a person can leave the commit service in when a merge lands the branch of a worktree that holds a
//!  commit behind a reference lock,
//!  through the built executable:
//!  a service that is not answering and one that ended before it did the work it was handed.
//! Why:
//!  The merge records the removal of that worktree for the commit service and returns
//!  (`src/native/worktree/worktree_landed_removal.rs`),
//!  so the worktree is not gone when the merge returns and the person is owed three things in that order:
//!  where the held commit is kept,
//!  the name of the record the removal waits in,
//!  and,
//! once a service runs that record,
//! the worktree's end.
//!  A control that read the directory at the moment the merge returned held the wrapper to an order of two processes
//!  instead:
//! it passed whenever the service happened to finish first,
//!  and the release gate on `0f07c4f0ee` failed it once in thirty-five gates because it did not
//!  (measured:
//! two of twenty runs of the same image on this machine).
//!  These controls make each order happen on purpose,
//!  so the held commit is proved kept under the kept name in both of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stop(services); const ran = await merge();  // the worktree stands, the commit is kept, the record is named
//! resume(services); await nextCommand();       // the record runs and the worktree goes
//! ```

/// Import the shared fixtures and the bounded wait,
/// which the sibling module of the held-commit controls owns.
use super::commit::wrapped_commit;
use super::support::{Observed, observe, observe_without_waiting_for_the_after_commit_work, remove, service_runtime, stderr_of};
use super::worktree_removal_held_commit::{
    BRANCH, HeldScene, a_held_commit_a_merge_lands, git_said, wait_until_the_removal_ran,
};
use std::path::PathBuf;

/// What:
///  Hold every commit service the fixture runs,
///  so the merge hands the removal over and nothing performs it.
/// Why:
///   A service that a person stopped,
///  or one that a manager is restarting,
///  answers nothing,
///  and the merge's wake is given a bound of its own for exactly that
///  (`HAND_OVER_GREETING_WAIT` in `src/native/service/wait_ceilings.rs`):
///  the record is written and the command returns,
///  which is the order this control reads.
pub(super) fn hold_the_services(scene: &HeldScene) -> Vec<u32> {
    let service = scene.fixture.service.lock().expect("the fixture's service lock");
    let started = service.as_ref().expect("the fixture serves its worktrees before the merge runs");
    let pids: Vec<u32> = started.process_ids();
    for pid in &pids {
        let command_line: String = std::fs::read_to_string(format!("/proc/{pid}/cmdline")).expect("the process's own command line");
        assert!(command_line.contains("service"), "the process {pid} is one of the fixture's commit services: {command_line}");
        // SAFETY: `kill` takes a process identifier and a signal, and this one is a service the fixture started and
        //  named above.
        assert_eq!(unsafe { libc::kill((*pid).cast_signed(), libc::SIGSTOP) }, 0, "the fixture's service {pid} is stopped");
    }
    return pids;
}

/// Let a stopped service run again,
/// which is the person restarting it.
pub(super) fn let_a_service_go(pid: u32) {
    // SAFETY: as above, with the signal that lets the fixture's own service run again.
    assert_eq!(unsafe { libc::kill(pid.cast_signed(), libc::SIGCONT) }, 0, "the fixture's service {pid} runs again");
}


/// What:
///  The next ordinary command of the repository that asks its commit service for a round,
///  which is a commit,
///  and with it the record the merge handed over.
/// Why:
///  A record outlives the command that wrote it,
///  and a service claims the records of every kind in a round of its own
///  (`SERVICE_KINDS` in `src/native/landing/service_work_handlers_replanning_and_acceptance.rs`),
///  so the command a person runs next is what performs a removal whose hand-over's own wake did not land.
///  A commit is run through the transaction's bound here,
///  because the harness refuses a commit under the shared bound of one Git command.
fn next_round_of_the_service(scene: &HeldScene) -> Observed {
    return observe(
        &mut wrapped_commit(&scene.fixture, scene.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=a command after the merge"]),
        b"",
    );
}

/// What:
///  A merge that lands the branch of a worktree holding a held commit,
///  whose commit service is not answering,
///  keeps that commit on a reference of this repository,
///  tells the person the record the removal waits in,
///  leaves the worktree where it is,
///  and the removal runs once the service opens the record.
/// Why:
///  The wrapper's own words say the removal is the commit service's and name the record and the command that removes
///  the worktree by hand,
///  so a person who reads them is not surprised that the worktree is still there when the merge returns;
///  what no order may change is the held commit's name,
/// which is written before the record is.
#[test]
fn a_merge_whose_service_does_not_answer_keeps_the_commit_and_the_removal_runs_afterwards() {
    let scene: HeldScene = a_held_commit_a_merge_lands("binary-removal-held-landed-service-held");
    let _runtime: PathBuf = service_runtime(&scene.fixture).expect("the fixture serves its worktrees");
    let stopped: Vec<u32> = hold_the_services(&scene);
    // The fixture's own wait for the service's after-commit work is left out here: the service is held on purpose, and
    //  that wait would end at its own bound before this control read anything.
    let ran: Observed = observe_without_waiting_for_the_after_commit_work(
        &mut wrapped_commit(&scene.fixture, scene.repo.as_path(), &["merge", "--no-edit", "--no-ff", "--message=land the feature", BRANCH]),
        b"",
    );
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "the merge never waits on a service it does not need: {said}");
    assert!(said.contains(scene.kept_name().as_str()), "the person is told where the commit is kept: {said}");
    assert!(said.contains("handed the removal"), "the person is told the removal is the commit service's: {said}");
    assert!(said.contains("subvolume-delete-"), "the person is told the record the removal waits in: {said}");
    assert!(scene.linked.is_dir(), "the worktree stands while the service handed the removal is not answering: {said}");
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(),
        scene.held,
        "the kept reference names the held commit although nothing removed the worktree: {said}"
    );
    for pid in &stopped {
        let_a_service_go(*pid);
    }
    // The record outlives the command: the service's next round claims it, and a commit is the ordinary command that
    //  asks the service for a round.
    let next: Observed = next_round_of_the_service(&scene);
    let after: String = stderr_of(&next);
    assert_eq!(next.code, Some(0), "the next command of the repository runs: {after}");
    wait_until_the_removal_ran(&scene, after.as_str());
    assert_eq!(git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(), scene.held, "the kept reference still names the held commit after the removal");
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit survives the removal the service performed");
    remove(&scene.fixture);
}

/// What:
///  A merge whose commit service ended before it did the work it was handed keeps the held commit on a reference of
///  this repository,
///  leaves the worktree where it is,
///  and the next command of the repository has a service perform the removal,
///  so nothing is left that nothing removes.
/// Why:
///  A service that ends with a record in its queue is the case a person meets after a crash or a stop,
///  and the queue outlives the program that writes to it:
///  the record is the one thing that still names that removal,
///  and every later command of the repository wakes a service that reads it.
#[test]
fn a_merge_whose_service_ends_before_the_removal_keeps_the_commit_and_the_next_command_removes_it() {
    let scene: HeldScene = a_held_commit_a_merge_lands("binary-removal-held-landed-service-ended");
    let _runtime: PathBuf = service_runtime(&scene.fixture).expect("the fixture serves its worktrees");
    let stopped: Vec<u32> = hold_the_services(&scene);
    let ran: Observed = observe_without_waiting_for_the_after_commit_work(
        &mut wrapped_commit(&scene.fixture, scene.repo.as_path(), &["merge", "--no-edit", "--no-ff", "--message=land the feature", BRANCH]),
        b"",
    );
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "the merge ran: {said}");
    assert!(said.contains(scene.kept_name().as_str()), "the person is told where the commit is kept: {said}");
    // The service ends while the record still waits, which is what a stop or a crash leaves.
    for pid in &stopped {
        let_a_service_go(*pid);
    }
    {
        let mut service = scene.fixture.service.lock().expect("the fixture's service lock");
        let started = service.as_mut().expect("the fixture serves its worktrees before the merge runs");
        started.stop_all();
    }
    assert!(scene.linked.is_dir(), "nothing removed the worktree the ended service was handed: {said}");
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(),
        scene.held,
        "the kept reference names the held commit although the service ended before the removal: {said}"
    );
    let next: Observed = next_round_of_the_service(&scene);
    let after: String = stderr_of(&next);
    assert_eq!(next.code, Some(0), "the next command of the repository runs: {after}");
    wait_until_the_removal_ran(&scene, after.as_str());
    assert_eq!(git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(), scene.held, "the kept reference still names the held commit after the removal a later service performed");
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit is not lost with the worktree a later command removed");
    remove(&scene.fixture);
}
