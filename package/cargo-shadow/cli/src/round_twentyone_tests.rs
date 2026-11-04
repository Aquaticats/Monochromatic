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
    let events_on_success: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("journal "); }).collect();
    assert_eq!(events_on_success, ["journal directory synced", "journal published"]);
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

/// Round 21 (check-to-start race): the cargo of an install step runs inside a scope, whose launcher takes the program as an argument and
/// starts it itself, so an open descriptor of this process cannot name it there. A compiled real cargo is started by its checked path
/// in that run (with the check repeated right before), and it must actually run: here a copy of `echo` stands in for it.
#[test]
fn a_compiled_real_cargo_runs_in_the_contained_runner_of_the_install_step() {
    use crate::cargo_run::{CargoRunner, RealCargo};
    let directory = scratch();
    let chosen: PathBuf = directory.path().join("cargo");
    fs::copy("/usr/bin/echo", &chosen).expect("a compiled stand-in for the real cargo");
    let cargo = RealCargo { program: chosen, toolchain: String::new(), bound: std::time::Duration::from_secs(60) };

    let outcome = cargo.run(directory.path(), &["ran".to_string(), "contained".to_string()]).expect("the stand-in starts");

    assert!(outcome.ok && outcome.stdout == "ran contained\n", "{outcome:?}");
}

/// Sixth pass, group 7 (K1, colliding names): copies kept under a collision-suffixed name count toward the warning. One name short of
/// the count is already beside the lockfile; two refusals through the real recovery add the first refused copy and a collision-named
/// second one, and only counting both crosses the count.
#[test]
fn refused_copies_saved_under_collision_names_count_toward_the_kept_files_warning() {
    let root = scratch();
    let crate_root: PathBuf = root.path().join(CRATE_DIR);
    fs::create_dir_all(&crate_root).expect("dir");
    let count: usize = crate::journal::KEPT_FILES_WARNING_COUNT;
    for index in 0..count - 1 {
        fs::write(crate_root.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a kept file");
    }
    let snapshot = || -> Vec<LockSnapshot> { return vec![LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() }]; };
    let refuse = |_before: &[LockSnapshot], _live: &[LockSnapshot]| -> Vec<String> { return vec!["dep@1.0.1 refused".to_string()]; };
    for run in 0..2 {
        fs::write(crate_root.join("Cargo.lock"), format!("unchecked output of killed run {run}")).expect("the killed run's output");
        plant(root.path(), &snapshot(), &[]);
        crate::journal::recover(root.path(), &refuse);
    }
    let refused: Vec<String> = fs::read_dir(&crate_root).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name| { return name.starts_with("Cargo.lock.refused"); }).collect();
    assert_eq!(refused.len(), 2, "the two refusals did not leave two saved copies, so the schedule was not built: {refused:?}");
    assert!(refused.iter().any(|name| { return name != "Cargo.lock.refused"; }), "no copy has a collision name: {refused:?}");

    let transaction = crate::journal::begin(root.path(), &snapshot(), false).expect("a kept file never refuses");

    let notice: &str = transaction.kept_notice().expect("the count was crossed only by counting the collision-named copy");
    assert!(notice.contains(&format!("{} files", count + 1)), "{notice}");
}
