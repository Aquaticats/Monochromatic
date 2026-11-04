//! What:
//! Round 14 and 15 tests: the start time of a launcher read from another time namespace (13 section 3, 14 sections 2 and 3).
//! Why:
//! The start time in `/proc/<pid>/stat` is relative to the boot-time offset of the time namespace of the process that reads it, so
//! a recovery in another time namespace reads a live launcher's start time differently and must not take that for a reused id.
//! Round 15: the namespace number that identifies the writer's time namespace is recycled once the namespace has no process, so it
//! proves nothing; a start time that differs from the record is unknown, whatever the namespace numbers say.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round fourteen', () => { it('another time namespace is unknown, not gone', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::cgroup_test_support::delegated_route;
use crate::control_group::{ControlGroup, Record, launcher_state, parse_record, pid_namespace_id, process_start};
use crate::crates::LockSnapshot;
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_ROOT";

/// The environment variable that gives the helper the recorded identity of the launcher, as `pid start pid_namespace`.
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

/// The live text of the lockfile the killed run left, which the recovery must not change while the launcher is uncertain.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// Asserts what the helper's real recovery and admission did in the other time namespace: the live output is unchanged, the journal is
/// kept (and is the only one: the refused admission left none behind), and a new transaction is refused.
fn assert_entry_points_held(result: &str) {
    for line in ["entry lock unchanged true", "entry journal kept true", "entry admission refused true", "entry journals 1"] {
        assert!(result.lines().any(|found| { return found == line; }), "the real recovery or admission did not hold the uncertain launcher ({line} is missing): {result}");
    }
}

/// Stops a process by its id when it drops, so a failed assertion leaves no launcher behind.
struct StopOnDrop(
    /// The process id.
    i32,
);

/// How the guard cleans up.
impl Drop for StopOnDrop {
    /// What:
    /// `fn drop(&mut self)` sends the process `SIGKILL` and does not mind if it is gone.
    /// Why:
    /// The launcher of the recycled-number test lives in a time namespace of its own and has no parent of the test.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        crate::contain_counterexample_tests::stop_quietly(self.0);
    }
}

/// Contract A1 (round 15): the launcher line reads back as written; a line of the form of round 14 (a time namespace number before the
/// path) is not understood, so it waits for an acknowledgement like any journal that cannot be read.
#[test]
fn a_launcher_record_reads_back_and_a_line_with_a_time_namespace_is_not_understood() {
    let record: Record = Record::Launcher { pid: 4242, start: 987_654, pid_namespace: 4_026_531_836, path: PathBuf::from("/sys/fs/cgroup/a b/c.scope") };
    assert_eq!(record.to_string(), "launcher 4242 987654 4026531836 /sys/fs/cgroup/a b/c.scope");
    assert_eq!(parse_record(&record.to_string()), Ok(record));
    for refused in ["launcher 4 5 6 7 /a", "launcher 4 5 6 7", "launcher 4 5 6 x /a", "launcher 4 5 6 relative"] {
        assert!(parse_record(refused).is_err(), "{refused:?} was understood");
    }
}

/// Acceptance rows A1, V2 (round 15): with everything else equal, a start time that differs from the record is unknown, never gone:
/// the record cannot show that the reader's time namespace is the writer's, and an equal start time is alive.
#[test]
fn a_start_time_that_differs_from_the_record_is_unknown_and_an_equal_one_is_alive() {
    let launcher = Held(Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts"));
    let pid: i32 = i32::try_from(launcher.0.id()).expect("a process id");
    let (start, namespace): (u64, u64) = (process_start(pid).expect("start"), pid_namespace_id().expect("namespace"));

    assert_eq!(launcher_state(pid, start, namespace), crate::control_group::LauncherState::Alive);
    for other in [start + 1, start.saturating_sub(1), start + 720_000] {
        assert!(matches!(launcher_state(pid, other, namespace), crate::control_group::LauncherState::Unknown(text) if text.contains("start time")), "start {other} against {start} was not unknown");
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
    let (pid, start, pid_namespace): (i32, u64, u64) = (i32::try_from(numbers[0]).expect("a pid"), numbers[1], numbers[2]);
    let own_start: String = process_start(pid).map_or_else(|error| { return format!("err {error}"); }, |found| { return found.to_string(); });
    let journal: PathBuf = crate::journal_support::journal_dirs(&root).into_iter().next().expect("a journal");
    let note: String = survivor_note(&journal).unwrap_or_else(|| { return "no survivor".to_string(); });
    // The real recovery and the real admission in this namespace (sixth pass, group 3): what they leave and what they decide.
    let refusing = |_before: &[LockSnapshot], _live: &[LockSnapshot]| -> Vec<String> { return vec!["dep@1.0.1 refused".to_string()]; };
    let recovered: Vec<String> = crate::journal::recover(&root, &refusing);
    let live: String = fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).unwrap_or_default();
    let admitted = crate::journal::begin(&root, &[snapshot("original")], false);
    let held: String = format!(
        "entry lock unchanged {}\nentry journal kept {}\nentry admission refused {}\nentry journals {}\nentry messages {}\n",
        live == LEFT_BY_THE_KILLED_RUN,
        journal.exists(),
        admitted.is_err(),
        crate::journal_support::journal_dirs(&root).len(),
        recovered.len()
    );
    fs::write(
        root.join("fixture-result"),
        format!(
            "{held}own start {own_start}\ntime namespace here {}\nquestion {:?}\nrecovery {}\n",
            fs::metadata("/proc/self/ns/time").map_or_else(|error| { return format!("err {error}"); }, |found| { return std::os::unix::fs::MetadataExt::ino(&found).to_string(); }),
            launcher_state(pid, start, pid_namespace),
            note.lines().next().unwrap_or("")
        ),
    )
    .expect("write the result");
}

/// Runs the helper (in a new user and time namespace with a boot-time offset when `shifted`) and returns what it wrote.
fn run_time_fixture(root: &Path, identity: &str, shifted: bool) -> String {
    return run_time_fixture_in(root, identity, shifted, None);
}

/// Like `run_time_fixture`; with `user_namespace_of`, the helper first joins the user namespace of that process (which it owns), so
/// that the new time namespace is the only namespace it creates and takes the lowest free namespace number.
fn run_time_fixture_in(root: &Path, identity: &str, shifted: bool, user_namespace_of: Option<i32>) -> String {
    let executable = std::env::current_exe().expect("this test executable");
    let mut command = if let Some(owner) = user_namespace_of {
        let mut enter = Command::new("nsenter");
        enter.arg("--preserve-credentials").arg(format!("--user=/proc/{owner}/ns/user")).args(["unshare", "--time", "--boottime", "7200", "--fork", "--"]).arg(&executable);
        enter
    } else if shifted {
        let mut unshare = Command::new("unshare");
        // A new user namespace owns the new time namespace; the process id, mount and cgroup namespaces stay as they are, so only the
        // reader's boot-time offset differs (an offset of two hours; the start time is in clock ticks, so it moves by hundreds of thousands).
        unshare.args(["--user", "--map-root-user", "--time", "--boottime", "7200", "--fork", "--"]).arg(&executable);
        unshare
    } else {
        Command::new(&executable)
    };
    fs::write(root.join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("the output of the killed run");
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
/// boot-time offset: the live launcher's start time reads differently there, and the question and the recovery answer "unknown", never
/// "gone".
/// The control is a recovery in the writer's own time namespace, where the same question answers "alive" and the start time is the one
/// recorded; the prerequisite is the shifted reader's own successful reading of a different start time, as a number.
#[test]
fn a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone() {
    let Some(route) = delegated_route() else {
        return;
    };
    let launcher = Held(Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts"));
    let pid: i32 = i32::try_from(launcher.0.id()).expect("a process id");
    let start: u64 = process_start(pid).expect("the start time here");
    let pid_namespace: u64 = pid_namespace_id().expect("pid namespace");
    let identity: String = format!("{pid} {start} {pid_namespace}");
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let record: Record = Record::Launcher { pid, start, pid_namespace, path: planned.directory().to_path_buf() };

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

    assert!(same.contains(&format!("own start {start}\n")) && same.contains("question Alive"), "the control did not move: {same}");
    // The prerequisite (14 section 3): the shifted reader read a start time successfully, as a number, and it differs from the recorded one.
    let shifted_start: u64 = shifted
        .lines()
        .find_map(|line| { return line.strip_prefix("own start "); })
        .and_then(|text| { return text.parse::<u64>().ok(); })
        .unwrap_or_else(|| { panic!("the shifted reader did not read a start time as a number: {shifted}"); });
    assert_ne!(shifted_start, start, "the prerequisite failed: the other time namespace read the same start time: {shifted}");
    assert!(shifted.contains("question Unknown("), "a live launcher read from another time namespace was not unknown: {shifted}");
    assert!(!shifted.contains("Gone"), "a live launcher read from another time namespace was read as gone: {shifted}");
    // The recovery's refusal names the cause (14 section 3), with the other views equal: it is the start-time comparison that is refused.
    let recovery: &str = shifted.lines().find_map(|line| { return line.strip_prefix("recovery "); }).expect("the recovery line");
    assert!(recovery != "no survivor" && recovery.contains("start time"), "the recovery in another time namespace did not refuse for the start times: {recovery}");
    assert_entry_points_held(&shifted);
}

/// The helper that stands for the wrapper in the review's schedule (14 section 2):
/// it runs in a time namespace T0 of its own and starts its launcher in another one, T1, so T0 is held by nothing but the wrapper and
/// the helper process that waits for the launcher; it reads the launcher's start time on its own (T0) basis and writes the identity.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone"]
fn wrapper_in_its_own_time_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let pid_file: PathBuf = root.join("launcher-pid");
    let script: String = format!("echo $$ > {file}.part; mv {file}.part {file}; exec sleep 600", file = pid_file.display());
    // The launcher is started in a time namespace of its own (the children of `unshare --time` are in it), so it does not keep this
    // helper's time namespace alive.
    let mut helper: Child = Command::new("unshare")
        .args(["--time", "--boottime", "7200", "--fork", "--", "sh", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("starts the launcher in another time namespace");
    crate::cgroup_test_support::wait_until("the launcher's process id", &|| { return fs::read_to_string(&pid_file).is_ok_and(|text| { return text.trim().parse::<i32>().is_ok(); }); });
    let pid: i32 = fs::read_to_string(&pid_file).expect("the id").trim().parse::<i32>().expect("a process id");
    let start: u64 = process_start(pid).expect("the start time on this time namespace's basis");
    // The time namespace of this helper, written beside the identity only so that the test can wait for its destruction.
    let time_namespace: u64 = std::os::unix::fs::MetadataExt::ino(&fs::metadata("/proc/self/ns/time").expect("the time namespace"));
    let identity: String = format!("{pid} {start} {} {time_namespace}", pid_namespace_id().expect("pid namespace"));
    fs::write(root.join("identity.part"), identity).expect("write the identity");
    fs::rename(root.join("identity.part"), root.join("identity")).expect("publish the identity");
    std::thread::sleep(std::time::Duration::from_secs(300));
    let _ = helper.wait();
}

/// Every process of the user that is in the time namespace with this inode number.
fn processes_in_time_namespace(inode: u64) -> Vec<i32> {
    use std::os::unix::fs::MetadataExt;
    let mut found: Vec<i32> = Vec::new();
    for entry in fs::read_dir("/proc").expect("lists /proc").flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|name| { return name.parse::<i32>().ok(); }) else {
            continue;
        };
        if fs::metadata(format!("/proc/{pid}/ns/time")).is_ok_and(|metadata| { return metadata.ino() == inode; }) {
            found.push(pid);
        }
    }
    return found;
}

/// Acceptance rows A1, V2 (14 section 2, the review's schedule): the writer's time namespace T0 is destroyed while the launcher lives in
/// another (T1), a new time namespace T2 is created and receives the freed inode number, and a recovery in T2 reads the live launcher's
/// start time on another basis. A recorded number that names a later, different namespace must not make that difference a reused id:
/// the question answers "unknown", never "gone", because no recorded number is used.
/// The prerequisites are asserted: the freed number was reused, and the recovery's own reading of the start time differs.
#[test]
fn a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone() {
    let Some(route) = delegated_route() else {
        return;
    };
    // The namespace numbers are handed out by one allocator for the whole host, so another session that creates a namespace between the
    // writer's death and the recovery's creation takes the freed number (which the first full gate of round 16 met). The schedule is
    // therefore tried again, a bounded number of times, until the freed number is reused; the test fails if it never is.
    let mut last: Option<Attempt> = None;
    for _attempt in 0..8 {
        let made: Attempt = recycled_attempt(&route);
        let reused: bool = made.reused();
        last = Some(made);
        if reused {
            break;
        }
    }
    let made: Attempt = last.expect("an attempt");
    assert!(made.still_alive, "the launcher did not outlive its writer's time namespace, so the schedule was not made");
    assert!(made.reused(), "the freed time namespace number was not reused by the recovery's namespace in any attempt, so the schedule was not made: {}", made.result);
    let result: &str = &made.result;
    let own: u64 = result.lines().find_map(|line| { return line.strip_prefix("own start "); }).and_then(|text| { return text.parse::<u64>().ok(); }).unwrap_or_else(|| { panic!("the recovery did not read a start time: {result}"); });
    assert_ne!(own, made.start, "the recovery read the recorded start time, so the bases were the same: {result}");
    assert!(result.contains("question Unknown("), "the question did not answer unknown: {result}");
    assert!(!result.contains("Gone"), "a live launcher was read as gone through a recycled time namespace number: {result}");
    let recovery: &str = result.lines().find_map(|line| { return line.strip_prefix("recovery "); }).expect("the recovery line");
    assert!(recovery != "no survivor" && recovery.contains("start time"), "the recovery did not refuse for the start times: {recovery}");
    assert_entry_points_held(result);
}

/// What one attempt at the recycled-number schedule saw.
struct Attempt {
    /// Whether the launcher was alive after its writer's time namespace had no process.
    still_alive: bool,
    /// The writer's time namespace number.
    time_namespace: u64,
    /// The start time the writer recorded.
    start: u64,
    /// What the recovery's helper wrote.
    result: String,
}

/// The methods of an attempt.
impl Attempt {
    /// Whether the recovery's time namespace received the number of the writer's destroyed one.
    fn reused(&self) -> bool {
        return self.result.lines().find_map(|line| { return line.strip_prefix("time namespace here "); }).and_then(|text| { return text.parse::<u64>().ok(); }) == Some(self.time_namespace);
    }
}

/// One attempt: a writer in a time namespace of its own with its launcher in another, the writer's processes killed, and a recovery
/// in a new time namespace created right after.
fn recycled_attempt(route: &crate::control_group::Route) -> Attempt {
    let root = scratch();
    let mut outer: Child = Command::new("unshare")
        .args(["--user", "--map-root-user", "--time", "--boottime", "3600", "--fork", "--"])
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_fourteen_tests::wrapper_in_its_own_time_namespace_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .stdin(Stdio::null())
        .spawn()
        .expect("the wrapper helper (it needs unshare with permission for a user and time namespace)");
    crate::cgroup_test_support::wait_until("the writer's identity", &|| { return root.path().join("identity").exists(); });
    let identity: String = fs::read_to_string(root.path().join("identity")).expect("the identity");
    let numbers: Vec<u64> = identity.split(' ').map(|part| { return part.parse::<u64>().expect("a number"); }).collect();
    let (pid, start, pid_namespace, time_namespace): (i32, u64, u64, u64) = (i32::try_from(numbers[0]).expect("a pid"), numbers[1], numbers[2], numbers[3]);
    let _stop_launcher: StopOnDrop = StopOnDrop(pid);

    // The writer's time namespace loses its last reference: its processes are killed (the launcher is in another namespace and lives).
    for member in processes_in_time_namespace(time_namespace) {
        let _ = Command::new("kill").args(["-KILL", &member.to_string()]).status();
    }
    crate::cgroup_test_support::wait_until("the writer's time namespace to have no process", &|| { return processes_in_time_namespace(time_namespace).is_empty(); });
    let _ = outer.wait();
    let still_alive: bool = crate::contain_counterexample_tests::is_running(pid);

    let planned: ControlGroup = ControlGroup::plan(route).expect("a scope name");
    let record: Record = Record::Launcher { pid, start, pid_namespace, path: planned.directory().to_path_buf() };
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    let journal: PathBuf = plant(root.path(), &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), format!("{}\n{record}\n", planned.record())).expect("records");
    // The recovery joins the user namespace that the launcher is still in, so its new time namespace is the only namespace it creates
    // (a new user namespace would take the freed number first).
    let result: String = run_time_fixture_in(root.path(), &format!("{pid} {start} {pid_namespace}"), true, Some(pid));
    crate::contain_counterexample_tests::stop_quietly(pid);
    return Attempt { still_alive, time_namespace, start, result };
}
