//! What:
//! Round 11 tests:
//! the launcher record (the owner's pick "Record the launcher" for the way out of 9a D1).
//! Why:
//! A run that never recorded its end is refused (round 10),
//! which sends every killed wrapper to a person.
//! After the launcher is spawned the wrapper notes its process id,
//! start time and process id namespace;
//! a recovery that finds the
//! launcher gone knows it can never join the scope,
//! so it may read the scope as final,
//! and a launcher that is alive keeps blocking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('launcher record', () => { it('a gone launcher lets a killed run recover by itself', ...) })
//! ```

use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};
use std::time::Duration;

use crate::acknowledge::acknowledge;
use crate::bounded_run::{
    Bound,
    output_within,
};
use crate::cgroup_test_support::{
    delegated_route,
    gate_looks,
    scope_directory_of,
    start_in_scope,
    wait_until,
    wait_up_to,
};
use crate::control_group::{
    ControlGroup,
    LauncherState,
    Record,
    Route,
    launcher_state,
    parse_record,
    pid_namespace_id,
    process_start,
    view_of,
};
use crate::crates::LockSnapshot;
use crate::journal::{
    begin,
    recover,
};
use crate::journal_recover::survivor_note;
use crate::journal_support::{
    journal_dirs,
    plant,
};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_ELEVEN_FIXTURE_ROOT";

/// How often the program of the killed-wrapper control looks at its release gate (issue 6576).
/// It ends by itself once `gate_patience` is spent in looks of this length,
/// so a control that fails before its release line
/// (a failed assertion, a panic, or the whole run killed from outside)
/// cannot leave a shell polling for hours.
const PAYLOAD_GATE_LOOK: Duration = Duration::from_millis(50);

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot {
        crate_dir: CRATE_DIR.to_string(),
        source: source.to_string(),
    };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(
        root.path().join(CRATE_DIR).join("Cargo.lock"),
        LEFT_BY_THE_KILLED_RUN,
    )
    .expect("lock");
    return root;
}

/// A judge that refuses the live lockfile,
/// so that a recovery which goes ahead restores the snapshot.
fn refusing(
    _before: &[LockSnapshot],
    _live: &[LockSnapshot],
) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Plants a journal whose `groups` file holds `records`.
fn journal_with_records(
    root: &Path,
    records: &str,
) -> PathBuf {
    let journal: PathBuf = plant(root, &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), records).expect("write the records");
    return journal;
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// A process that stands for a launcher that has not reached the manager:
/// it waits at a gate file that the tests never create.
struct HeldLauncher {
    /// The process.
    child: Child,
}

/// The methods of the held launcher.
impl HeldLauncher {
    /// Starts it.
    fn start(directory: &Path) -> HeldLauncher {
        let gate: PathBuf = directory.join("gate");
        // The gate is one the tests never make, so the loop counts its looks and gives up on its own (issue 6576): a control that
        // fails before it ends this process leaves nothing polling at a gate nobody will open.
        let script: String = format!(
            "i=0; while [ ! -e {gate} ] && [ $i -lt {looks} ]; do sleep 0.02; i=$((i+1)); done; exec sleep 600",
            gate = gate.display(),
            looks = gate_looks(Duration::from_millis(20))
        );
        let child: Child = Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("starts the held launcher");
        return HeldLauncher {
            child,
        };
    }

    /// The process id.
    fn pid(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("a process id");
    }

    /// The record the wrapper would have appended for this process and the scope.
    fn record(
        &self,
        scope: &Path,
    ) -> Record {
        return Record::Launcher {
            pid: self.pid(),
            start: process_start(self.pid()).expect("the start time"),
            pid_namespace: pid_namespace_id().expect("the namespace"),
            path: scope.to_path_buf(),
        };
    }

    /// Ends it and reaps it,
    /// so the process id is gone.
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

/// A route whose scopes are made inside a delegated holder scope,
/// so a test can create a control group by hand where a manager would.
fn route_inside(
    holder: &Path,
    route: &Route,
) -> Route {
    return Route {
        slice_dir: holder.to_path_buf(),
        slice_view: view_of(holder).expect("the view of the holder"),
        ..route.clone()
    };
}

/// Contract A1 (round 11):
/// the record names the launcher,
/// reads back as written,
/// and a damaged one is not understood.
#[test]
fn a_launcher_record_reads_back_and_a_damaged_one_is_not_understood() {
    let record: Record = Record::Launcher {
        pid: 4242,
        start: 987_654,
        pid_namespace: 4_026_531_836,
        path: PathBuf::from("/sys/fs/cgroup/a b/c.scope"),
    };
    assert_eq!(parse_record(&record.to_string()), Ok(record));
    for damaged in [
        "launcher",
        "launcher 4242",
        "launcher 4242 5 6",
        "launcher 0 5 6 /x",
        "launcher -4 5 6 /x",
        "launcher x 5 6 /x",
        "launcher 4 y 6 /x",
        "launcher 4 5 z /x",
    ] {
        assert!(parse_record(damaged).is_err(), "{damaged:?} was understood");
    }
}

/// Contract A1 (round 11),
/// the question the recovery asks first:
/// a live process is alive,
/// a reaped one is gone,
/// a different start time is unknown (a reused id cannot be told from a start time read on another time basis),
/// another process id namespace is unknown,
/// and a zombie that is not reaped yet is alive because only its reaping shows it ended.
#[test]
fn the_launcher_question_tells_gone_from_alive_reused_other_namespace_and_zombie() {
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let (pid, start, namespace): (i32, u64, u64) = (
        held.pid(),
        process_start(held.pid()).expect("start"),
        pid_namespace_id().expect("namespace"),
    );
    assert_eq!(launcher_state(pid, start, namespace), LauncherState::Alive);
    assert!(
        matches!(launcher_state(pid, start + 1, namespace), LauncherState::Unknown(text) if text.contains("start time")),
        "another start time was read as a gone launcher (round 15: it cannot be told from another time basis)"
    );
    assert!(
        matches!(
            launcher_state(pid, start, namespace + 1),
            LauncherState::Unknown(_)
        ),
        "another process id namespace was read as an answer"
    );
    held.end();
    assert_eq!(launcher_state(pid, start, namespace), LauncherState::Gone);

    let mut zombie: Child = Command::new("true").spawn().expect("starts");
    let zombie_pid: i32 = i32::try_from(zombie.id()).expect("a process id");
    let zombie_start: u64 = process_start(zombie_pid).expect("start");
    wait_until("the process to end and wait for its reaping", &|| {
        return fs::read_to_string(format!("/proc/{zombie_pid}/stat")).is_ok_and(|text| {
            return text.rsplit(')').next().is_some_and(|rest| {
                return rest.trim_start().starts_with('Z');
            });
        });
    });
    assert_eq!(
        launcher_state(zombie_pid, zombie_start, namespace),
        LauncherState::Alive,
        "an unreaped zombie was read as gone"
    );
    let _ = zombie.wait();
    assert_eq!(
        launcher_state(zombie_pid, zombie_start, namespace),
        LauncherState::Gone
    );
}

/// The real recovery and the real admission refuse while the launcher lives (certification,
/// third pass):
/// the recovery names the
/// running launcher,
/// restores nothing and keeps the journal,
/// and a new transaction is not admitted beside it;
/// this holds also after
/// an acknowledgement,
/// which no entry point may let override a live launcher.
fn assert_entry_points_refuse(
    root: &Path,
    journal: &Path,
    label: &str,
) {
    let messages: Vec<String> = recover(root, &refusing);
    assert!(
        messages.iter().any(|message| {
            return message.contains("launcher") && message.contains("still running");
        }),
        "{label}: the recovery did not name the running launcher: {messages:?}"
    );
    assert_eq!(
        live_lock(root),
        LEFT_BY_THE_KILLED_RUN,
        "{label}: a recovery went ahead beside a live launcher: {messages:?}"
    );
    assert!(
        journal.exists(),
        "{label}: the journal was removed beside a live launcher: {messages:?}"
    );
    assert!(
        begin(root, &[snapshot("original")], false).is_err(),
        "{label}: a transaction was admitted beside a live launcher"
    );
    assert_eq!(
        journal_dirs(root).len(),
        1,
        "{label}: the refused admission left a journal behind"
    );
}

/// Acceptance rows A1,
/// V2 (round 11):
/// a launcher that is alive keeps blocking,
/// whatever the scope looks like,
/// and no acknowledgement
/// overrides it:
/// the scope is absent (the launcher has not reached the manager) and the scope exists and is empty (created,
/// not attached),
/// the two round 10 schedules of 9a D1.
#[test]
fn a_live_launcher_blocks_whether_its_scope_is_absent_or_empty() {
    let route = delegated_route();
    let holder = start_in_scope(Command::new("sleep").arg("600"));
    let inside: Route = route_inside(holder.group.directory(), &route);
    let directory = scratch();
    let held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    let root = repository();
    let journal: PathBuf = journal_with_records(
        root.path(),
        &format!(
            "{}\n{}\n",
            planned.record(),
            held.record(planned.directory())
        ),
    );

    let absent: Option<String> = survivor_note(&journal);
    assert_entry_points_refuse(root.path(), &journal, "absent");
    acknowledge(&journal).expect("acknowledges");
    let absent_acknowledged: Option<String> = survivor_note(&journal);
    assert_entry_points_refuse(root.path(), &journal, "absent and acknowledged");
    fs::remove_file(journal.join("acknowledged")).expect("take the acknowledgement away");

    fs::create_dir(planned.directory())
        .expect("the manager makes the control group before it attaches the launcher");
    let empty: Option<String> = survivor_note(&journal);
    assert_entry_points_refuse(root.path(), &journal, "empty");
    acknowledge(&journal).expect("acknowledges");
    let empty_acknowledged: Option<String> = survivor_note(&journal);
    assert_entry_points_refuse(root.path(), &journal, "empty and acknowledged");
    fs::remove_dir(planned.directory()).expect("remove it again");

    for (label, note) in [
        ("absent", absent),
        ("absent and acknowledged", absent_acknowledged),
        ("empty", empty),
        ("empty and acknowledged", empty_acknowledged),
    ] {
        let text: String = note.unwrap_or_else(|| {
            panic!("{label}: a live launcher was cleared");
        });
        assert!(
            text.contains("launcher") && text.contains("still running"),
            "{label}: {text}"
        );
    }
}

/// Acceptance rows A1,
/// V2 (round 11):
/// once the launcher is gone,
/// nothing can join the scope later,
/// so the scope is final:
/// absent and empty both recover without an acknowledgement,
/// and a scope that has a member still blocks.
/// A process id with another start time is unknown,
/// not gone (round 15):
/// the record's start time may be on another time basis.
#[test]
fn a_gone_launcher_lets_the_recovery_read_the_scope_as_final() {
    let route = delegated_route();
    let holder = start_in_scope(Command::new("sleep").arg("600"));
    let inside: Route = route_inside(holder.group.directory(), &route);
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    let root = repository();
    let record: Record = held.record(planned.directory());
    let Record::Launcher {
        pid,
        start,
        pid_namespace,
        path,
    } = record.clone()
    else {
        panic!("a launcher record");
    };
    let reused: Record = Record::Launcher {
        pid,
        start: start + 1,
        pid_namespace,
        path,
    };
    let journal: PathBuf =
        journal_with_records(root.path(), &format!("{}\n{reused}\n", planned.record()));
    let reused_note: Option<String> = survivor_note(&journal);

    fs::write(
        journal.join("groups"),
        format!("{}\n{record}\n", planned.record()),
    )
    .expect("the real record");
    held.end();
    let gone_absent: Option<String> = survivor_note(&journal);
    fs::create_dir(planned.directory()).expect("the control group exists and is empty");
    let gone_empty: Option<String> = survivor_note(&journal);
    let _ = fs::remove_dir(planned.directory());

    assert!(
        reused_note.is_some_and(|text| {
            return text.contains("start time") && text.contains("cargo-shadow acknowledge");
        }),
        "another start time was read as a gone launcher, or the way out is not named"
    );
    assert_eq!(
        gone_absent, None,
        "a gone launcher and an absent scope still needed an acknowledgement"
    );
    assert_eq!(
        gone_empty, None,
        "a gone launcher and an empty scope still needed an acknowledgement"
    );
    let recovered: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(
        live_lock(root.path()),
        "original",
        "the journal was not settled without an acknowledgement: {recovered:?}"
    );
}

/// Acceptance rows A1,
/// V2 (sixth pass,
/// groups 3 and 10):
/// a gone launcher and a scope that still exists and is empty go through the
/// real recovery,
/// which settles the journal without an acknowledgement (restoration and removal),
/// and then through admission,
/// which
/// admits a new transaction.
/// The scope directory is still there when the recovery runs.
#[test]
fn a_gone_launcher_with_an_existing_empty_scope_is_settled_by_the_real_recovery_and_admission() {
    let route = delegated_route();
    let holder = start_in_scope(Command::new("sleep").arg("600"));
    let inside: Route = route_inside(holder.group.directory(), &route);
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    let root = repository();
    let record: Record = held.record(planned.directory());
    let journal: PathBuf =
        journal_with_records(root.path(), &format!("{}\n{record}\n", planned.record()));
    held.end();
    fs::create_dir(planned.directory()).expect("the control group exists and is empty");
    let still_there: bool = planned.directory().is_dir();

    let recovered: Vec<String> = recover(root.path(), &refusing);

    let _ = fs::remove_dir(planned.directory());
    assert!(
        still_there,
        "the empty scope was not there when the recovery ran, so the schedule was not built"
    );
    assert_eq!(
        live_lock(root.path()),
        "original",
        "the real recovery did not settle a gone launcher with an empty scope: {recovered:?}"
    );
    assert!(
        !journal.exists(),
        "the settled journal was not removed: {recovered:?}"
    );
    assert!(
        begin(root.path(), &[snapshot("original")], false).is_ok(),
        "a transaction was refused after the journal was settled"
    );
}

/// Acceptance rows A1,
/// V2 (11b 2.4):
/// a gone launcher does not clear a scope that has a member,
/// and no acknowledgement overrides it:
/// the descendants of a program that ended are still in the scope.
#[test]
fn a_gone_launcher_does_not_clear_a_scope_with_a_member() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let scope = start_in_scope(Command::new("sleep").arg("600"));
    let directory = scratch();
    let mut stand_in: HeldLauncher = HeldLauncher::start(directory.path());
    let record: Record = stand_in.record(scope.group.directory());
    stand_in.end();
    let root = repository();
    let journal: PathBuf = journal_with_records(
        root.path(),
        &format!("{}\n{record}\n", scope.group.record()),
    );
    assert!(
        scope.group.members().is_ok_and(|found| {
            return !found.is_empty();
        }),
        "the scope has no member, so the schedule is not made"
    );

    let blocked: Option<String> = survivor_note(&journal);
    // The real recovery and admission refuse too (fourth certification pass), before and after the acknowledgement: the member-specific
    // refusal, the live lockfile unchanged, the journal retained, no transaction admitted.
    assert_entry_points_refuse_for_a_member(root.path(), &journal, "unacknowledged");
    acknowledge(&journal).expect("acknowledges");
    let acknowledged: Option<String> = survivor_note(&journal);
    assert_entry_points_refuse_for_a_member(root.path(), &journal, "acknowledged");

    assert!(
        blocked.is_some_and(|text| {
            return text.contains("still running");
        }),
        "a gone launcher cleared a scope with a member"
    );
    assert!(
        acknowledged.is_some_and(|text| {
            return text.contains("still running");
        }),
        "an acknowledgement overrode a scope with a member"
    );
    // The control: with the member stopped the same journal is recovered, so the refusals above were the member's.
    scope
        .group
        .stop(std::time::Duration::from_secs(5))
        .expect("stop the member");
    let after: Vec<String> = recover(root.path(), &refusing);
    assert!(
        journal_dirs(root.path()).is_empty(),
        "the control: with the member gone the journal was not recovered: {after:?}"
    );
}

/// The real recovery and admission refuse for a live scope member:
/// the refusal names a process that is still running,
/// nothing is
/// restored,
/// the journal stays,
/// and a new transaction is not admitted.
fn assert_entry_points_refuse_for_a_member(
    root: &Path,
    journal: &Path,
    label: &str,
) {
    let messages: Vec<String> = recover(root, &refusing);
    assert!(
        messages.iter().any(|message| {
            return message.contains("still running");
        }),
        "{label}: the recovery did not name the running member: {messages:?}"
    );
    assert_eq!(
        live_lock(root),
        LEFT_BY_THE_KILLED_RUN,
        "{label}: a recovery went ahead beside a live scope member: {messages:?}"
    );
    assert!(
        journal.exists(),
        "{label}: the journal was removed beside a live member"
    );
    assert!(
        begin(root, &[snapshot("original")], false).is_err(),
        "{label}: a transaction was admitted beside a live member"
    );
    assert_eq!(
        journal_dirs(root).len(),
        1,
        "{label}: the refused admission left a journal behind"
    );
}

/// The real recovery and admission hold an uncertainty the launcher record cannot settle:
/// the recovery says how to acknowledge,
/// nothing
/// is restored,
/// the journal stays,
/// and a new transaction is not admitted (sixth pass,
/// group 3).
fn assert_entry_points_hold_for_uncertainty(
    root: &Path,
    journal: &Path,
    label: &str,
) {
    let messages: Vec<String> = recover(root, &refusing);
    assert!(
        messages.iter().any(|message| {
            return message.contains("cargo-shadow acknowledge");
        }),
        "{label}: the recovery did not name the way out: {messages:?}"
    );
    assert_eq!(
        live_lock(root),
        LEFT_BY_THE_KILLED_RUN,
        "{label}: a recovery went ahead on an uncertain launcher: {messages:?}"
    );
    assert!(
        journal.exists(),
        "{label}: the journal was removed on an uncertain launcher"
    );
    assert!(
        begin(root, &[snapshot("original")], false).is_err(),
        "{label}: a transaction was admitted on an uncertain launcher"
    );
    assert_eq!(
        journal_dirs(root).len(),
        1,
        "{label}: the refused admission left a journal behind"
    );
}

/// Acceptance rows A1,
/// V2 (round 11):
/// what the launcher record cannot prove still waits for an acknowledgement:
/// another process id namespace,
/// a missing launcher record (a journal of an older version),
/// a launcher record that names no scope of
/// the journal,
/// and a launcher record that is not understood.
/// Each is cleared by `cargo-shadow acknowledge` and not otherwise.
#[test]
fn what_the_launcher_record_cannot_prove_waits_for_an_acknowledgement() {
    let route = delegated_route();
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let Record::Launcher {
        pid,
        start,
        pid_namespace,
        ..
    } = held.record(planned.directory())
    else {
        panic!("a launcher record");
    };
    held.end();
    let other_namespace: Record = Record::Launcher {
        pid,
        start,
        pid_namespace: pid_namespace + 1,
        path: planned.directory().to_path_buf(),
    };
    let elsewhere: Record = Record::Launcher {
        pid,
        start,
        pid_namespace,
        path: planned.directory().with_file_name("another.scope"),
    };
    let cases: Vec<(&str, String)> = vec![
        (
            "another process id namespace",
            format!("{}\n{other_namespace}\n", planned.record()),
        ),
        ("no launcher record", format!("{}\n", planned.record())),
        (
            "a launcher record for another scope",
            format!("{}\n{elsewhere}\n", planned.record()),
        ),
        (
            "a launcher record that is not understood",
            format!("{}\nlauncher {pid} x\n", planned.record()),
        ),
    ];
    for (label, records) in cases {
        let root = repository();
        let journal: PathBuf = journal_with_records(root.path(), &records);
        let blocked: Option<String> = survivor_note(&journal);
        assert!(
            blocked.as_ref().is_some_and(|text| {
                return text.contains("cargo-shadow acknowledge");
            }),
            "{label}: not refused with the way out: {blocked:?}"
        );
        // Sixth pass, group 3: the real recovery and admission hold the same uncertainty, then the acknowledgement lets the recovery settle.
        assert_entry_points_hold_for_uncertainty(root.path(), &journal, label);
        acknowledge(&journal).expect("acknowledges");
        assert_eq!(
            survivor_note(&journal),
            None,
            "{label}: the acknowledgement did not clear it"
        );
        let settled: Vec<String> = recover(root.path(), &refusing);
        assert_eq!(
            live_lock(root.path()),
            "original",
            "{label}: the acknowledged journal was not settled by the recovery: {settled:?}"
        );
        assert!(
            !journal.exists(),
            "{label}: the settled journal was not removed"
        );
    }
}

/// Durability of the launcher record (round 11):
/// the line is appended without a sync,
/// which is enough because a lost or torn line
/// is read as no proof.
/// Every proper prefix of the bytes production appends (the line and its line feed),
/// as a torn write or a lost tail leaves it,
/// keeps
/// the journal blocked,
/// including the whole text with only its line feed missing;
/// none reads as a gone launcher (11b 2.6).
#[test]
fn a_torn_launcher_line_never_reads_as_a_gone_launcher() {
    let route = delegated_route();
    let directory = scratch();
    let mut held: HeldLauncher = HeldLauncher::start(directory.path());
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    // The bytes `append_record_with` writes: the display text of the record and one line feed.
    let bytes: String = format!("{}\n", held.record(planned.directory()));
    held.end();
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "");
    for length in 0..bytes.len() {
        if !bytes.is_char_boundary(length) {
            continue;
        }
        fs::write(
            journal.join("groups"),
            format!("{}\n{}", planned.record(), &bytes[..length]),
        )
        .expect("write a torn line");
        assert!(
            survivor_note(&journal).is_some(),
            "a launcher line cut after {length} of {} bytes ({:?}) read as a gone launcher",
            bytes.len(),
            &bytes[..length]
        );
    }
    fs::write(
        journal.join("groups"),
        format!("{}\n{bytes}", planned.record()),
    )
    .expect("the whole line");
    assert_eq!(
        survivor_note(&journal),
        None,
        "the whole line did not count, so the cuts above prove nothing"
    );
}

/// The identity of the program the killed-wrapper control runs, as the program publishes it:
/// the process id, start time and process id namespace the run's own launcher record holds.
#[derive(Clone, Copy)]
struct PublishedProgram {
    /// The program's process id.
    pid: i32,
    /// Its start time in procfs.
    start: u64,
    /// The inode of its process id namespace.
    pid_namespace: u64,
}

/// What the killed-wrapper control holds while its program runs:
/// the fixture that runs the program through a delegated scope,
/// and what this process needs to end that program and its scope however the control ends.
/// Why:
/// The fixture is the process the control kills on purpose,
/// so it cannot be what ends the program:
/// a control that fails after that kill (a failed assertion, a panic)
/// left the program polling at its release gate, in a scope of the manager's that outlived the run
/// (issue 6576:
/// twenty-five such shells on this host on 2026-11-27,
/// the oldest twelve hours old).
/// The program's own bound ends it even when nothing here runs;
/// this guard ends it at once on the paths that do run, and lets a test prove the program and its scope gone.
struct ProgramInItsScope {
    /// The fixture: it begins a transaction and runs the program through the bounded runner.
    fixture: Child,
    /// Where the program publishes who it is and where its release gate is made.
    root: PathBuf,
    /// The program's identity, once it has published it.
    program: Option<PublishedProgram>,
    /// The scope the program runs in, read from the program's own control group once it has published it.
    scope: Option<PathBuf>,
}

/// The methods of the type above.
impl ProgramInItsScope {
    /// What:
    /// `fn start(root: &Path) -> ProgramInItsScope` spawns the fixture for `root`.
    /// Why:
    /// The guard owns the fixture from the first moment,
    /// so a failure before the control's deliberate kill still ends it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static start(root: string): ProgramInItsScope
    /// ```
    fn start(root: &Path) -> ProgramInItsScope {
        let fixture: Child = Command::new(std::env::current_exe().expect("this test executable"))
            .args([
                "--ignored",
                "--exact",
                "round_eleven_tests::wrapper_running_a_program_fixture",
                "--nocapture",
            ])
            .env(FIXTURE_ROOT, root)
            .spawn()
            .expect("the wrapper helper");
        return ProgramInItsScope {
            fixture,
            root: root.to_path_buf(),
            program: None,
            scope: None,
        };
    }

    /// What:
    /// `fn published(&mut self) -> PublishedProgram` waits for the program to publish its process id and answers that identity,
    /// which the guard keeps for its end.
    /// Why:
    /// The record the wrapper writes is checked against what the program says about itself,
    /// and the guard must ask about the same process later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// published(): PublishedProgram
    /// ```
    fn published(&mut self) -> PublishedProgram {
        let id_file: PathBuf = self.root.join("payload-pid");
        wait_until("the program's process id", &|| {
            return id_file.exists();
        });
        let pid: i32 = fs::read_to_string(&id_file)
            .expect("the program's id")
            .trim()
            .parse::<i32>()
            .expect("a process id");
        let program: PublishedProgram = PublishedProgram {
            pid,
            start: process_start(pid).expect("the program's start time"),
            pid_namespace: pid_namespace_id().expect("the namespace"),
        };
        self.program = Some(program);
        self.scope = scope_of_cgroup_file(&self.root.join("payload-cgroup"));
        return program;
    }

    /// What:
    /// `fn scope(&self) -> Option<PathBuf>` is the scope the program reported, once it has.
    /// Why:
    /// A test asserts about the scope the program itself named,
    /// on the path where it reads no journal at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scope(): string | undefined
    /// ```
    fn scope(&self) -> Option<PathBuf> {
        return self.scope.clone();
    }

    /// What:
    /// `fn kill_the_fixture(&mut self)` kills and reaps the fixture.
    /// Why:
    /// This is the control's deliberate kill:
    /// the program must stay alive across it,
    /// so it cannot be what ends the program,
    /// and the kill must not leave the fixture behind either.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// killTheFixture(): void
    /// ```
    fn kill_the_fixture(&mut self) {
        let _ = self.fixture.kill();
        let _ = self.fixture.wait();
    }

    /// What:
    /// `fn release(&self)` makes the release gate the program waits at.
    /// Why:
    /// The control makes it at its own line of the schedule,
    /// and the guard makes the same file again when that line is never reached;
    /// making it twice is the same as making it once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// release(): void
    /// ```
    fn release(&self) {
        let _ = fs::write(self.root.join("release"), "go");
    }
}

/// How the guard cleans up.
impl Drop for ProgramInItsScope {
    /// What:
    /// `fn drop(&mut self)` ends the fixture,
    /// releases the program,
    /// and,
    /// where the program or its scope is still there,
    /// kills them and waits for both to be gone.
    /// Why:
    /// A control that fails before its release line must leave neither a program polling at a gate nobody will open
    /// nor a scope of the manager's behind (issue 6576).
    /// Nothing here may panic,
    /// because a panic while a test unwinds ends the test binary:
    /// every step ignores its failure and every wait has a bound,
    /// and a test's own last lines are what prove the program and the scope gone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = self.fixture.kill();
        let _ = self.fixture.wait();
        let _ = fs::write(self.root.join("release"), "go");
        if let Some(program) = self.program {
            let _ = wait_up_to(end_wait(), &|| {
                return launcher_state(program.pid, program.start, program.pid_namespace)
                    == LauncherState::Gone;
            });
        }
        if let Some(scope) = &self.scope {
            if scope.exists() {
                let _ = fs::write(scope.join("cgroup.kill"), "1");
            }
            let _ = wait_up_to(end_wait(), &|| {
                return !scope.exists();
            });
        }
    }
}

/// What:
/// `fn end_wait() -> Duration` is how long the guard gives the program to end and the manager to collect its scope.
/// Why:
/// The release gate is looked at every `PAYLOAD_GATE_LOOK`,
/// so the program ends within a look or two of the release,
/// and a stop that has to be forced is what `QUIESCE_BOUND` bounds;
/// twice that is a wait no correct host spends and no broken one waits out for long.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endWait(): number  // milliseconds
/// ```
fn end_wait() -> Duration {
    return crate::bounded_run::QUIESCE_BOUND * 2;
}

/// What:
/// `fn scope_of_cgroup_file(path: &Path) -> Option<PathBuf>` is the scope the program's own `/proc/self/cgroup` text names,
/// as a path below the cgroup root,
/// or nothing where the program has not written that file yet.
/// Why:
/// The guard must stop the scope even where the control failed before it read the journal's record,
/// and this host writes a version 1 line before the version 2 one (issue 6549),
/// so only the `0::` line names the group.
/// It answers instead of panicking because it runs in `Drop`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scopeOfCgroupFile(path: string): string | undefined
/// ```
fn scope_of_cgroup_file(path: &Path) -> Option<PathBuf> {
    let text: String = fs::read_to_string(path).ok()?;
    let v2: &str = text.lines().map(str::trim).find(|line| {
        return line.starts_with("0::");
    })?;
    let relative: &str = v2.strip_prefix("0::")?.trim_start_matches('/');
    return Some(PathBuf::from(crate::control_group::CGROUP_ROOT).join(relative));
}

/// The helper process of the killed-wrapper test:
/// it begins a transaction and runs a program through the bounded runner;
/// the program publishes its own process id,
/// then waits at a
/// release gate file (the wrapper is killed while it waits).
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone"]
fn wrapper_running_a_program_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let _transaction = begin(&root, &[snapshot("original")], false).expect("begin");
    // The program is the launcher (same process), so `$$` is the launcher's process id; the pid file appears whole.
    // It looks at its release gate for `gate_patience` in looks of `PAYLOAD_GATE_LOOK` and then ends by itself, so a control that
    // fails before its release line (a failed assertion, a panic, or the run killed from outside) leaves no polling shell behind
    // (issue 6576). The bound is three times the control's own worst case before that line: two `wait_until` waits of one
    // `STATE_WAIT` each.
    let script: String = format!(
        "cat /proc/self/cgroup > {cgroup}.part; mv {cgroup}.part {cgroup}; echo $$ > {pid}.part; mv {pid}.part {pid}; \
         i=0; while [ ! -e {release} ] && [ $i -lt {looks} ]; do sleep {look}; i=$((i+1)); done",
        cgroup = root.join("payload-cgroup").display(),
        pid = root.join("payload-pid").display(),
        release = root.join("release").display(),
        looks = gate_looks(PAYLOAD_GATE_LOOK),
        look = PAYLOAD_GATE_LOOK.as_secs_f64()
    );
    let bound: Bound = Bound {
        limit: Duration::from_secs(600),
        what: "the contained command".to_string(),
        rerun: "contained-command".to_string(),
    };
    let _ = output_within(Command::new("sh").args(["-c", &script]), None, &bound);
}

/// Acceptance rows A1,
/// V2,
/// V3/V4 (round 11,
/// the owner's pick),
/// end to end:
/// the real wrapper records the real launcher (its process id,
/// start time and namespace are compared with the program that says who
/// it is),
/// is killed once that record is whole while the program waits at a release gate,
/// and the recovery refuses while the program
/// lives;
/// after the program is released and the launcher is shown gone,
/// and the scope collected,
/// the same recovery settles the
/// journal with no acknowledgement.
#[test]
fn a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let root = repository();
    let mut control: ProgramInItsScope = ProgramInItsScope::start(root.path());
    let published: PublishedProgram = control.published();
    let payload: i32 = published.pid;
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");
    let groups: String = journal.join("groups").display().to_string();
    let scope: PathBuf = scope_directory_of(&groups);
    // The launcher line is whole (it ends with its line feed) and names the program that says who it is, before the wrapper is killed:
    // a kill inside the window between the spawn and the append is the contract's fallback, not this test's schedule.
    wait_until("a whole launcher record for the program", &|| {
        return fs::read_to_string(&groups).is_ok_and(|text| {
            return text.ends_with('\n')
                && text.lines().any(|line| {
                    return matches!(parse_record(line), Ok(Record::Launcher { .. }));
                });
        });
    });
    let record: String = fs::read_to_string(&groups).expect("the records");
    let Some(Ok(Record::Launcher {
        pid,
        start,
        pid_namespace,
        path,
    })) = record.lines().map(parse_record).find(|parsed| {
        return matches!(parsed, Ok(Record::Launcher { .. }));
    })
    else {
        panic!("no launcher record in {record:?}");
    };
    // The kill the control makes on purpose: the program stays alive across it, and the guard is what ends the program on every
    // path that does not reach the release line below (issue 6576).
    control.kill_the_fixture();

    assert_eq!(
        pid, payload,
        "the record names another process than the program"
    );
    assert_eq!(
        start,
        process_start(payload).expect("the program's start time"),
        "the record holds another start time than the program has"
    );
    assert_eq!(
        pid_namespace,
        pid_namespace_id().expect("the namespace"),
        "the record holds another process id namespace"
    );
    assert_eq!(path, scope, "the record names another scope");
    // The scope the journal records is compared with the cgroup the program itself reports (12b 3.1), not with the journal again, and
    // the program's group differs from the wrapper's own.
    crate::cgroup_test_support::assert_ran_in_recorded_scope(
        &groups,
        &fs::read_to_string(root.path().join("payload-cgroup")).expect("the program's own cgroup"),
        &crate::cgroup_test_support::wrapper_group(),
    );
    let while_running: Option<String> = survivor_note(&journal);
    control.release();
    wait_until("the launcher to be gone", &|| {
        return launcher_state(pid, start, pid_namespace) == LauncherState::Gone;
    });
    wait_until("the manager to collect the scope", &|| {
        return !scope.exists();
    });
    let after: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(
        while_running.is_some_and(|text| {
            return text.contains("still running");
        }),
        "the journal did not block while the program lived"
    );
    assert_eq!(
        after, None,
        "the launcher ended and the scope was collected, and the journal still waited: {record:?}"
    );
    assert_eq!(
        live_lock(root.path()),
        "original",
        "the killed run was not recovered by itself: {messages:?}"
    );
    assert!(
        journal_dirs(root.path()).is_empty(),
        "the journal was not completed and removed: {messages:?}"
    );
    // The control proves at its end that the program's process and its scope are gone (issue 6576): the two things a failure
    // before the release line used to leave behind.
    assert_eq!(
        launcher_state(payload, start, pid_namespace),
        LauncherState::Gone,
        "the program was still there when the control ended"
    );
    assert!(
        !scope.exists(),
        "the program's scope was still there when the control ended"
    );
}

/// The control for the leak itself (issue 6576):
/// a control that fails before its release line, the way a failed assertion or a panic does,
/// leaves neither the program it started nor that program's scope behind.
/// It judges only what it started:
/// other tests' programs and scopes are alive while it looks,
/// and it asks about none of them.
/// What no single test can judge is a scope left by a test it did not start;
/// there the program's own bound (`gate_patience`) is what ends it, and this test does not see it.
#[test]
fn a_control_that_fails_before_its_release_line_leaves_no_program_and_no_scope() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let root = repository();
    let mut began: Option<(PublishedProgram, PathBuf)> = None;
    let failed: std::thread::Result<()> =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut control: ProgramInItsScope = ProgramInItsScope::start(root.path());
            let program: PublishedProgram = control.published();
            let scope: PathBuf = control.scope().expect("the program's scope");
            began = Some((program, scope.clone()));
            // Both things this test judges were there to be left behind, so a green run of it proves something.
            assert_eq!(
                launcher_state(program.pid, program.start, program.pid_namespace),
                LauncherState::Alive,
                "the program was not running when the control failed"
            );
            assert!(
                scope.exists(),
                "the program's scope was not there when the control failed"
            );
            panic!("the control fails before its release line, as a failed assertion does");
        }));
    let (program, scope) = began.expect("the control started a program");
    assert!(
        failed.is_err(),
        "the control did not fail, so this case proves nothing"
    );
    wait_until("the program to be gone", &|| {
        return launcher_state(program.pid, program.start, program.pid_namespace)
            == LauncherState::Gone;
    });
    wait_until("the manager to collect the scope", &|| {
        return !scope.exists();
    });
}
