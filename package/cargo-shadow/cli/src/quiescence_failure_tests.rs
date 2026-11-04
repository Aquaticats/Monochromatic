//! What:
//! Round 8 tests of the failures on the containment path after a program started (7a F3),
//! of the isolation of a start from other work of the process (7a F1, F2),
//! and of the bounds on the waits the wrapper controls (V6).
//! Why:
//! Contract acceptance rows "A2, R4 (7a F3)", "A1, V3 (spawn isolation)" and "V6".
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

use crate::bounded_run::{Bound, RunError, on_start_for_test, output_within};
use crate::cgroup_test_support::{assert_scope_record, delegated_route, scope_directory_of, wait_until};
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

/// The script of a program that starts a writer in a session of its own, waits until the writer says it is running, and exits.
fn script_leaving_a_writer(ready: &Path) -> String {
    return format!(
        "setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; exit 0",
        ready = ready.display()
    );
}

/// Stops a scope a test left behind, through its recorded path, and waits until it is empty or gone.
fn clean_up(scope: &Path) {
    let _ = fs::write(scope.join("cgroup.kill"), "1");
    wait_until("the scope's end", &|| { return !scope.exists() || fs::read_to_string(scope.join("cgroup.events")).is_ok_and(|text| { return text.contains("populated 0"); }); });
}

/// Acceptance row "A2, R4 (7a F3)":
/// the program and a writer started, and then the stop of the scope fails (an error),
/// the inspection of its members fails,
/// or a member cannot be stopped within the bound:
/// each is `NotQuiescent`, the scope reports that writers may remain, the journal is retained, and nothing is downgraded to an
/// ordinary I/O failure.
#[test]
fn a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent() {
    if delegated_route().is_none() {
        return;
    }
    for fault in [Point::ContainmentStopError, Point::ContainmentInspection, Point::ContainmentStopStuck] {
        let root = repository();
        let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
        let journal: PathBuf = journal_dirs(root.path()).into_iter().next().expect("a journal");
        let ready: PathBuf = root.path().join("writer-ready");
        arm(fault);

        let result = output_within(Command::new("sh").args(["-c", &script_leaving_a_writer(&ready)]), None, &bound(Duration::from_secs(30)));

        let groups: String = journal.join("groups").display().to_string();
        assert_scope_record(&groups);
        let scope: PathBuf = scope_directory_of(&groups);
        let writer_alive: bool = fs::read_to_string(scope.join("cgroup.procs")).is_ok_and(|text| { return !text.trim().is_empty(); });
        clean_up(&scope);
        assert!(matches!(result, Err(RunError::NotQuiescent { .. })), "{fault:?}: {result:?}");
        if !matches!(fault, Point::ContainmentInspection) {
            // The inspection fails after `cgroup.kill`, so the writer is already gone in that case; the verdict is conservative anyway.
            assert!(writer_alive, "{fault:?}: the writer was not left in the scope, so the test shows nothing");
        }
        assert!(transaction.writers_may_remain(), "{fault:?}: the scope was not told that writers may remain");
        assert!(journal.exists() && !journal.to_string_lossy().ends_with(".done"), "{fault:?}: the journal was not retained");
        drop(transaction);
    }
}

/// Acceptance row "A1, V3 (spawn isolation)":
/// a bounded run is held inside its start, after its record and before its program,
/// while an unrelated ordinary spawn and a second bounded run both happen;
/// then the first run goes on.
/// The wrapper stays in its original group throughout, the two runs have separate scopes,
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
        return output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(30)));
    });
    at_start.recv_timeout(Duration::from_secs(60)).expect("the first run reached its start");

    let mut unrelated = Command::new("sleep").arg("600").spawn().expect("an unrelated child");
    let group_during: String = fs::read_to_string("/proc/self/cgroup").expect("own group");
    let second = output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(30))).expect("the second run completes while the first is inside its start");
    release_sender.send(()).expect("release the first run");
    let first_output = first.join().expect("first run thread").expect("the first run completes");

    let unrelated_alive: bool = unrelated.try_wait().expect("asks").is_none();
    let unrelated_group: String = fs::read_to_string(format!("/proc/{}/cgroup", unrelated.id())).unwrap_or_default();
    let _ = unrelated.kill();
    let _ = unrelated.wait();
    let first_group: String = String::from_utf8_lossy(&first_output.stdout).trim().to_string();
    let second_group: String = String::from_utf8_lossy(&second.stdout).trim().to_string();
    assert_eq!(group_during, original, "the wrapper changed its group during a start");
    assert_eq!(fs::read_to_string("/proc/self/cgroup").expect("own group"), original);
    assert!(first_group.ends_with(".scope") && second_group.ends_with(".scope"), "{first_group} / {second_group}");
    assert_ne!(first_group, second_group, "the two runs shared a scope");
    assert!(unrelated_alive, "an unrelated child was stopped by a cleanup");
    assert_eq!(unrelated_group, original, "the unrelated child was moved into a run's scope");
}

/// Acceptance row "V6":
/// a manager that does not answer ends the discovery at its deadline with the reason,
/// and the run takes the weaker path;
/// nothing waits for the manager without end.
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
/// (`$$`, `${NAME}`, `%`, backslashes, quotes, spaces, a line feed, a leading dash, a semicolon, an empty argument, non-UTF-8 bytes)
/// reach the program exactly as given, also in a command with a cleared environment.
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
