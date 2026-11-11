//! Verifies the sweep of the scopes of the earlier design against a fake `systemctl` that logs its calls
//! and a fake cgroup tree:
//! what is reset,
//! what is stopped,
//! what must survive,
//! and what the log line says.

/// Sweep under test.
use crate::leftover_sweep::{
    describe_sweep, is_old_style_scope_name, read_populated, sweep_leftover_per_process_scopes,
    sweep_leftover_scopes, sweep_with, SweepReport, SweepSetup,
};
/// The pass over the origin scopes under test.
use crate::origin_scope_sweep::stop_empty_origin_scopes;
/// Where a scope lives.
use crate::scope_naming::scope_cgroup_directory;
/// The client the sweep runs.
use crate::user_client::UserClient;
/// Executable mode for the fake client.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::Duration;

/// Slice used by the tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";
/// A scope of the current design that holds no process,
/// which the sweep must never touch.
const CURRENT_DESIGN_SCOPE: &str = "wg-quicker-update-app.slice-claude\\x2dcode\\x2dbash.scope";

/// Our own uid and gid.
fn own_ids() -> (u32, u32) {
    // SAFETY: getuid and getgid have no preconditions.
    return unsafe { (libc::getuid(), libc::getgid()) };
}

/// Fixture of one test.
struct Place {
    /// The setup under test.
    setup: SweepSetup,
    /// The fake manager's list,
    /// one unit a line.
    listing: PathBuf,
    /// The calls the fake received,
    /// one a line.
    log: PathBuf,
}

/// Builds a fake `systemctl` that prints the list file for `list-units`,
/// logs every call,
/// and exits with `list_code` for the list and `change_code` for `reset-failed` and `stop`.
fn place(name: &str, list_code: i32, change_code: i32) -> Place {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-sweep-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.join("cgroup")).expect("make the fake cgroup tree");
    std::fs::create_dir_all(directory.join("gate")).expect("make the fake gate directory");
    let listing = directory.join("listing");
    let log = directory.join("log");
    std::fs::write(&listing, "").expect("write the empty list");
    let body = format!(
        "#!/bin/sh\necho \"$*\" >> '{log}'\ncase \"$2\" in\nlist-units) cat '{listing}'; exit {list_code} ;;\n*) exit {change_code} ;;\nesac\n",
        log = log.display(),
        listing = listing.display()
    );
    let program = directory.join("systemctl");
    std::fs::write(&program, body).expect("write the fake");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("make the fake executable");
    let (uid, gid) = own_ids();
    let setup = SweepSetup {
        uid,
        gid,
        slice: SLICE.to_string(),
        cgroup_root: directory.join("cgroup"),
        gate_root: directory.join("gate"),
        gate_wait: Duration::from_millis(50),
        client: UserClient { program: program.to_string_lossy().into_owned(), timeout: Duration::from_secs(20) },
        prefix: "wg-quicker-update-".to_string(),
    };
    return Place { setup, listing, log };
}

/// One line of the fake manager's list.
fn unit_line(name: &str, active: &str, sub: &str) -> String {
    return format!("{name} loaded {active} {sub} wg-quicker update traffic that bypasses the tunnel\n");
}

/// Makes a scope's cgroup directory with the given populated state.
fn scope_directory(place: &Place, name: &str, populated: bool) -> PathBuf {
    let directory = scope_cgroup_directory(&place.setup.cgroup_root, place.setup.uid, SLICE, name);
    std::fs::create_dir_all(&directory).expect("make the scope");
    let flag = if populated { 1 } else { 0 };
    std::fs::write(directory.join("cgroup.events"), format!("populated {flag}\nfrozen 0\n")).expect("write the state");
    return directory;
}

/// The calls the fake received.
fn calls(place: &Place) -> Vec<String> {
    return std::fs::read_to_string(&place.log).unwrap_or_default().lines().map(str::to_string).collect();
}

/// The calls that changed something,
/// with the verb first.
fn changes(place: &Place) -> Vec<String> {
    return calls(place).into_iter().filter(|call| return !call.contains("list-units")).collect();
}

/// Only two runs of digits joined by a dash,
/// between the prefix and `.scope`,
/// are the earlier design's names;
/// the path-named scopes of the current design and near misses are not.
#[test]
fn only_the_earlier_designs_names_are_swept() {
    for name in ["wg-quicker-update-1-2.scope", "wg-quicker-update-838116-15257815.scope", "wg-quicker-update-0-0.scope"] {
        assert!(is_old_style_scope_name(name, "wg-quicker-update-"), "{name}");
    }
    for name in [
        CURRENT_DESIGN_SCOPE,
        "wg-quicker-update-app.slice-x.scope",
        "wg-quicker-update-1-2-3.scope",
        "wg-quicker-update--2.scope",
        "wg-quicker-update-1-.scope",
        "wg-quicker-update-1.scope",
        "wg-quicker-update-1-2.service",
        "wg-quicker-update-1a-2.scope",
        "wg-quicker-update-1-2",
        "other-1-2.scope",
        "",
    ] {
        assert!(!is_old_style_scope_name(name, "wg-quicker-update-"), "{name}");
    }
    assert!(is_old_style_scope_name("wgtest-old-1-2.scope", "wgtest-old-"));
    assert!(!is_old_style_scope_name("wg-quicker-update-1-2.scope", "wgtest-old-"), "another prefix never matches a real name");
}

/// Failed units are reset in one call,
/// empty running scopes are stopped in one call,
/// and nothing else is named in any call:
/// a scope that holds a process,
/// an inactive unit,
/// a scope whose state cannot be read,
/// and a scope of the current design all survive.
#[test]
fn failed_units_are_reset_and_empty_scopes_stopped_and_a_populated_scope_survives() {
    let place = place("mixed", 0, 0);
    let mut list = String::new();
    for index in 1..=3 {
        list.push_str(&unit_line(&format!("wg-quicker-update-{index}-10.scope"), "failed", "failed"));
    }
    for index in 4..=6 {
        let name = format!("wg-quicker-update-{index}-10.scope");
        list.push_str(&unit_line(&name, "active", "running"));
        scope_directory(&place, &name, false);
    }
    for index in 7..=8 {
        let name = format!("wg-quicker-update-{index}-10.scope");
        list.push_str(&unit_line(&name, "active", "running"));
        scope_directory(&place, &name, true);
    }
    list.push_str(&unit_line("wg-quicker-update-9-10.scope", "inactive", "dead"));
    list.push_str(&unit_line("wg-quicker-update-11-10.scope", "active", "running"));
    list.push_str(&unit_line(CURRENT_DESIGN_SCOPE, "active", "running"));
    scope_directory(&place, CURRENT_DESIGN_SCOPE, false);
    std::fs::write(&place.listing, list).expect("write the list");
    let report = sweep_leftover_scopes(&place.setup);
    assert_eq!(
        report,
        SweepReport { listed: 10, origins_listed: 1, reset: 3, stopped: 3, origins_stopped: 1, origins_deferred: 0, held: 2, unreadable: 1, problems: vec![] },
        "{report:?}"
    );
    assert_eq!(
        changes(&place),
        vec![
            "--user reset-failed wg-quicker-update-1-10.scope wg-quicker-update-2-10.scope wg-quicker-update-3-10.scope".to_string(),
            "--user stop wg-quicker-update-4-10.scope wg-quicker-update-5-10.scope wg-quicker-update-6-10.scope".to_string(),
            format!("--user stop {CURRENT_DESIGN_SCOPE}"),
        ]
    );
    let everything = calls(&place).join("\n");
    for survivor in ["-7-10", "-8-10", "-9-10", "-11-10"] {
        assert!(!changes(&place).join("\n").contains(survivor), "{survivor} was named in a change: {everything}");
    }
    assert!(calls(&place)[0].contains("list-units --all --plain --no-legend --no-pager wg-quicker-update-*.scope"), "{everything}");
}

/// A scope that gains a process between the first read and the stop is left alone,
/// because each is read again right before its batch is sent.
#[test]
fn a_scope_that_gains_a_process_between_the_reads_is_not_stopped() {
    let place = place("recheck", 0, 0);
    let mut list = String::new();
    for index in 1..=3 {
        let name = format!("wg-quicker-update-{index}-10.scope");
        list.push_str(&unit_line(&name, "active", "running"));
        scope_directory(&place, &name, false);
    }
    std::fs::write(&place.listing, list).expect("write the list");
    let reads = std::cell::RefCell::new(std::collections::HashMap::<PathBuf, u32>::new());
    let populated = |directory: &Path| {
        let mut counts = reads.borrow_mut();
        let seen = counts.entry(directory.to_path_buf()).or_insert(0);
        *seen += 1;
        // The second scope has a process by the second read.
        return Some(*seen >= 2 && directory.to_string_lossy().contains("-2-10"));
    };
    let report = sweep_with(&place.setup, &populated);
    assert_eq!((report.stopped, report.held), (2, 1), "{report:?}");
    assert_eq!(changes(&place), ["--user stop wg-quicker-update-1-10.scope wg-quicker-update-3-10.scope"]);
}

/// Stops go in batches,
/// and a list larger than a pipe holds is read whole.
#[test]
fn a_long_list_is_read_whole_and_stops_go_in_batches() {
    let place = place("long", 0, 0);
    let mut list = String::new();
    for index in 0..3000 {
        list.push_str(&unit_line(&format!("wg-quicker-update-{index}-99.scope"), "failed", "failed"));
    }
    for index in 5000..5060 {
        let name = format!("wg-quicker-update-{index}-99.scope");
        list.push_str(&unit_line(&name, "active", "running"));
        scope_directory(&place, &name, false);
    }
    assert!(list.len() > 300_000, "the list is {} bytes", list.len());
    std::fs::write(&place.listing, list).expect("write the list");
    let report = sweep_leftover_scopes(&place.setup);
    assert_eq!((report.listed, report.reset, report.stopped), (3060, 3000, 60), "{report:?}");
    let verbs: Vec<String> = changes(&place).iter().map(|call| return call.split(' ').nth(1).unwrap_or("").to_string()).collect();
    assert_eq!(verbs.iter().filter(|verb| return verb.as_str() == "reset-failed").count(), 30);
    assert_eq!(verbs.iter().filter(|verb| return verb.as_str() == "stop").count(), 3);
}

/// A list that cannot be read changes nothing and is reported.
#[test]
fn a_list_that_cannot_be_read_changes_nothing() {
    let place = place("list-fails", 1, 0);
    std::fs::write(&place.listing, unit_line("wg-quicker-update-1-10.scope", "failed", "failed")).expect("write the list");
    let report = sweep_leftover_scopes(&place.setup);
    assert_eq!(report.listed, 0);
    assert_eq!(report.problems.len(), 1);
    assert!(report.problems[0].contains("listing the scopes failed"), "{report:?}");
    assert!(changes(&place).is_empty());
}

/// A reset or a stop that fails is reported and not counted.
#[test]
fn a_failing_change_is_reported_and_not_counted() {
    let place = place("change-fails", 0, 1);
    let mut list = unit_line("wg-quicker-update-1-10.scope", "failed", "failed");
    list.push_str(&unit_line("wg-quicker-update-2-10.scope", "active", "running"));
    scope_directory(&place, "wg-quicker-update-2-10.scope", false);
    std::fs::write(&place.listing, list).expect("write the list");
    let report = sweep_leftover_scopes(&place.setup);
    assert_eq!((report.reset, report.stopped), (0, 0), "{report:?}");
    assert_eq!(report.problems.len(), 2, "{report:?}");
    assert!(report.problems[0].contains("resetting 1 failed units failed"), "{report:?}");
    assert!(report.problems[1].contains("stopping 1 empty scopes failed"), "{report:?}");
}

/// The populated state is read from `cgroup.events`.
#[test]
fn the_populated_state_comes_from_cgroup_events() {
    let place = place("events", 0, 0);
    let directory = scope_directory(&place, "wg-quicker-update-1-1.scope", true);
    assert_eq!(read_populated(&directory), Some(true));
    std::fs::write(directory.join("cgroup.events"), "populated 0\nfrozen 0\n").expect("empty");
    assert_eq!(read_populated(&directory), Some(false));
    std::fs::write(directory.join("cgroup.events"), "frozen 0\n").expect("no populated line");
    assert_eq!(read_populated(&directory), None);
    assert_eq!(read_populated(&directory.join("absent")), None);
}

/// The log line states the counts once and names the problems,
/// and nothing is logged when there was nothing to sweep.
#[test]
fn the_log_line_states_the_counts_and_is_silent_when_there_was_nothing() {
    assert_eq!(describe_sweep(&SweepReport::default()), None);
    let line = describe_sweep(&SweepReport {
        listed: 9,
        origins_listed: 2,
        reset: 4,
        stopped: 3,
        origins_stopped: 1,
        origins_deferred: 0,
        held: 1,
        unreadable: 1,
        problems: vec!["x failed".to_string()],
    })
    .expect("a line");
    for phrase in [
        "9 of the earlier design listed",
        "4 failed units reset",
        "3 empty and stopped",
        "2 scopes of one origin listed",
        "1 of them empty and stopped",
        "1 scopes that hold a process left alone",
        "1 left alone because their state could not be read",
        "; x failed",
    ] {
        assert!(line.contains(phrase), "missing {phrase:?} in {line}");
    }
    assert!(describe_sweep(&SweepReport { problems: vec!["listing failed".to_string()], ..SweepReport::default() }).is_some());
}

/// Runs the sweep with the real `systemctl` against throwaway scopes of the user's own manager,
/// named `wgtest-old-<pid>-<n>.scope` in a throwaway slice so no real scope is ever named:
/// one holds a process,
/// one adopted a zombie and runs empty,
/// one adopted a process that was dying and failed.
/// The held one must survive and the other two must be gone.
/// It needs a user manager and its bus.
#[test]
#[ignore = "needs the user systemd manager"]
fn the_real_manager_sweeps_throwaway_scopes_and_spares_the_one_holding_a_process() {
    let slice = "app-wgtestslice.slice";
    let (uid, gid) = own_ids();
    let place = place("real", 0, 0);
    let setup = SweepSetup {
        client: UserClient { program: "systemctl".to_string(), timeout: Duration::from_secs(30) },
        cgroup_root: PathBuf::from("/sys/fs/cgroup"),
        slice: slice.to_string(),
        prefix: "wgtest-old-".to_string(),
        ..place.setup
    };
    let create = |name: &str, pid: u32| {
        let bus = std::process::Command::new("busctl")
            .args(["--user", "call", "org.freedesktop.systemd1", "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "StartTransientUnit", "ssa(sv)a(sa(sv))", name, "fail", "3", "Slice", "s", slice, "PIDs", "au", "1", &pid.to_string(), "Description", "s", "wgtest old style", "0"])
            .output()
            .expect("run busctl");
        assert!(bus.status.success(), "{}", String::from_utf8_lossy(&bus.stderr));
    };
    let mut zombie = std::process::Command::new("true").spawn().expect("start the zombie");
    std::thread::sleep(Duration::from_millis(200));
    let mut dying = std::process::Command::new("python3")
        .args(["-c", "import sys,time\nb = bytearray(600 * 2**20)\nprint('ready', flush=True)\ntime.sleep(300)"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("start the dying process");
    let mut ready = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(dying.stdout.take().expect("pipe")), &mut ready).expect("wait for the process");
    let mut holder = std::process::Command::new("sleep").arg("300").spawn().expect("start the holder");
    let names = [format!("wgtest-old-{}-1.scope", zombie.id()), format!("wgtest-old-{}-2.scope", dying.id()), format!("wgtest-old-{}-3.scope", holder.id())];
    create(&names[0], zombie.id());
    let _killed = dying.kill();
    create(&names[1], dying.id());
    create(&names[2], holder.id());
    std::thread::sleep(Duration::from_millis(500));
    let report = sweep_leftover_scopes(&setup);
    std::thread::sleep(Duration::from_millis(500));
    let state = |name: &str| {
        let shown = std::process::Command::new("systemctl").args(["--user", "show", "--property=ActiveState", name]).output().expect("ask the manager");
        return String::from_utf8_lossy(&shown.stdout).trim().to_string();
    };
    let states: Vec<String> = names.iter().map(|name| return state(name)).collect();
    let _killed_holder = holder.kill();
    let _reaped = [zombie.wait(), dying.wait(), holder.wait()];
    for name in &names {
        let _stopped = std::process::Command::new("systemctl").args(["--user", "stop", name]).status();
        let _reset = std::process::Command::new("systemctl").args(["--user", "reset-failed", name]).status();
    }
    let _slice_stopped = std::process::Command::new("systemctl").args(["--user", "stop", slice]).status();
    println!("{report:?} states {states:?} uid {uid} gid {gid}");
    assert_eq!(report.listed, 3, "{report:?}");
    assert_eq!(report.held, 1, "{report:?}");
    assert_eq!(report.reset + report.stopped, 2, "{report:?}");
    assert!(report.problems.is_empty(), "{report:?}");
    assert_eq!(states[2], "ActiveState=active", "the scope holding a process was touched");
    assert_eq!(states[0], "ActiveState=inactive");
    assert_eq!(states[1], "ActiveState=inactive");
}

/// The production setup runs `systemctl` against the updates slice and the real cgroup filesystem.
#[test]
fn the_production_setup_targets_systemctl_and_the_updates_slice() {
    let setup = SweepSetup::production(1000, 1000);
    assert_eq!(setup.client.program, "systemctl");
    assert_eq!(setup.slice, SLICE);
    assert_eq!(setup.cgroup_root, PathBuf::from("/sys/fs/cgroup"));
    assert!(setup.client.timeout >= Duration::from_secs(30));
}

/// The pass that runs while the watcher adopts stops no origin scope.
///
/// A process joining an origin scope is a process the watcher just moved there,
/// and stopping a scope ends every process in it:
/// a check of `populated 0` cannot answer for the moment the stop request arrives.
/// The origin scopes are stopped before the first adoption instead,
/// which the next case covers.
#[test]
fn the_sweep_that_runs_beside_adoption_leaves_origin_scopes_alone() {
    let place = place("origins-run-beside", 0, 0);
    let origin = "wg-quicker-update-app.slice-xx.scope";
    std::fs::write(&place.listing, unit_line(origin, "active", "running")).expect("write the list");
    let _directory = scope_directory(&place, origin, false);
    let report = sweep_leftover_per_process_scopes(&place.setup, &|_path| return Some(false));
    assert_eq!(report.origins_stopped, 0);
    assert!(changes(&place).iter().all(|call| return !call.contains(" stop ")), "nothing was stopped: {:?}", changes(&place));
}

/// The origin scopes are stopped before the first adoption,
/// so a scope an earlier watcher left empty is gone before any process can join it.
#[test]
fn the_origin_scopes_are_stopped_before_the_first_adoption() {
    let place = place("origins-before-adoption", 0, 0);
    let empty_origin = "wg-quicker-update-app.slice-xx.scope";
    let held_origin = "wg-quicker-update-app.slice-yy.scope";
    let mut list = unit_line(empty_origin, "active", "running");
    list.push_str(&unit_line(held_origin, "active", "running"));
    std::fs::write(&place.listing, list).expect("write the list");
    let _empty = scope_directory(&place, empty_origin, false);
    let _held = scope_directory(&place, held_origin, true);
    let report = stop_empty_origin_scopes(&place.setup, &read_populated);
    assert_eq!(report.origins_stopped, 1);
    assert_eq!(report.held, 1);
    let stopping = changes(&place);
    assert!(stopping.iter().any(|call| return call.contains(" stop ") && call.contains("app.slice-xx.scope")), "{stopping:?}");
    assert!(stopping.iter().all(|call| return !call.contains("app.slice-yy.scope")), "the scope that holds a process survives: {stopping:?}");
}

/// A fake `systemctl` that behaves,
/// for `stop`,
/// like the user's manager on one point:
/// it ends every process the stopped scope holds,
/// and records each one it ended.
/// It also moves a process into the scope right before it handles the stop,
/// which is what another watcher's adoption does,
/// and it does that only while it holds the adoption gate,
/// exactly as the production adoption path does.
fn place_with_adoption(name: &str, victim: i32) -> Place {
    let place = place(name, 0, 0);
    let gate = crate::adoption_gate::adoption_gate_path(&place.setup.gate_root, place.setup.uid);
    let scope_parent = scope_cgroup_directory(&place.setup.cgroup_root, place.setup.uid, SLICE, "probe.scope")
        .parent()
        .expect("the slice directory")
        .to_path_buf();
    let body = format!(
        "#!/bin/sh\necho \"$*\" >> '{log}'\ncase \"$2\" in\nlist-units) cat '{listing}'; exit 0 ;;\nstop)\n  shift 2\n  for unit in \"$@\"; do\n    directory='{scopes}'/\"$unit\"\n    if [ -d \"$directory\" ] && flock --exclusive --nonblock '{gate}' true; then\n      printf '%s\\n' {victim} > \"$directory/cgroup.procs\"\n      echo \"move-landed {victim} $unit\" >> '{log}'\n    else\n      echo \"move-deferred {victim} $unit\" >> '{log}'\n    fi\n    if [ -s \"$directory/cgroup.procs\" ]; then\n      for ended in $(cat \"$directory/cgroup.procs\"); do\n        kill -9 \"$ended\" 2>/dev/null\n        echo \"ended $ended $unit\" >> '{log}'\n      done\n    fi\n  done\n  exit 0 ;;\n*) exit 0 ;;\nesac\n",
        log = place.log.display(),
        listing = place.listing.display(),
        gate = gate.display(),
        victim = victim,
        scopes = scope_parent.display(),
    );
    let program = place.setup.client.program.clone();
    std::fs::write(&program, body).expect("write the fake manager");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("make the fake manager executable");
    return place;
}

/// An origin stop is read and stopped under the adoption gate,
/// so a process another watcher moves into that scope just then is not ended by the stop.
///
/// The manager ends every process in a scope it stops,
/// and the read of `populated 0` cannot be one step with the stop:
/// the fake manager here moves a real process into the scope at the instant it handles the stop,
/// which is the window a second watcher's adoption lands in,
/// and then ends what the scope holds.
/// With the gate held across the read and the stop,
/// that move is refused inside the window,
/// so the stop finds nothing to end.
#[test]
fn a_move_into_an_origin_scope_during_its_stop_is_refused_and_no_process_is_ended() {
    let mut victim = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the process another watcher moves");
    let place = place_with_adoption("origin-stop-gate", i32::try_from(victim.id()).expect("pid fits"));
    let origin = "wg-quicker-update-app.slice-zz.scope";
    std::fs::write(&place.listing, unit_line(origin, "active", "running")).expect("write the list");
    let _directory = scope_directory(&place, origin, false);
    let report = stop_empty_origin_scopes(&place.setup, &read_populated);
    let logged = calls(&place).join("\n");
    let ended = victim.try_wait().expect("ask whether the process ended");
    let _killed = victim.kill();
    let _reaped = victim.wait();
    assert!(!logged.contains("ended "), "the stop ended a process another watcher moved in: {logged} {report:?}");
    assert!(logged.contains(&format!("move-deferred {}", victim.id())), "the move was not refused inside the stop window: {logged}");
    assert!(ended.is_none(), "the process another watcher moved in was ended: {report:?}");
}

/// A batch whose adoption gate stays held is left for the next watcher start,
/// and the line says how many scopes that was and what was waited for.
#[test]
fn an_origin_batch_whose_gate_stays_held_is_left_and_reported() {
    let place = place("origin-gate-held", 0, 0);
    let origin = "wg-quicker-update-app.slice-held.scope";
    std::fs::write(&place.listing, unit_line(origin, "active", "running")).expect("write the list");
    let _directory = scope_directory(&place, origin, false);
    let held = crate::adoption_gate::try_acquire(&place.setup.gate_root, place.setup.uid).expect("take the gate").expect("the gate is free");
    let report = stop_empty_origin_scopes(&place.setup, &read_populated);
    assert_eq!(report.origins_deferred, 1, "{report:?}");
    assert_eq!(report.origins_stopped, 0, "{report:?}");
    assert!(changes(&place).iter().all(|call| return !call.contains(" stop ")), "an origin scope was stopped under another holder: {:?}", changes(&place));
    let line = describe_sweep(&report).expect("the deferral is said");
    assert!(line.contains("left for the next watcher start"), "{line}");
    assert!(line.contains("adoption gate"), "{line}");
    drop(held);
}
