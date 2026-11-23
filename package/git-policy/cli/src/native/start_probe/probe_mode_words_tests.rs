//! What:
//!  Controls of the mode's verdict line and its source readings:
//!  each verdict is one sentence naming what a person must look at,
//!  a refused slot is told its route,
//!  and a timed-out record names both steps.
//! Why:
//!  The line is the journal entry a boot failure is read from,
//!  and it must name the arm or the step rather than only that something
//!  failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verdictLine({ kind: 'failed', step: 'move' })).toContain('move');
//! ```

use super::{source_of, source_refusal_word, table_listing, table_word, verdict_line};

/// An empty report,
/// which every verdict line reads without a step.
fn report() -> ProbeReport {
    return ProbeReport::new();
}
use super::super::probe_arms::{ProbeArm, ProbeStep};
use super::super::probe_report::{ProbeReport, ProbeVerdict};
use super::super::probe_status_line::StepLine;

/// What:
///  The listing the mode prints when the table told it no single checkout.
/// Why:
///  The project's second answer of 2026-11-22: the probe prints each mounted
///  dataset it found with its path.
///  The word alone cannot do that, so the mode recomputes the look-up and prints
///  the refusal's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(tableListing(path, twoInstances)).toContain('/home/user/second');
/// ```
#[test]
fn the_table_words_ask_for_a_listing_that_names_every_row() {
    let mut several: ProbeReport = ProbeReport::new();
    several.record(StepLine::not_exercised(ProbeStep::CheckoutRoot, "several-mounted-checkouts", None));
    assert_eq!(table_word(&several), Some("several-mounted-checkouts"));
    let mut none: ProbeReport = ProbeReport::new();
    none.record(StepLine::not_exercised(ProbeStep::ReadSourceName, "no-mounted-checkout", None));
    assert_eq!(table_word(&none), Some("no-mounted-checkout"));
    let mut other: ProbeReport = ProbeReport::new();
    other.record(StepLine::not_exercised(ProbeStep::CheckoutRoot, "no-checkout-path", None));
    assert_eq!(table_word(&other), None);
    assert_eq!(table_word(&ProbeReport::new()), None);

    let fixture: std::path::PathBuf = crate::test_support::fixture("probe-listing-none");
    let missing: std::path::PathBuf = fixture.join("nothing.jsonc");
    let rows: Vec<String> = vec![
        crate::testing::mounted_dataset_table::row(40, 1, "/", "/home/user/Monochromatic", "zfs", "pool/home/monochromatic-git-checkout"),
        crate::testing::mounted_dataset_table::row(41, 1, "/", "/home/user/second", "zfs", "other/home/monochromatic-git-checkout"),
    ];
    let table = crate::testing::mounted_dataset_table::ScriptedMountTable::holding(&rows);
    let listing: String = table_listing(missing.as_path(), &table).expect("two instances are no single checkout");
    assert!(listing.contains("/home/user/Monochromatic") && listing.contains("/home/user/second"), "the listing names both rows with their paths: {listing}");
    assert!(listing.contains("will not choose between them"), "the listing says why nothing was taken: {listing}");
}

/// Every verdict's line names what the reader must act on.
#[test]
fn every_verdict_line_names_what_matters() {
    let mut failed: ProbeReport = ProbeReport::new();
    failed.record(StepLine::failed(ProbeStep::Move, Some("EBUSY")));
    assert!(verdict_line(ProbeVerdict::Passed, &report()).contains("enabled"));
    let failure: String = verdict_line(ProbeVerdict::Failed { step: ProbeStep::Move }, &failed);
    assert!(failure.contains("move") && failure.contains("EBUSY") && failure.contains("move-unmount"), "{failure}");
    assert!(verdict_line(ProbeVerdict::Blocked { arm: ProbeArm::Source }, &report()).contains("source"));
    assert!(verdict_line(ProbeVerdict::Blocked { arm: ProbeArm::Source }, &report()).contains("reflink"));
    let timed_out: String = verdict_line(ProbeVerdict::TimedOut { in_flight: Some(ProbeStep::Unmount), last_finished: Some(ProbeStep::Move) }, &report());
    assert!(timed_out.contains("unmount") && timed_out.contains("move"), "{timed_out}");
    assert!(verdict_line(ProbeVerdict::Skipped { gate_step: "member" }, &report()).contains("member"));
}

/// The source and its refusal are read from the lines the worker sent.
#[test]
fn the_source_and_its_refusal_come_from_the_lines() {
    let mut report: ProbeReport = ProbeReport::new();
    assert_eq!(source_of(&report), None);
    assert_eq!(source_refusal_word(&report), None);
    let mut line: StepLine = StepLine::passed(ProbeStep::ReadSourceName);
    line.source = Some(String::from("pool/ds@ready"));
    report.record(line);
    assert_eq!(source_of(&report), Some(String::from("pool/ds@ready")));
    let mut missing: ProbeReport = ProbeReport::new();
    missing.record(StepLine::not_exercised(ProbeStep::ReadSourceName, "name-absent", None));
    assert_eq!(source_refusal_word(&missing), Some("name-absent"));
}
