//! What:
//!  The wait that reads what the commit service did after a command,
//!  and what one empty reading of a repository's queue is worth.
//! Why:
//!  The wait lists three state directories,
//!  and three listings are three different moments:
//!  a record a claim moves from `pending` to `recovering` to `claimed`
//!  is in none of them when it leaves `recovering` between the second listing and the third,
//!  although it is in the queue at every moment.
//!  On 2026-11-14 a control of the package repair then read a queue as settled
//!  and the service ran the relink it held three milliseconds later
//!  (`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`,
//!  `/home/user/temp/agent/integration-pass-45-scratch/keep/logs/binary-package-repair-tests-tip.log`).
//!  No control can make a claim straddle a listing on a real filesystem,
//!  because the two renames of a claim sit microseconds apart,
//!  so this control gives the reading the wait asks
//!  and proves that one reading of nothing does not end the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settleServiceWorkReadWith(repo, (work) => [0, 1, 0, 0][readings++]); // the wait must not stop at the first 0
//! ```

/// Import the fixture, the repository maker and the wait under test.
use super::support::{Fixture, fixture, remove, repository, settle_service_work_read_with};
use git_policy_cli::work_item::WORK_DIRECTORY;
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  The wait for the after-commit work reads a queue that answers no work at its first reading and work at its second,
///  and it does not answer settled before it has read the queue twice with nothing in it.
/// Why:
///   The record the add hands the service is in the queue before the add returns
/// (`repair::package_repair_relink::hand_over_with` writes it and wakes the service),
/// and a claim moves it between the states the wait counts,
/// so a reading can miss it although the work is queued.
/// What this control proves: the wait asks again after a reading that found nothing,
/// and it answers only two readings with nothing in the queue, a pause apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readings = [0, 1, 0, 0]; settleServiceWorkReadWith(repo, () => readings.shift()); expect(asked).toBeGreaterThan(2);
/// ```
#[test]
fn a_queue_that_reads_empty_once_is_not_work_that_settled() {
    let fixture: Fixture = fixture("settle-two-readings");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let asked: Cell<usize> = Cell::new(0);
    let read: RefCell<PathBuf> = RefCell::new(PathBuf::new());
    // The first reading finds nothing, the second finds the work a claim moved between the wait's listings, and the queue
    //  is empty from the third on: a wait that stops at its first empty reading answers settled while work is queued.
    let reading = |work: &Path| -> usize {
        *read.borrow_mut() = work.to_path_buf();
        let taken: usize = asked.get() + 1;
        asked.set(taken);
        return match taken {
            1 => 0,
            2 => 1,
            _ => 0,
        };
    };
    settle_service_work_read_with(repo.as_path(), &reading);
    assert_eq!(*read.borrow(), repo.join(".git").join(WORK_DIRECTORY), "the wait read a queue that is not this repository's");
    assert!(
        asked.get() > 2,
        "the wait answered settled after {} reading(s) of the queue: a first reading that finds nothing is not work that settled, because a record is in the queue at every moment while a claim moves it between two of the states the wait lists",
        asked.get()
    );
    remove(&fixture);
}
