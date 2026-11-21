//! What:
//!  Controls of the mode's side of the holder:
//!  a holder started without mount authority answers its attach as a failure
//!  rather than passing,
//!  and its stop byte and reap are the mode's own.
//! Why:
//!  The scan's first pass requires the holder's mount,
//!  so a holder that could not attach must answer `F` rather than let the
//!  worker scan a namespace that does not exist.
//!  A control on a machine that cannot unshare a mount namespace exercises
//!  exactly that refusal without making a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holder = Holder.start(place, 'pool/ds@ready'); expect(holder.drain(feed, report)).toEqual({ kind: 'attach', passed: false });
//! ```

use super::{Arrival, Holder};
use super::super::probe_record_files::RecordPlace;
use super::super::probe_report::ProbeReport;
use super::super::probe_status_line::LineFeed;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::testing::test_support::{fixture, wait_until};
use std::path::PathBuf;

/// A holder started without mount authority reports its attach as failed,
/// and its pipe then ends.
#[test]
fn a_holder_without_mount_authority_reports_a_failed_attach() {
    let root: PathBuf = fixture("start-probe-holder");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    place.prepare().expect("the record place is made");
    let started: Holder = Holder::start(&place, "pool/ds@ready").expect("the holder forks");
    let held: RefCell<Holder> = RefCell::new(started);
    let feed: RefCell<LineFeed> = RefCell::new(LineFeed::new());
    let report: RefCell<ProbeReport> = RefCell::new(ProbeReport::new());
    let arrival: Cell<Arrival> = Cell::new(Arrival::Nothing);
    let answered: bool = wait_until(&|| {
        let next: Arrival = held.borrow_mut().drain(&mut feed.borrow_mut(), &mut report.borrow_mut());
        if next == Arrival::Nothing {
            return false;
        }
        arrival.set(next);
        return true;
    });
    assert!(answered, "the holder did not answer within the control's deadline");
    assert_eq!(arrival.get(), Arrival::Attach { passed: false }, "a holder that cannot unshare must not pass");
    assert!(report.borrow().lines().iter().any(|line| return line.step == super::super::probe_arms::ProbeStep::HolderAttach));
    let mut finished: Holder = held.into_inner();
    finished.stop();
    let _ = finished.reap();
    finished.end();
}

/// The old count of polls cannot see an answer that arrives after the last
/// poll,
/// and the deadline wait does.
/// This forces the failing order rather than repeating the control's run:
/// the answer cannot be stored until the count has finished,
/// so the count must miss it.
#[test]
fn a_late_answer_is_missed_by_a_count_of_polls_and_seen_by_the_deadline() {
    let answered: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let (count_finished, after_the_count) = std::sync::mpsc::channel::<()>();
    let answering = Arc::clone(&answered);
    let answer = std::thread::spawn(move || {
        // The count's sender is dropped once the count has run, so the answer is stored only after it.
        let _released = after_the_count.recv();
        answering.store(true, Ordering::SeqCst);
    });
    let mut seen: bool = false;
    for _ in 0..1000 {
        if answered.load(Ordering::SeqCst) {
            seen = true;
            break;
        }
    }
    assert!(!seen, "the count of polls saw an answer that had not arrived when the count began");
    drop(count_finished);
    assert!(wait_until(&|| return answered.load(Ordering::SeqCst)), "the deadline wait did not see the late answer");
    answer.join().expect("the late answer's thread ends");
}
