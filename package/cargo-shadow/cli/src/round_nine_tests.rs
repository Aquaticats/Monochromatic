//! What:
//! Round 9 tests: the pending launcher (8a F1), the bounds on discovery and the internal locks (8a F2, F3), the strict and durable
//! acknowledgement (8a F4), the unknown inventory (8a K1), and the boot identity of the contract (assumption c).
//! Why:
//! The reviews of round 8 named schedules inside existing rows of the acceptance list;
//! each test here is one of them, with a handshake at the boundary and not an elapsed sleep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round nine', () => { it('a pending launcher is not a run that ended', ...) })
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::acknowledge::acknowledge;
use crate::bounded_run::{Bound, RouteCache, output_within};
use crate::cgroup_test_support::{assert_scope_record, delegated_route, scope_directory_of, wait_until};
use crate::control_group::{ControlGroup, Identity, Route, RouteSettings, View, current_identity, detect_route, use_identity_for_test};
use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::{begin, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::{journal_dirs, plant};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "unchecked output of the killed run").expect("lock");
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

/// The state letter of a process in `/proc/<pid>/stat`, or `None` when it is gone.
fn state_of(pid: i32) -> Option<char> {
    let stat: String = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    return stat.rsplit(')').next()?.trim_start().chars().next();
}

/// A bound for the commands of these tests.
fn bound() -> Bound {
    return Bound { limit: Duration::from_secs(30), what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// Acceptance rows A1, V2, V3 (8a F1):
/// the wrapper is killed after its launcher was spawned and before the manager made the scope.
/// The launcher is held there by a handshake (it stops itself before it can ask the manager),
/// the scope is absent and its record is on disk, and a recovery must not read that as a run that ended;
/// when the launcher goes on and the scope appears with a member, the recovery sees the member.
#[test]
fn a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let root = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));
    let launch: Command = planned.launcher(&route, Command::new("sleep").arg("600"), false);
    // The launcher is held at a gate before it asks the manager for anything: a shell waits for a file and then becomes the launcher.
    // `spawn` returns at once (the shell has exec'd), and the scope is absent until the test opens the gate.
    let gate: PathBuf = root.path().join("gate");
    let mut held: Command = Command::new("sh");
    held.arg("-c").arg(format!("while [ ! -e {} ]; do sleep 0.02; done; exec \"$@\"", gate.display())).arg("sh").arg(launch.get_program()).args(launch.get_args());
    held.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    let mut launcher = held.spawn().expect("starts the held launcher");
    let pid: i32 = i32::try_from(launcher.id()).expect("a process id");
    assert_eq!(state_of(pid).map(|state| { return state != 'Z'; }), Some(true), "the held launcher is not running");
    assert!(!planned.directory().exists(), "the scope exists before the launcher asked for it");

    let while_pending: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    let lock_while_pending: String = live_lock(root.path());
    fs::write(&gate, "open").expect("open the gate");
    wait_until("the scope to be made", &|| { return planned.members().map(|found| { return !found.is_empty(); }).unwrap_or(false); });
    let after: Option<String> = survivor_note(&journal);
    let stopped: bool = planned.stop(Duration::from_secs(10)).expect("stops");
    let _ = launcher.wait();

    let pending: String = while_pending.expect("a pending launcher blocks");
    assert!(pending.contains("did not record its end") && pending.contains("may not have finished entering"), "{pending}");
    assert_eq!(lock_while_pending, "unchecked output of the killed run", "a recovery restored files while a program could still start: {messages:?}");
    assert!(after.is_some_and(|text| { return text.contains("still running"); }), "the member of the scope that appeared was not seen");
    assert!(stopped);
}

/// Acceptance rows A1, V3:
/// a run that ended records its end,
/// and a recovery then reads its collected, absent scope as ended.
#[test]
fn a_settled_run_whose_scope_was_collected_is_read_as_ended() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");

    output_within(&mut Command::new("true"), None, &bound()).expect("runs");

    let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
    assert_scope_record(&journal.join("groups").display().to_string());
    assert!(record.lines().last().is_some_and(|line| { return line.starts_with("settled "); }), "the run did not record its end: {record:?}");
    // The scope must be collected, not merely empty (9b 3.4): the recovery below then takes the settled-and-absent branch.
    let scope: PathBuf = scope_directory_of(&journal.join("groups").display().to_string());
    wait_until("the manager to collect the scope", &|| { return !scope.exists(); });
    assert!(!scope.exists());
    assert_eq!(survivor_note(&journal), None);
    drop(transaction);
}

/// Acceptance row V6 (8a F2):
/// a discovery program whose descendant keeps its pipes open after it exited is still answered by the deadline,
/// not after the descendant ends.
#[test]
fn a_discovery_whose_descendant_holds_the_pipes_ends_at_its_deadline() {
    let directory = scratch();
    let shell_file: PathBuf = directory.path().join("shell");
    let ready_file: PathBuf = directory.path().join("descendant-ready");
    let descendant_file: PathBuf = directory.path().join("descendant");
    // The direct program records its own process id, starts a descendant that records its id, says it is ready and holds the
    // inherited pipes open, waits for that readiness, and exits: both facts of the claim are established before the deadline.
    let script: String = format!(
        "echo $$ > {shell}; sh -c 'echo $$ > {descendant}; : > {ready}; exec sleep 5' & while [ ! -e {ready} ]; do sleep 0.01; done; exit 0",
        shell = shell_file.display(),
        descendant = descendant_file.display(),
        ready = ready_file.display()
    );
    let bound: Duration = Duration::from_millis(2500);
    let settings: RouteSettings = RouteSettings {
        systemctl: "sh".to_string(),
        question: Some(vec!["-c".to_string(), script]),
        discovery_bound: bound,
        ..RouteSettings::default()
    };
    // An observer, alongside the discovery, establishes at the boundary (not after the return) that the direct program has exited
    // while the descendant is alive and ready, and when that was.
    let observer = std::thread::spawn({
        let (watched_shell, watched_ready, watched_descendant) = (shell_file.clone(), ready_file.clone(), descendant_file.clone());
        move || -> (Instant, bool) {
            crate::cgroup_test_support::wait_until("the descendant to be ready", &|| { return watched_ready.exists() && watched_descendant.exists(); });
            let shell: i32 = crate::contain_counterexample_tests::read_pid(&watched_shell);
            let descendant: i32 = crate::contain_counterexample_tests::read_pid(&watched_descendant);
            crate::cgroup_test_support::wait_until("the direct program to exit", &|| { return !crate::contain_counterexample_tests::is_running(shell); });
            return (Instant::now(), crate::contain_counterexample_tests::is_running(descendant));
        }
    });
    let started = Instant::now();

    let refused = detect_route(&settings).expect_err("a manager whose answer never ends is a missing capability");
    let returned: Instant = Instant::now();
    let (exited_at, descendant_alive_then): (Instant, bool) = observer.join().expect("the observer");
    crate::contain_counterexample_tests::stop_quietly(crate::contain_counterexample_tests::read_pid(&descendant_file));

    assert!(descendant_alive_then, "the descendant was not alive when the direct program exited, so it did not hold the pipes");
    assert!(exited_at < started + bound, "the direct program exited only at the deadline, so the boundary of the claim was not reached before it");
    assert!(exited_at < returned, "the direct program had not exited when the discovery returned");
    assert!(refused.to_string().contains("did not answer"), "{refused}");
    assert!(returned.duration_since(started) < bound + Duration::from_millis(1500), "the discovery waited {:?} for a descendant that lives five seconds", returned.duration_since(started));
}

/// Acceptance row V6 (8a F3):
/// the registry of unsynchronized directories never makes a caller wait,
/// and a busy registry is read as "unsynchronized".
#[test]
fn a_busy_unsynced_registry_returns_at_once_and_answers_conservatively() {
    let (ready_sender, ready) = mpsc::channel::<()>();
    let (done_sender, done) = mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        crate::journal_replace::hold_unsynced_registry_for_test(|| {
            ready_sender.send(()).expect("announce the held registry");
            done.recv_timeout(Duration::from_secs(30)).expect("released");
        });
    });
    ready.recv_timeout(Duration::from_secs(30)).expect("the registry is held");
    let (answer_sender, answer) = mpsc::channel::<bool>();
    let caller = std::thread::spawn(move || {
        let directory = scratch();
        answer_sender.send(crate::journal_replace::registry_calls_for_test(directory.path())).expect("answer");
    });

    let answered: Result<bool, mpsc::RecvTimeoutError> = answer.recv_timeout(Duration::from_secs(10));
    done_sender.send(()).expect("release the holder");
    holder.join().expect("holder");
    caller.join().expect("caller");

    assert_eq!(answered, Ok(true), "a caller waited for a busy registry, or answered that the directory was durable");
}

/// A route that was found, for the cache test.
fn some_route() -> Route {
    return Route {
        systemd_run: "systemd-run".to_string(),
        slice: "app.slice".to_string(),
        slice_dir: PathBuf::from("/sys/fs/cgroup/app.slice"),
        namespace: 1,
        slice_view: View { mount: 1, inode: 1 },
    };
}

/// Acceptance row V6 (8a F3), the production cache:
/// a caller that finds another caller in the middle of the discovery does not wait for it.
#[test]
fn the_route_cache_does_not_make_a_caller_wait_for_a_discovery_in_flight() {
    static CACHE: RouteCache = RouteCache::new();
    let (started_sender, started) = mpsc::channel::<()>();
    let (release_sender, release) = mpsc::channel::<()>();
    let slow = std::thread::spawn(move || {
        return CACHE.get_or_discover(&|| {
            started_sender.send(()).expect("announce the discovery");
            release.recv_timeout(Duration::from_secs(30)).expect("released");
            return Ok(some_route());
        });
    });
    started.recv_timeout(Duration::from_secs(30)).expect("a discovery is in flight");
    let (answer_sender, answer) = mpsc::channel::<bool>();
    let other = std::thread::spawn(move || {
        answer_sender.send(CACHE.get_or_discover(&|| { return Ok(some_route()); }).is_ok()).expect("answer");
    });

    let answered: Result<bool, mpsc::RecvTimeoutError> = answer.recv_timeout(Duration::from_secs(10));
    release_sender.send(()).expect("release the slow discovery");
    let slow_result = slow.join().expect("slow");
    other.join().expect("other");

    assert_eq!(answered, Ok(true), "a caller waited for the discovery in flight");
    assert!(slow_result.is_ok());
    assert!(CACHE.get_or_discover(&|| { return Err(crate::control_group::GroupUnavailable { reason: "must not be asked".to_string() }); }).is_ok(), "the answer was not kept");
}

/// Acceptance row V6 (certification, third pass), the production cache with its own mutex held:
/// a caller that finds the cache's mutex held answers at once, discovering for itself, and does not wait for the holder.
/// The holder keeps the mutex until the answer arrived, so a caller that waited for the lock would never answer in the window.
#[test]
fn the_route_cache_answers_at_once_while_its_own_mutex_is_held() {
    static CACHE: RouteCache = RouteCache::new();
    CACHE.hold_for_test(|| {
        let answered: bool = crate::round_sixteen_tests::answers_within("a caller of the route cache", Duration::from_secs(10), || { return CACHE.get_or_discover(&|| { return Ok(some_route()); }).is_ok(); });
        assert!(answered, "the caller did not get an answer from its own discovery");
    });
}

/// Acceptance row "Weaker path" (8a F4):
/// a marker that has three lines but is not the whole versioned record does not count.
#[test]
fn an_acknowledgement_with_three_lines_that_are_not_a_process_id_and_a_time_does_not_count() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    for incomplete in ["acknowledged v1\n\n\n", "acknowledged v1\n123\n\n", "acknowledged v1\nx\ny\n", "acknowledged v1\n123\nsoon\n", "acknowledged v1\n-1\n5\n", "acknowledged v1\n123\n456\nextra\n"] {
        fs::write(journal.join("acknowledged"), incomplete).expect("write");
        assert!(survivor_note(&journal).is_some(), "{incomplete:?} counted as an acknowledgement");
    }
    fs::write(journal.join("acknowledged"), "acknowledged v1\n123\n456\n").expect("write");
    assert_eq!(survivor_note(&journal), None, "a whole record did not count");
}

/// Acceptance row "Weaker path" (8a, the acknowledgement-durability boundary; 9b 3.6):
/// a recovery relies on an acknowledgement only after it made the file and then its directory entry durable itself;
/// a failure of either step means the acknowledgement does not count, each step is failed on its own with its fault reached,
/// and it counts once both can be made.
#[test]
fn an_acknowledgement_the_reader_cannot_make_durable_does_not_count_until_it_can() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    acknowledge(&journal).expect("acknowledges");

    for (label, step) in [("the file sync", Point::AcknowledgementFileSync), ("the directory sync", Point::AcknowledgementDirectorySync)] {
        arm(step);
        let failing: Option<String> = survivor_note(&journal);
        assert!(!crate::failpoints::fires(step), "{label}: the fault was never reached");
        assert!(failing.is_some(), "{label}: an acknowledgement that could not be made durable was relied on");
    }
    assert_eq!(survivor_note(&journal), None, "a whole, durable acknowledgement did not count");
}

/// Acceptance row "Weaker path" (9b 3.6, the interrupted publication):
/// a publication that stopped after the complete text was written under its temporary name and before the rename leaves the journal
/// unacknowledged, a recovery does not settle it, and acknowledging again publishes it whole.
/// The state is built by hand from a real acknowledgement (the rename is not injectable); the sibling name is the temporary kind that
/// the journal's own listing already tolerates.
#[test]
fn an_interrupted_publication_leaves_the_journal_unacknowledged_until_it_is_made_again() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    acknowledge(&journal).expect("acknowledges");
    let whole: String = fs::read_to_string(journal.join("acknowledged")).expect("the acknowledgement");
    fs::remove_file(journal.join("acknowledged")).expect("take the published name away");
    fs::write(journal.join("acknowledged.interrupted.tmp"), &whole).expect("the temporary the publication stopped at");

    let note: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    let kept: String = live_lock(root.path());

    assert!(note.is_some(), "a temporary file was read as an acknowledgement");
    assert_eq!(kept, "unchecked output of the killed run", "the journal was settled without a published acknowledgement: {messages:?}");
    acknowledge(&journal).expect("acknowledges again");
    let again: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the second acknowledgement did not settle the journal: {again:?}");
}

/// Acceptance row "Weaker path":
/// the command that cannot make its acknowledgement durable says so and does not claim it,
/// and one whose publication fails leaves no file and the journal unacknowledged.
/// Each fault must have been reached (9b 3.6).
#[test]
fn an_acknowledgement_whose_barrier_or_publication_fails_is_not_reported_as_made() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");

    arm(Point::AcknowledgementPublish);
    let unpublished = acknowledge(&journal);
    let exists_after_publish_failure: bool = journal.join("acknowledged").exists();
    assert!(!crate::failpoints::fires(Point::AcknowledgementPublish), "the publication fault was never reached");
    arm(Point::AcknowledgementDurability);
    let undurable = acknowledge(&journal);
    assert!(!crate::failpoints::fires(Point::AcknowledgementDurability), "the directory-sync fault was never reached");

    assert!(unpublished.is_err() && !exists_after_publish_failure, "a failed publication left a file or claimed success: {unpublished:?}");
    assert!(undurable.is_err_and(|text| { return text.contains("durable"); }), "a failed barrier was reported as an acknowledgement");
}

/// Acceptance row "Weaker path" (8b 2.7):
/// the whole schedule the name promises:
/// a journal of the weaker path is acknowledged, a recovery settles it like any other,
/// the lockfile is put back and the refused copy is kept.
#[test]
fn an_acknowledged_weaker_path_journal_is_settled_like_any_other() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
    assert!(survivor_note(&journal).is_some(), "the journal should block before the acknowledgement");

    acknowledge(&journal).expect("acknowledges");
    let messages: Vec<String> = recover(root.path(), &refusing);

    assert_eq!(live_lock(root.path()), "original", "the acknowledged journal was not settled: {messages:?}");
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the refused copy"), "unchecked output of the killed run");
    assert!(journal_dirs(root.path()).is_empty(), "the journal was not removed");
}

/// Acceptance row K1 (8a K1):
/// a crate root that cannot be listed gives a warning that the inventory is unknown,
/// also when no count is past the threshold; it never refuses.
#[test]
fn an_unreadable_crate_root_gives_an_unknown_inventory_warning_and_never_refuses() {
    let root = repository();
    let crate_root: PathBuf = root.path().join(CRATE_DIR);
    fs::set_permissions(&crate_root, fs::Permissions::from_mode(0o311)).expect("make the crate root unreadable");
    if fs::read_dir(&crate_root).is_ok() {
        fs::set_permissions(&crate_root, fs::Permissions::from_mode(0o755)).expect("restore");
        eprintln!("SKIPPED: this user reads a directory without the read permission (a privileged user), so the schedule cannot be made");
        return;
    }

    let begun = begin(root.path(), &[snapshot("original")], false);
    fs::set_permissions(&crate_root, fs::Permissions::from_mode(0o755)).expect("restore");

    let transaction = begun.expect("an unreadable inventory never refuses");
    let notice: String = transaction.kept_notice().map(str::to_string).expect("an unknown inventory is not silence");
    assert!(notice.contains("could not be listed") && notice.contains("unknown") && notice.contains(&crate_root.display().to_string()), "{notice}");
}

/// The identity this test pretends to have after a reboot of the same machine.
fn after_a_reboot() -> Identity {
    let now: Identity = current_identity().expect("this machine has an identity");
    return Identity { boot: with_first_digit_changed(&now.boot), machine: now.machine };
}

/// An id of the same format whose first digit differs (round 10, 9a D2: the identity must stay well formed to count).
fn with_first_digit_changed(id: &str) -> String {
    let mut bytes: Vec<u8> = id.as_bytes().to_vec();
    bytes[0] = if bytes[0] == b'0' { b'1' } else { b'0' };
    return String::from_utf8(bytes).expect("ascii");
}

/// Contract assumption c, the boot identity:
/// a recovery that reads the same machine id and a different boot id treats the run's process liveness as discharged,
/// and settles the journal of the weaker path without an acknowledgement, with every other check still running.
#[test]
fn a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
    assert!(survivor_note(&journal).is_some(), "without a reboot the journal blocks");

    use_identity_for_test(Some(Ok(after_a_reboot())));
    let note: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    use_identity_for_test(None);

    assert_eq!(note, None, "a reboot of the same machine did not discharge the liveness");
    assert_eq!(live_lock(root.path()), "original", "the journal was not settled after the reboot: {messages:?}");
    assert!(journal_dirs(root.path()).is_empty());
}

/// Contract assumption c, the boot identity:
/// another machine id, an unreadable identity of this system, a journal without a recorded identity, and the same boot
/// (a suspend and resume keeps the boot id) all keep the refusal.
#[test]
fn another_machine_an_unreadable_identity_no_record_or_the_same_boot_keeps_the_refusal() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
    let now: Identity = current_identity().expect("an identity");
    let cases: Vec<(&str, Option<Result<Identity, String>>)> = vec![
        ("another machine", Some(Ok(Identity { boot: with_first_digit_changed(&now.boot), machine: with_first_digit_changed(&now.machine) }))),
        ("an unreadable identity", Some(Err("the boot id cannot be read".to_string()))),
        ("the same boot (suspend and resume)", None),
    ];
    for (label, identity) in cases {
        use_identity_for_test(identity);
        let note: Option<String> = survivor_note(&journal);
        use_identity_for_test(None);
        assert!(note.is_some(), "{label}: the refusal was dropped");
    }

    let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
    let without_identity: String = recorded.lines().next().expect("a view line").to_string() + "\n";
    fs::write(journal.join("view"), without_identity).expect("lose the identity");
    use_identity_for_test(Some(Ok(after_a_reboot())));
    let no_record: Option<String> = survivor_note(&journal);
    use_identity_for_test(None);
    assert!(no_record.is_some(), "a journal without a recorded identity was cleared by a reboot");
}

/// Contract assumption c, the boot identity:
/// the discharge is of process liveness only:
/// a missing snapshot and an incomplete listing of the journals still keep the journal,
/// and a recorded scope that has a member after a reboot is a contradiction that blocks.
#[test]
fn the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
    fs::remove_file(journal.join("0.lock")).expect("lose a snapshot");
    use_identity_for_test(Some(Ok(after_a_reboot())));
    let messages: Vec<String> = recover(root.path(), &refusing);
    use_identity_for_test(None);
    assert_eq!(live_lock(root.path()), "unchecked output of the killed run", "a journal with a missing snapshot was settled: {messages:?}");
    assert!(journal.exists());

    let other = repository();
    let first: PathBuf = plant(other.path(), &[snapshot("original")], &[]);
    let _second = plant(other.path(), &[snapshot("original")], &[]);
    fs::write(first.join("groups"), "pgroup starting\npgroup 2147483646\n").expect("record");
    use_identity_for_test(Some(Ok(after_a_reboot())));
    arm(Point::ListingEntryNotFound);
    let listing: Vec<String> = recover(other.path(), &refusing);
    use_identity_for_test(None);
    assert_eq!(live_lock(other.path()), "unchecked output of the killed run", "an incomplete listing was recovered over: {listing:?}");

    let Some(route) = delegated_route() else {
        return;
    };
    let contradiction = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let scoped: PathBuf = journal_with_records(contradiction.path(), &format!("{}\n", planned.record()));
    let mut launch: Command = planned.launcher(&route, Command::new("sleep").arg("600"), false);
    launch.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    let mut member = launch.spawn().expect("starts");
    wait_until("the scope to hold a member", &|| { return planned.members().map(|found| { return !found.is_empty(); }).unwrap_or(false); });
    use_identity_for_test(Some(Ok(after_a_reboot())));
    let note: Option<String> = survivor_note(&scoped);
    use_identity_for_test(None);
    let _ = member.kill();
    let _ = planned.stop(Duration::from_secs(10));
    let _ = member.wait();
    assert!(note.is_some_and(|text| { return text.contains("although the boot id changed"); }), "a scope with a member after a reboot was cleared");
}

/// Acceptance rows A1 and V6 (reaping):
/// a direct child that cannot be reaped within its bound makes the run not quiescent,
/// with writers possibly remaining and the journal retained, and the return is not a wait without end.
#[test]
fn a_direct_child_that_cannot_be_reaped_makes_the_run_not_quiescent() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
    arm(Point::ReapStuck);
    let started = Instant::now();

    let result = output_within(&mut Command::new("true"), None, &bound());

    assert_scope_record(&journal.join("groups").display().to_string());
    assert!(matches!(result, Err(crate::bounded_run::RunError::NotQuiescent { .. })), "{result:?}");
    assert!(transaction.writers_may_remain(), "the scope was not told that writers may remain");
    assert!(journal.exists() && !journal.to_string_lossy().ends_with(".done"));
    assert!(started.elapsed() < Duration::from_secs(10), "the run waited {:?}", started.elapsed());
    let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
    assert!(!record.lines().any(|line| { return line.starts_with("settled "); }), "a run that could not show its end recorded one: {record:?}");
    drop(transaction);
}

/// Acceptance rows V3 and V4 (8b 2.5):
/// a record that cannot be written, or cannot be made durable, starts no program,
/// in the delegated path and in the weaker path.
/// The failure of the sync is the proof that the sync comes before the start.
#[test]
fn a_record_that_cannot_be_written_or_made_durable_starts_no_program() {
    for weaker in [false, true] {
        for fault in [Point::RecordWrite, Point::RecordSync] {
            let plain = scratch();
            if weaker {
                crate::bounded_run::use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
            } else if delegated_route().is_none() {
                return;
            }
            let root = repository();
            let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
            let marker: PathBuf = root.path().join("ran");
            let _ = crate::bounded_run::take_trace_for_test();
            arm(fault);

            let result = output_within(Command::new("sh").args(["-c", &format!(": > {}", marker.display())]), None, &bound());
            crate::bounded_run::use_route_settings_for_test(None);
            // The weaker-path notice is traced too and is observed by its own test; this case compares the order of the record events.
            let trace: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return !event.starts_with("notice "); }).collect();
            // The containment the runner actually attempted at the fault (certification group 2), also when the write itself fails:
            // the first record it tried to make, not the route that discovery could have given.
            let kind: &str = if weaker { "pgroup-starting" } else { "scope" };
            let mut expected_trace: Vec<String> = if matches!(fault, Point::RecordWrite) {
                vec![format!("attempt {kind}")]
            } else {
                vec![format!("attempt {kind}"), format!("written {kind}"), format!("sync {kind} whole=true")]
            };
            // A scope that will never be made is settled afterwards (nothing launched it); the weaker path has nothing to settle.
            if !weaker {
                expected_trace.extend(["attempt settled", "written settled", "sync settled whole=true", "synced settled"].map(String::from));
            }
            assert_eq!(trace, expected_trace, "weaker={weaker} {fault:?}: the runner did not attempt the containment kind the case claims, or went on past the fault");

            // The fault must have been reached (9b 3.6): a refusal before the record was made would leave it armed.
            // `RecordWrite` is the write of the appended record line; `RecordSync` is the sync of the record file after that write.
            assert!(!crate::failpoints::fires(fault), "weaker={weaker} {fault:?}: the fault was never reached, so the refusal came from somewhere else");
            let refusal: String = result.expect_err("the program ran although its record was not durable").to_string();
            let expected: &str = if matches!(fault, Point::RecordWrite) { "injected failure to write the containment record" } else { "injected failure to make the containment record durable" };
            assert!(refusal.contains(expected), "weaker={weaker} {fault:?}: the refusal is not the injected fault: {refusal}");
            assert!(!marker.exists(), "weaker={weaker} {fault:?}: the program started");
            drop(transaction);
        }
    }
}

/// Acceptance rows V3 and V4 (preparation windows):
/// a journal cut off after its record, or after its record and view, holds no child,
/// and a recovery removes it without touching the lockfile.
#[test]
fn a_journal_cut_off_during_its_preparation_is_removed_without_touching_the_lockfile() {
    for files in [vec!["groups"], vec!["groups", "view"], vec!["groups", "view", "owner"]] {
        let root = repository();
        let cut_off: PathBuf = root.path().join(crate::journal::JOURNAL_DIR).join("cut-off");
        fs::create_dir_all(&cut_off).expect("a journal directory");
        for file in &files {
            fs::write(cut_off.join(file), "").expect("a file of the preparation");
        }

        let note: Option<String> = survivor_note(&cut_off);
        let messages: Vec<String> = recover(root.path(), &refusing);

        assert_eq!(note, None, "{files:?}: an empty record shows that no child started");
        assert_eq!(live_lock(root.path()), "unchecked output of the killed run", "{files:?}: {messages:?}");
        assert!(!cut_off.exists(), "{files:?}: the cut-off journal stayed: {messages:?}");
    }
}

/// Acceptance row V5:
/// a journal of two roots with their manifests, one root refused and the other accepted:
/// the refused root's lockfile and manifest go back together, the accepted root keeps both,
/// and the refused lockfile is saved without overwriting an existing refused copy.
#[test]
fn two_roots_with_manifests_are_restored_or_kept_together_per_root() {
    let root = scratch();
    let first: &str = "package/n/first";
    let second: &str = "package/n/second";
    for crate_dir in [first, second] {
        fs::create_dir_all(root.path().join(crate_dir)).expect("dir");
        fs::write(root.path().join(crate_dir).join("Cargo.lock"), format!("output of {crate_dir}")).expect("lock");
        fs::write(root.path().join(crate_dir).join("Cargo.toml"), format!("edited manifest of {crate_dir}")).expect("manifest");
    }
    fs::write(root.path().join(first).join("Cargo.lock.refused"), "an earlier refused copy").expect("an earlier refused copy");
    let snapshots: Vec<LockSnapshot> = [first, second].iter().map(|crate_dir| { return LockSnapshot { crate_dir: (*crate_dir).to_string(), source: format!("snapshot of {crate_dir}") }; }).collect();
    let manifests: Vec<crate::manifest_edit::ManifestEdit> = [first, second]
        .iter()
        .map(|crate_dir| { return crate::manifest_edit::ManifestEdit { crate_dir: (*crate_dir).to_string(), path: root.path().join(crate_dir).join("Cargo.toml"), before: format!("manifest of {crate_dir}"), produced: None }; })
        .collect();
    let _journal = plant(root.path(), &snapshots, &manifests);

    let messages: Vec<String> = recover(root.path(), &|before, _live| {
        return if before[0].crate_dir == first { vec!["dep@1.0.1 refused".to_string()] } else { Vec::new() };
    });

    let read = |crate_dir: &str, name: &str| -> String { return fs::read_to_string(root.path().join(crate_dir).join(name)).expect("a file"); };
    assert_eq!(read(first, "Cargo.lock"), format!("snapshot of {first}"), "{messages:?}");
    assert_eq!(read(first, "Cargo.toml"), format!("manifest of {first}"), "the refused root's manifest was not put back with its lockfile");
    assert_eq!(read(first, "Cargo.lock.refused"), "an earlier refused copy", "an existing refused copy was overwritten");
    let saved: Vec<String> = fs::read_dir(root.path().join(first)).expect("lists").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).filter(|name| { return name.starts_with("Cargo.lock.refused."); }).collect();
    assert_eq!(saved.len(), 1, "the refused lockfile was not saved under its own name: {saved:?}");
    // The new copy must hold the refused output itself, not an empty or other file (9b 3.3).
    assert_eq!(read(first, &saved[0]), format!("output of {first}"), "the new refused copy does not hold the refused output");
    assert_eq!(read(second, "Cargo.lock"), format!("output of {second}"), "the accepted root's lockfile was restored");
    assert_eq!(read(second, "Cargo.toml"), format!("edited manifest of {second}"), "the accepted root's manifest was restored");
    assert!(journal_dirs(root.path()).is_empty());
}
