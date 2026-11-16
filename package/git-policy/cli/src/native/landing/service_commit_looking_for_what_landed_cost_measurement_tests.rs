//! The cost of the look for a landed commit against the number of finished records the journal holds.
//! What:
//!  Time one read of the finished records with 16, with 160 and with 1,600 of them,
//!  the request's own record being the oldest.
//! Why:
//!  The look reads every record the journal holds, so its cost must follow the journal's bound
//!  (`DONE_RETAINED`) and not how long the service has run.

use super::*;
use crate::service_journal::{DONE_EXTENSION, batch_path, intent_text};
use std::time::Instant;

/// One finished record naming `id`.
fn a_record(seq: u64, id: &str) -> Intent {
    let commit: String = format!("{seq:040}");
    return Intent {
        seq,
        repository: PathBuf::from("/nowhere/repo"),
        common_dir: PathBuf::from("/nowhere/repo/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: String::from("0").repeat(40),
        new: commit.clone(),
        ids: vec![String::from(id)],
        commits: vec![commit],
        holders: Vec::new(),
        paths: Vec::new(),
        derived: true,
        cleanup: None,
        index: PathBuf::from("/nowhere/repo/.git/index"),
        git_dir: PathBuf::from("/nowhere/repo/.git"),
    };
}

/// What:
///  Print one look's own cost with 16, 160 and 1,600 finished records in the journal.
/// Why:
///  The brief of the round asks for the figure, and a reader comparing it to the journal's bound needs it measured on
/// this host rather than argued.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (n of [16, 160, 1600]) { writeNRecords(n); console.log(n, time(() => aFinishedBatchMade(journal, id))) }
/// ```
#[test]
fn a_look_at_a_finished_record_costs_what_the_journal_holds() {
    for count in [16usize, 160, 1600] {
        let root: PathBuf = crate::test_support::fixture(format!("looking-cost-{count}").as_str());
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir_all(journal.as_path()).expect("journal");
        let id: String = String::from("0d1e2f3a-4b5c-6d7e-8f90-a1b2c3d4e5f6");
        for seq in 1..=count as u64 {
            let record: Intent = a_record(seq, if seq == 1 { id.as_str() } else { "a-successor" });
            std::fs::write(batch_path(journal.as_path(), seq, DONE_EXTENSION), intent_text(&record)).expect("a finished record");
        }
        let mut timings: Vec<Duration> = Vec::new();
        for _round in 0..5 {
            let began: Instant = Instant::now();
            let found: Option<String> = a_finished_batch_made(journal.as_path(), id.as_str(), began + Duration::from_secs(60));
            timings.push(began.elapsed());
            assert!(found.is_some(), "the request's record was not found with {count} records");
        }
        timings.sort();
        eprintln!("MEASURE a finished record look with {count} records: median {:?}, least {:?}, most {:?}", timings[2], timings[0], timings[4]);
        crate::test_support::remove(root.as_path());
    }
}
