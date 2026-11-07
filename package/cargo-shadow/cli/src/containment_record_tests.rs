//! What:
//! Rounds 7 and 8 tests of what a journal records about containment and what a recovery and an admission do with it:
//! the weaker path and its acknowledgement,
//! the capability classes,
//! the views of amended assumption c,
//! the record before the start,
//! and the windows of a wrapper that is killed during a start.
//! Why:
//! The contract (`doc/planning/cargo-shadow-guarantees.md`) says what each of these promises;
//! the acceptance rows "V2,
//! amended c",
//! "V3,
//! V4" and "Weaker path" are shown here.
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

/// A judge that refuses the live lockfile,
/// so that a recovery which goes ahead restores the snapshot.
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

/// The settings of the host classes that lack a capability,
/// with the words the reason must hold.
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
        // The manager names the root group, which has no `cgroup.kill`: discovery tells it apart from the others.
        ("a manager group without cgroup.kill", RouteSettings { systemctl: "sh".to_string(), question: Some(vec!["-c".to_string(), "echo /".to_string()]), ..RouteSettings::default() }, "cgroup.kill"),
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
        let _ = crate::bounded_run::take_trace_for_test();
        let seen_at_start: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let observer: Rc<RefCell<String>> = Rc::clone(&seen_at_start);
        let watched: PathBuf = journal.join("groups");
        on_start_for_test(Some(Rc::new(move || { *observer.borrow_mut() = fs::read_to_string(&watched).unwrap_or_default(); })));

        let result = output_within(Command::new("sh").args(["-c", "echo ok"]), None, &bound());
        on_start_for_test(None);
        use_route_settings_for_test(None);

        let output = result.expect(class);
        // The notice the run itself gave (certification, second pass): one per run, with the class's reason and the promised wording.
        let given: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("notice "); }).collect();
        assert_eq!(given.len(), 1, "{class}: the run gave {} notices: {given:?}", given.len());
        assert_eq!(given[0], format!("notice {notice}"), "{class}: the run's notice is not the one for this class");
        assert!(given[0].contains(words) && given[0].contains("quiescence is not shown") && given[0].contains("cargo-shadow acknowledge"), "{class}: {}", given[0]);
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

/// The environment variable that tells the notice helper process which capability class to run under (an index into
/// `missing_capability_classes`).
const FIXTURE_CLASS: &str = "CARGO_SHADOW_NOTICE_FIXTURE_CLASS";

/// The environment variable that gives the notice helper process a directory that is not a cgroup mount.
const FIXTURE_PLAIN: &str = "CARGO_SHADOW_NOTICE_FIXTURE_PLAIN";

/// The helper process of the notice test:
/// it runs one bounded command under the settings of one capability class and exits.
/// A fresh process per class,
/// because the notice is given once per process.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of each_missing_capability_class_prints_the_notice_on_the_real_standard_error"]
fn weaker_notice_fixture() {
    let (Ok(class), Ok(plain)) = (std::env::var(FIXTURE_CLASS), std::env::var(FIXTURE_PLAIN)) else {
        return;
    };
    let index: usize = class.parse().expect("a class index");
    let (_name, settings, _words) = missing_capability_classes(Path::new(&plain)).swap_remove(index);
    use_route_settings_for_test(Some(settings));
    output_within(Command::new("sh").args(["-c", "echo ok"]), None, &bound()).expect("the command runs on the weaker path");
}

/// Acceptance row "Weaker path" (certification,
/// third pass):
/// the notice reaches the real standard error of the run,
/// once,
/// as one line
/// with the reason of the class,
/// for every missing-capability class including the manager group without `cgroup.kill`.
/// Each class runs in a fresh process whose standard error is captured;
/// the trace of the other test shows which notice a run selects,
/// this shows that it is printed (a test that removed the printing and kept the trace would pass that one).
#[test]
fn each_missing_capability_class_prints_the_notice_on_the_real_standard_error() {
    let plain = scratch();
    let classes = missing_capability_classes(plain.path());
    for (index, (class, settings, words)) in classes.into_iter().enumerate() {
        let unavailable = detect_route(&settings).expect_err(class);
        let expected: String = weaker_notice(&unavailable);
        let output = Command::new(std::env::current_exe().expect("this test executable"))
            .args(["--ignored", "--exact", "containment_record_tests::weaker_notice_fixture", "--nocapture"])
            .env(FIXTURE_CLASS, index.to_string())
            .env(FIXTURE_PLAIN, plain.path())
            .output()
            .expect("the notice helper");

        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(output.status.success(), "{class}: the helper failed: {stderr}");
        let printed: Vec<&str> = stderr.lines().filter(|line| { return line.starts_with("cargo-shadow: no delegated control group"); }).collect();
        assert_eq!(printed.len(), 1, "{class}: the notice was not printed exactly once on standard error: {stderr}");
        assert_eq!(printed[0], expected, "{class}: the printed notice is not the one for this class");
        assert!(printed[0].contains(words) && printed[0].contains("quiescence is not shown") && printed[0].contains("cargo-shadow acknowledge"), "{class}: {}", printed[0]);
    }
}

/// Acceptance row "Weaker path" (certification,
/// third pass):
/// an acknowledgement that cannot be read is not an acknowledgement.
/// The marker is a directory,
/// so opening it succeeds and reading it fails on every system and for every user;
/// the real recovery and the
/// real admission both stay blocked,
/// the journal stays and nothing is restored;
/// a real acknowledgement in its place (the control)
/// lets the recovery go on.
#[test]
fn an_unreadable_acknowledgement_does_not_count_for_the_recovery_or_the_admission() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");
    fs::create_dir(journal.join("acknowledged")).expect("a marker that cannot be read as a file");

    let messages: Vec<String> = recover(root.path(), &refusing);
    let admitted = begin(root.path(), &[snapshot("original")], false);

    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "a recovery went ahead on an acknowledgement it could not read: {messages:?}");
    assert!(journal.exists(), "the journal was removed on an unreadable acknowledgement: {messages:?}");
    assert!(admitted.is_err(), "a transaction was admitted beside a journal whose acknowledgement cannot be read");
    assert!(survivor_note(&journal).is_some());
    assert_eq!(journal_dirs(root.path()), vec![journal.clone()]);

    // Control: a real acknowledgement in its place lets the same recovery go on.
    fs::remove_dir(journal.join("acknowledged")).expect("take the unreadable marker away");
    acknowledge(&journal).expect("acknowledges");
    let after: Vec<String> = recover(root.path(), &refusing);
    assert!(journal_dirs(root.path()).is_empty(), "the control: the acknowledged journal was not recovered: {after:?}");
}

/// The weaker path stops the process group at the bound,
/// as round 5 did.
#[test]
fn the_weaker_path_still_stops_the_process_group_at_the_bound() {
    let plain = scratch();
    use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
    let root = repository();
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let marker = plain.path().join("late");
    let ready = plain.path().join("writer-ready");
    // A writer of the process group is running and ready (a handshake) before the bound passes; it would write after the stop.
    // The bound counts the start of the run, so an attempt whose bound ended before the writer was ready is repeated with a doubled
    // bound, and the writer's own delay follows the bound (a second after it) so that it still writes only after the stop.
    let writer_file = plain.path().join("writer-pid");
    let started = std::time::Instant::now();
    let (result, limit, judged) = crate::cgroup_test_support::run_until_the_payload_started(
        Duration::from_millis(1500),
        &|| { let _ = fs::remove_file(&ready); let _ = fs::remove_file(&marker); },
        &|| { return ready.exists(); },
        &mut |limit| {
            let delay: f64 = limit.as_secs_f64() + 0.5;
            let script: String = format!("(: > {ready}; sleep {delay}; : > {late}) & echo $! > {pid}; while [ ! -e {ready} ]; do sleep 0.02; done; sleep 600", ready = ready.display(), late = marker.display(), pid = writer_file.display());
            return output_within(Command::new("sh").args(["-c", &script]), None, &Bound { limit, ..bound() });
        },
    );
    use_route_settings_for_test(None);
    let ready_before: bool = ready.exists();
    // The run ended at its bound and not long after it (the stop and the reaping have their own small allowances), and the writer is
    // gone the moment it returned (fourth certification pass).
    let elapsed: Duration = started.elapsed();
    let writer: i32 = crate::contain_counterexample_tests::read_pid(&writer_file);
    let alive_at_return: bool = crate::contain_counterexample_tests::is_running(writer);
    std::thread::sleep(limit + Duration::from_millis(1_200));
    assert!(!alive_at_return, "the writer of the process group still ran when the run returned");
    assert!(elapsed < limit * 8 + crate::bounded_run::QUIESCE_BOUND * 2, "the run returned after {elapsed:?}, long after its bound of {limit:?}");
    assert!(judged >= limit && judged < limit + crate::bounded_run::QUIESCE_BOUND * 2, "the judged run ended after {judged:?}, not at its bound of {limit:?}");

    // The result is the timeout (certification group 3), the writer started, and the run was the weaker path.
    assert!(matches!(result, Err(crate::bounded_run::RunError::TimedOut { .. })), "the run should have passed its bound: {result:?}");
    assert!(ready_before, "the writer never ran, so a failure before the spawn would also leave no late marker");
    assert!(fs::read_to_string(journal.join("groups")).expect("the record").starts_with("pgroup starting\npgroup "), "the run was not the weaker path");
    assert!(!marker.exists(), "a member of the process group wrote after the stop");
    drop(transaction);
}

/// Acceptance row "Weaker path" (certification,
/// third pass):
/// an acknowledgement overrides neither a failed enumeration of the journals
/// nor a failed mandatory step of finishing the recovery.
/// Two acknowledged journals (the second makes the listing fail at its entry):
/// the recovery changes nothing and keeps both;
/// one acknowledged journal whose completion marker cannot be made durable is a
/// mandatory failure,
/// with the journal retained.
#[test]
fn an_acknowledged_journal_is_still_held_by_a_failed_listing_and_a_failed_mandatory_finish() {
    use crate::failpoints::{Point, arm, fires};
    // A failed enumeration.
    let root = repository();
    let first: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
    acknowledge(&first).expect("acknowledges");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    let second: PathBuf = {
        // Planting begins a transaction, which an unacknowledged weaker-path journal would refuse; the first is acknowledged.
        let planted: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
        fs::write(planted.join("groups"), "pgroup starting\npgroup 2147483646\n").expect("records");
        acknowledge(&planted).expect("acknowledges");
        planted
    };
    arm(Point::ListingIteration);
    let messages: Vec<String> = recover(root.path(), &refusing);
    assert!(!fires(Point::ListingIteration), "the listing fault was never reached");
    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "an acknowledged recovery went ahead on a journal set it could not list: {messages:?}");
    assert!(messages.iter().any(|message| { return message.contains("could not be listed"); }), "{messages:?}");
    assert!(first.exists() && second.exists(), "an acknowledged journal was removed on a failed listing");

    // A failed mandatory finish.
    let other = repository();
    let acknowledged: PathBuf = journal_with_records(other.path(), "pgroup starting\npgroup 2147483646\n");
    acknowledge(&acknowledged).expect("acknowledges");
    arm(Point::CompletionSync);
    let report = crate::journal::recover_report(other.path(), &refusing);
    assert!(!fires(Point::CompletionSync), "the completion fault was never reached");
    assert!(report.mandatory_failure, "an acknowledged recovery whose marker could not be made durable was not a mandatory failure: {:?}", report.messages);
    assert_eq!(journal_dirs(other.path()).len(), 1, "the journal was not retained");
}

/// Acceptance row "Weaker path" (certification,
/// third pass):
/// the process group is stopped at the normal exit as well as at the bound.
/// A same-group writer announces itself (a handshake),
/// the direct payload exits successfully with the writer still running,
/// and the
/// writer would write into the workspace later;
/// after the run returns the marker must never appear and the writer must be gone.
#[test]
fn the_weaker_path_stops_a_writer_of_the_process_group_at_a_normal_exit() {
    let plain = scratch();
    use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
    let marker = plain.path().join("late");
    let ready = plain.path().join("writer-ready");
    let pid_file = plain.path().join("writer-pid");
    // The writer closes its inherited pipes (so the runner's drain does not wait for it) and keeps running after the payload exits.
    let script: String = format!(
        "(: > {ready}; sleep 2; : > {late}) >/dev/null 2>&1 </dev/null & echo $! > {pid}; while [ ! -e {ready} ]; do sleep 0.02; done; exit 0",
        pid = pid_file.display(),
        ready = ready.display(),
        late = marker.display()
    );

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound());
    use_route_settings_for_test(None);
    let ready_before: bool = ready.exists();
    // The writer is judged the moment the run returned (fourth certification pass), through the fail-closed oracle: a runner that
    // returned with the writer alive and let a later cleanup kill it before its write would leave no marker and pass the check below.
    let writer: i32 = crate::contain_counterexample_tests::read_pid(&pid_file);
    let alive_at_return: bool = crate::contain_counterexample_tests::is_running(writer);
    std::thread::sleep(Duration::from_millis(2_700));

    assert!(!alive_at_return, "the writer of the process group still ran when the run returned: the exit cleanup of the group was not done by then");
    assert!(result.is_ok_and(|output| { return output.status.success(); }), "the payload should have exited successfully");
    assert!(ready_before, "the writer never ran, so its absence proves nothing");
    assert!(!marker.exists(), "a member of the process group wrote after the run returned: the exit cleanup of the group is missing");
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
/// the command works under the workspace lock,
/// durably acknowledges exactly one journal,
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

    // The holder is still held when the answer must come, and it is released only afterwards (certification group 1).
    let target: PathBuf = journal.clone();
    let refused = crate::round_sixteen_tests::answers_within("the acknowledgement", Duration::from_secs(3), move || { return acknowledge(&target); });

    assert!(refused.as_ref().is_err_and(|text| { return text.contains("another cargo-shadow run") || text.contains("nothing was acknowledged"); }), "{refused:?}");
    assert!(!journal.join("acknowledged").exists());
    drop(_held);
    acknowledge(&journal).expect("the control: with the lock free the same acknowledgement works");
    assert!(journal.join("acknowledged").exists());
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
    // The real recovery and admission hold the acknowledged journal too while the group lives (fourth certification pass); the group is
    // stopped only afterwards.
    let held: Vec<String> = recover(root.path(), &refusing);
    let admitted = begin(root.path(), &[snapshot("second")], false);
    let held_lock: String = live_lock(root.path());
    let kept: bool = live.exists();
    let _ = orphan.kill();
    let _ = orphan.wait();

    assert!(note.is_some_and(|text| { return text.contains("still running"); }), "a live recorded group was overridden");
    assert!(held.iter().any(|message| { return message.contains("still running"); }), "the recovery did not name the live group: {held:?}");
    assert_eq!(held_lock, LEFT_BY_THE_KILLED_RUN, "an acknowledged recovery went ahead beside a live group: {held:?}");
    assert!(kept, "the acknowledged journal was removed while its group lives");
    assert!(admitted.is_err(), "a transaction was admitted beside an acknowledged live group");

    let other = repository();
    let invalid: PathBuf = journal_with_records(other.path(), "pgroup starting\n");
    acknowledge(&invalid).expect("acknowledges");
    fs::remove_file(invalid.join("0.lock")).expect("lose a snapshot");
    let messages: Vec<String> = recover(other.path(), &refusing);
    assert_eq!(live_lock(other.path()), LEFT_BY_THE_KILLED_RUN, "an acknowledged journal with a missing snapshot was restored: {messages:?}");
    assert!(invalid.exists(), "the journal with a missing snapshot was removed");
}

/// Acceptance row "Weaker path":
/// the command refuses a path that is not a journal of a workspace,
/// and a finished journal.
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

/// Acceptance row "V2,
/// amended c" (the journal view):
/// a journal seen with another mount id or inode than the one recorded when it was made
/// is refused,
/// with the reason,
/// and kept;
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

    // Each changed view (another mount, another inode, no view record) is given to the real recovery: it must refuse, name the view,
    // leave the live output as it was and keep the journal (certification group 1: a classification alone is not a refusal).
    let original_view: String = format!("view {}:{}\n", recorded.mount, recorded.inode);
    let cases: Vec<(&str, Option<String>)> = vec![
        ("another mount", Some(format!("view {}:{}\n", recorded.mount + 1, recorded.inode))),
        ("another inode", Some(format!("view {}:{}\n", recorded.mount, recorded.inode + 1))),
        ("no view record", None),
    ];
    for (label, view) in cases {
        match &view {
            Some(text) => fs::write(journal.join("view"), text).expect("write the changed view"),
            None => fs::remove_file(journal.join("view")).expect("lose the view record"),
        }
        let note: Option<String> = survivor_note(&journal);
        let messages: Vec<String> = recover(root.path(), &refusing);

        let text: String = note.unwrap_or_else(|| { panic!("{label}: a journal in another view did not block"); });
        assert!(text.contains("mount") && text.contains("cargo-shadow acknowledge"), "{label}: {text}");
        assert!(messages.iter().any(|message| { return message.contains("mount") && message.contains("cargo-shadow acknowledge"); }), "{label}: the recovery did not refuse for the view: {messages:?}");
        assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{label}: the live output changed: {messages:?}");
        assert!(journal.exists(), "{label}: the journal was not retained");
    }
    // The control: with the recorded view back the same recovery settles the journal, so the refusals above came from the view.
    fs::write(journal.join("view"), original_view).expect("put the recorded view back");
    let settled: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the control: the journal in its recorded view was not settled: {settled:?}");
}

/// Acceptance row "V2,
/// amended c" (the cgroup-parent view):
/// a scope record made where the directory that held the scope is seen with another mount id or inode
/// is refused,
/// with the reason,
/// even though the scope path is absent.
#[test]
fn a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone() {
    let Some(route) = delegated_route() else {
        return;
    };
    // The mount id and the inode of the parent are each changed on their own (fifth certification pass): a check of only one of them
    // is red for the other; the unchanged view is the control, which is read as ended.
    for (label, mount_step, inode_step) in [("an unchanged view", 0_u64, 0_u64), ("another mount id", 1, 0), ("another inode", 0, 1)] {
        let root = repository();
        let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
        let Record::Scope { namespace, parent_view, path } = planned.record() else {
            panic!("a scope record");
        };
        let seen: Record = Record::Scope { namespace, parent_view: View { mount: parent_view.mount + mount_step, inode: parent_view.inode + inode_step }, path };
        let journal: PathBuf = journal_with_records(root.path(), &format!("{seen}\nsettled {}\n", planned.directory().display()));

        let note: Option<String> = survivor_note(&journal);
        let messages: Vec<String> = recover(root.path(), &refusing);

        if mount_step == 0 && inode_step == 0 {
            assert_eq!(note, None, "{label}: the control did not read the settled, absent scope as ended: {messages:?}");
            assert!(!journal.exists(), "{label}: the control journal was not recovered: {messages:?}");
        } else {
            let text: String = note.unwrap_or_else(|| { panic!("{label}: a different parent view did not block"); });
            assert!(text.contains("recorded path may lead somewhere else"), "{label}: {text}");
            assert!(messages.iter().any(|message| { return message.contains("recorded path may lead somewhere else"); }), "{label}: the recovery did not give the reason: {messages:?}");
            assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{label}: {messages:?}");
            assert!(journal.exists(), "{label}: the journal was removed");
            assert!(begin(root.path(), &[snapshot("second")], false).is_err(), "{label}: a transaction was admitted beside a journal whose parent view differs");
        }
    }
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

/// Acceptance row "V2,
/// amended c",
/// the same-cgroup-namespace schedule of 7a F6:
/// a recovery in another mount namespace (a new user and mount namespace,
/// the cgroup namespace unchanged)
/// refuses,
/// says why,
/// and keeps the journal and the lockfile.
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

/// Acceptance row "V3,
/// V4":
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
/// without waiting for a reader,
/// in both containments.
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

/// Acceptance row "V3,
/// V4":
/// a wrapper killed at the boundary inside its start,
/// after the record and before the program,
/// is recovered conservatively:
/// a scope that was never made is absent but unsettled,
/// so the recovery refuses and names the missing end record,
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

/// Acceptance row "V3,
/// V4":
/// a nonempty journal that lost its record is not read as empty.
#[test]
fn a_nonempty_journal_without_its_record_is_not_treated_as_empty() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "");
    fs::remove_file(journal.join("groups")).expect("lose the record");

    let note: Option<String> = survivor_note(&journal);

    assert!(note.is_some_and(|text| { return text.contains("missing"); }));
}

/// Advisor finding on the weaker path (the shape of 6b2 2),
/// kept under the round 8 rule:
/// a wrapper killed after announcing a child and before recording its process group
/// leaves a journal that a recovery must read as a possible survivor.
#[test]
fn a_started_child_whose_process_group_was_never_recorded_is_a_possible_survivor() {
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\n");

    let note = survivor_note(&journal);

    assert!(note.is_some_and(|text| { return text.contains("never recorded"); }), "a recovery would restore files a child may still be writing");
}

/// The environment variable that tells the notice-concurrency helper process to run.
const FIXTURE_NOTICE_RACE: &str = "CARGO_SHADOW_NOTICE_RACE_FIXTURE";

/// The helper process of the notice-concurrency test (a fresh process,
/// because the notice is given once per process):
/// the first
/// weaker-path run wins the notice and is held inside its emission;
/// a second weaker-path run on another thread must complete while
/// the winner is still held.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_run_does_not_wait_for_another_runs_slow_notice"]
fn notice_concurrency_fixture() {
    use std::sync::atomic::Ordering;
    if std::env::var(FIXTURE_NOTICE_RACE).is_err() {
        return;
    }
    let plain = scratch();
    let settings = RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() };
    let gate = crate::bounded_run::NOTICE_GATE.lock().expect("the gate");
    let first_settings = settings.clone();
    let first = std::thread::spawn(move || {
        use_route_settings_for_test(Some(first_settings));
        return output_within(Command::new("sh").args(["-c", "echo first"]), None, &bound());
    });
    crate::cgroup_test_support::wait_until("the first run to be inside its notice", &|| { return crate::bounded_run::NOTICE_ENTERED.load(Ordering::SeqCst); });
    let second = crate::round_sixteen_tests::answers_within("the second weaker-path run", Duration::from_secs(20), move || {
        use_route_settings_for_test(Some(settings));
        return output_within(Command::new("sh").args(["-c", "echo second"]), None, &bound());
    });
    assert!(second.is_ok_and(|output| { return output.status.success(); }), "the second run did not complete");
    drop(gate);
    assert!(first.join().expect("first thread").is_ok_and(|output| { return output.status.success(); }));
}

/// Acceptance rows V6 and "Weaker path" (fifth certification pass):
/// the weaker-path notice,
/// given once per process,
/// does not make
/// another run wait for the winner's slow write.
/// A helper process holds the winner inside the emission and requires the second run to
/// answer;
/// the notice is on standard error exactly once.
#[test]
fn a_run_does_not_wait_for_another_runs_slow_notice() {
    let output = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "containment_record_tests::notice_concurrency_fixture", "--nocapture"])
        .env(FIXTURE_NOTICE_RACE, "1")
        .output()
        .expect("the helper");

    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "the helper failed: {stderr}");
    let notices: usize = stderr.lines().filter(|line| { return line.starts_with("cargo-shadow: no delegated control group"); }).count();
    assert_eq!(notices, 1, "the notice was not given exactly once: {stderr}");
}
