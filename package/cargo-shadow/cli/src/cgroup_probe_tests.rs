//! What:
//! The probes of rounds 7 and 8 as tests:
//! what a delegated scope does on the host, asserted so that a kernel or a host that differs is noticed.
//! Why:
//! The approach stands on these facts (`doc/planning/cargo-shadow-update.md`, "Round 7: control group probes", and the probe of the
//! scope route in `doc/planning/cargo-shadow-guarantees.md`).
//! Every test that needs a scope starts it through `start_in_scope`, the launcher a run uses, so it shows that delegated containment
//! was in force.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('scope probes', () => { it('probe 2: a zombie leader with a live thread stays populated', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::cgroup_test_support::{delegated_route, start_in_scope, wait_until};
use crate::contain_counterexample_tests::{compile_leader, is_running};
use crate::control_group::{Membership, Record, RouteSettings, View, detect_route, kill_file_present, parse_record, query, view_of};
use crate::test_scratch::scratch;

/// How long a stop may take in these probes.
const STOP: Duration = Duration::from_secs(10);

/// The state letter of a process in `/proc/<pid>/stat`, or `None` when it is gone.
fn state_of(pid: i32) -> Option<char> {
    let stat: String = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    return stat.rsplit(')').next()?.trim_start().chars().next();
}

/// The group a process is in, as `/proc/<pid>/cgroup` says.
fn group_of(pid: &str) -> String {
    return fs::read_to_string(format!("/proc/{pid}/cgroup")).expect("a group").trim().to_string();
}

/// Probe 1:
/// the program started through the launcher runs in the scope that was planned,
/// and the starting process stays in its own group.
#[test]
fn probe_1_a_delegated_scope_is_started_and_the_program_runs_in_it() {
    let before: String = group_of("self");
    let Some(scope) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };

    let members: Vec<i32> = scope.group.members().expect("lists");
    let program_group: String = group_of(&scope.child.id().to_string());

    let scope_name: String = scope.group.directory().file_name().expect("a name").to_string_lossy().to_string();
    assert!(program_group.ends_with(&scope_name), "{program_group} should end with {scope_name}");
    assert_eq!(members, vec![i32::try_from(scope.child.id()).expect("a process id")], "the program replaced systemd-run, so its pid is the member");
    assert_eq!(group_of("self"), before, "the starting process changed its group");
}

/// Probe 1:
/// a host without a cgroup v2 mount names the missing capability.
#[test]
fn probe_1_a_host_without_a_cgroup_v2_mount_names_the_missing_capability() {
    let plain = scratch();
    let settings: RouteSettings = RouteSettings { cgroup_root: plain.path().to_path_buf(), ..RouteSettings::default() };

    let refused = detect_route(&settings).expect_err("a plain directory is no cgroup v2 mount");

    assert!(refused.to_string().contains("cgroup v2"), "{refused}");
}

/// Probe 1:
/// a kernel whose groups have no `cgroup.kill` (before Linux 5.14) is told apart by the file.
#[test]
fn probe_1_a_group_directory_without_cgroup_kill_is_an_older_kernel() {
    let plain = scratch();
    assert!(!kill_file_present(plain.path()));
    fs::write(plain.path().join("cgroup.kill"), "").expect("the file");
    assert!(kill_file_present(plain.path()));
}

/// Probe 2:
/// after the stop an ordinary member is gone and the scope is unpopulated or verifiably absent.
#[test]
fn probe_2_an_ordinary_member_is_stopped_and_the_scope_is_empty_or_absent() {
    let Some(mut scope) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    assert!(scope.group.populated().expect("reads"), "a member makes the scope populated");

    let _ = scope.child.kill();
    let stopped: bool = scope.group.stop(STOP).expect("stops");
    let _ = scope.child.wait();

    assert!(stopped);
    assert!(scope.group.members().expect("lists").is_empty());
}

/// Probe 2 (the sixth review's counterexample, 6a R1 A):
/// a thread group whose leader exited while another thread lives is listed, is populated, and is stopped.
#[test]
fn probe_2_a_thread_group_with_an_exited_leader_stays_populated_until_it_is_stopped() {
    let directory = scratch();
    let leader: PathBuf = compile_leader(directory.path());
    let target: PathBuf = directory.path().join("target");
    let Some(mut scope) = start_in_scope(Command::new(&leader).arg("600").arg(&target)) else {
        return;
    };
    let pid: i32 = i32::try_from(scope.child.id()).expect("a process id");
    wait_until("the leader thread's end", &|| { return state_of(pid) == Some('Z'); });

    let listed: Vec<i32> = scope.group.members().expect("lists");
    let populated: bool = scope.group.populated().expect("reads");
    let threads: String = fs::read_to_string(scope.group.directory().join("cgroup.threads")).expect("threads");
    let stopped: bool = scope.group.stop(STOP).expect("stops");
    let _ = scope.child.wait();

    assert_eq!(listed, vec![pid], "the process is listed by its leader's id");
    assert!(populated, "a live thread of a zombie leader makes the scope populated");
    assert!(threads.lines().any(|line| { return line.trim() != pid.to_string(); }), "the live thread is listed: {threads:?}");
    assert!(stopped, "the scope was not stopped");
}

/// Probe 2 (acceptance row "rapid empty-scope collection and the chosen valid-absence rule"):
/// a scope whose program ended at once is collected by the manager,
/// and its absence is accepted as proof because the directory that held it is seen as it was planned.
#[test]
fn probe_2_a_scope_that_is_collected_at_once_is_accepted_as_empty_by_verified_absence() {
    let Some(mut scope) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let _ = scope.child.kill();
    let _ = scope.child.wait();
    let directory: PathBuf = scope.group.directory().to_path_buf();
    wait_until("the manager's collection of the empty scope", &|| { return !directory.exists(); });

    let stopped: bool = scope.group.stop(STOP).expect("an absent scope is stopped by verified absence");

    assert!(stopped);
}

/// Contract A1 (the invariant of the absence rule, probed):
/// a scope whose program exited while a descendant lives stays observable and populated,
/// and the manager does not move that descendant elsewhere.
#[test]
fn probe_2_a_scope_stays_observable_while_a_descendant_outlives_the_program() {
    let Some(mut scope) = start_in_scope(Command::new("sh").args(["-c", "(setsid sleep 600 </dev/null >/dev/null 2>&1 &); exit 0"])) else {
        return;
    };
    let _ = scope.child.wait();

    let populated: bool = scope.group.populated().expect("the scope is observable");
    let members: Vec<i32> = scope.group.members().expect("lists");

    assert!(populated);
    assert_eq!(members.len(), 1, "{members:?}");
    assert!(group_of(&members[0].to_string()).contains(".scope"), "the descendant was moved out of its scope");
    assert!(scope.group.stop(STOP).expect("stops"));
}

/// Probe 3:
/// a descendant that calls `setsid` and one that is reparented after a double fork are still members.
#[test]
fn probe_3_a_setsid_and_double_fork_descendant_stays_a_member() {
    let script: &str = "setsid sh -c '(sleep 600 &) ; exec sleep 600' </dev/null >/dev/null 2>&1 & exec sleep 600";
    let Some(mut scope) = start_in_scope(Command::new("sh").args(["-c", script])) else {
        return;
    };

    wait_until("three members", &|| { return scope.group.members().map(|found| { return found.len() >= 3; }).unwrap_or(false); });
    let sessions: std::collections::BTreeSet<String> = scope
        .group
        .members()
        .expect("lists")
        .iter()
        .filter_map(|pid| { return fs::read_to_string(format!("/proc/{pid}/stat")).ok(); })
        .map(|stat| { return stat.rsplit(')').next().unwrap_or("").split_whitespace().nth(3).unwrap_or("").to_string(); })
        .collect();
    let stopped: bool = scope.group.stop(STOP).expect("stops");
    let _ = scope.child.wait();

    assert!(sessions.len() >= 2, "the descendants did not leave the session: {sessions:?}");
    assert!(stopped);
}

/// Probe 3, the limit (contract assumption a):
/// a member of the same user that writes itself into another group's `cgroup.procs` leaves the scope.
/// The scope contains ordinary escapes, not a deliberate one;
/// if this ever fails, the assumption can be lifted.
#[test]
fn probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_assumption_a() {
    if delegated_route().is_none() {
        return;
    }
    let own: String = group_of("self");
    let leaf: PathBuf = Path::new("/sys/fs/cgroup").join(own.trim_start_matches("0::").trim_start_matches('/'));
    let Some(mut scope) = start_in_scope(Command::new("sh").args(["-c", "echo 0 > \"$1\"/cgroup.procs; exec sleep 600", "sh"]).arg(&leaf)) else {
        return;
    };
    let pid: i32 = i32::try_from(scope.child.id()).expect("a process id");

    wait_until("the member's move out", &|| { return scope.group.members().map(|found| { return found.is_empty(); }).unwrap_or(false); });
    let escaped_alive: bool = is_running(pid);
    let _ = scope.child.kill();
    let _ = scope.child.wait();

    assert!(escaped_alive, "the member that left should be running outside the scope");
}

/// Probe 4:
/// a scope is found again by its record alone while a member lives,
/// is `Gone` once it is empty,
/// and a record whose cgroup namespace or parent view differs from what this process sees is `Unknown`, never `Gone`.
#[test]
fn probe_4_a_scope_is_found_by_its_record_and_a_different_view_is_unknown() {
    let Some(mut scope) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let line: String = scope.group.record().to_string();
    let Record::Scope { namespace, parent_view, path } = parse_record(&line).expect("the record parses") else {
        panic!("a scope record was read as another kind");
    };

    let alive: Membership = query(namespace, parent_view, &path);
    let other_namespace: Membership = query(namespace + 1, parent_view, &path);
    let other_mount: Membership = query(namespace, View { mount: parent_view.mount + 1, inode: parent_view.inode }, &path);
    let other_inode: Membership = query(namespace, View { mount: parent_view.mount, inode: parent_view.inode + 1 }, &path);
    let _ = scope.child.kill();
    let stopped: bool = scope.group.stop(STOP).expect("stops");
    let _ = scope.child.wait();
    let after: Membership = query(namespace, parent_view, &path);

    assert_eq!(alive, Membership::Alive);
    assert!(matches!(other_namespace, Membership::Unknown(_)), "{other_namespace:?}");
    assert!(matches!(other_mount, Membership::Unknown(_)), "{other_mount:?}");
    assert!(matches!(other_inode, Membership::Unknown(_)), "{other_inode:?}");
    assert!(stopped);
    assert_eq!(after, Membership::Gone);
}

/// Probe 5 (acceptance row "spawn isolation", part of it):
/// the starting process stays in its own group while a scope is started,
/// and an ordinary child another thread starts at the same time stays out of the scope.
#[test]
fn probe_5_the_spawner_never_changes_group_and_an_unrelated_child_stays_out() {
    let before: String = group_of("self");
    let unrelated = std::thread::spawn(|| { return Command::new("sleep").arg("600").spawn().expect("an unrelated child"); });
    let Some(mut scope) = start_in_scope(Command::new("sleep").arg("600")) else {
        let mut other = unrelated.join().expect("thread");
        let _ = other.kill();
        let _ = other.wait();
        return;
    };
    let mut other = unrelated.join().expect("thread");

    let other_group: String = group_of(&other.id().to_string());
    let scope_members: Vec<i32> = scope.group.members().expect("lists");
    let _ = other.kill();
    let _ = other.wait();

    assert_eq!(group_of("self"), before, "the starting process changed its group");
    assert_eq!(other_group, before, "the unrelated child was moved");
    assert!(!scope_members.contains(&i32::try_from(other.id()).expect("a process id")), "{scope_members:?}");
    let _ = scope.child.kill();
}

/// The mount view of a path is stable for one process.
#[test]
fn a_view_is_the_same_twice_and_differs_between_directories() {
    let first = scratch();
    let second = scratch();

    let one: View = view_of(first.path()).expect("views");
    let again: View = view_of(first.path()).expect("views");
    let other: View = view_of(second.path()).expect("views");

    assert_eq!(one, again);
    assert_ne!(one.inode, other.inode);
}

/// The journal lines round trip, and a damaged line is rejected with the line in the reason.
#[test]
fn the_record_lines_round_trip_and_damaged_lines_are_rejected() {
    let scope: Record = Record::Scope { namespace: 9, parent_view: View { mount: 7, inode: 8 }, path: Path::new("/sys/fs/cgroup/a b/c.scope").to_path_buf() };
    let process_group: Record = Record::ProcessGroup(4242);

    assert_eq!(parse_record(&scope.to_string()), Ok(scope));
    assert_eq!(parse_record(&process_group.to_string()), Ok(process_group));
    assert_eq!(parse_record(&Record::ProcessGroupStarting.to_string()), Ok(Record::ProcessGroupStarting));
    for damaged in ["", "4242", "pgroup x", "pgroup 0", "scope 1 2:3", "scope a 2:3 /x", "scope 1 b /x", "scope 1 2:c /x", "cgroup 1 2 /x"] {
        let reason: String = parse_record(damaged).expect_err("a damaged line");
        assert!(reason.contains(&format!("{damaged:?}")), "{reason}");
    }
}
