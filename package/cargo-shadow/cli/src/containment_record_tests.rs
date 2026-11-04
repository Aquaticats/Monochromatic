//! What:
//! Rounds 7 and 8 tests of what a journal records about containment and what a recovery and an admission do with it:
//! the weaker path and its acknowledgement, the capability classes, the views of amended assumption c, the record before the start,
//! and the windows of a wrapper that is killed during a start.
//! Why:
//! The contract (`doc/planning/cargo-shadow-guarantees.md`) says what each of these promises;
//! the acceptance rows "V2, amended c", "V3, V4" and "Weaker path" are shown here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('containment record', () => { it('names the missing capability', ...) })
//! ```

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::rc::Rc;
use std::time::Duration;

use crate::acknowledge::acknowledge;
use crate::bounded_run::{Bound, on_start_for_test, output_within, use_route_settings_for_test, weaker_notice};
use crate::cgroup_test_support::{assert_scope_record, delegated_route};
use crate::contain_counterexample_tests::wait_for_file;
use crate::control_group::{ControlGroup, Record, RouteSettings, View, detect_route, parse_record, view_of};
use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::journal_dirs;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/k/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_RECORD_FIXTURE_ROOT";

/// The environment variable that tells the helper process which containment to use (`weaker` or the strong default).
const FIXTURE_MODE: &str = "CARGO_SHADOW_RECORD_FIXTURE_MODE";

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

/// A bound for the commands of these tests.
fn bound() -> Bound {
    return Bound { limit: Duration::from_secs(20), what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// A judge that refuses the live lockfile, so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// Plants a journal whose `groups` file holds `records`,
/// and returns its directory.
fn journal_with_records(root: &Path, records: &str) -> PathBuf {
    let journal: PathBuf = crate::journal_support::plant(root, &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), records).expect("write the records");
    return journal;
}

/// The settings of the host classes that lack a capability, with the words the reason must hold.
fn missing_capability_classes(plain: &Path) -> Vec<(&'static str, RouteSettings, &'static str)> {
    return vec![
        ("no systemd-run", RouteSettings { systemd_run: "cargo-shadow-no-such-program".to_string(), ..RouteSettings::default() }, "is not installed"),
        (
            "an unreachable user manager",
            RouteSettings {
                environment: vec![
                    ("XDG_RUNTIME_DIR".to_string(), Some("/nonexistent".to_string())),
                    ("DBUS_SESSION_BUS_ADDRESS".to_string(), Some("unix:path=/nonexistent/bus".to_string())),
                ],
                ..RouteSettings::default()
            },
            "user manager cannot",
        ),
        ("no cgroup v2 mount", RouteSettings { cgroup_root: plain.to_path_buf(), ..RouteSettings::default() }, "cgroup v2"),
        ("a unit the manager does not have", RouteSettings { slice: "cargo-shadow-no-such.service".to_string(), ..RouteSettings::default() }, "user manager cannot"),
    ];
}

/// Acceptance row "Weaker path":
/// each unavailable-capability class produces the promised notice
/// and a run that goes on in a process group whose record is written before the start (`pgroup starting`) and after it.
#[test]
fn each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start() {
    let plain = scratch();
    for (class, settings, words) in missing_capability_classes(plain.path()) {
        let unavailable = detect_route(&settings).expect_err(class);
        let notice: String = weaker_notice(&unavailable);
        assert!(unavailable.to_string().contains(words), "{class}: {unavailable}");
        assert!(notice.contains("quiescence is not shown") && notice.contains("cargo-shadow acknowledge") && notice.lines().count() == 1, "{class}: {notice}");

        use_route_settings_for_test(Some(settings));
        let root = repository();
        let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        let seen_at_start: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let observer: Rc<RefCell<String>> = Rc::clone(&seen_at_start);
        let watched: PathBuf = journal.join("groups");
        on_start_for_test(Some(Rc::new(move || { *observer.borrow_mut() = fs::read_to_string(&watched).unwrap_or_default(); })));

        let result = output_within(Command::new("sh").args(["-c", "echo ok"]), None, &bound());
        on_start_for_test(None);
        use_route_settings_for_test(None);

        let output = result.expect(class);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok", "{class}");
        assert_eq!(seen_at_start.borrow().as_str(), "pgroup starting\n", "{class}: the announcement was not on disk before the start");
        let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
        let kinds: Vec<Record> = record.lines().map(|line| { return parse_record(line).expect("a record"); }).collect();
        assert!(matches!(kinds.as_slice(), [Record::ProcessGroupStarting, Record::ProcessGroup(_)]), "{class}: {record:?}");
        assert!(transaction.process_group_only(), "{class}: the transaction was not told the guarantee is weaker");
        assert!(!transaction.writers_may_remain(), "{class}");
        drop(transaction);
    }
}

/// The weaker path stops the process group at the bound,
/// as round 5 did.
#[test]
fn the_weaker_path_still_stops_the_process_group_at_the_bound() {
    let plain = scratch();
    use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
    let marker = plain.path().join("late");
    let script: String = format!("(sleep 2; : > {}) & sleep 30", marker.display());

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &Bound { limit: Duration::from_millis(400), ..bound() });
    use_route_settings_for_test(None);
    std::thread::sleep(Duration::from_millis(2_700));

    assert!(result.is_err(), "the run should have passed its bound");
    assert!(!marker.exists(), "a member of the process group wrote after the stop");
}

/// Acceptance row "Weaker path":
/// both `pgroup starting` alone and a completed record whose group is gone block a recovery and a new transaction
/// until the journal is acknowledged,
/// and the refusal names the journal and the command.
#[test]
fn a_weaker_path_journal_is_refused_by_recovery_and_admission_before_it_is_acknowledged() {
    for records in ["pgroup starting\n", "pgroup starting\npgroup 2147483646\n"] {
        let root = repository();
        let journal: PathBuf = journal_with_records(root.path(), records);

        let note: String = survivor_note(&journal).expect("a weaker-path journal blocks");
        let messages: Vec<String> = recover(root.path(), &refusing);
        let admitted = begin(root.path(), &[snapshot("second")], false);

        assert!(note.contains(&journal.display().to_string()) && note.contains("cargo-shadow acknowledge"), "{note}");
        assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{records:?}: a recovery restored a file a child may still write: {messages:?}");
        assert!(journal.exists(), "{records:?}: the journal was not retained");
        assert!(admitted.is_err(), "{records:?}: a transaction started beside an unacknowledged journal");
    }
}

/// Acceptance row "Weaker path":
/// the command works under the workspace lock, durably acknowledges exactly one journal,
/// and a recovery then settles that journal like any other,
/// while the other journal keeps blocking.
#[test]
fn acknowledging_one_of_two_journals_leaves_the_other_blocking_everything() {
    let root = repository();
    let first: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    let second: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    fs::write(first.join("groups"), "pgroup starting\n").expect("record");
    fs::write(second.join("groups"), "pgroup starting\n").expect("record");

    let said: String = acknowledge(&first).expect("acknowledges");

    assert!(said.contains("asserted") || said.contains("assert"), "{said}");
    assert_eq!(survivor_note(&first), None);
    assert!(survivor_note(&second).is_some(), "the second journal was acknowledged too");
    assert!(begin(root.path(), &[snapshot("third")], false).is_err(), "the unacknowledged journal no longer blocks a transaction");
    let text: String = fs::read_to_string(first.join("acknowledged")).expect("the file");
    assert!(text.starts_with("acknowledged v1\n") && text.lines().count() == 3, "{text:?}");
}

/// Acceptance row "Weaker path":
/// the command takes the workspace lock without waiting,
/// so a live transaction makes it refuse at once and acknowledge nothing.
#[test]
fn acknowledging_while_a_transaction_holds_the_workspace_is_refused() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    let _held = crate::journal_transaction::take_workspace(root.path()).expect("tries").expect("takes the lock");

    let refused = acknowledge(&journal);

    assert!(refused.is_err(), "{refused:?}");
    assert!(!journal.join("acknowledged").exists());
}

/// Acceptance row "Weaker path":
/// an interrupted publication leaves the journal unacknowledged:
/// a partial or unreadable file does not count.
#[test]
fn a_partial_acknowledgement_does_not_count() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    for partial in ["", "acknowledged v1\n", "acknowledged v1\n123\n", "acknowledged v2\n1\n2\n", "acknowledged v1\n1\n2"] {
        fs::write(journal.join("acknowledged"), partial).expect("write");
        assert!(survivor_note(&journal).is_some(), "{partial:?} counted as an acknowledgement");
    }
}

/// Acceptance row "Weaker path":
/// the acknowledgement overrides the uncertainty of the containment context only:
/// a recorded process group that is positively observed to have a member still blocks,
/// and an invalid snapshot still keeps the journal.
#[test]
fn an_acknowledgement_does_not_override_a_live_group_or_unrelated_validation() {
    use std::os::unix::process::CommandExt;
    let root = repository();
    let live: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    let mut orphan: Child = Command::new("sleep").arg("600").process_group(0).spawn().expect("a live group");
    fs::write(live.join("groups"), format!("pgroup starting\npgroup {}\n", orphan.id())).expect("record the live group");
    acknowledge(&live).expect("acknowledges");

    let note: Option<String> = survivor_note(&live);
    let _ = orphan.kill();
    let _ = orphan.wait();

    assert!(note.is_some_and(|text| { return text.contains("still running"); }), "a live recorded group was overridden");

    let other = repository();
    let invalid: PathBuf = journal_with_records(other.path(), "pgroup starting\n");
    acknowledge(&invalid).expect("acknowledges");
    fs::remove_file(invalid.join("0.lock")).expect("lose a snapshot");
    let messages: Vec<String> = recover(other.path(), &refusing);
    assert_eq!(live_lock(other.path()), LEFT_BY_THE_KILLED_RUN, "an acknowledged journal with a missing snapshot was restored: {messages:?}");
    assert!(invalid.exists(), "the journal with a missing snapshot was removed");
}

/// Acceptance row "Weaker path":
/// the command refuses a path that is not a journal of a workspace, and a finished journal.
#[test]
fn acknowledge_refuses_what_is_not_an_open_journal_of_a_workspace() {
    let root = repository();
    let elsewhere = scratch();

    assert!(acknowledge(elsewhere.path()).is_err());
    assert!(acknowledge(&root.path().join("no-such-journal")).is_err());
    // A journal that is already finished (marked done) is refused too.
    let journal: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    let finished: PathBuf = crate::journal_transaction::publish_done(&journal).expect("marks the journal done");
    assert!(acknowledge(&finished).is_err_and(|text| { return text.contains("already finished"); }), "a finished journal was acknowledged");
    assert!(!finished.join("acknowledged").exists());
}

/// Acceptance row "V2, amended c" (the journal view):
/// a journal seen with another mount id or inode than the one recorded when it was made
/// is refused, with the reason, and kept;
/// an absent recorded scope is not read as gone.
#[test]
fn a_journal_seen_through_another_view_is_refused_and_kept() {
    let Some(route) = delegated_route() else {
        return;
    };
    let root = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\nsettled {}\n", planned.record(), planned.directory().display()));
    assert_eq!(survivor_note(&journal), None, "the unchanged view should be accepted: the settled scope is absent and its parent unchanged");
    let recorded: View = view_of(&journal).expect("views");

    fs::write(journal.join("view"), format!("view {}:{}\n", recorded.mount + 1, recorded.inode)).expect("another mount");
    let other_mount: Option<String> = survivor_note(&journal);
    fs::write(journal.join("view"), format!("view {}:{}\n", recorded.mount, recorded.inode + 1)).expect("another inode");
    let other_inode: Option<String> = survivor_note(&journal);
    fs::remove_file(journal.join("view")).expect("lose the view record");
    let lost: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);

    for note in [other_mount, other_inode, lost] {
        let text: String = note.expect("a journal in another view blocks");
        assert!(text.contains("mount") && text.contains("cargo-shadow acknowledge"), "{text}");
    }
    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{messages:?}");
    assert!(journal.exists());
}

/// Acceptance row "V2, amended c" (the cgroup-parent view):
/// a scope record made where the directory that held the scope is seen with another mount id or inode
/// is refused, with the reason, even though the scope path is absent.
#[test]
fn a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone() {
    let Some(route) = delegated_route() else {
        return;
    };
    let root = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let Record::Scope { namespace, parent_view, path } = planned.record() else {
        panic!("a scope record");
    };
    let wrong: Record = Record::Scope { namespace, parent_view: View { mount: parent_view.mount + 1, inode: parent_view.inode }, path };
    let journal: PathBuf = journal_with_records(root.path(), &format!("{wrong}\nsettled {}\n", planned.directory().display()));

    let note: String = survivor_note(&journal).expect("a different parent view blocks");
    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(note.contains("recorded path may lead somewhere else"), "{note}");
    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{messages:?}");
    assert!(journal.exists());
}

/// The helper process of the mount-view test:
/// it runs in a new mount namespace and recovers the workspace it is given,
/// writing what the recovery said to a file.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal"]
fn recovery_in_another_mount_namespace_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    // The journal is given the view this namespace has of it (as if it had been made here), so that the journal view agrees and
    // only the view of the directory that held the scope differs.
    for journal in journal_dirs(&root) {
        let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
        let rest: String = recorded.lines().skip(1).map(|line| { return format!("{line}\n"); }).collect();
        fs::write(journal.join("view"), format!("view {}\n{rest}", view_of(&journal).expect("views"))).expect("rewrite the view");
    }
    let messages: Vec<String> = recover(&root, &refusing);
    fs::write(root.join("fixture-messages"), messages.join("\n")).expect("write the messages");
}

/// Acceptance row "V2, amended c", the same-cgroup-namespace schedule of 7a F6:
/// a recovery in another mount namespace (a new user and mount namespace, the cgroup namespace unchanged)
/// refuses, says why, and keeps the journal and the lockfile.
#[test]
fn a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal() {
    let Some(route) = delegated_route() else {
        return;
    };
    let root = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\nsettled {}\n", planned.record(), planned.directory().display()));
    assert_eq!(survivor_note(&journal), None, "in this namespace the settled, absent scope is gone");

    let status = Command::new("unshare")
        .args(["--user", "--map-root-user", "--mount", "--"])
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "containment_record_tests::recovery_in_another_mount_namespace_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .status()
        .expect("unshare must be installed and allowed to make a user and mount namespace for this test");

    assert!(status.success(), "the helper process failed: {status:?}");
    let said: String = fs::read_to_string(root.path().join("fixture-messages")).expect("the helper's messages");
    assert!(said.contains("recorded path may lead somewhere else"), "the refusal is not about the cgroup parent view: {said}");
    assert!(!said.contains("another mount or path"), "the journal view differed too, so the cgroup view was not isolated: {said}");
    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "the other view restored a file");
    assert!(journal.exists(), "the journal was not retained");
}

/// Acceptance row "V3, V4":
/// the record of a scope is on disk before the program starts,
/// seen at the boundary by a hook that runs inside the start,
/// and the program writes a marker only after that.
#[test]
fn the_scope_record_is_visible_before_the_program_starts() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let marker: PathBuf = root.path().join("program-ran");
    let at_start: Rc<RefCell<(String, bool)>> = Rc::new(RefCell::new((String::new(), true)));
    let observer: Rc<RefCell<(String, bool)>> = Rc::clone(&at_start);
    let (watched, watched_marker): (PathBuf, PathBuf) = (journal.join("groups"), marker.clone());
    on_start_for_test(Some(Rc::new(move || { *observer.borrow_mut() = (fs::read_to_string(&watched).unwrap_or_default(), watched_marker.exists()); })));

    let result = output_within(Command::new("sh").args(["-c", &format!(": > {}", marker.display())]), None, &bound());
    on_start_for_test(None);

    result.expect("runs");
    let (record, ran_before): (String, bool) = at_start.borrow().clone();
    assert!(matches!(parse_record(record.trim_end()), Ok(Record::Scope { .. })), "the scope record was not on disk at the start: {record:?}");
    assert!(!ran_before, "the program ran before the record");
    assert!(marker.exists());
    assert_scope_record(&journal.join("groups").display().to_string());
    drop(transaction);
}

/// Acceptance row "V3":
/// a record that cannot be published (a pipe where the record file was) stops the start at once,
/// without waiting for a reader, in both containments.
#[test]
fn a_record_that_cannot_be_published_starts_no_program_and_does_not_wait() {
    for weaker in [false, true] {
        let plain = scratch();
        if weaker {
            use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
        } else if delegated_route().is_none() {
            return;
        }
        let root = repository();
        let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        fs::remove_file(journal.join("groups")).expect("remove the record");
        let fifo = Command::new("mkfifo").arg(journal.join("groups")).status().expect("mkfifo");
        assert!(fifo.success());
        let marker: PathBuf = root.path().join("ran");

        let started = std::time::Instant::now();
        let result = output_within(Command::new("sh").args(["-c", &format!(": > {}", marker.display())]), None, &bound());
        use_route_settings_for_test(None);

        assert!(result.is_err(), "weaker={weaker}: the program ran with no record");
        assert!(!marker.exists(), "weaker={weaker}: the program started");
        assert!(started.elapsed() < Duration::from_secs(5), "weaker={weaker}: the start waited for a reader of the record");
        drop(transaction);
    }
}

/// The helper process of the start-window test:
/// it begins a transaction and starts a run whose start is held by a hook at the boundary after the record,
/// so that the test can kill it there.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_wrapper_killed_inside_its_start_is_recovered_conservatively"]
fn wrapper_held_inside_its_start_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    if std::env::var(FIXTURE_MODE).as_deref() == Ok("weaker") {
        use_route_settings_for_test(Some(RouteSettings { cgroup_root: root.join("not-a-cgroup"), ..RouteSettings::default() }));
    }
    let _transaction = begin(&root, &[snapshot("original")], false).expect("begin");
    let ready: PathBuf = root.join("ready");
    on_start_for_test(Some(Rc::new(move || {
        fs::write(&ready, "held").expect("announce the held start");
        std::thread::sleep(Duration::from_secs(300));
    })));
    let _ = output_within(&mut Command::new("true"), None, &Bound { limit: Duration::from_secs(600), ..bound() });
}

/// Acceptance row "V3, V4":
/// a wrapper killed at the boundary inside its start, after the record and before the program,
/// is recovered conservatively:
/// a scope that was never made is verifiably absent and the recovery goes on,
/// while a weaker-path start that only announced a child blocks until acknowledged.
#[test]
fn a_wrapper_killed_inside_its_start_is_recovered_conservatively() {
    for weaker in [false, true] {
        if !weaker && delegated_route().is_none() {
            return;
        }
        let root = repository();
        fs::create_dir_all(root.path().join("not-a-cgroup")).expect("a plain directory");
        let mut command = Command::new(std::env::current_exe().expect("this test executable"));
        command
            .args(["--ignored", "--exact", "containment_record_tests::wrapper_held_inside_its_start_fixture", "--nocapture"])
            .env(FIXTURE_ROOT, root.path());
        if weaker {
            command.env(FIXTURE_MODE, "weaker");
        }
        let mut wrapper: Child = command.spawn().expect("the wrapper helper");
        wait_for_file(&root.path().join("ready"));
        wrapper.kill().expect("kill the wrapper at the boundary");
        let _ = wrapper.wait();

        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
        let messages: Vec<String> = recover(root.path(), &refusing);

        if weaker {
            assert_eq!(record, "pgroup starting\n");
            assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "a weaker-path start that only announced a child was recovered: {messages:?}");
            assert!(journal.exists());
        } else {
            assert!(matches!(parse_record(record.trim_end()), Ok(Record::Scope { .. })), "{record:?}");
            // The record is on disk and the run's end is not: an absent scope may be one a launcher has not made yet, so the
            // recovery refuses (round 9, 8a F1) and says why.
            assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "an unsettled run with an absent scope was recovered: {messages:?}");
            assert!(journal.exists());
            let note: String = survivor_note(&journal).expect("an unsettled run blocks");
            assert!(note.contains("did not record its end"), "{note}");
        }
    }
}

/// Round 7 (6b2 2):
/// the directory a begin cut off after its creation leaves is empty,
/// no child can have started,
/// and a recovery removes it.
#[test]
fn a_recovery_removes_an_empty_journal_directory() {
    let root = repository();
    let cut_off = root.path().join(JOURNAL_DIR).join("cut-off");
    fs::create_dir_all(&cut_off).expect("an empty journal directory");

    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(!cut_off.exists(), "the empty directory stayed: {messages:?}");
}

/// Acceptance row "V3, V4":
/// a nonempty journal that lost its record is not read as empty.
#[test]
fn a_nonempty_journal_without_its_record_is_not_treated_as_empty() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "");
    fs::remove_file(journal.join("groups")).expect("lose the record");

    let note: Option<String> = survivor_note(&journal);

    assert!(note.is_some_and(|text| { return text.contains("missing"); }));
}

/// Advisor finding on the weaker path (the shape of 6b2 2), kept under the round 8 rule:
/// a wrapper killed after announcing a child and before recording its process group
/// leaves a journal that a recovery must read as a possible survivor.
#[test]
fn a_started_child_whose_process_group_was_never_recorded_is_a_possible_survivor() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");

    let note = survivor_note(&journal);

    assert!(note.is_some_and(|text| { return text.contains("never recorded"); }), "a recovery would restore files a child may still be writing");
}
