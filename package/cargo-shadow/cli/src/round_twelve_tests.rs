//! What:
//! Round 12 tests: the launcher check against a procfs of another process id namespace, the boot id read from the object that was
//! checked, the reaping error, the whole view header, and the writer side of an atomic publication (11a and 11b).
//! Why:
//! The review of rounds 10 and 11 (`cargo-shadow-sol-11a.txt`, `-11b.txt`) named schedules inside existing rows of the acceptance list;
//! each test here is one of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round twelve', () => { it('a failed wait is not a reap', ...) })
//! ```

use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use crate::acknowledge::acknowledge;
use crate::contain_counterexample_tests::wait_for_file;
use crate::control_group::{Identity, RouteSettings, current_identity, detect_route, launcher_state, pid_namespace_id, process_start};
use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::{before_publish_for_test, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// The environment variable that gives a helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_TWELVE_FIXTURE_ROOT";

/// The environment variable that tells a helper process which schedule to make.
const FIXTURE_MODE: &str = "CARGO_SHADOW_ROUND_TWELVE_FIXTURE_MODE";

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

/// A process in a process group of its own, which is alive until the test ends, however it ends.
struct LiveGroup(
    /// The process that leads the group.
    Child,
);

/// The methods of the guard.
impl LiveGroup {
    /// The process id, which is the group's number.
    fn id(&self) -> u32 {
        return self.0.id();
    }
}

/// How the guard cleans up.
impl Drop for LiveGroup {
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

/// Starts a live process group.
fn live_group() -> LiveGroup {
    return LiveGroup(Command::new("sleep").arg("600").process_group(0).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a live group"));
}

/// This machine's boot id with one digit changed, as a real reboot would change it.
fn another_boot() -> Identity {
    let now: Identity = current_identity().expect("this machine has an identity");
    let mut boot: Vec<u8> = now.boot.into_bytes();
    boot[0] = if boot[0] == b'0' { b'1' } else { b'0' };
    return Identity { boot: String::from_utf8(boot).expect("ascii"), machine: now.machine };
}

/// Acceptance row V6 (11a, reaping errors):
/// a wait that fails is not a reap: the discovery's failure says that the killed program could not be reaped.
#[test]
fn a_wait_that_fails_is_not_a_reap() {
    let settings: RouteSettings = RouteSettings {
        systemctl: "sh".to_string(),
        question: Some(vec!["-c".to_string(), "exec sleep 30".to_string()]),
        discovery_bound: Duration::from_millis(300),
        ..RouteSettings::default()
    };
    arm(Point::ReapError);

    let refused = detect_route(&settings).expect_err("a manager that does not answer is a missing capability");

    assert!(!crate::failpoints::fires(Point::ReapError), "the wait fault was never reached");
    assert!(refused.to_string().contains("could not be reaped"), "a failed wait was counted as a reap: {refused}");
}

/// Acceptance rows V2, amended c (11a, the view header):
/// a record whose first line is not `view <mount>:<inode>` has no identity, however good the identity lines are,
/// so a live process group keeps blocking; the control is a whole header, which clears it.
#[test]
fn a_view_header_that_is_not_mount_and_inode_gives_no_identity() {
    let root = repository();
    let group = live_group();
    let journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
    let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
    let whole: String = recorded.lines().next().expect("a view line").to_string();
    let other: Identity = another_boot();
    for header in ["view nonsense", "view ", "view 1:", "view :2", "view 1:2:3", "view 1: 2", "view -1:2", "view 1:2 x", "viewed 1:2", ""] {
        fs::write(journal.join("view"), format!("{header}\nboot {}\nmachine {}\n", other.boot, other.machine)).expect("write");
        let note: Option<String> = survivor_note(&journal);
        assert!(note.is_some_and(|text| { return text.contains("still running"); }), "{header:?}: a malformed header gave an identity and cleared a live group");
    }
    fs::write(journal.join("view"), format!("{whole}\nboot {}\nmachine {}\n", other.boot, other.machine)).expect("write");
    assert_eq!(survivor_note(&journal), None, "the control: a whole header with another boot did not clear the group");
}

/// The helper process of the procfs namespace test:
/// it runs as the first process of a new process id namespace, with the procfs it inherited (mode `ancestor`, which numbers the
/// processes of the ancestor namespace) or a procfs of its own (mode `own`), starts a launcher stand-in, and writes what the
/// launcher question says to a file.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_procfs_of_another_pid_namespace_never_reads_as_a_gone_launcher"]
fn procfs_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let mode: String = std::env::var(FIXTURE_MODE).expect("a mode");
    let mut launcher: Child = Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts");
    let pid: i32 = i32::try_from(launcher.id()).expect("a process id");
    let namespace: u64 = pid_namespace_id().expect("the namespace");
    let recorded_start: Result<u64, String> = process_start(pid).map_err(|error| { return error.to_string(); });
    // What a numeric lookup would find: the stat file that this procfs has for the same number, and the name it shows.
    let collision: String = fs::read_to_string(format!("/proc/{pid}/stat")).map_or_else(
        |error| { return format!("unreadable ({error})"); },
        |text| { return text.split_once('(').and_then(|(_, rest)| { return rest.rsplit_once(')'); }).map_or("not understood".to_string(), |(name, _)| { return name.to_string(); }); },
    );
    let start_error: String = recorded_start.as_ref().err().cloned().unwrap_or_default();
    // An arbitrary start time stands for a launcher recorded earlier; in a consistent procfs a live process has another start time
    // only if it is another process, and in an inconsistent one nothing can be said.
    let arbitrary: u64 = recorded_start.as_ref().map_or(1, |start| { return start + 1; });
    let against_other_start: String = format!("{:?}", launcher_state(pid, arbitrary, namespace, crate::control_group::time_namespace_id().ok()));
    let against_own_start: String = recorded_start.as_ref().map_or_else(|_| { return "no start".to_string(); }, |start| { return format!("{:?}", launcher_state(pid, *start, namespace, crate::control_group::time_namespace_id().ok())); });
    let _ = launcher.kill();
    let _ = launcher.wait();
    let after_end: String = recorded_start.as_ref().map_or_else(|_| { return "no start".to_string(); }, |start| { return format!("{:?}", launcher_state(pid, *start, namespace, crate::control_group::time_namespace_id().ok())); });
    fs::write(
        root.join("fixture-result"),
        format!(
            "{mode}\nrecorded {}\nother start {against_other_start}\nown start {against_own_start}\nafter end {after_end}\ncollision name {collision}\nstart error {start_error}\n",
            if recorded_start.is_ok() { "ok" } else { "err" }
        ),
    )
    .expect("write the result");
}

/// Runs the procfs namespace helper in a new user and process id namespace.
fn run_procfs_fixture(mode: &str) -> String {
    let root = scratch();
    let mut command = Command::new("unshare");
    command.args(["--user", "--map-root-user", "--pid", "--fork"]);
    if mode == "own" {
        command.arg("--mount-proc");
    }
    let status = command
        .arg("--")
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_twelve_tests::procfs_namespace_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .env(FIXTURE_MODE, mode)
        .status()
        .expect("unshare must be installed and allowed to make a user and process id namespace for this test");
    assert!(status.success(), "{mode}: the helper process failed: {status:?}");
    return fs::read_to_string(root.path().join("fixture-result")).expect("the helper's result");
}

/// Acceptance rows A1, V2 (11a, the procfs of another process id namespace):
/// a process in a child process id namespace that still uses the procfs of its ancestor (`kill` and `/proc/<pid>` then name different
/// processes) cannot establish a launcher's start time, and the question answers "unknown", never "gone".
/// The control is a procfs of the namespace's own: the start time is read, the live launcher is alive, a reused id (another start
/// time) and an ended launcher are gone.
#[test]
fn a_procfs_of_another_pid_namespace_never_reads_as_a_gone_launcher() {
    let own: String = run_procfs_fixture("own");
    assert!(own.contains("recorded ok") && own.contains("other start Gone") && own.contains("own start Alive") && own.contains("after end Gone"), "the control did not move: {own}");
    let ancestor: String = run_procfs_fixture("ancestor");
    assert!(ancestor.contains("recorded err"), "a start time was read through a procfs of another namespace: {ancestor}");
    // The prerequisite (12b 3.2): the ancestor's procfs has a readable entry under the launcher's number that is another process, so a
    // lookup without the numbering guard would have read it; and the refusal is the numbering mismatch, not some other failure.
    assert!(ancestor.contains("collision name ") && !ancestor.contains("collision name unreadable") && !ancestor.contains("collision name sleep\n"), "no readable other process under the launcher's number: {ancestor}");
    assert!(ancestor.contains("start error ") && ancestor.contains("another process id namespace"), "the refusal is not the numbering mismatch: {ancestor}");
    assert!(ancestor.contains("other start Unknown(") && !ancestor.contains("Gone"), "a launcher was read as gone through a procfs of another namespace: {ancestor}");
}

/// Acceptance row "Weaker path" (11b 2.6, the writer's file sync):
/// an acknowledgement whose temporary file cannot be made durable is not published and does not count,
/// and acknowledging again, with the sync working, publishes it whole and clears the journal.
#[test]
fn an_acknowledgement_whose_file_cannot_be_synced_is_not_published() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    arm(Point::PublicationFileSync);

    let failed = acknowledge(&journal);
    let reached: bool = !crate::failpoints::fires(Point::PublicationFileSync);
    let published: bool = journal.join("acknowledged").exists();
    let blocked: Option<String> = survivor_note(&journal);
    acknowledge(&journal).expect("acknowledges once the sync works");
    let cleared: Option<String> = survivor_note(&journal);

    assert!(reached, "the file sync fault was never reached");
    assert!(failed.is_err_and(|text| { return text.contains("could not be written") && text.contains("nothing was acknowledged"); }), "a failed file sync was reported as an acknowledgement");
    assert!(!published, "a name was published although its file could not be synced");
    assert!(blocked.is_some(), "an unsynced acknowledgement counted");
    assert_eq!(cleared, None);
}

/// The helper process of the interrupted publication test:
/// it acknowledges the journal of the workspace, and its publisher stops between the finished temporary file and the rename that
/// publishes it (it says so and waits to be killed). Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_publisher_killed_before_its_rename_leaves_the_journal_unacknowledged"]
fn publisher_held_before_its_rename_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let journal: PathBuf = crate::journal_support::journal_dirs(&root).into_iter().next().expect("a journal");
    let ready: PathBuf = root.join("ready");
    before_publish_for_test(Some(Rc::new(move || {
        fs::write(&ready, "held").expect("announce the held publisher");
        std::thread::sleep(Duration::from_secs(300));
    })));
    let _ = acknowledge(&journal);
}

/// Acceptance row "Weaker path" (11b 2.6, a real interrupted publication):
/// a real publisher killed between its finished temporary file and the rename leaves the journal unacknowledged (the complete text is
/// under its temporary name, no published name), a recovery does not settle it, and acknowledging again settles it.
#[test]
fn a_publisher_killed_before_its_rename_leaves_the_journal_unacknowledged() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    let mut publisher: Child = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_twelve_tests::publisher_held_before_its_rename_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .spawn()
        .expect("the publisher helper");
    wait_for_file(&root.path().join("ready"));
    publisher.kill().expect("kill the publisher between the temporary file and the rename");
    let _ = publisher.wait();

    let temporaries: Vec<PathBuf> = fs::read_dir(&journal).expect("lists").flatten().map(|entry| { return entry.path(); }).filter(|path| { return path.file_name().is_some_and(|name| { let text = name.to_string_lossy(); return text.starts_with(".acknowledged.") && text.ends_with(".tmp"); }); }).collect();
    let published: bool = journal.join("acknowledged").exists();
    let note: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    let kept: String = live_lock(root.path());

    assert!(!published, "the rename happened although the publisher was held before it");
    assert_eq!(temporaries.len(), 1, "the real publisher did not leave its finished temporary file: {temporaries:?}");
    assert!(fs::read_to_string(&temporaries[0]).is_ok_and(|text| { return text.starts_with("acknowledged v1\n") && text.ends_with('\n'); }), "the temporary file is not a whole acknowledgement");
    assert!(note.is_some(), "a temporary file was read as an acknowledgement");
    assert_eq!(kept, LEFT_BY_THE_KILLED_RUN, "the journal was settled without a published acknowledgement: {messages:?}");
    acknowledge(&journal).expect("acknowledges again");
    let again: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the second acknowledgement did not settle the journal: {again:?}");
    // Settling is the completed journal, not only the restored text (12b 3.3).
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "the recovery after the second acknowledgement left a journal behind: {again:?}");
}
