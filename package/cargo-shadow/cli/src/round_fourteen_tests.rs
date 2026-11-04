//! What:
//! Round 14 tests: the start time of a launcher read from another time namespace (13, section 3).
//! Why:
//! The start time in `/proc/<pid>/stat` is relative to the boot-time offset of the time namespace of the process that reads it, so
//! a recovery in another time namespace reads a live launcher's start time differently and must not take that for a reused id.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round fourteen', () => { it('another time namespace is unknown, not gone', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::cgroup_test_support::delegated_route;
use crate::control_group::{ControlGroup, Record, launcher_state, parse_record, pid_namespace_id, process_start, time_namespace_id};
use crate::crates::LockSnapshot;
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_ROOT";

/// The environment variable that gives the helper the recorded identity of the launcher, as `pid start pid_namespace time_namespace`.
const FIXTURE_IDENTITY: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_IDENTITY";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A live process that stands for a launcher that has not reached the manager, killed when the guard drops.
struct Held(
    /// The process.
    Child,
);

/// How the guard cleans up.
impl Drop for Held {
    /// What:
    /// `fn drop(&mut self)` kills and reaps the process.
    /// Why:
    /// A failed assertion must not leave a `sleep` behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Contract A1 (round 14): the record of a launcher carries the time namespace of its writer, reads back as written, and a line of an
/// older version (without it) still parses, with no time namespace.
#[test]
fn a_launcher_record_carries_the_time_namespace_and_an_older_line_has_none() {
    let record: Record = Record::Launcher { pid: 4242, start: 987_654, pid_namespace: 4_026_531_836, time_namespace: Some(4_026_531_834), path: PathBuf::from("/sys/fs/cgroup/a b/c.scope") };
    assert_eq!(parse_record(&record.to_string()), Ok(record));
    let older: Record = Record::Launcher { pid: 4242, start: 987_654, pid_namespace: 4_026_531_836, time_namespace: None, path: PathBuf::from("/sys/fs/cgroup/a b/c.scope") };
    assert_eq!(older.to_string(), "launcher 4242 987654 4026531836 /sys/fs/cgroup/a b/c.scope");
    assert_eq!(parse_record(&older.to_string()), Ok(older));
    for damaged in ["launcher 4 5 6 7", "launcher 4 5 6 x /a", "launcher 4 5 6 7 ", "launcher 4 5 6 -7 /a"] {
        assert!(parse_record(damaged).is_err(), "{damaged:?} was understood");
    }
}

/// The helper process of the time namespace test:
/// it asks what the recorded launcher is, with the start time it reads itself, and what a recovery of the planted journal says.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone"]
fn time_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let identity: String = std::env::var(FIXTURE_IDENTITY).expect("an identity");
    let numbers: Vec<u64> = identity.split(' ').map(|part| { return part.parse::<u64>().expect("a number"); }).collect();
    let (pid, start, pid_namespace, recorded_time): (i32, u64, u64, u64) = (i32::try_from(numbers[0]).expect("a pid"), numbers[1], numbers[2], numbers[3]);
    let own_start: String = process_start(pid).map_or_else(|error| { return format!("err {error}"); }, |found| { return found.to_string(); });
    let journal: PathBuf = crate::journal_support::journal_dirs(&root).into_iter().next().expect("a journal");
    let note: String = survivor_note(&journal).unwrap_or_else(|| { return "no survivor".to_string(); });
    fs::write(
        root.join("fixture-result"),
        format!(
            "own start {own_start}\ntime namespace here {}\nwith the recorded time namespace {:?}\nwith none (an older line) {:?}\nwith another time namespace {:?}\nrecovery {}\n",
            time_namespace_id().map_or_else(|error| { return format!("err {error}"); }, |found| { return found.to_string(); }),
            launcher_state(pid, start, pid_namespace, Some(recorded_time)),
            launcher_state(pid, start, pid_namespace, None),
            launcher_state(pid, start, pid_namespace, Some(recorded_time + 1)),
            note.lines().next().unwrap_or("")
        ),
    )
    .expect("write the result");
}

/// Runs the helper (in a new user and time namespace with a boot-time offset when `shifted`) and returns what it wrote.
fn run_time_fixture(root: &Path, identity: &str, shifted: bool) -> String {
    let executable = std::env::current_exe().expect("this test executable");
    let mut command = if shifted {
        let mut unshare = Command::new("unshare");
        // A new user namespace owns the new time namespace; the process id, mount and cgroup namespaces stay as they are, so only the
        // reader's boot-time offset differs (an offset of two hours; the start time is in clock ticks, so it moves by hundreds of thousands).
        unshare.args(["--user", "--map-root-user", "--time", "--boottime", "7200", "--fork", "--"]).arg(&executable);
        unshare
    } else {
        Command::new(&executable)
    };
    let status = command
        .args(["--ignored", "--exact", "round_fourteen_tests::time_namespace_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root)
        .env(FIXTURE_IDENTITY, identity)
        .status()
        .expect("the helper (shifted runs need unshare with permission for a user and time namespace)");
    assert!(status.success(), "the helper failed: {status:?}");
    return fs::read_to_string(root.join("fixture-result")).expect("the helper's result");
}

/// Acceptance rows A1, V2 (13 section 3): the same process id namespace, procfs and journal, a recovery in another time namespace with a
/// boot-time offset: the live launcher's start time reads differently there, and the question answers "unknown", never "gone"; a line
/// without a time namespace (an older version) is unknown as well.
/// The control is a recovery in the writer's own time namespace, where the same question answers "alive" and the start time is the one
/// recorded; the prerequisite is shown by the helper's own reading of a different start time in the other namespace.
#[test]
fn a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone() {
    let Some(route) = delegated_route() else {
        return;
    };
    let launcher = Held(Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts"));
    let pid: i32 = i32::try_from(launcher.0.id()).expect("a process id");
    let start: u64 = process_start(pid).expect("the start time here");
    let (pid_namespace, time_namespace): (u64, u64) = (pid_namespace_id().expect("pid namespace"), time_namespace_id().expect("time namespace"));
    let identity: String = format!("{pid} {start} {pid_namespace} {time_namespace}");
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let record: Record = Record::Launcher { pid, start, pid_namespace, time_namespace: Some(time_namespace), path: planned.directory().to_path_buf() };

    let same_root = scratch();
    fs::create_dir_all(same_root.path().join(CRATE_DIR)).expect("dir");
    let journal: PathBuf = plant(same_root.path(), &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), format!("{}\n{record}\n", planned.record())).expect("records");
    let same: String = run_time_fixture(same_root.path(), &identity, false);

    let other_root = scratch();
    fs::create_dir_all(other_root.path().join(CRATE_DIR)).expect("dir");
    let other_journal: PathBuf = plant(other_root.path(), &[snapshot("original")], &[]);
    fs::write(other_journal.join("groups"), format!("{}\n{record}\n", planned.record())).expect("records");
    let shifted: String = run_time_fixture(other_root.path(), &identity, true);

    assert!(same.contains(&format!("own start {start}\n")) && same.contains("with the recorded time namespace Alive"), "the control did not move: {same}");
    assert!(same.contains("with none (an older line) Unknown(") && same.contains("with another time namespace Unknown("), "the same time namespace did not check the record: {same}");
    assert!(shifted.contains("own start ") && !shifted.contains(&format!("own start {start}\n")), "the prerequisite failed: the other time namespace read the same start time: {shifted}");
    assert!(shifted.contains("with the recorded time namespace Unknown("), "a live launcher read from another time namespace was not unknown: {shifted}");
    assert!(shifted.contains("with none (an older line) Unknown("), "{shifted}");
    assert!(!shifted.contains("Gone"), "a live launcher read from another time namespace was read as gone: {shifted}");
    assert!(!shifted.contains("recovery no survivor"), "the recovery in another time namespace cleared the journal of a live launcher: {shifted}");
}
