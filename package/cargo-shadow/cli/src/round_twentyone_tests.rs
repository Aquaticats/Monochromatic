//! What:
//! Round 21 tests: the fixes asked for by the sixth certification pass (`cargo-shadow-cert6-sol-1.txt` to `-14.txt`).
//! Why:
//! The sixth pass found two production defects (a zero process id counted as an acknowledging process; the printed Fix B record command
//! built invalid JSON for a path with quotation marks or backslashes) and assertions that a weaker outcome than the guarantee would also
//! satisfy; each test here discriminates the guarantee from the weaker outcome named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round twenty-one', () => { it('a zero process id is not an acknowledgement', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::arrival::arrival_message;
use crate::crates::LockSnapshot;
use crate::guard::{Guard, GuardKind};
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The largest process id Linux allows (`PID_MAX_LIMIT` for 64-bit kernels, `include/linux/threads.h`).
const LARGEST_LINUX_PROCESS_ID: u64 = 4_194_304;

/// A disposable repository with a ready weaker-path journal whose acknowledgement is `marker`.
fn journal_acknowledged_with(marker: &str) -> (tempfile::TempDir, PathBuf) {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "live").expect("lock");
    let journal: PathBuf = plant(root.path(), &[LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() }], &[]);
    fs::write(journal.join("groups"), "pgroup starting\n").expect("write the records");
    fs::write(journal.join("acknowledged"), marker).expect("write the marker");
    return (root, journal);
}

/// Group 13 (sixth pass, production defect): a process id that no process can have is not an acknowledging process.
/// Zero and a number above the largest Linux process id are refused, the smallest and the largest real ones count; none of them has to
/// name a process that still exists.
#[test]
fn an_acknowledgement_naming_a_process_id_no_process_can_have_does_not_count() {
    for impossible in [0, LARGEST_LINUX_PROCESS_ID + 1, u64::from(u32::MAX)] {
        let (_root, journal) = journal_acknowledged_with(&format!("acknowledged v1\n{impossible}\n456\n"));
        assert!(survivor_note(&journal).is_some(), "process id {impossible} counted as an acknowledgement");
    }
    for possible in [1, LARGEST_LINUX_PROCESS_ID] {
        let (_root, journal) = journal_acknowledged_with(&format!("acknowledged v1\n{possible}\n456\n"));
        assert_eq!(survivor_note(&journal), None, "process id {possible} did not count as an acknowledgement");
    }
}

/// Group 14 (sixth pass, production defect): the printed Fix B command writes a record that parses, whatever the path of the
/// cargo-shadow executable contains. The executable path here holds a quotation mark, a backslash, a single quote and a space; the
/// printed line is run by `/bin/sh` as printed and the record it writes is parsed as JSON and must give the path back unchanged.
#[test]
fn the_printed_record_command_writes_valid_json_for_a_path_with_quotes_and_backslashes() {
    let directory = scratch();
    let launcher: PathBuf = directory.path().join("cargo");
    fs::write(&launcher, "#!/bin/sh\n").expect("write the launcher");
    let executable: &Path = Path::new("/home/u/q\"uote\\back's slash/cargo-shadow");
    let guard = Guard { kind: GuardKind::Spawn, pid: std::process::id(), hops: 1, skipped: vec![launcher.clone()] };

    let said: String = arrival_message(&guard, executable, "/usr/bin");

    let line: &str = said.lines().map(str::trim).find(|line| { return line.starts_with("printf "); }).expect("a record line");
    let written = std::process::Command::new("/bin/sh").arg("-c").arg(line).output().expect("runs the record line");
    assert!(written.status.success(), "the record line does not run as printed: {line}: {}", String::from_utf8_lossy(&written.stderr));
    let record_text: String = fs::read_to_string(directory.path().join("cargo.launch")).expect("the record was written");
    let record: serde_json::Value = serde_json::from_str(&record_text).unwrap_or_else(|error| { panic!("the record is not JSON ({error}): {record_text}"); });
    assert_eq!(record.get("real").and_then(serde_json::Value::as_str), Some(executable.to_str().expect("utf-8")), "{record_text}");
}

/// Sixth pass, group 12 (V3): a new journal is made durable and then the name that lists it, in that order, before `begin` returns and
/// so before any recorded child can start. Each barrier is failed on its own: the transaction is refused, with the fault reached, the
/// error naming the barrier, and the "journal published" event absent.
#[test]
fn a_failed_barrier_of_the_new_journal_refuses_the_transaction_before_anything_can_start() {
    use crate::failpoints::{Point, arm, fires};
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "live").expect("lock");
    let snapshots: Vec<LockSnapshot> = vec![LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() }];

    // The order of the two barriers on success.
    let _ = crate::bounded_run::take_trace_for_test();
    let transaction = crate::journal::begin(root.path(), &snapshots, false).expect("a transaction begins");
    let events: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("journal "); }).collect();
    assert_eq!(events, ["journal directory synced", "journal published"]);
    drop(transaction);

    for (point, reason, surviving_events) in [(Point::JournalDirectorySync, "journal directory", 0), (Point::JournalParentSync, "journal's name", 1)] {
        let other = scratch();
        fs::create_dir_all(other.path().join(CRATE_DIR)).expect("dir");
        fs::write(other.path().join(CRATE_DIR).join("Cargo.lock"), "live").expect("lock");
        arm(point);
        let _ = crate::bounded_run::take_trace_for_test();

        let refused = crate::journal::begin(other.path(), &snapshots, false);

        assert!(!fires(point), "{point:?}: the barrier was never reached");
        let message: String = refused.err().map(|error| { return error.to_string(); }).unwrap_or_else(|| { panic!("{point:?}: a transaction began although a barrier of its journal failed"); });
        assert!(message.contains(reason), "{point:?}: {message}");
        let events: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("journal "); }).collect();
        assert_eq!(events.len(), surviving_events, "{point:?}: {events:?}");
        assert!(!events.iter().any(|event| { return event == "journal published"; }), "{point:?}: {events:?}");
    }
}
