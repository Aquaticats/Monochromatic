//! What:
//! Round 14 and 15 tests:
//! the start time of a launcher read from another time namespace (13 section 3,
//! 14 sections 2 and 3).
//! Why:
//! The start time in `/proc/<pid>/stat` is relative to the boot-time offset of the time namespace of the process that reads it,
//! so
//! a recovery in another time namespace reads a live launcher's start time differently and must not take that for a reused id.
//! Round 15:
//! the namespace number that identifies the writer's time namespace is recycled once the namespace has no process,
//! so it
//! proves nothing;
//! a start time that differs from the record is unknown,
//! whatever the namespace numbers say.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round fourteen', () => { it('another time namespace is unknown, not gone', ...) })
//! ```

use std::fs;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};

use crate::cgroup_test_support::delegated_route;
use crate::control_group::{
    ControlGroup,
    Record,
    launcher_state,
    parse_record,
    pid_namespace_id,
    process_start,
};
use crate::crates::LockSnapshot;
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_ROOT";

/// The environment variable that gives the helper the recorded identity of the launcher,
/// as `pid start pid_namespace`.
const FIXTURE_IDENTITY: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_IDENTITY";

/// The environment variable that gives the armed reader the writer's own process id,
/// which is the process whose time namespace it holds.
const FIXTURE_WRITER: &str = "CARGO_SHADOW_ROUND_FOURTEEN_FIXTURE_WRITER";

/// The file the armed reader publishes once it holds the writer's time namespace,
/// which is the moment the test may end the processes in it.
const NAMESPACE_HELD: &str = "namespace-held";

/// How many time namespace numbers the armed reader takes before it says the recycled-number schedule was not made.
/// Why it is not one:
/// the kernel hands out the lowest free number,
/// and the numbers below the writer's are free by then whenever a namespace made before it ended while this test planted its journal,
/// so the reader passes over them one at a time.
/// Measured on 2026-11-27 (issue 6599):
/// eight runs of this control passed over none or one,
/// and the three runs before the reader passed over them were handed numbers 44,
/// 4 and 2 below the writer's,
/// so this bound is far above every one of those and still ends a host that never hands the number back.
const READER_ASSIGNMENTS: u32 = 4096;

/// What:
/// `fn namespace_number(path: &Path) -> u64` is the number the kernel's namespace file system gives the namespace a path names.
/// Why:
/// a time namespace is identified by the inode number of its `/proc/<pid>/ns/time` file,
/// and a test that builds a schedule from those numbers reads them the same way `/proc` publishes them.
fn namespace_number(path: &Path) -> u64 {
    // What: `use` inside a function imports a trait for this body only; `MetadataExt` adds the Unix fields of a file's metadata.
    // Why: the inode number is one of those fields, and the plain `fs::Metadata` of the standard library does not expose it.
    use std::os::unix::fs::MetadataExt;
    return fs::metadata(path)
        .unwrap_or_else(|error| {
            panic!("the namespace {path:?} cannot be read: {error}");
        })
        .ino();
}

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot {
        crate_dir: CRATE_DIR.to_string(),
        source: source.to_string(),
    };
}

/// A live process that stands for a launcher that has not reached the manager,
/// killed when the guard drops.
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

/// The live text of the lockfile the killed run left,
/// which the recovery must not change while the launcher is uncertain.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// Asserts what the helper's real recovery and admission did in the other time namespace:
/// the live output is unchanged,
/// the journal is
/// kept (and is the only one:
/// the refused admission left none behind),
/// and a new transaction is refused.
fn assert_entry_points_held(result: &str) {
    for line in [
        "entry lock unchanged true",
        "entry journal kept true",
        "entry admission refused true",
        "entry journals 1",
    ] {
        assert!(
            result.lines().any(|found| {
                return found == line;
            }),
            "the real recovery or admission did not hold the uncertain launcher ({line} is missing): {result}"
        );
    }
}

/// Stops a process by its id when it drops,
/// so a failed assertion leaves no launcher behind.
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

/// Contract A1 (round 15):
/// the launcher line reads back as written;
/// a line of the form of round 14 (a time namespace number before the
/// path) is not understood,
/// so it waits for an acknowledgement like any journal that cannot be read.
#[test]
fn a_launcher_record_reads_back_and_a_line_with_a_time_namespace_is_not_understood() {
    let record: Record = Record::Launcher {
        pid: 4242,
        start: 987_654,
        pid_namespace: 4_026_531_836,
        path: PathBuf::from("/sys/fs/cgroup/a b/c.scope"),
    };
    assert_eq!(
        record.to_string(),
        "launcher 4242 987654 4026531836 /sys/fs/cgroup/a b/c.scope"
    );
    assert_eq!(parse_record(&record.to_string()), Ok(record));
    for refused in [
        "launcher 4 5 6 7 /a",
        "launcher 4 5 6 7",
        "launcher 4 5 6 x /a",
        "launcher 4 5 6 relative",
    ] {
        assert!(parse_record(refused).is_err(), "{refused:?} was understood");
    }
}

/// Acceptance rows A1,
/// V2 (round 15):
/// with everything else equal,
/// a start time that differs from the record is unknown,
/// never gone:
/// the record cannot show that the reader's time namespace is the writer's,
/// and an equal start time is alive.
#[test]
fn a_start_time_that_differs_from_the_record_is_unknown_and_an_equal_one_is_alive() {
    let launcher = Held(
        Command::new("sleep")
            .arg("600")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("starts"),
    );
    let pid: i32 = i32::try_from(launcher.0.id()).expect("a process id");
    let (start, namespace): (u64, u64) = (
        process_start(pid).expect("start"),
        pid_namespace_id().expect("namespace"),
    );

    assert_eq!(
        launcher_state(pid, start, namespace),
        crate::control_group::LauncherState::Alive
    );
    for other in [start + 1, start.saturating_sub(1), start + 720_000] {
        assert!(
            matches!(launcher_state(pid, other, namespace), crate::control_group::LauncherState::Unknown(text) if text.contains("start time")),
            "start {other} against {start} was not unknown"
        );
    }
}

/// The helper process of the time namespace test:
/// it asks what the recorded launcher is,
/// with the start time it reads itself,
/// and what a recovery of the planted journal says.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone"]
fn time_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let identity: String = std::env::var(FIXTURE_IDENTITY).expect("an identity");
    let numbers: Vec<u64> = identity
        .split(' ')
        .map(|part| {
            return part.parse::<u64>().expect("a number");
        })
        .collect();
    let (pid, start, pid_namespace): (i32, u64, u64) = (
        i32::try_from(numbers[0]).expect("a pid"),
        numbers[1],
        numbers[2],
    );
    let own_start: String = process_start(pid).map_or_else(
        |error| {
            return format!("err {error}");
        },
        |found| {
            return found.to_string();
        },
    );
    let journal: PathBuf = crate::journal_support::journal_dirs(&root)
        .into_iter()
        .next()
        .expect("a journal");
    let note: String = survivor_note(&journal).unwrap_or_else(|| {
        return "no survivor".to_string();
    });
    // The real recovery and the real admission in this namespace (sixth pass, group 3): what they leave and what they decide.
    let refusing = |_before: &[LockSnapshot], _live: &[LockSnapshot]| -> Vec<String> {
        return vec!["dep@1.0.1 refused".to_string()];
    };
    let recovered: Vec<String> = crate::journal::recover(&root, &refusing);
    let live: String =
        fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).unwrap_or_default();
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
            fs::metadata("/proc/self/ns/time").map_or_else(
                |error| {
                    return format!("err {error}");
                },
                |found| {
                    return std::os::unix::fs::MetadataExt::ino(&found).to_string();
                }
            ),
            launcher_state(pid, start, pid_namespace),
            note.lines().next().unwrap_or("")
        ),
    )
    .expect("write the result");
}

/// The armed reader of the recycled-number schedule (14 section 2).
/// What:
/// it holds the writer's time namespace by an open descriptor,
/// waits for the test to end the processes in it,
/// gives the number back with its own `close`,
/// and takes it with the allocations it makes next,
/// passing over the free numbers below it (each held,
/// so the next ask moves past it) until the kernel hands the writer's back,
/// and then runs the fixture in the time namespace it was handed.
/// Why:
/// The kernel hands a freed namespace number to whoever asks next,
/// so a schedule that frees it and then writes files before asking again loses it to another session on a busy host (issue 6599).
/// Here the number is the test's own steps,
/// and a number this reader cannot get is said in its own words instead of being read as the code under test being wrong.
/// Run on its own it does nothing.
#[test]
#[ignore = "the armed reader of a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone"]
fn recycled_number_reader_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let identity: String = std::env::var(FIXTURE_IDENTITY).expect("an identity");
    let recorded: u64 = identity
        .split(' ')
        .nth(3)
        .expect("the recorded time namespace number")
        .parse::<u64>()
        .expect("a number");
    let writer: i32 = std::env::var(FIXTURE_WRITER)
        .expect("the writer's process id")
        .parse::<i32>()
        .expect("a process id");
    let writer_namespace: PathBuf = PathBuf::from(format!("/proc/{writer}/ns/time"));
    assert_eq!(
        namespace_number(&writer_namespace),
        recorded,
        "the writer process {writer} is not in the time namespace its identity names, so the recycled-number schedule was not made"
    );
    // What: `fs::File::open` on `/proc/<pid>/ns/time` asks the kernel for the namespace file itself, not a copy of its name.
    // Why: while this descriptor is open the kernel counts the namespace as used, so its number cannot be handed to another process.
    let held: fs::File = fs::File::open(&writer_namespace).unwrap_or_else(|error| {
        panic!("the writer's time namespace cannot be held: {error}");
    });
    fs::write(root.join(format!("{NAMESPACE_HELD}.part")), "held").expect("write the hold");
    fs::rename(
        root.join(format!("{NAMESPACE_HELD}.part")),
        root.join(NAMESPACE_HELD),
    )
    .expect("publish the hold");
    // The test releases this reader with one line on its standard input once that namespace has no process and the journal is planted.
    // An end of file means the test ended first, so this reader says so instead of waiting for a test that is gone.
    let mut released: String = String::new();
    let read: usize = std::io::stdin()
        .read_line(&mut released)
        .expect("the test's release");
    assert!(
        read != 0,
        "the test ended before it released this reader, so the recycled-number schedule was not made"
    );
    // What: `drop(held)` closes the descriptor, which is the kernel's last use of the writer's time namespace.
    // Why: this is where the number goes back to the kernel's allocator, and the `unshare` below is the next ask for one.
    drop(held);
    // What: `rustix::thread::unshare_unsafe(UnshareFlags::NEWTIME)` asks for a time namespace this thread's children are made in.
    // Why: the calling process is not moved into a new time namespace (a running process keeps its clocks), so the fixture has to be a
    // child, which the `Command` below makes. It is unsafe because a namespace unshare can leave descriptors and threads in different
    // namespaces; this thread asks for no other namespace, and only the children it makes see this one.
    //
    // The kernel hands out the lowest free number, and on a busy host the numbers below the writer's are free too, because namespaces
    // made before it end while this test works (issue 6599). Each one this reader is handed is held by a descriptor, which keeps its
    // number taken, so the next ask moves past it, until the writer's number itself is the lowest free one and this reader is handed it.
    let mut passed_over: Vec<fs::File> = Vec::new();
    let mut made: u64 = 0;
    for _ask in 0..READER_ASSIGNMENTS {
        unsafe {
            rustix::thread::unshare_unsafe(rustix::thread::UnshareFlags::NEWTIME).expect(
                "a new time namespace (this host lets the writer's user namespace make one)",
            );
        }
        made = namespace_number(Path::new("/proc/thread-self/ns/time_for_children"));
        if made == recorded {
            break;
        }
        passed_over.push(
            fs::File::open("/proc/thread-self/ns/time_for_children")
                .expect("a descriptor on the time namespace this thread just took"),
        );
    }
    assert_eq!(
        made,
        recorded,
        "{} time namespace numbers below the writer's {recorded} were handed to this reader and none was the writer's, so the recycled-number schedule was not made (a precondition this host did not give, not a finding about the code under test)",
        passed_over.len()
    );
    // The numbers this reader passed over go back to the kernel's allocator; the namespace the fixture will be made in is kept by this
    // thread itself, so nothing can take it while the fixture starts.
    drop(passed_over);
    // A child of this thread is made in the namespace just taken, and its boot-time offset stays the kernel's default of zero, against
    // the writer's namespace offset of one hour, so the start time it reads is not the recorded one.
    let status = Command::new(std::env::current_exe().expect("this test executable"))
        .args([
            "--ignored",
            "--exact",
            "round_fourteen_tests::time_namespace_fixture",
            "--nocapture",
        ])
        .env(FIXTURE_ROOT, &root)
        .env(FIXTURE_IDENTITY, &identity)
        .status()
        .expect("the fixture, a child of this thread, so in the time namespace this reader took");
    assert!(status.success(), "the fixture failed: {status:?}");
}

/// Runs the helper (in a new user and time namespace with a boot-time offset when `shifted`) and returns what it wrote.
fn run_time_fixture(
    root: &Path,
    identity: &str,
    shifted: bool,
) -> String {
    let executable = std::env::current_exe().expect("this test executable");
    let mut command = if shifted {
        let mut unshare = Command::new("unshare");
        // A new user namespace owns the new time namespace; the process id, mount and cgroup namespaces stay as they are, so only the
        // reader's boot-time offset differs (an offset of two hours; the start time is in clock ticks, so it moves by hundreds of thousands).
        unshare
            .args([
                "--user",
                "--map-root-user",
                "--time",
                "--boottime",
                "7200",
                "--fork",
                "--",
            ])
            .arg(&executable);
        unshare
    } else {
        Command::new(&executable)
    };
    fs::write(
        root.join(CRATE_DIR).join("Cargo.lock"),
        LEFT_BY_THE_KILLED_RUN,
    )
    .expect("the output of the killed run");
    let status = command
        .args([
            "--ignored",
            "--exact",
            "round_fourteen_tests::time_namespace_fixture",
            "--nocapture",
        ])
        .env(FIXTURE_ROOT, root)
        .env(FIXTURE_IDENTITY, identity)
        .status()
        .expect(
            "the helper (shifted runs need unshare with permission for a user and time namespace)",
        );
    assert!(status.success(), "the helper failed: {status:?}");
    return fs::read_to_string(root.join("fixture-result")).expect("the helper's result");
}

/// Acceptance rows A1,
/// V2 (13 section 3):
/// the same process id namespace,
/// procfs and journal,
/// a recovery in another time namespace with a
/// boot-time offset:
/// the live launcher's start time reads differently there,
/// and the question and the recovery answer "unknown",
/// never
/// "gone".
/// The control is a recovery in the writer's own time namespace,
/// where the same question answers "alive" and the start time is the one
/// recorded;
/// the prerequisite is the shifted reader's own successful reading of a different start time,
/// as a number.
#[test]
fn a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone() {
    let route = delegated_route();
    let launcher = Held(
        Command::new("sleep")
            .arg("600")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("starts"),
    );
    let pid: i32 = i32::try_from(launcher.0.id()).expect("a process id");
    let start: u64 = process_start(pid).expect("the start time here");
    let pid_namespace: u64 = pid_namespace_id().expect("pid namespace");
    let identity: String = format!("{pid} {start} {pid_namespace}");
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let record: Record = Record::Launcher {
        pid,
        start,
        pid_namespace,
        path: planned.directory().to_path_buf(),
    };

    let same_root = scratch();
    fs::create_dir_all(same_root.path().join(CRATE_DIR)).expect("dir");
    let journal: PathBuf = plant(same_root.path(), &[snapshot("original")], &[]);
    fs::write(
        journal.join("groups"),
        format!("{}\n{record}\n", planned.record()),
    )
    .expect("records");
    let same: String = run_time_fixture(same_root.path(), &identity, false);

    let other_root = scratch();
    fs::create_dir_all(other_root.path().join(CRATE_DIR)).expect("dir");
    let other_journal: PathBuf = plant(other_root.path(), &[snapshot("original")], &[]);
    fs::write(
        other_journal.join("groups"),
        format!("{}\n{record}\n", planned.record()),
    )
    .expect("records");
    let shifted: String = run_time_fixture(other_root.path(), &identity, true);

    assert!(
        same.contains(&format!("own start {start}\n")) && same.contains("question Alive"),
        "the control did not move: {same}"
    );
    // The prerequisite (14 section 3): the shifted reader read a start time successfully, as a number, and it differs from the recorded one.
    let shifted_start: u64 = shifted
        .lines()
        .find_map(|line| {
            return line.strip_prefix("own start ");
        })
        .and_then(|text| {
            return text.parse::<u64>().ok();
        })
        .unwrap_or_else(|| {
            panic!("the shifted reader did not read a start time as a number: {shifted}");
        });
    assert_ne!(
        shifted_start, start,
        "the prerequisite failed: the other time namespace read the same start time: {shifted}"
    );
    assert!(
        shifted.contains("question Unknown("),
        "a live launcher read from another time namespace was not unknown: {shifted}"
    );
    assert!(
        !shifted.contains("Gone"),
        "a live launcher read from another time namespace was read as gone: {shifted}"
    );
    // The recovery's refusal names the cause (14 section 3), with the other views equal: it is the start-time comparison that is refused.
    let recovery: &str = shifted
        .lines()
        .find_map(|line| {
            return line.strip_prefix("recovery ");
        })
        .expect("the recovery line");
    assert!(
        recovery != "no survivor" && recovery.contains("start time"),
        "the recovery in another time namespace did not refuse for the start times: {recovery}"
    );
    assert_entry_points_held(&shifted);
}

/// The helper that stands for the wrapper in the review's schedule (14 section 2):
/// it runs in a time namespace T0 of its own and starts its launcher in another one,
/// T1,
/// so T0 is held by nothing but the wrapper and
/// the helper process that waits for the launcher;
/// it reads the launcher's start time on its own (T0) basis and writes the identity,
/// which ends with its own process id so the reader can hold T0 once it has moved on.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone"]
fn wrapper_in_its_own_time_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let pid_file: PathBuf = root.join("launcher-pid");
    let script: String = format!(
        "echo $$ > {file}.part; mv {file}.part {file}; exec sleep 600",
        file = pid_file.display()
    );
    // The launcher is started in a time namespace of its own (the children of `unshare --time` are in it), so it does not keep this
    // helper's time namespace alive.
    let mut helper: Child = Command::new("unshare")
        .args([
            "--time",
            "--boottime",
            "7200",
            "--fork",
            "--",
            "sh",
            "-c",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("starts the launcher in another time namespace");
    crate::cgroup_test_support::wait_until("the launcher's process id", &|| {
        return fs::read_to_string(&pid_file).is_ok_and(|text| {
            return text.trim().parse::<i32>().is_ok();
        });
    });
    let pid: i32 = fs::read_to_string(&pid_file)
        .expect("the id")
        .trim()
        .parse::<i32>()
        .expect("a process id");
    let start: u64 = process_start(pid).expect("the start time on this time namespace's basis");
    // The time namespace of this helper, written beside the identity only so that the test can wait for its destruction.
    let time_namespace: u64 = std::os::unix::fs::MetadataExt::ino(
        &fs::metadata("/proc/self/ns/time").expect("the time namespace"),
    );
    // The writer's own process id is the fifth number: the armed reader holds the namespace this process is in, and only this process
    // can name it once the launcher has moved on.
    let identity: String = format!(
        "{pid} {start} {} {time_namespace} {}",
        pid_namespace_id().expect("pid namespace"),
        std::process::id()
    );
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
        let Some(pid) = entry.file_name().to_str().and_then(|name| {
            return name.parse::<i32>().ok();
        }) else {
            continue;
        };
        if fs::metadata(format!("/proc/{pid}/ns/time")).is_ok_and(|metadata| {
            return metadata.ino() == inode;
        }) {
            found.push(pid);
        }
    }
    return found;
}

/// Acceptance rows A1,
/// V2 (14 section 2,
/// the review's schedule):
/// the writer's time namespace T0 is destroyed while the launcher lives in
/// another (T1),
/// a new time namespace T2 is made and receives the freed inode number,
/// and a recovery in T2 reads the live launcher's
/// start time on another basis.
/// A recorded number that names a later,
/// different namespace must not make that difference a reused id:
/// the question answers "unknown",
/// never "gone",
/// because no recorded number is used.
/// The schedule is built from facts this test owns (issue 6599):
/// the reader holds T0 by an open descriptor while this test ends T0's processes and plants the journal,
/// then gives the number back and takes it with its own close and its own `unshare`,
/// so the number is never free while the test is busy.
/// What it cannot build it says as a precondition it could not build,
/// and never as the code under test being wrong:
/// the freed number went elsewhere,
/// or the launcher did not outlive its writer's namespace.
#[test]
fn a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone() {
    let route = delegated_route();
    // The number is this test's own two steps now, so one attempt is one schedule; the loop stays because a host can still hand the
    // number to another process in the moment between those two steps, and a fresh attempt is a fresh number.
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
    assert!(
        made.still_alive,
        "the launcher did not outlive its writer's time namespace, so the recycled-number schedule was not made: {}",
        made.report()
    );
    assert!(
        made.reused(),
        "the freed time namespace number was not reused by the recovery's namespace in any attempt, so the recycled-number schedule was not made (a precondition this host did not give, not a finding about the code under test): {}",
        made.report()
    );
    let result: &str = &made.result;
    let own: u64 = result
        .lines()
        .find_map(|line| {
            return line.strip_prefix("own start ");
        })
        .and_then(|text| {
            return text.parse::<u64>().ok();
        })
        .unwrap_or_else(|| {
            panic!("the recovery did not read a start time: {result}");
        });
    assert_ne!(
        own, made.start,
        "the recovery read the recorded start time, so the bases were the same: {result}"
    );
    assert!(
        result.contains("question Unknown("),
        "the question did not answer unknown: {result}"
    );
    assert!(
        !result.contains("Gone"),
        "a live launcher was read as gone through a recycled time namespace number: {result}"
    );
    let recovery: &str = result
        .lines()
        .find_map(|line| {
            return line.strip_prefix("recovery ");
        })
        .expect("the recovery line");
    assert!(
        recovery != "no survivor" && recovery.contains("start time"),
        "the recovery did not refuse for the start times: {recovery}"
    );
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
    /// The namespace the planted other session made,
    /// when the plant was asked for:
    /// its number and the program that keeps it.
    foreign: Option<ForeignNamespace>,
}

/// The methods of an attempt.
impl Attempt {
    /// What an attempt says about its schedule:
    /// the writer's number,
    /// the number the recovery's namespace got,
    /// and what the planted other session took.
    fn report(&self) -> String {
        let here: Option<&str> = self.result.lines().find_map(|line| {
            return line.strip_prefix("time namespace here ");
        });
        return format!(
            "the writer's time namespace was {} and the recovery's is {here:?}; the planted other session took {}. {} the result was: {}",
            self.time_namespace,
            self.foreign_number(),
            if self.still_alive {
                "The launcher outlived the writer's time namespace."
            } else {
                "The launcher did not outlive the writer's time namespace."
            },
            self.result
        );
    }

    /// What the planted other session took,
    /// when one was planted.
    fn foreign_number(&self) -> String {
        return self.foreign.as_ref().map_or_else(
            || {
                return "nothing (no other session was planted)".to_string();
            },
            |planted| {
                return planted.number.clone();
            },
        );
    }

    /// Whether the recovery's time namespace received the number of the writer's destroyed one.
    fn reused(&self) -> bool {
        return self
            .result
            .lines()
            .find_map(|line| {
                return line.strip_prefix("time namespace here ");
            })
            .and_then(|text| {
                return text.parse::<u64>().ok();
            })
            == Some(self.time_namespace);
    }
}

/// The variable that makes the test plant the namespace another session makes in the gap:
/// the window between the writer's time namespace losing its last process and the recovery's namespace being made.
/// Why it exists:
/// the schedule's precondition is a number the kernel hands back,
/// and the run that opened issue 6599 met a run of the whole suite where another session's namespace took it instead.
/// Set to any value to plant that allocation;
/// the test passes only because it does not depend on it.
const PLANT_FOREIGN_NAMESPACE: &str = "CARGO_SHADOW_ROUND_FOURTEEN_PLANT_FOREIGN_NAMESPACE";

/// A time namespace another session made while the writer's time namespace had no process,
/// held until the attempt ends.
/// This is a stand-in the test writes itself for the other session the schedule's comment names:
/// it makes one namespace where a run of the whole suite on this host makes many.
struct ForeignNamespace {
    /// What makes the namespace and keeps it alive:
    /// `nsenter` into the writer's user namespace,
    /// a new time namespace,
    /// and a `sleep` in it.
    holder: Child,
    /// The number of the namespace it made,
    /// as `/proc/<pid>/ns/time` names it.
    number: String,
}

/// How the guard cleans up.
impl Drop for ForeignNamespace {
    /// What:
    /// `fn drop(&mut self)` ends the program that holds the namespace,
    /// and with it the namespace.
    /// Why:
    /// a failed assertion must leave neither the holder nor the namespace behind.
    fn drop(&mut self) {
        let _ = self.holder.kill();
        let _ = self.holder.wait();
    }
}

/// What:
/// `fn plant_foreign_namespace(root: &Path, owner: i32) -> ForeignNamespace` makes one time namespace the way another session on
/// this host does,
/// in the user namespace that owns the writer's time namespace (a time namespace cannot be made outside a user namespace that owns
/// it),
/// and answers once that namespace exists.
/// Why:
/// Another session's namespace takes the number a destroyed one frees,
/// because the kernel hands out the lowest free number to whoever asks next;
/// this is that ask,
/// made at the one point where it can break the schedule.
fn plant_foreign_namespace(
    root: &Path,
    owner: i32,
) -> ForeignNamespace {
    let number_file: PathBuf = root.join("foreign-namespace");
    // The program says which namespace it is in before it waits, so the test knows the allocation happened and not only the start.
    let script: String = format!(
        "stat -L -c %i /proc/self/ns/time > {file}.part; mv {file}.part {file}; exec sleep 30",
        file = number_file.display()
    );
    let holder: Child = Command::new("nsenter")
        .arg("--preserve-credentials")
        .arg(format!("--user=/proc/{owner}/ns/user"))
        .args([
            "unshare",
            "--time",
            "--boottime",
            "7200",
            "--fork",
            "--",
            "sh",
            "-c",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect(
            "the other session's namespace (it needs nsenter and unshare with permission for a user and time namespace)",
        );
    crate::cgroup_test_support::wait_until("the other session's namespace", &|| {
        return number_file.exists();
    });
    let number: String = fs::read_to_string(&number_file)
        .expect("the other session's namespace number")
        .trim()
        .to_string();
    return ForeignNamespace {
        holder,
        number,
    };
}

/// One attempt,
/// built from facts this test owns:
/// a writer in a time namespace of its own with its launcher in another,
/// a reader that holds that namespace by its own descriptor,
/// the writer's processes ended by this test,
/// and the freed number given back and taken again by the reader's own two steps,
/// which makes the recovery's namespace the one that carries the writer's number.
fn recycled_attempt(route: &crate::control_group::Route) -> Attempt {
    let root = scratch();
    let mut outer: Child = Command::new("unshare")
        .args([
            "--user",
            "--map-root-user",
            "--time",
            "--boottime",
            "3600",
            "--fork",
            "--",
        ])
        .arg(std::env::current_exe().expect("this test executable"))
        .args([
            "--ignored",
            "--exact",
            "round_fourteen_tests::wrapper_in_its_own_time_namespace_fixture",
            "--nocapture",
        ])
        .env(FIXTURE_ROOT, root.path())
        .stdin(Stdio::null())
        .spawn()
        .expect(
            "the wrapper helper (it needs unshare with permission for a user and time namespace)",
        );
    crate::cgroup_test_support::wait_until("the writer's identity", &|| {
        return root.path().join("identity").exists();
    });
    let identity: String = fs::read_to_string(root.path().join("identity")).expect("the identity");
    let numbers: Vec<u64> = identity
        .split(' ')
        .map(|part| {
            return part.parse::<u64>().expect("a number");
        })
        .collect();
    let (pid, start, pid_namespace, time_namespace, writer): (i32, u64, u64, u64, i32) = (
        i32::try_from(numbers[0]).expect("a pid"),
        numbers[1],
        numbers[2],
        numbers[3],
        i32::try_from(numbers[4]).expect("a pid"),
    );
    let _stop_launcher: StopOnDrop = StopOnDrop(pid);

    // The reader is armed before the writer's namespace loses its processes: it holds that namespace, so no other session can take the
    // number while this test ends them and plants the journal.
    let mut reader: Child = Command::new("nsenter")
        .arg("--preserve-credentials")
        .arg(format!("--user=/proc/{pid}/ns/user"))
        .arg(std::env::current_exe().expect("this test executable"))
        .args([
            "--ignored",
            "--exact",
            "round_fourteen_tests::recycled_number_reader_fixture",
            "--nocapture",
        ])
        .env(FIXTURE_ROOT, root.path())
        .env(FIXTURE_IDENTITY, &identity)
        .env(FIXTURE_WRITER, writer.to_string())
        .stdin(Stdio::piped())
        .spawn()
        .expect(
            "the armed reader (it needs nsenter with permission for the user namespace the launcher is in)",
        );
    crate::cgroup_test_support::wait_until(
        "the armed reader to hold the writer's time namespace",
        &|| {
            return root.path().join(NAMESPACE_HELD).exists();
        },
    );

    // The writer's time namespace loses its last process: its processes are killed (the launcher is in another namespace and lives).
    for member in processes_in_time_namespace(time_namespace) {
        let _ = Command::new("kill")
            .args(["-KILL", &member.to_string()])
            .status();
    }
    crate::cgroup_test_support::wait_until(
        "the writer's time namespace to have no process",
        &|| {
            return processes_in_time_namespace(time_namespace).is_empty();
        },
    );
    let _ = outer.wait();
    let still_alive: bool = crate::contain_counterexample_tests::is_running(pid);
    // The planted allocation sits exactly where the schedule was vulnerable: the writer's namespace has no process and the recovery's
    // is not made yet.
    let foreign: Option<ForeignNamespace> =
        std::env::var_os(PLANT_FOREIGN_NAMESPACE).map(|_asked| {
            return plant_foreign_namespace(root.path(), pid);
        });

    let planned: ControlGroup = ControlGroup::plan(route).expect("a scope name");
    let record: Record = Record::Launcher {
        pid,
        start,
        pid_namespace,
        path: planned.directory().to_path_buf(),
    };
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(
        root.path().join(CRATE_DIR).join("Cargo.lock"),
        LEFT_BY_THE_KILLED_RUN,
    )
    .expect("the output of the killed run");
    let journal: PathBuf = plant(root.path(), &[snapshot("original")], &[]);
    fs::write(
        journal.join("groups"),
        format!("{}\n{record}\n", planned.record()),
    )
    .expect("records");
    // The journal and its record stand before the reader takes the number, because the recovery they describe runs once it is released.
    {
        let mut release: std::process::ChildStdin =
            reader.stdin.take().expect("the reader's input");
        release
            .write_all(b"released\n")
            .expect("release the reader");
    }
    let status: std::process::ExitStatus = reader.wait().expect("the armed reader");
    crate::contain_counterexample_tests::stop_quietly(pid);
    assert!(
        status.success(),
        "the armed reader did not make the recovery's time namespace: {status:?}"
    );
    let result: String =
        fs::read_to_string(root.path().join("fixture-result")).expect("the fixture's result");
    return Attempt {
        still_alive,
        time_namespace,
        start,
        result,
        foreign,
    };
}
