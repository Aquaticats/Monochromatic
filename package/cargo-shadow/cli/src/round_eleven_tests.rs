//! What:
//! Round 11 tests: the launcher record (the owner's pick "Record the launcher" for the way out of 9a D1).
//! Why:
//! A run that never recorded its end is refused (round 10), which sends every killed wrapper to a person.
//! After the launcher is spawned the wrapper notes its process id, start time and process id namespace; a recovery that finds the
//! launcher gone knows it can never join the scope, so it may read the scope as final, and a launcher that is alive keeps blocking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('launcher record', () => { it('a gone launcher lets a killed run recover by itself', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::acknowledge::acknowledge;
use crate::bounded_run::{Bound, output_within};
use crate::cgroup_test_support::{delegated_route, scope_directory_of, start_in_scope, wait_until};
use crate::contain_counterexample_tests::wait_for_file;
use crate::control_group::{ControlGroup, LauncherState, Record, Route, launcher_state, parse_record, pid_namespace_id, process_start, view_of};
use crate::crates::LockSnapshot;
use crate::journal::{begin, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::{journal_dirs, plant};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_ELEVEN_FIXTURE_ROOT";

/// The environment variable that gives the helper's program the seconds it lives.
const FIXTURE_SECONDS: &str = "CARGO_SHADOW_ROUND_ELEVEN_FIXTURE_SECONDS";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile, so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Plants a journal whose `groups` file holds `records`.
fn journal_with_records(root: &Path, records: &str) -> PathBuf {
    let journal: PathBuf = plant(root, &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), records).expect("write the records");
    return journal;
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// A process that stands for a launcher that has not reached the manager: it waits at a gate file that the tests never create.
struct HeldLauncher {
    /// The process.
    child: Child,
}

/// The methods of the held launcher.
impl HeldLauncher {
    /// Starts it.
    fn start(directory: &Path) -> HeldLauncher {
        let gate: PathBuf = directory.join("gate");
        let child: Child = Command::new("sh")
            .arg("-c")
            .arg(format!("while [ ! -e {} ]; do sleep 0.02; done; exec sleep 600", gate.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("starts the held launcher");
        return HeldLauncher { child };
    }

    /// The process id.
    fn pid(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("a process id");
    }

    /// The record the wrapper would have appended for this process and the scope.
    fn record(&self, scope: &Path) -> Record {
        return Record::Launcher { pid: self.pid(), start: process_start(self.pid()).expect("the start time"), pid_namespace: pid_namespace_id().expect("the namespace"), path: scope.to_path_buf() };
    }

    /// Ends it and reaps it, so the process id is gone.
    fn end(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// How the guard cleans up.
impl Drop for HeldLauncher {
    /// What:
    /// `fn drop(&mut self)` ends the process.
    /// Why:
    /// A failed assertion must not leave a process behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        self.end();
    }
}

/// A route whose scopes are made inside a delegated holder scope, so a test can create a control group by hand where a manager would.
fn route_inside(holder: &Path, route: &Route) -> Route {
    return Route { slice_dir: holder.to_path_buf(), slice_view: view_of(holder).expect("the view of the holder"), ..route.clone() };
}

/// Contract A1 (round 11): the record names the launcher, reads back as written, and a damaged one is not understood.
#[test]
fn a_launcher_record_reads_back_and_a_damaged_one_is_not_understood() {
    let record: Record = Record::Launcher { pid: 4242, start: 987_654, pid_namespace: 4_026_531_836, path: PathBuf::from("/sys/fs/cgroup/a b/c.scope") };
    assert_eq!(parse_record(&record.to_string()), Ok(record));
    for damaged in ["launcher", "launcher 4242", "launcher 4242 5 6", "launcher 0 5 6 /x", "launcher -4 5 6 /x", "launcher x 5 6 /x", "launcher 4 y 6 /x", "launcher 4 5 z /x"] {
        assert!(parse_record(damaged).is_err(), "{damaged:?} was understood");
    }
}

/// Contract A1 (round 11), the question the recovery asks first:
/// a live process is alive, a reaped one is gone, a reused id (another start time) is gone, another process id namespace is unknown,
/// and a zombie that is not reaped yet is alive because only its reaping shows it ended.
#[test]
fn the_launcher_question_tells_gone_from_alive_reused_other_namespace_and_zombie() {
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let (pid, start, namespace): (i32, u64, u64) = (held.pid(), process_start(held.pid()).expect("start"), pid_namespace_id().expect("namespace"));
    assert_eq!(launcher_state(pid, start, namespace), LauncherState::Alive);
    assert_eq!(launcher_state(pid, start + 1, namespace), LauncherState::Gone, "a reused process id (another start time) was read as the launcher");
    assert!(matches!(launcher_state(pid, start, namespace + 1), LauncherState::Unknown(_)), "another process id namespace was read as an answer");
    held.end();
    assert_eq!(launcher_state(pid, start, namespace), LauncherState::Gone);

    let mut zombie: Child = Command::new("true").spawn().expect("starts");
    let zombie_pid: i32 = i32::try_from(zombie.id()).expect("a process id");
    let zombie_start: u64 = process_start(zombie_pid).expect("start");
    wait_until("the process to end and wait for its reaping", &|| { return fs::read_to_string(format!("/proc/{zombie_pid}/stat")).is_ok_and(|text| { return text.rsplit(')').next().is_some_and(|rest| { return rest.trim_start().starts_with('Z'); }); }); });
    assert_eq!(launcher_state(zombie_pid, zombie_start, namespace), LauncherState::Alive, "an unreaped zombie was read as gone");
    let _ = zombie.wait();
    assert_eq!(launcher_state(zombie_pid, zombie_start, namespace), LauncherState::Gone);
}

/// Acceptance rows A1, V2 (round 11): a launcher that is alive keeps blocking, whatever the scope looks like, and no acknowledgement
/// overrides it:
/// the scope is absent (the launcher has not reached the manager) and the scope exists and is empty (created, not attached),
/// the two round 10 schedules of 9a D1.
#[test]
fn a_live_launcher_blocks_whether_its_scope_is_absent_or_empty() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let directory = scratch();
    let held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n{}\n", planned.record(), held.record(planned.directory())));

    let absent: Option<String> = survivor_note(&journal);
    acknowledge(&journal).expect("acknowledges");
    let absent_acknowledged: Option<String> = survivor_note(&journal);
    fs::remove_file(journal.join("acknowledged")).expect("take the acknowledgement away");

    fs::create_dir(planned.directory()).expect("the manager makes the control group before it attaches the launcher");
    let empty: Option<String> = survivor_note(&journal);
    fs::remove_dir(planned.directory()).expect("remove it again");

    for (label, note) in [("absent", absent), ("absent and acknowledged", absent_acknowledged), ("empty", empty)] {
        let text: String = note.unwrap_or_else(|| { panic!("{label}: a live launcher was cleared"); });
        assert!(text.contains("launcher") && text.contains("still running"), "{label}: {text}");
    }
}

/// Acceptance rows A1, V2 (round 11): once the launcher is gone, nothing can join the scope later, so the scope is final:
/// absent and empty both recover without an acknowledgement, and a scope that has a member still blocks.
/// A reused process id (another start time) is read as gone, which is the control that makes the live cases above mean something.
#[test]
fn a_gone_launcher_lets_the_recovery_read_the_scope_as_final() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    let root = repository();
    let record: Record = held.record(planned.directory());
    let Record::Launcher { pid, start, pid_namespace, path } = record.clone() else {
        panic!("a launcher record");
    };
    let reused: Record = Record::Launcher { pid, start: start + 1, pid_namespace, path };
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n{reused}\n", planned.record()));
    let reused_note: Option<String> = survivor_note(&journal);

    fs::write(journal.join("groups"), format!("{}\n{record}\n", planned.record())).expect("the real record");
    held.end();
    let gone_absent: Option<String> = survivor_note(&journal);
    fs::create_dir(planned.directory()).expect("the control group exists and is empty");
    let gone_empty: Option<String> = survivor_note(&journal);
    let _ = fs::remove_dir(planned.directory());

    assert_eq!(reused_note, None, "a reused process id kept the journal blocked");
    assert_eq!(gone_absent, None, "a gone launcher and an absent scope still needed an acknowledgement");
    assert_eq!(gone_empty, None, "a gone launcher and an empty scope still needed an acknowledgement");
    let recovered: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the journal was not settled without an acknowledgement: {recovered:?}");
}

/// Acceptance rows A1, V2 (round 11): what the launcher record cannot prove still waits for an acknowledgement:
/// another process id namespace, a missing launcher record (a journal of an older version), a launcher record that names no scope of
/// the journal, and a launcher record that is not understood.
/// Each is cleared by `cargo-shadow acknowledge` and not otherwise.
#[test]
fn what_the_launcher_record_cannot_prove_waits_for_an_acknowledgement() {
    let Some(route) = delegated_route() else {
        return;
    };
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let Record::Launcher { pid, start, pid_namespace, .. } = held.record(planned.directory()) else {
        panic!("a launcher record");
    };
    held.end();
    let other_namespace: Record = Record::Launcher { pid, start, pid_namespace: pid_namespace + 1, path: planned.directory().to_path_buf() };
    let elsewhere: Record = Record::Launcher { pid, start, pid_namespace, path: planned.directory().with_file_name("another.scope") };
    let cases: Vec<(&str, String)> = vec![
        ("another process id namespace", format!("{}\n{other_namespace}\n", planned.record())),
        ("no launcher record", format!("{}\n", planned.record())),
        ("a launcher record for another scope", format!("{}\n{elsewhere}\n", planned.record())),
        ("a launcher record that is not understood", format!("{}\nlauncher {pid} x\n", planned.record())),
    ];
    for (label, records) in cases {
        let root = repository();
        let journal: PathBuf = journal_with_records(root.path(), &records);
        let blocked: Option<String> = survivor_note(&journal);
        assert!(blocked.as_ref().is_some_and(|text| { return text.contains("cargo-shadow acknowledge"); }), "{label}: not refused with the way out: {blocked:?}");
        acknowledge(&journal).expect("acknowledges");
        assert_eq!(survivor_note(&journal), None, "{label}: the acknowledgement did not clear it");
    }
}

/// Durability of the launcher record (round 11): the line is appended without a sync, which is enough because a lost or torn line
/// is read as no proof.
/// Every proper prefix of a launcher line, as a torn write or a lost tail leaves it, keeps the journal blocked; none reads as gone.
#[test]
fn a_torn_launcher_line_never_reads_as_a_gone_launcher() {
    let Some(route) = delegated_route() else {
        return;
    };
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let line: String = held.record(planned.directory()).to_string();
    held.end();
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "");
    for length in 0..line.len() {
        if !line.is_char_boundary(length) {
            continue;
        }
        fs::write(journal.join("groups"), format!("{}\n{}", planned.record(), &line[..length])).expect("write a torn line");
        assert!(survivor_note(&journal).is_some(), "a launcher line cut after {length} bytes ({:?}) read as a gone launcher", &line[..length]);
    }
    fs::write(journal.join("groups"), format!("{}\n{line}\n", planned.record())).expect("the whole line");
    assert_eq!(survivor_note(&journal), None, "the whole line did not count, so the cuts above prove nothing");
}

/// The helper process of the killed-wrapper test:
/// it begins a transaction and runs a program through the bounded runner; the program says when it started and then lives for the
/// given seconds. Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone"]
fn wrapper_running_a_program_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let seconds: String = std::env::var(FIXTURE_SECONDS).expect("seconds");
    let _transaction = begin(&root, &[snapshot("original")], false).expect("begin");
    let started: PathBuf = root.join("started");
    let script: String = format!(": > {}; exec sleep {seconds}", started.display());
    let bound: Bound = Bound { limit: Duration::from_secs(600), what: "the contained command".to_string(), rerun: "contained-command".to_string() };
    let _ = output_within(Command::new("sh").args(["-c", &script]), None, &bound);
}

/// Acceptance rows A1, V2, V3/V4 (round 11, the owner's pick), end to end:
/// a wrapper is killed while its program runs; while the program lives the recovery refuses (a member, then a live launcher);
/// once the program has ended and the manager collected the scope, the same recovery settles the journal with no acknowledgement.
#[test]
fn a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    let mut wrapper: Child = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_eleven_tests::wrapper_running_a_program_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .env(FIXTURE_SECONDS, "3")
        .spawn()
        .expect("the wrapper helper");
    wait_for_file(&root.path().join("started"));
    wrapper.kill().expect("kill the wrapper while its program runs");
    let _ = wrapper.wait();
    let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let groups: String = journal.join("groups").display().to_string();
    let record: String = fs::read_to_string(&groups).expect("the records");
    let scope: PathBuf = scope_directory_of(&groups);

    let while_running: Option<String> = survivor_note(&journal);
    wait_until("the program to end and the manager to collect the scope", &|| { return !scope.exists(); });
    let after: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(record.lines().any(|line| { return matches!(parse_record(line), Ok(Record::Launcher { .. })); }), "the wrapper did not note its launcher: {record:?}");
    assert!(while_running.is_some_and(|text| { return text.contains("still running"); }), "the journal did not block while the program lived");
    assert_eq!(after, None, "the launcher ended and the scope was collected, and the journal still waited: {record:?}");
    assert_eq!(live_lock(root.path()), "original", "the killed run was not recovered by itself: {messages:?}");
}
