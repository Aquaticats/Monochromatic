//! What:
//! Round 16 tests: the fixes asked for by the certification pass of groups 1 and 2 (`cargo-shadow-cert-sol-1.txt`,
//! `cargo-shadow-cert-sol-2.txt`).
//! Why:
//! The pass found tests that pass although a weaker outcome than the guarantee would also pass them; each test here discriminates the
//! guarantee from the weaker outcome named in the pass, and the planted regression that would have passed before is run as a
//! control where one can be built.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round sixteen', () => { it('a launcher born at the launch boundary is in the wrapper group', ...) })
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use crate::bounded_run::{Bound, RunError, on_start_for_test, output_within, take_trace_for_test, use_route_settings_for_test};
use crate::cgroup_test_support::{delegated_route, start_in_scope, wait_until};
use crate::contain_counterexample_tests::wait_for_file;
use crate::control_group::RouteSettings;
use crate::test_scratch::scratch;

/// A bound for the commands of these tests.
fn bound(limit: Duration) -> Bound {
    return Bound { limit, what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// Runs `work` on a thread and requires its answer within `window` while the caller still holds whatever the work contends for.
/// A contender that waits for the holder would answer only after the caller released it, which this observation never does before the
/// answer (certification group 1, V6: a busy lock returns at once).
pub(crate) fn answers_within<T: Send + 'static>(what: &str, window: Duration, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel::<T>();
    std::thread::spawn(move || { let _ = sender.send(work()); });
    return receiver.recv_timeout(window).unwrap_or_else(|_| { panic!("{what} did not answer within {window:?} while the lock was held: it waited for the holder"); });
}

/// The group line of a process, or an empty text for one that cannot be read.
fn group_of(pid: &str) -> String {
    return fs::read_to_string(format!("/proc/{pid}/cgroup")).unwrap_or_default().trim().to_string();
}

//region the launch boundary
// What the region covers: the actual spawn of the launcher and its association with a scope, held under a handshake.

/// The environment variable that gives the helper process of the control its gate directory.
const FIXTURE_GATES: &str = "CARGO_SHADOW_ROUND_SIXTEEN_GATES";

/// The environment variable that gives the helper process of the control the `cgroup.procs` it moves itself into.
const FIXTURE_MOVE_INTO: &str = "CARGO_SHADOW_ROUND_SIXTEEN_MOVE_INTO";

/// A stand-in for `systemd-run` that holds the launcher at the boundary the manager crosses to put it into its scope:
/// the script takes a numbered slot, publishes its own process id, waits for the gate file of its slot, and only then becomes the real
/// `systemd-run`.
/// A launcher held there has been spawned by the wrapper and is not in a scope yet, so what group it is in is the group the wrapper
/// had at the moment of the spawn.
fn gated_systemd_run(directory: &Path) -> PathBuf {
    let script: PathBuf = directory.join("gated-systemd-run");
    let text: String = format!(
        "#!/bin/sh\nd={dir}\ni=1\nwhile ! mkdir \"$d/slot-$i\" 2>/dev/null; do i=$((i+1)); done\necho $$ > \"$d/pid-$i.part\"; mv \"$d/pid-$i.part\" \"$d/pid-$i\"\nwhile [ ! -e \"$d/open-$i\" ]; do sleep 0.01; done\nexec systemd-run \"$@\"\n",
        dir = directory.display()
    );
    fs::write(&script, text).expect("write the stand-in");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return script;
}

/// What the wrapper and the held launchers were seen in, at the boundary and after it.
struct Seen {
    /// The group the wrapper had before anything started.
    original: String,
    /// The wrapper's group while the first launcher was held.
    wrapper_first: String,
    /// The first launcher's group while it was held.
    launcher_first: String,
    /// The wrapper's group while both launchers were held and the unrelated child had been spawned.
    wrapper_second: String,
    /// The second launcher's group while it was held.
    launcher_second: String,
    /// The unrelated child's group when it was spawned between the two spawns.
    unrelated: String,
    /// Whether the unrelated child lived to the end.
    unrelated_alive: bool,
    /// The scope the first program reported.
    first_scope: String,
    /// The scope the second program reported.
    second_scope: String,
    /// The wrapper's group at the end.
    wrapper_after: String,
}

/// Runs the scenario of the row A1, V3: a bounded run whose launcher is held at the launch boundary, an unrelated ordinary spawn, and
/// a second bounded run whose launcher is held there too; then both are let go.
/// With `move_into`, the first run's start moves the whole process into that group right before the launcher is spawned (the
/// regression the pass describes, which is not restored), so the launcher is born in the wrong group.
fn at_the_launch_boundary(directory: &Path, move_into: Option<PathBuf>) -> Seen {
    let original: String = group_of("self");
    let settings: RouteSettings = RouteSettings { systemd_run: gated_systemd_run(directory).display().to_string(), ..RouteSettings::default() };
    let first_settings: RouteSettings = settings.clone();
    let first = std::thread::spawn(move || {
        use_route_settings_for_test(Some(first_settings));
        if let Some(procs) = move_into {
            on_start_for_test(Some(Rc::new(move || { fs::write(&procs, std::process::id().to_string()).expect("move the wrapper"); })));
        }
        return output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(60)));
    });
    wait_for_file(&directory.join("pid-1"));
    let first_launcher: String = fs::read_to_string(directory.join("pid-1")).expect("pid").trim().to_string();
    let wrapper_first: String = group_of("self");
    let launcher_first: String = group_of(&first_launcher);

    let second = std::thread::spawn(move || {
        use_route_settings_for_test(Some(settings));
        return output_within(Command::new("sh").args(["-c", "cat /proc/self/cgroup"]), None, &bound(Duration::from_secs(60)));
    });
    wait_for_file(&directory.join("pid-2"));
    let second_launcher: String = fs::read_to_string(directory.join("pid-2")).expect("pid").trim().to_string();
    let mut unrelated: Child = Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("an unrelated child");
    let unrelated_group: String = group_of(&unrelated.id().to_string());
    let wrapper_second: String = group_of("self");
    let launcher_second: String = group_of(&second_launcher);

    fs::write(directory.join("open-2"), "go").expect("let the second launcher go");
    fs::write(directory.join("open-1"), "go").expect("let the first launcher go");
    let first_output = first.join().expect("first thread").expect("the first run completes");
    let second_output = second.join().expect("second thread").expect("the second run completes");
    let unrelated_alive: bool = unrelated.try_wait().expect("asks").is_none();
    let _ = unrelated.kill();
    let _ = unrelated.wait();
    return Seen {
        original,
        wrapper_first,
        launcher_first,
        wrapper_second,
        launcher_second,
        unrelated: unrelated_group,
        unrelated_alive,
        first_scope: String::from_utf8_lossy(&first_output.stdout).trim().to_string(),
        second_scope: String::from_utf8_lossy(&second_output.stdout).trim().to_string(),
        wrapper_after: group_of("self"),
    };
}

/// Acceptance row "A1, V3 (spawn isolation)" (certification group 2):
/// the actual launch of two bounded runs, each held at the boundary where the manager would put its launcher into a scope, with an
/// unrelated ordinary spawn between them.
/// A launcher is born in the group its spawner has at the spawn, so a launcher held there that is in the wrapper's original group
/// shows that the wrapper was not in another group at the spawn; the wrapper, the unrelated child and both launchers are in that group
/// at the boundary, the two programs then run in separate scopes, and the unrelated child survives both cleanups.
#[test]
fn two_launchers_held_at_the_launch_boundary_and_an_unrelated_spawn_share_the_wrappers_group() {
    if delegated_route().is_none() {
        return;
    }
    let directory = scratch();

    let seen: Seen = at_the_launch_boundary(directory.path(), None);

    assert_eq!(seen.wrapper_first, seen.original, "the wrapper changed its group at the first launch");
    assert_eq!(seen.launcher_first, seen.original, "the first launcher was born in another group than the wrapper's");
    assert_eq!(seen.unrelated, seen.original, "the unrelated child was born in another group than the wrapper's");
    assert_eq!(seen.wrapper_second, seen.original, "the wrapper changed its group at the second launch");
    assert_eq!(seen.launcher_second, seen.original, "the second launcher was born in another group than the wrapper's");
    assert!(seen.first_scope.ends_with(".scope") && seen.second_scope.ends_with(".scope"), "{} / {}", seen.first_scope, seen.second_scope);
    assert_ne!(seen.first_scope, seen.second_scope, "the two runs shared a scope");
    assert_ne!(seen.first_scope, seen.original.trim_start_matches("0::"), "a program ran in the wrapper's own group");
    assert!(seen.unrelated_alive, "an unrelated child was stopped by a cleanup");
    assert_eq!(seen.wrapper_after, seen.original);
}

/// The helper process of the control: it is the wrapper, and a regression that moves it into another group right before a launcher is
/// spawned is planted in it.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_wrapper_moved_before_the_spawn_puts_the_launcher_in_the_wrong_group_which_the_boundary_test_sees"]
fn moved_wrapper_fixture() {
    let Ok(gates) = std::env::var(FIXTURE_GATES) else {
        return;
    };
    let procs: PathBuf = PathBuf::from(std::env::var(FIXTURE_MOVE_INTO).expect("a group to move into"));
    let directory: PathBuf = PathBuf::from(gates);
    let seen: Seen = at_the_launch_boundary(&directory, Some(procs));
    fs::write(directory.join("seen"), format!("{}\n{}\n{}\n", seen.original, seen.launcher_first, seen.wrapper_first)).expect("write what was seen");
}

/// Control of the test above (certification group 2): the regression the pass describes, planted, is seen.
/// A wrapper that moves into another group right before it spawns a launcher (and is never restored here, which a spawn-then-restore
/// regression would do afterwards) puts the launcher in that other group, and the launcher held at the boundary shows it; so the
/// equality the test above asserts fails under that regression.
#[test]
fn a_wrapper_moved_before_the_spawn_puts_the_launcher_in_the_wrong_group_which_the_boundary_test_sees() {
    if delegated_route().is_none() {
        return;
    }
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let directory = scratch();
    let status = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_sixteen_tests::moved_wrapper_fixture", "--nocapture"])
        .env(FIXTURE_GATES, directory.path())
        .env(FIXTURE_MOVE_INTO, holder.group.directory().join("cgroup.procs"))
        .status()
        .expect("the helper");

    assert!(status.success(), "the helper failed: {status:?}");
    let seen: String = fs::read_to_string(directory.path().join("seen")).expect("what the helper saw");
    let lines: Vec<&str> = seen.lines().collect();
    let holder_name: String = holder.group.directory().file_name().expect("a name").to_string_lossy().to_string();
    assert!(lines[1].ends_with(&holder_name) && lines[1] != lines[0], "the launcher born after the move was not seen in the holder's group: {seen}");
    assert!(lines[2].ends_with(&holder_name), "the wrapper was not seen in the holder's group at the boundary: {seen}");
}
//endregion

//region the order of the record and the launch
// What the region covers: the real boundaries of a run's recording and launching, read from the ordered trace of the runner.

/// Acceptance rows "V3, V4" (certification group 2): in a run the record of the containment is written, then made durable (the
/// sync sees the whole record in the file), and only then is the program launched; the launcher line comes after the launch, without
/// a sync, and the end is recorded and synced last.
/// A sync made before the append, followed by an unsynced append, would show as a sync whose file does not hold the record yet.
/// Both containments are run, and the trace says which was attempted.
#[test]
fn the_record_is_written_made_durable_and_only_then_the_program_is_launched() {
    for weaker in [false, true] {
        let plain = scratch();
        if weaker {
            use_route_settings_for_test(Some(RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() }));
        } else if delegated_route().is_none() {
            return;
        }
        let root = scratch();
        fs::create_dir_all(root.path().join("package/s/one")).expect("dir");
        fs::write(root.path().join("package/s/one/Cargo.lock"), "lock").expect("lock");
        let transaction = crate::journal::begin(root.path(), &[crate::crates::LockSnapshot { crate_dir: "package/s/one".to_string(), source: "lock".to_string() }], false).expect("begin");
        let _ = take_trace_for_test();

        output_within(&mut Command::new("true"), None, &bound(Duration::from_secs(30))).expect("runs");
        use_route_settings_for_test(None);
        let trace: Vec<String> = take_trace_for_test();

        let kind: &str = if weaker { "pgroup-starting" } else { "scope" };
        let launch: &str = if weaker { "launch pgroup" } else { "launch scope" };
        let intended: Vec<String> = vec![format!("attempt {kind}"), format!("written {kind}"), format!("sync {kind} whole=true"), format!("synced {kind}"), launch.to_string()];
        assert_eq!(trace[..intended.len()], intended[..], "weaker={weaker}: the order is not record, sync, launch: {trace:?}");
        let rest: Vec<&str> = trace[intended.len()..].iter().map(String::as_str).collect();
        if weaker {
            assert_eq!(rest[..4], ["attempt pgroup", "written pgroup", "sync pgroup whole=true", "synced pgroup"], "weaker: the group record after the launch: {trace:?}");
        } else {
            assert_eq!(rest[..2], ["attempt launcher", "written launcher"], "the launcher line comes after the launch and is not synced: {trace:?}");
            assert!(!rest.iter().any(|event| { return event.starts_with("sync launcher"); }), "the launcher line was synced: {trace:?}");
            assert_eq!(rest[2..], ["attempt settled", "written settled", "sync settled whole=true", "synced settled"], "the end is recorded and synced last: {trace:?}");
        }
        drop(transaction);
    }
}
//endregion

//region the listing helper
// What the region covers: the shared helper that lists the journals of a workspace fails closed (certification group 3).

/// The helper that lists the journals reads a missing journal directory as no journals and stops on every other failure, so that an
/// unreadable journal parent cannot make "every journal is gone" pass.
#[test]
fn the_journal_listing_helper_stops_on_an_unreadable_directory_and_reads_only_absence_as_empty() {
    let root = scratch();
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "an absent journal directory is no journals");
    let journals: PathBuf = root.path().join(crate::journal::JOURNAL_DIR);
    fs::create_dir_all(journals.join("one")).expect("a journal");
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 1);
    fs::set_permissions(&journals, fs::Permissions::from_mode(0o000)).expect("make the journals unreadable");
    if fs::read_dir(&journals).is_ok() {
        fs::set_permissions(&journals, fs::Permissions::from_mode(0o755)).expect("restore");
        eprintln!("SKIPPED: this user reads a directory without permission, so the schedule cannot be made");
        return;
    }
    let outcome = std::panic::catch_unwind(|| { return crate::journal_support::journal_dirs(root.path()); });
    fs::set_permissions(&journals, fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(outcome.is_err(), "an unreadable journal directory was read as an empty listing");
}
//endregion

//region reaping of the direct child
// What the region covers: the direct child is reaped, observed by a checked wait (certification group 1, A1).

/// Acceptance row A1 (certification group 1): after an ordinary return and after a timeout, the program the wrapper started (the
/// direct child, which publishes its own process id) can no longer be waited for: a checked `waitpid` gives `ECHILD`.
/// The run used a delegated scope (its journal records one).
#[test]
fn the_direct_child_is_reaped_after_a_normal_return_and_after_a_timeout() {
    if delegated_route().is_none() {
        return;
    }
    for at_the_bound in [false, true] {
        let root = scratch();
        fs::create_dir_all(root.path().join("package/s/one")).expect("dir");
        fs::write(root.path().join("package/s/one/Cargo.lock"), "lock").expect("lock");
        let transaction = crate::journal::begin(root.path(), &[crate::crates::LockSnapshot { crate_dir: "package/s/one".to_string(), source: "lock".to_string() }], false).expect("begin");
        let journal: PathBuf = crate::journal_support::journal_dirs(root.path()).into_iter().next().expect("a journal");
        let direct: PathBuf = root.path().join("direct");
        let script: String = format!("echo $$ > {file}.part; mv {file}.part {file}; {finish}", file = direct.display(), finish = if at_the_bound { "exec sleep 600" } else { "echo done" });

        let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(if at_the_bound { 3 } else { 30 })));

        crate::cgroup_test_support::assert_scope_record(&journal.join("groups").display().to_string());
        if at_the_bound {
            assert!(matches!(result, Err(RunError::TimedOut { .. })), "{result:?}");
        } else {
            result.expect("runs");
        }
        crate::contain_counterexample_tests::assert_reaped(crate::contain_counterexample_tests::read_pid(&direct), if at_the_bound { "at the bound" } else { "after a normal return" });
        drop(transaction);
    }
}
//endregion

//region recovery and admission (V2, certification group 1)
// What the region covers: schedules where the blocking cause must be the one the test claims.

/// The crate root the journals of this region snapshot.
const CRATE_DIR: &str = "package/s/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> crate::crates::LockSnapshot {
    return crate::crates::LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "unchecked output of the killed run").expect("lock");
    return root;
}

/// A judge that refuses the live lockfile, so that a recovery that goes ahead restores the snapshot.
fn refusing(_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// Acceptance row "V2, amended c" (certification group 1): a journal whose only containment record is malformed is a possible
/// survivor: the recovery keeps it and the lockfile and says that the record is not understood.
/// An implementation that ignored malformed lines would recover it (nothing else in the journal blocks), so the cause of the refusal is
/// the malformed record; the control is the acknowledgement, after which the same recovery settles the journal.
#[test]
fn a_journal_whose_only_record_is_malformed_is_kept_with_its_diagnostic_until_it_is_acknowledged() {
    let root = repository();
    let journal: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), "not a number\n").expect("write a malformed record");

    let messages: Vec<String> = crate::journal::recover(root.path(), &refusing);

    assert_eq!(live_lock(root.path()), "unchecked output of the killed run", "a journal with a malformed record was recovered: {messages:?}");
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 1, "the journal was not kept");
    assert!(messages.iter().any(|message| { return message.contains("not understood") && message.contains("cargo-shadow acknowledge"); }), "the diagnostic of the malformed record is missing: {messages:?}");
    crate::acknowledge::acknowledge(&journal).expect("acknowledges");
    let after: Vec<String> = crate::journal::recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the control: the acknowledged journal was not settled: {after:?}");
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
}

/// A scope with a live descendant and a journal that records it with a resolved lifecycle (the scope and its settled end), so the
/// member is the only thing that can block.
fn journal_with_a_scope_that_has_a_member(root: &Path) -> Option<(crate::cgroup_test_support::TestScope, PathBuf)> {
    let scope = start_in_scope(Command::new("sleep").arg("600"))?;
    let journal: PathBuf = crate::journal_support::plant(root, &[snapshot("first")], &[]);
    fs::write(journal.join("groups"), format!("{}\nsettled {}\n", scope.group.record(), scope.group.directory().display())).expect("record the scope");
    assert!(scope.group.members().is_ok_and(|found| { return !found.is_empty(); }), "the scope has no member, so the schedule is not made");
    return Some((scope, journal));
}

/// Stops a scope and waits until it is gone.
fn end_scope(scope: &crate::cgroup_test_support::TestScope) {
    let _ = scope.group.stop(Duration::from_secs(10));
    let directory: PathBuf = scope.group.directory().to_path_buf();
    wait_until("the scope to be collected", &|| { return !directory.exists(); });
}

/// Acceptance row "V2, amended c" (certification group 1): a positively observed member of one journal's scope stops the recovery of
/// another journal.
/// The scope's lifecycle is resolved (its end is recorded), so nothing but the live member blocks; once the member is gone the same
/// recovery settles both journals, which is the control that no other uncertainty was blocking.
#[test]
fn a_member_in_the_scope_of_one_journal_stops_the_recovery_of_another_until_it_ends() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    // The other journal is planted first: planting begins a transaction, which a journal with a live member would refuse.
    let _other = crate::journal_support::plant(root.path(), &[snapshot("second")], &[]);
    let Some((scope, _journal)) = journal_with_a_scope_that_has_a_member(root.path()) else {
        return;
    };

    let blocked: Vec<String> = crate::journal::recover(root.path(), &refusing);
    let kept: String = live_lock(root.path());
    end_scope(&scope);
    let after: Vec<String> = crate::journal::recover(root.path(), &refusing);

    assert_eq!(kept, "unchecked output of the killed run", "a live member did not stop the recovery: {blocked:?}");
    assert!(blocked.iter().any(|message| { return message.contains("still running"); }), "the refusal does not name the member: {blocked:?}");
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "the control: with the member gone the journals were not settled: {after:?}");
}

/// Acceptance row "V2, amended c" (certification group 1): a new transaction does not start while a journal's scope has a member, and
/// starts once it is gone (the control).
#[test]
fn a_new_transaction_does_not_start_beside_a_scope_with_a_member_until_it_ends() {
    if delegated_route().is_none() {
        return;
    }
    let root = repository();
    let Some((scope, _journal)) = journal_with_a_scope_that_has_a_member(root.path()) else {
        return;
    };

    let refused = crate::journal::begin(root.path(), &[snapshot("second")], false);
    end_scope(&scope);
    let started = crate::journal::begin(root.path(), &[snapshot("second")], false);

    assert!(refused.as_ref().is_err_and(|error| { return error.to_string().contains("still running"); }), "a transaction started beside a live member, or the refusal does not name it: {:?}", refused.as_ref().err());
    assert!(started.is_ok(), "the control: with the member gone the transaction was still refused: {:?}", started.err());
}
//endregion

//region deadlines of the scope (V6, certification group 1)
// What the region covers: a stalled scope startup and an unsuccessful shutdown, at their real boundaries, with elapsed-time bounds.

/// Acceptance row V6 (certification group 1): a scope startup that stalls before the payload runs is bounded.
/// The launcher (the stand-in for `systemd-run` that the manager would answer) is held at its gate and never released, so the
/// program never runs; the run ends at its bound with `TimedOut`, the payload never started, the direct child (the held launcher) was
/// reaped, and quiescence was shown (nothing could have started), so the transaction reports no writers that may remain.
#[test]
fn a_scope_startup_that_stalls_before_the_payload_ends_at_the_bound() {
    if delegated_route().is_none() {
        return;
    }
    let directory = scratch();
    let root = scratch();
    fs::create_dir_all(root.path().join("package/s/one")).expect("dir");
    fs::write(root.path().join("package/s/one/Cargo.lock"), "lock").expect("lock");
    let transaction = crate::journal::begin(root.path(), &[crate::crates::LockSnapshot { crate_dir: "package/s/one".to_string(), source: "lock".to_string() }], false).expect("begin");
    let journal: PathBuf = crate::journal_support::journal_dirs(root.path()).into_iter().next().expect("a journal");
    use_route_settings_for_test(Some(RouteSettings { systemd_run: gated_systemd_run(directory.path()).display().to_string(), ..RouteSettings::default() }));
    let marker: PathBuf = directory.path().join("payload-ran");
    let started = std::time::Instant::now();

    let result = output_within(Command::new("sh").args(["-c", &format!(": > {}", marker.display())]), None, &bound(Duration::from_secs(2)));
    let elapsed: Duration = started.elapsed();
    use_route_settings_for_test(None);

    assert!(directory.path().join("pid-1").exists(), "the launcher never started, so the startup was not held");
    assert!(matches!(result, Err(RunError::TimedOut { .. })), "a stalled startup did not end as a timeout: {result:?}");
    assert!(elapsed >= Duration::from_secs(2) && elapsed < Duration::from_secs(8), "the run returned after {elapsed:?}: not at its bound of two seconds");
    assert!(!marker.exists(), "the payload ran although its startup was held");
    let launcher: i32 = fs::read_to_string(directory.path().join("pid-1")).expect("pid").trim().parse::<i32>().expect("a pid");
    crate::contain_counterexample_tests::assert_reaped(launcher, "a stalled startup");
    assert!(!transaction.writers_may_remain(), "nothing could have started, yet the scope reports writers that may remain");
    crate::cgroup_test_support::assert_scope_record(&journal.join("groups").display().to_string());
}

/// Acceptance row V6 (certification group 1): an unsuccessful shutdown has a deadline and produces uncertainty.
/// A member survives every signal (the stop runs its real loop, which signals again every interval), the writer is ready before the
/// stop, and the run returns `NotQuiescent` only once the quiescence bound has passed, and not much later; the scope reports that
/// writers may remain and the journal is retained.
#[test]
fn an_unsuccessful_shutdown_returns_not_quiescent_at_the_quiescence_bound() {
    if delegated_route().is_none() {
        return;
    }
    let root = scratch();
    fs::create_dir_all(root.path().join("package/s/one")).expect("dir");
    fs::write(root.path().join("package/s/one/Cargo.lock"), "lock").expect("lock");
    let transaction = crate::journal::begin(root.path(), &[crate::crates::LockSnapshot { crate_dir: "package/s/one".to_string(), source: "lock".to_string() }], false).expect("begin");
    let journal: PathBuf = crate::journal_support::journal_dirs(root.path()).into_iter().next().expect("a journal");
    let ready: PathBuf = root.path().join("writer-ready");
    let script: String = format!("setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; exit 0", ready = ready.display());
    crate::failpoints::arm(crate::failpoints::Point::ContainmentSurvivor);
    let started = std::time::Instant::now();

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(60)));
    let elapsed: Duration = started.elapsed();

    let groups: String = journal.join("groups").display().to_string();
    let scope: PathBuf = crate::cgroup_test_support::scope_directory_of(&groups);
    let _ = fs::write(scope.join("cgroup.kill"), "1");
    assert!(!crate::failpoints::fires(crate::failpoints::Point::ContainmentSurvivor), "the surviving member was never reached");
    assert!(ready.exists(), "the writer was not ready before the shutdown");
    assert!(matches!(result, Err(RunError::NotQuiescent { .. })), "{result:?}");
    let bound_of_the_stop: Duration = crate::bounded_run::QUIESCE_BOUND;
    assert!(elapsed >= bound_of_the_stop && elapsed < bound_of_the_stop + Duration::from_secs(6), "the shutdown returned after {elapsed:?}, not at its bound of {bound_of_the_stop:?}");
    assert!(transaction.writers_may_remain(), "the scope was not told that writers may remain");
    assert!(journal.exists() && !journal.to_string_lossy().ends_with(".done"), "the journal was not retained");
    wait_until("the scope's end", &|| { return !scope.exists() || fs::read_to_string(scope.join("cgroup.events")).is_ok_and(|text| { return text.contains("populated 0"); }); });
}
//endregion

//region the finish of an active recovery (V5, certification group 3)
// What the region covers: a recovery that restores the files finishes with the same mandatory phases as a transaction.

/// The journals of a workspace in the order they are named: whether each is a completed (`.done`) one.
fn journal_states(root: &Path) -> Vec<bool> {
    return crate::journal_support::journal_dirs(root).iter().map(|journal| { return journal.to_string_lossy().ends_with(".done"); }).collect();
}

/// Acceptance row V5 (certification group 3): an active recovery that restores the files finishes with its own mandatory phases, and
/// each failure of one is visible and leaves a state the next recovery completes:
/// the output barrier fails (the journal stays active, no marker), the completion rename fails (the journal stays active), the sync
/// of the marker's directory fails (the marker is published, the journal is `.done` and a later recovery only removes it), and the
/// removal after the marker fails (the cleanup-only case: `.done` stays and is removed later).
/// Each fault must have been consumed, the restored lockfile is the snapshot in every case, and the later recovery completes the
/// journal and judges nothing again.
#[test]
fn an_active_recovery_finishes_with_its_own_mandatory_phases_and_tells_cleanup_apart() {
    use crate::failpoints::{Point, arm};
    // (fault, journal is `.done` after the failed recovery)
    for (fault, done_afterwards) in [(Point::RecoverDurability, false), (Point::CompletionRename, false), (Point::CompletionSync, true), (Point::FinishCleanup, true)] {
        let root = repository();
        let _journal = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
        arm(fault);

        let failed: Vec<String> = crate::journal::recover(root.path(), &refusing);

        assert!(!crate::failpoints::fires(fault), "{fault:?}: the fault was never reached");
        assert_eq!(live_lock(root.path()), "original", "{fault:?}: the files were not restored before the finish: {failed:?}");
        assert!(failed.iter().any(|message| { return message.contains("journal was kept") || message.contains("could not be removed") || message.contains("tried again"); }), "{fault:?}: the failure of the finish is not reported: {failed:?}");
        assert_eq!(journal_states(root.path()), vec![done_afterwards], "{fault:?}: the journal is not in the state that this failure leaves: {failed:?}");

        let judged: std::cell::Cell<u32> = std::cell::Cell::new(0);
        let later: Vec<String> = crate::journal::recover(root.path(), &|before, live| { judged.set(judged.get() + 1); return refusing(before, live); });
        assert!(journal_states(root.path()).is_empty(), "{fault:?}: the later recovery did not complete the journal: {later:?}");
        assert_eq!(live_lock(root.path()), "original");
        if done_afterwards {
            assert_eq!(judged.get(), 0, "{fault:?}: a journal that was already marked done was judged again");
        }
    }
}
//endregion

//region the weaker path (certification group 3)
// What the region covers: the capability classes that were not exercised, recovery with one of two journals acknowledged.

/// A stand-in for `systemd-run` that the manager refuses: it says so and exits, and never runs the program.
fn refusing_systemd_run(directory: &Path) -> PathBuf {
    let script: PathBuf = directory.join("refusing-systemd-run");
    fs::write(&script, "#!/bin/sh\necho 'Failed to start transient scope unit: delegation refused' >&2\nexit 1\n").expect("write the stand-in");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return script;
}

/// Acceptance row "Weaker path" (certification group 3), the capability classes the other tests omit:
/// a kernel whose cgroup directory has no `cgroup.kill` (the manager reports the root group, which has none) is told apart and the
/// run takes the weaker path with its record before the start;
/// and a scope that the manager refuses to start (an actual delegation failure, after discovery succeeded) is not the weaker path and
/// not a program that ran: the program does not run, the run's end is recorded (nothing could have started), and a recovery then
/// needs no acknowledgement.
#[test]
fn a_directory_without_a_kill_file_takes_the_weaker_path_and_a_refused_scope_runs_nothing() {
    if delegated_route().is_none() {
        return;
    }
    // No cgroup.kill: discovery names it.
    let no_kill: RouteSettings = RouteSettings { systemctl: "sh".to_string(), question: Some(vec!["-c".to_string(), "echo /".to_string()]), ..RouteSettings::default() };
    let refused = crate::control_group::detect_route(&no_kill).expect_err("the root group has no cgroup.kill");
    assert!(refused.to_string().contains("cgroup.kill"), "{refused}");

    let root = repository();
    let transaction = crate::journal::begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal: PathBuf = crate::journal_support::journal_dirs(root.path()).into_iter().next().expect("a journal");
    use_route_settings_for_test(Some(no_kill));
    let weaker = output_within(&mut Command::new("true"), None, &bound(Duration::from_secs(30)));
    use_route_settings_for_test(None);
    weaker.expect("runs in a process group");
    let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
    assert!(record.starts_with("pgroup starting\npgroup "), "the kernel without cgroup.kill did not take the weaker path: {record:?}");
    drop(transaction);

    // A scope that the manager refuses.
    let other = repository();
    let directory = scratch();
    let second = crate::journal::begin(other.path(), &[snapshot("original")], false).expect("begin");
    let second_journal: PathBuf = crate::journal_support::journal_dirs(other.path()).into_iter().next().expect("a journal");
    use_route_settings_for_test(Some(RouteSettings { systemd_run: refusing_systemd_run(directory.path()).display().to_string(), ..RouteSettings::default() }));
    let marker: PathBuf = directory.path().join("payload-ran");
    let _ = take_trace_for_test();
    let result = output_within(Command::new("sh").args(["-c", &format!(": > {}", marker.display())]), None, &bound(Duration::from_secs(30)));
    use_route_settings_for_test(None);
    let trace: Vec<String> = take_trace_for_test();
    drop(second);

    assert!(!marker.exists(), "the program ran although the manager refused its scope");
    assert!(result.as_ref().map_or(true, |output| { return !output.status.success(); }), "a refused scope was read as a successful run: {result:?}");
    assert_eq!(trace[..2], ["attempt scope", "written scope"], "the run did not record a scope before the refusal: {trace:?}");
    let groups: String = fs::read_to_string(second_journal.join("groups")).expect("the record");
    assert!(groups.lines().any(|line| { return line.starts_with("settled "); }), "the run's end was not recorded: {groups:?}");
    let messages: Vec<String> = crate::journal::recover(other.path(), &refusing);
    assert!(crate::journal_support::journal_dirs(other.path()).is_empty(), "a refused scope needed an acknowledgement: {messages:?}");
}

/// Acceptance row "Weaker path" (certification group 3): recovery, not only admission, with two journals of the weaker path of which
/// one is acknowledged: the recovery restores nothing while the other blocks, and settles both once the other is acknowledged too.
#[test]
fn recovery_with_one_of_two_weaker_path_journals_acknowledged_restores_nothing_until_both_are() {
    let root = repository();
    let first: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    let second: PathBuf = crate::journal_support::plant(root.path(), &[snapshot("original")], &[]);
    fs::write(first.join("groups"), "pgroup starting\n").expect("record");
    fs::write(second.join("groups"), "pgroup starting\n").expect("record");
    crate::acknowledge::acknowledge(&first).expect("acknowledges the first");

    let blocked: Vec<String> = crate::journal::recover(root.path(), &refusing);

    assert_eq!(live_lock(root.path()), "unchecked output of the killed run", "a recovery restored files while an unacknowledged weaker-path journal stood: {blocked:?}");
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 2, "a journal was removed: {blocked:?}");
    assert!(blocked.iter().any(|message| { return message.contains(&second.display().to_string()) && message.contains("cargo-shadow acknowledge"); }), "the refusal does not name the unacknowledged journal: {blocked:?}");
    crate::acknowledge::acknowledge(&second).expect("acknowledges the second");
    let settled: Vec<String> = crate::journal::recover(root.path(), &refusing);
    assert_eq!(live_lock(root.path()), "original", "the control: with both acknowledged the recovery did not restore: {settled:?}");
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "{settled:?}");
}
//endregion
