//! What:
//! Round 8 tests of the failures on the containment path after a program started (7a F3),
//! of the isolation of a start from other work of the process (7a F1,
//! F2),
//! and of the bounds on the waits the wrapper controls (V6).
//! Why:
//! Contract acceptance rows "A2,
//! R4 (7a F3)",
//! "A1,
//! V3 (spawn isolation)" and "V6".
//! Every test here uses a delegated scope and shows it (`assert_scope_record` or the program's own group).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('quiescence failures', () => { it('a failure to stop the scope is NotQuiescent', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::bounded_run::{Bound, RunError, on_start_for_test, on_startup_for_test, output_within};
use crate::cgroup_test_support::{assert_ran_in_recorded_scope, assert_scope_record, delegated_route, scope_directory_of, wait_until};
use crate::control_group::{RouteSettings, detect_route};
use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::begin;
use crate::journal_support::journal_dirs;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/q/one";

/// A snapshot of the lockfile of the case.
fn snapshot() -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() };
}

/// A disposable repository whose crate root holds a lockfile.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "original").expect("lock");
    return root;
}

/// A bound for the commands of these tests.
fn bound(limit: Duration) -> Bound {
    return Bound { limit, what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// The script of a program that starts a writer in a session of its own,
/// waits until the writer says it is running,
/// and exits.
/// The groups file of the single journal of a repository,
/// as text.
fn control_root_groups(root: &Path) -> String {
    return journal_dirs(root).into_iter().next().expect("a journal").join("groups").display().to_string();
}

fn script_leaving_a_writer(ready: &Path) -> String {
    return format!(
        // The handshake is inside the payload (sixth certification pass): the writer's ready marker is seen first, then the payload
        // records its own control group and that it saw the marker, and only then ends, so the stop (and a fault in it) comes after.
        "setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; cat /proc/self/cgroup > {ready}.group; : > {ready}.seen; exit 0",
        ready = ready.display()
    );
}

/// Stops a scope a test left behind,
/// through its recorded path,
/// and waits until it is empty or gone.
fn clean_up(scope: &Path) {
    let _ = fs::write(scope.join("cgroup.kill"), "1");
    wait_until("the scope's end", &|| { return !scope.exists() || fs::read_to_string(scope.join("cgroup.events")).is_ok_and(|text| { return text.contains("populated 0"); }); });
}

/// Acceptance row "A2,
/// R4 (7a F3)" (certification group 2):
/// the program and a writer started,
/// and then the stop of the scope fails (an error),
/// the inspection of its members fails,
/// or a member cannot be stopped within the bound:
/// each is `NotQuiescent`,
/// the scope reports that writers may remain,
/// the journal is retained,
/// and nothing is downgraded to an
/// ordinary I/O failure.
/// Each run has a control without the fault (it returns normally and stops the writer),
/// so the injected failure is the cause of the
/// refusal and not some other conservative outcome;
/// the fault must have been armed and then consumed,
/// and the writer's ready marker
/// must exist (the writer was running before the fault).
#[test]
fn a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent() {
    if delegated_route().is_none() {
        return;
    }
    for fault in [Point::ContainmentStopError, Point::ContainmentInspection, Point::ContainmentStopStuck] {
        // The control: no fault armed, the same program and writer.
        let control_root = repository();
        let control_transaction = begin(control_root.path(), &[snapshot()], false).expect("begin");
        let control_ready: PathBuf = control_root.path().join("writer-ready");
        let control = output_within(Command::new("sh").args(["-c", &script_leaving_a_writer(&control_ready)]), None, &bound(Duration::from_secs(30)));
        assert!(control.is_ok(), "{fault:?}: the control without the fault was refused, so the refusal below has another cause: {control:?}");
        // The control's writer is gone when the run returned: its scope is absent or empty.
        let control_scope: PathBuf = scope_directory_of(&control_root_groups(control_root.path()));
        assert!(!control_scope.exists() || fs::read_to_string(control_scope.join("cgroup.procs")).is_ok_and(|text| { return text.trim().is_empty(); }), "{fault:?}: the control's writer still runs after the run returned");
        assert!(control_ready.exists() && !control_transaction.writers_may_remain(), "{fault:?}: the control did not run its writer or reported writers that may remain");
        drop(control_transaction);

        let root = repository();
        let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        let ready: PathBuf = root.path().join("writer-ready");
        arm(fault);

        let result = output_within(Command::new("sh").args(["-c", &script_leaving_a_writer(&ready)]), None, &bound(Duration::from_secs(30)));

        let consumed: bool = !crate::failpoints::fires(fault);
        let groups: String = journal.join("groups").display().to_string();
        assert_scope_record(&groups);
        let scope: PathBuf = scope_directory_of(&groups);
        let writer_alive: bool = fs::read_to_string(scope.join("cgroup.procs")).is_ok_and(|text| { return !text.trim().is_empty(); });
        clean_up(&scope);
        assert!(consumed, "{fault:?}: the armed fault was never reached, so the refusal has another cause");
        assert!(ready.exists(), "{fault:?}: the writer was not ready before the fault");
        // The payload saw the writer ready before it ended (and so before the stop that the fault is in), and it ran in the very scope
        // the journal recorded, so the writer that is left is a member of the scope under test.
        assert!(root.path().join("writer-ready.seen").exists(), "{fault:?}: the payload ended without having seen the writer ready");
        assert_ran_in_recorded_scope(&groups, &fs::read_to_string(root.path().join("writer-ready.group")).expect("the payload's group"), &crate::cgroup_test_support::wrapper_group());
        assert!(matches!(result, Err(RunError::NotQuiescent { .. })), "{fault:?}: {result:?}");
        if !matches!(fault, Point::ContainmentInspection) {
            assert!(writer_alive, "{fault:?}: the writer was not left in the scope, so the test shows nothing");
        }
        assert!(transaction.writers_may_remain(), "{fault:?}: the scope was not told that writers may remain");
        assert!(journal.exists() && !journal.to_string_lossy().ends_with(".done"), "{fault:?}: the journal was not retained");
        drop(transaction);
    }
}

/// Acceptance row "A2,
/// R4 (7a F3)" (certification group 2,
/// the startup completion):
/// the program has started and a writer is running and ready,
/// and then the completion of the startup fails.
/// When the scope can then not be shown empty the failure is `NotQuiescent`,
/// the scope reports that writers may remain,
/// and the
/// journal is retained,
/// the same as for a failed stop;
/// when it can,
/// the writer is stopped,
/// quiescence is shown and the error is the
/// startup failure itself (the control,
/// which shows that the refusal comes from the failed stop and not from the startup failure alone).
#[test]
fn a_failed_startup_completion_with_a_writer_running_is_not_quiescent_when_the_scope_cannot_be_emptied() {
    if delegated_route().is_none() {
        return;
    }
    for stuck in [true, false] {
        let root = repository();
        let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        let ready: PathBuf = root.path().join("writer-ready");
        let hook_ready: PathBuf = ready.clone();
        on_startup_for_test(Some(Rc::new(move || {
            // The child and the writer are running and ready before the startup is made to fail.
            wait_until("the writer to be ready", &|| { return hook_ready.exists(); });
            if stuck {
                arm(Point::ContainmentStopStuck);
            }
            return true;
        })));
        let script: String = format!("setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & exec sleep 600", ready = ready.display());

        let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(30)));
        on_startup_for_test(None);

        let groups: String = journal.join("groups").display().to_string();
        assert_scope_record(&groups);
        let scope: PathBuf = scope_directory_of(&groups);
        let writer_alive: bool = fs::read_to_string(scope.join("cgroup.procs")).is_ok_and(|text| { return !text.trim().is_empty(); });
        let consumed: bool = !crate::failpoints::fires(Point::ContainmentStopStuck);
        let record: String = fs::read_to_string(&groups).expect("the record");
        clean_up(&scope);
        assert!(ready.exists(), "stuck={stuck}: the writer was not ready before the startup failed");
        if stuck {
            assert!(consumed, "the stuck stop was never reached");
            assert!(matches!(result, Err(RunError::NotQuiescent { .. })), "a failed startup completion with a scope that cannot be emptied was downgraded: {result:?}");
            assert!(writer_alive, "the writer was not left in the scope, so the test shows nothing");
            assert!(transaction.writers_may_remain(), "the scope was not told that writers may remain");
            assert!(journal.exists() && !record.lines().any(|line| { return line.starts_with("settled "); }), "the journal was settled or lost: {record:?}");
        } else {
            let message: String = result.expect_err("the startup failure is the error").to_string();
            assert!(message.contains("injected failure to complete the startup"), "{message}");
            assert!(!transaction.writers_may_remain(), "quiescence was shown, yet the scope reports writers that may remain");
            assert!(!writer_alive, "the writer was left running although the stop succeeded");
            assert!(record.lines().any(|line| { return line.starts_with("settled "); }), "the run's end was not recorded: {record:?}");
        }
        drop(transaction);
    }
}

/// Acceptance row "A1,
/// V3 (spawn isolation)":
/// a bounded run is held inside its start,
/// after its record and before its program,
/// while an unrelated ordinary spawn and a second bounded run both happen;
/// then the first run goes on.
/// The wrapper stays in its original group throughout,
/// the two runs have separate scopes,
/// and the unrelated child is untouched by either cleanup.
#[test]
fn two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary() {
    if delegated_route().is_none() {
        return;
    }
    let original: String = fs::read_to_string("/proc/self/cgroup").expect("own group");
    let (at_start_sender, at_start) = mpsc::channel::<()>();
    let (release_sender, release) = mpsc::channel::<()>();
    let first = std::thread::spawn(move || {
        on_start_for_test(Some(Rc::new(move || {
            at_start_sender.send(()).expect("announce the held start");
            release.recv_timeout(Duration::from_secs(60)).expect("released");
        })));
        let _ = crate::bounded_run::take_trace_for_test();
        let output = output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(30)));
        return (output, crate::bounded_run::take_trace_for_test());
    });
    at_start.recv_timeout(Duration::from_secs(60)).expect("the first run reached its start");

    let mut unrelated = Command::new("sleep").arg("600").spawn().expect("an unrelated child");
    let group_during: String = fs::read_to_string("/proc/self/cgroup").expect("own group");
    let _ = crate::bounded_run::take_trace_for_test();
    let second = output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(30))).expect("the second run completes while the first is inside its start");
    let second_planned: String = crate::round_sixteen_tests::launched_scope(&crate::bounded_run::take_trace_for_test());
    release_sender.send(()).expect("release the first run");
    let (first_result, first_trace) = first.join().expect("first run thread");
    let first_output = first_result.expect("the first run completes");
    let first_planned: String = crate::round_sixteen_tests::launched_scope(&first_trace);

    let unrelated_alive: bool = unrelated.try_wait().expect("asks").is_none();
    let unrelated_group: String = fs::read_to_string(format!("/proc/{}/cgroup", unrelated.id())).unwrap_or_default();
    let _ = unrelated.kill();
    let _ = unrelated.wait();
    let first_group: String = String::from_utf8_lossy(&first_output.stdout).trim().to_string();
    let second_group: String = String::from_utf8_lossy(&second.stdout).trim().to_string();
    assert_eq!(group_during, original, "the wrapper changed its group during a start");
    assert_eq!(fs::read_to_string("/proc/self/cgroup").expect("own group"), original);
    // Compared as control group paths (the file's `0::` prefix removed from every one), each against the scope its own run launched.
    let (first_path, second_path, own_path) = (crate::round_sixteen_tests::reported_path(&first_group), crate::round_sixteen_tests::reported_path(&second_group), crate::round_sixteen_tests::reported_path(&original));
    assert!(first_path.ends_with(&first_planned) && second_path.ends_with(&second_planned), "{first_path} / {second_path} against {first_planned} / {second_planned}");
    assert_ne!(first_path, second_path, "the two runs shared a scope");
    assert!(first_path != own_path && second_path != own_path, "a program ran in the wrapper's own group");
    assert!(unrelated_alive, "an unrelated child was stopped by a cleanup");
    assert_eq!(unrelated_group, original, "the unrelated child was moved into a run's scope");
}

/// Acceptance row "V6":
/// a manager that does not answer ends the discovery at its deadline with the reason as a missing capability;
/// nothing waits for the manager without end.
/// This test calls only the discovery;
/// that a run then takes the weaker path with its record is shown by
/// `each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start`.
#[test]
fn a_manager_that_does_not_answer_ends_the_discovery_at_its_deadline() {
    let settings: RouteSettings = RouteSettings {
        systemctl: "sleep".to_string(),
        question: Some(vec!["60".to_string()]),
        discovery_bound: Duration::from_millis(300),
        ..RouteSettings::default()
    };
    let started = Instant::now();

    let refused = detect_route(&settings).expect_err("a silent manager is a missing capability");

    assert!(refused.to_string().contains("did not answer"), "{refused}");
    assert!(started.elapsed() < Duration::from_secs(10), "the discovery waited {:?}", started.elapsed());
}

/// Contract boundary test (STB) for the launcher:
/// arguments with the characters `systemd-run` reads as a grammar of its own
/// (`$$`,
/// `${NAME}`,
/// `%`,
/// backslashes,
/// quotes,
/// spaces,
/// a line feed,
/// a leading dash,
/// a semicolon,
/// an empty argument,
/// non-UTF-8 bytes)
/// reach the program exactly as given,
/// also in a command with a cleared environment.
#[test]
fn arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged() {
    use std::os::unix::ffi::OsStrExt;
    if delegated_route().is_none() {
        return;
    }
    let arguments: Vec<&[u8]> = vec![b"a$$b", b"${HOME}", b"$HOME", b"$", b"%h %% %n", b"back\\slash", b"it's \"quoted\"", b"two words", b"line\nfeed", b"--looks-like-an-option", b"-", b"semi;colon", b"", b"\xff\xfe not utf-8", b"tab\there"];
    for clean in [false, true] {
        let mut command: Command = Command::new("printf");
        command.arg("%s\\0");
        for argument in &arguments {
            command.arg(std::ffi::OsStr::from_bytes(argument));
        }
        command.env("CARGO_SHADOW_PROBE_VARIABLE", "v$$w${HOME}");

        let output = if clean {
            crate::bounded_run::output_within_clean_environment(&mut command, None, &bound(Duration::from_secs(30)))
        } else {
            output_within(&mut command, None, &bound(Duration::from_secs(30)))
        }
        .expect("runs");

        let received: Vec<&[u8]> = output.stdout.split(|byte| { return *byte == 0; }).collect();
        let expected_with_end: Vec<&[u8]> = arguments.iter().copied().chain(std::iter::once(&b""[..])).collect();
        assert_eq!(received, expected_with_end, "clean={clean}: an argument was changed on the way");
    }
}
