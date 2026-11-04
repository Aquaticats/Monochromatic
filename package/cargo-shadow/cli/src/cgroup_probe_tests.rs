//! What:
//! The probes of round 7 as tests:
//! what a delegated control group does on the host, asserted so that a kernel or a host that differs is noticed.
//! Why:
//! The approach stands on these facts (`doc/planning/cargo-shadow-update.md`, "Round 7: control group probes");
//! each test name carries the number of its probe.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('control group probes', () => { it('probe 2: a zombie leader with a live thread stays populated', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::cgroup_test_support::test_group;
use crate::contain_counterexample_tests::{compile_leader, is_running};
use crate::control_group::{ControlGroup, Membership, Record, parse_record, query};
use crate::test_scratch::scratch;

/// How long a probe waits for a process to reach the state it is looking for.
const STATE_WAIT: Duration = Duration::from_secs(20);

/// How long a stop may take in these probes.
const STOP: Duration = Duration::from_secs(10);

/// Waits until the condition holds, up to `STATE_WAIT`.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let deadline: Instant = Instant::now() + STATE_WAIT;
    while !condition() {
        assert!(Instant::now() < deadline, "{what} did not happen in time");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The state letter of a process in `/proc/<pid>/stat`, or `None` when it is gone.
fn state_of(pid: i32) -> Option<char> {
    let stat: String = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    return stat.rsplit(')').next()?.trim_start().chars().next();
}

/// Probe 1:
/// a child of the process's own group can be made and removed by an unprivileged process.
#[test]
fn probe_1_a_child_of_the_own_group_can_be_made_and_removed() {
    let Some(made) = test_group() else {
        return;
    };

    assert!(made.group.directory().join("cgroup.procs").exists());
    assert!(!made.group.populated().expect("reads the events"), "a new group has no member");
    made.group.remove().expect("an empty group is removed");
    assert!(!made.group.directory().exists());
}

/// Probe 1:
/// a directory that is not on cgroup v2 is reported by the capability it lacks,
/// and nothing is left behind.
#[test]
fn probe_1_a_directory_that_is_not_a_control_group_names_the_missing_capability() {
    let directory = scratch();

    let refused = ControlGroup::create_in(directory.path()).expect_err("a plain directory is no control group");

    assert!(refused.to_string().contains("cgroup v2"), "{refused}");
    assert_eq!(fs::read_dir(directory.path()).expect("lists").count(), 0, "a directory was left behind");
}

/// Probe 2:
/// after the kill an ordinary member is gone and the kernel says the group is unpopulated.
#[test]
fn probe_2_an_ordinary_member_is_stopped_and_the_group_reports_unpopulated() {
    let Some(made) = test_group() else {
        return;
    };
    let mut child = made.group.spawn(Command::new("sleep").arg("600")).expect("spawns in the group");
    assert!(made.group.populated().expect("reads"), "a member makes the group populated");

    let stopped: bool = made.group.stop(STOP).expect("stops");
    let _ = child.wait();

    assert!(stopped);
    assert!(!made.group.populated().expect("reads"));
    assert!(made.group.members().expect("lists").is_empty());
}

/// Probe 2 (the sixth review's counterexample, 6a R1 A):
/// a thread group whose leader exited while another thread lives is listed, is populated, and is stopped.
#[test]
fn probe_2_a_thread_group_with_an_exited_leader_stays_populated_until_it_is_stopped() {
    let Some(made) = test_group() else {
        return;
    };
    let directory = scratch();
    let leader: PathBuf = compile_leader(directory.path());
    let target: PathBuf = directory.path().join("target");
    let mut child = made.group.spawn(Command::new(&leader).arg("600").arg(&target)).expect("spawns in the group");
    let pid: i32 = i32::try_from(child.id()).expect("a process id");
    wait_until("the leader thread's end", &|| { return state_of(pid) == Some('Z'); });

    let listed: Vec<i32> = made.group.members().expect("lists");
    let populated: bool = made.group.populated().expect("reads");
    let threads: String = fs::read_to_string(made.group.directory().join("cgroup.threads")).expect("threads");
    let stopped: bool = made.group.stop(STOP).expect("stops");
    let _ = child.wait();

    assert_eq!(listed, vec![pid], "the process is listed by its leader's id");
    assert!(populated, "a live thread of a zombie leader makes the group populated");
    assert!(threads.lines().any(|line| { return line.trim() != pid.to_string(); }), "the live thread is listed: {threads:?}");
    assert!(stopped, "the group was not stopped");
    assert!(!made.group.populated().expect("reads"));
}

/// Probe 2:
/// a zombie that nobody reaped, alone in the group, does not make it populated,
/// so the direct child must be reaped on its own.
#[test]
fn probe_2_an_unreaped_zombie_alone_does_not_make_the_group_populated() {
    let Some(made) = test_group() else {
        return;
    };
    let mut child = made.group.spawn(&mut Command::new("true")).expect("spawns in the group");
    let pid: i32 = i32::try_from(child.id()).expect("a process id");
    wait_until("the exit of `true`", &|| { return state_of(pid) == Some('Z'); });

    let populated: bool = made.group.populated().expect("reads");
    let listed: Vec<i32> = made.group.members().expect("lists");
    let _ = child.wait();

    assert!(!populated);
    assert!(listed.is_empty(), "{listed:?}");
}

/// Probe 3:
/// a descendant that calls `setsid` and one that is reparented after a double fork are still members.
#[test]
fn probe_3_a_setsid_and_double_fork_descendant_stays_a_member() {
    let Some(made) = test_group() else {
        return;
    };
    let script: &str = "setsid sh -c '(sleep 600 &) ; exec sleep 600' </dev/null >/dev/null 2>&1 & exec sleep 600";
    let mut child = made.group.spawn(Command::new("sh").args(["-c", script])).expect("spawns in the group");

    wait_until("three members", &|| { return made.group.members().map(|found| { return found.len() >= 3; }).unwrap_or(false); });
    let sessions: std::collections::BTreeSet<String> = made
        .group
        .members()
        .expect("lists")
        .iter()
        .filter_map(|pid| { return fs::read_to_string(format!("/proc/{pid}/stat")).ok(); })
        .map(|stat| { return stat.rsplit(')').next().unwrap_or("").split_whitespace().nth(3).unwrap_or("").to_string(); })
        .collect();
    let stopped: bool = made.group.stop(STOP).expect("stops");
    let _ = child.wait();

    assert!(sessions.len() >= 2, "the descendants did not leave the session: {sessions:?}");
    assert!(stopped);
}

/// Probe 3, the limit:
/// a member of the same user that writes itself into another group's `cgroup.procs` leaves the group.
/// The group contains ordinary escapes, not a deliberate one;
/// if this ever fails, the limit in the README and the planning record can be lifted.
#[test]
fn probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_the_stated_limit() {
    let Some(made) = test_group() else {
        return;
    };
    let mut child = made
        .group
        .spawn(Command::new("sh").args(["-c", "echo 0 > \"$1\"/cgroup.procs; exec sleep 600", "sh"]).arg(&made.parent))
        .expect("spawns in the group");
    let pid: i32 = i32::try_from(child.id()).expect("a process id");

    wait_until("the member's move out", &|| { return made.group.members().map(|found| { return found.is_empty(); }).unwrap_or(false); });
    let escaped_alive: bool = is_running(pid);
    let _ = child.kill();
    let _ = child.wait();

    assert!(escaped_alive, "the member that left should be running outside the group");
}

/// Probe 4:
/// a group is found again by its record alone while a member lives,
/// cannot be removed meanwhile,
/// and is `Gone` once it is empty or a different group sits at the path.
#[test]
fn probe_4_a_group_is_found_by_its_record_and_cannot_be_removed_while_a_member_lives() {
    let Some(made) = test_group() else {
        return;
    };
    let line: String = made.group.record().to_string();
    let Record::Group { id, namespace, path } = parse_record(&line).expect("the record parses") else {
        panic!("a control group record was read as another kind");
    };
    let mut child = made.group.spawn(Command::new("sleep").arg("600")).expect("spawns in the group");

    let alive: Membership = query(id, namespace, &path);
    let removal = made.group.remove();
    let replaced: Membership = query(id + 1, namespace, &path);
    let other_namespace: Membership = query(id, namespace + 1, &path);
    let stopped: bool = made.group.stop(STOP).expect("stops");
    let _ = child.wait();
    let empty: Membership = query(id, namespace, &path);
    made.group.remove().expect("an empty group is removed");
    let removed: Membership = query(id, namespace, &path);

    assert_eq!(alive, Membership::Alive);
    assert!(removal.is_err(), "a group with a member was removed");
    assert_eq!(replaced, Membership::Gone, "a group with another id is not the recorded one");
    assert!(matches!(other_namespace, Membership::Unknown(_)), "{other_namespace:?}");
    assert!(stopped);
    assert_eq!(empty, Membership::Gone);
    assert_eq!(removed, Membership::Gone);
}

/// Probe 5:
/// a child started through the group is a member from its first instruction, so are its children,
/// and the spawning process is back in its own group.
#[test]
fn probe_5_a_child_inherits_the_group_without_a_hook_and_the_spawner_stays_out() {
    let Some(made) = test_group() else {
        return;
    };
    let before: String = fs::read_to_string("/proc/self/cgroup").expect("own group");

    let mut child = made.group.spawn(Command::new("sh").args(["-c", "sleep 600 & exec sleep 600"])).expect("spawns in the group");
    let after: String = fs::read_to_string("/proc/self/cgroup").expect("own group");
    wait_until("two members", &|| { return made.group.members().map(|found| { return found.len() >= 2; }).unwrap_or(false); });
    let stopped: bool = made.group.stop(STOP).expect("stops");
    let _ = child.wait();

    assert_eq!(before, after, "the spawning process did not return to its own group");
    assert!(stopped);
}

/// The journal lines round trip, and a damaged line is rejected with the line in the reason.
#[test]
fn the_record_lines_round_trip_and_damaged_lines_are_rejected() {
    let group: Record = Record::Group { id: 7, namespace: 9, path: Path::new("/sys/fs/cgroup/a b/c").to_path_buf() };
    let process_group: Record = Record::ProcessGroup(4242);

    assert_eq!(parse_record(&group.to_string()), Ok(group));
    assert_eq!(parse_record(&process_group.to_string()), Ok(process_group));
    for damaged in ["", "4242", "pgroup x", "pgroup 0", "cgroup 1 2", "cgroup a 2 /x", "cgroup 1 b /x", "group 1 2 /x"] {
        let reason: String = parse_record(damaged).expect_err("a damaged line");
        assert!(reason.contains(&format!("{damaged:?}")), "{reason}");
    }
}
