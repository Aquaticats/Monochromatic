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
    let Ok(directory) = std::env::var(FIXTURE_GATES) else {
        return;
    };
    let procs: PathBuf = PathBuf::from(std::env::var(FIXTURE_MOVE_INTO).expect("a group to move into"));
    let directory: PathBuf = PathBuf::from(directory);
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
