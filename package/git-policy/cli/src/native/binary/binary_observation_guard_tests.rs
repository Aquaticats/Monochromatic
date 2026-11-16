//! What:
//!  The wait that reads what the commit service did after a command,
//!  and what one look that finds nothing in a repository's queue is worth.
//! Why:
//!  The wait lists three state directories,
//!  and three listings are three different moments:
//!  a record a claim carries is seen,
//!  but a record an expired claim returns from `claimed` to `pending` through `recovering`
//!  runs the other way,
//!  and one of its two renames can fall between two of the listings,
//!  although the record is in the queue at every moment.
//!  On 2026-11-14 a control of the package repair then read a queue as settled
//!  and the service ran the relink it held three milliseconds later
//!  (`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`,
//!  `/home/user/temp/agent/integration-pass-45-scratch/keep/logs/binary-package-repair-tests-tip.log`).
//!  No control can make a rename straddle a listing on a real filesystem,
//!  because the renames of a claim sit microseconds apart,
//!  so this control gives the wait the listing it asks for
//!  and proves that one look of nothing does not end the wait.
//!  A record moved between two of a look's listings is another control's question
//!  (`settle_the_after_commit_work::a_recovery_moved_at_every_gap_is_never_read_as_settled`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settleServiceWorkListWith(repo, (work, state) => listings++ === LISTINGS_IN_A_LOOK ? 1 : 0); // the wait must not stop at the first empty look
//! ```

/// Import the fixture,
/// the repository maker and the wait under test.
use super::support::{Fixture, LISTINGS_IN_A_LOOK, fixture, remove, repository, settle_service_work_list_with};
use git_policy_cli::work_item::WORK_DIRECTORY;
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  The wait for the after-commit work reads a queue that answers no work for its first look and work for the look after it,
///  and it does not answer settled before two looks in a row have found nothing in it.
/// Why:
///   The record the add hands the service is in the queue before the add returns
/// (`repair::package_repair_relink::hand_over_with` writes it and wakes the service),
/// and a claim moves it between the states the wait counts,
/// so a look can miss it although the work is queued.
/// What this control proves:
/// the wait looks again after a look that found nothing,
/// and it answers only two looks with nothing in the queue,
/// a pause apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleServiceWorkListWith(repo, () => listings++ === LISTINGS_IN_A_LOOK ? 1 : 0); expect(listings).toBeGreaterThan(2 * LISTINGS_IN_A_LOOK);
/// ```
#[test]
fn a_queue_that_reads_empty_once_is_not_work_that_settled() {
    let fixture: Fixture = fixture("settle-two-looks");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let asked: Cell<usize> = Cell::new(0);
    let read: RefCell<PathBuf> = RefCell::new(PathBuf::new());
    // The first look finds nothing,
    // the look after it finds the work a claim moved between the wait's listings,
    // and the queue is empty from the third look on:
    // a wait that stops at its first empty look answers settled while work is queued.
    let records_in = |work: &Path, _state: &str| -> Result<usize, String> {
        *read.borrow_mut() = work.to_path_buf();
        let taken: usize = asked.get() + 1;
        asked.set(taken);
        // The listing the look after the first one asks for is the work the wait must not have called settled.
        return Ok(if taken == LISTINGS_IN_A_LOOK + 1 { 1 } else { 0 });
    };
    settle_service_work_list_with(repo.as_path(), &records_in);
    assert_eq!(*read.borrow(), repo.join(".git").join(WORK_DIRECTORY), "the wait read a queue that is not this repository's");
    assert!(
        asked.get() > 2 * LISTINGS_IN_A_LOOK,
        "the wait answered settled after {} reading(s) of the queue: a first reading that finds nothing is not work that settled, because a record is in the queue at every moment while a claim moves it between two of the states the wait lists",
        asked.get()
    );
    remove(&fixture);
}
