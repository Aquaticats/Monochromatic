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
use crate::testing::test_support::fixture;
use std::path::PathBuf;

/// A holder started without mount authority reports its attach as failed,
/// and its pipe then ends.
#[test]
fn a_holder_without_mount_authority_reports_a_failed_attach() {
    let root: PathBuf = fixture("start-probe-holder");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    place.prepare().expect("the record place is made");
    let mut holder: Holder = Holder::start(&place, "pool/ds@ready").expect("the holder forks");
    let mut feed: LineFeed = LineFeed::new();
    let mut report: ProbeReport = ProbeReport::new();
    let mut arrival: Arrival = Arrival::Nothing;
    for _ in 0..1000 {
        arrival = holder.drain(&mut feed, &mut report);
        if arrival != Arrival::Nothing {
            break;
        }
    }
    assert_eq!(arrival, Arrival::Attach { passed: false }, "a holder that cannot unshare must not pass");
    assert!(report.lines().iter().any(|line| return line.step == super::super::probe_arms::ProbeStep::HolderAttach));
    holder.stop();
    let _ = holder.reap();
    holder.end();
}
