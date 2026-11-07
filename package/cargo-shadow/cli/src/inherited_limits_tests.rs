//! What:
//! The tests of the limits a run's scope keeps from its caller:
//! the tightest value of each limit read from a fixture control group tree,
//! limits that stay absent,
//! the `default` line of `io.weight`,
//! the properties the launcher passes to `systemd-run`,
//! and, on a host with a user manager, the limits a real scope carries.
//! Why:
//! A run under a capped `systemd-run --user --scope` ran uncapped, because its scope sits under the manager's slice
//! (contract `doc/planning/cargo-shadow-guarantees.md`, "The spawn route", clause on the caller's limits).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('limits kept from the caller', () => { it('takes the tightest of the ancestors', ...) })
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use crate::cgroup_test_support::delegated_route;
use crate::control_group::{ControlGroup, Route, RouteSettings, detect_route};
use crate::inherited_limits::{InheritedLimits, caller_limits, own_cgroup, read_inherited_limits};
use crate::test_scratch::scratch;

/// The kernel's default I/O weight, which is no weight to carry.
const KERNEL_DEFAULT_IO_WEIGHT: u64 = 100;

/// Microseconds of CPU time in one second, the unit of the quota the tests compare.
const SECOND_IN_MICROSECONDS: u64 = 1_000_000;

/// The file of a cgroup directory in a fixture tree, made with its parents.
fn write_limit(root: &Path, cgroup: &str, name: &str, text: &str) {
    let directory = root.join(cgroup.trim_start_matches('/'));
    fs::create_dir_all(&directory).expect("make the cgroup directory");
    fs::write(directory.join(name), text).expect("write the limit file");
}

/// The limits of one cgroup directory of a fixture tree, written as the kernel prints them.
fn write_all(root: &Path, cgroup: &str, memory: &str, cpu: &str, pids: &str, io_weight: &str) {
    write_limit(root, cgroup, "memory.max", memory);
    write_limit(root, cgroup, "cpu.max", cpu);
    write_limit(root, cgroup, "pids.max", pids);
    write_limit(root, cgroup, "io.weight", io_weight);
}

/// The fixture tree of the tightest-of-ancestors tests:
/// each of the four limits is tightest on a different cgroup of the path `/a/b/c`.
fn tree_where_each_limit_is_tightest_somewhere(root: &Path) {
    write_all(root, "/a", "1000\n", "200000 100000\n", "50\n", "default 80\n");
    write_all(root, "/a/b", "500\n", "50000 100000\n", "max\n", "default 40\n8:0 10\n");
    write_all(root, "/a/b/c", "max\n", "max 100000\n", "70\n", "default 100\n");
}

/// Acceptance (limits kept): the tightest finite value of each limit among the cgroup and its ancestors is the one carried.
/// Memory is tightest on `/a/b`, tasks on `/a`, CPU on `/a/b` (a period that is not a second is converted), I/O weight on `/a/b`.
#[test]
fn the_tightest_value_of_each_limit_on_the_way_up_is_the_one_carried() {
    let root = scratch();
    tree_where_each_limit_is_tightest_somewhere(root.path());

    let limits = read_inherited_limits(root.path(), "/a/b/c", "/elsewhere").expect("the limits");

    assert_eq!(
        limits,
        InheritedLimits { memory_max: Some(500), cpu_quota_per_sec_usec: Some(SECOND_IN_MICROSECONDS / 2), tasks_max: Some(50), io_weight: Some(40) }
    );
}

/// The ancestors the origin shares with the slice keep applying by ancestry,
/// so they are not read: a tight limit on a shared ancestor is not copied onto the scope.
#[test]
fn limits_of_the_ancestors_the_slice_shares_are_not_copied() {
    let root = scratch();
    tree_where_each_limit_is_tightest_somewhere(root.path());
    write_all(root.path(), "/a", "1\n", "1000 100000\n", "1\n", "default 1\n");

    let limits = read_inherited_limits(root.path(), "/a/b/c", "/a/slice").expect("the limits");

    assert_eq!(
        limits,
        InheritedLimits { memory_max: Some(500), cpu_quota_per_sec_usec: Some(SECOND_IN_MICROSECONDS / 2), tasks_max: Some(70), io_weight: Some(40) }
    );
}

/// A limit that nobody sets stays absent: `max` files, missing files (a controller that is off) and a default weight give no limit,
/// and no property is passed for them.
#[test]
fn limits_nobody_sets_stay_absent() {
    let root = scratch();
    write_all(root.path(), "/a", "max\n", "max 100000\n", "max\n", "default 100\n");
    write_limit(root.path(), "/a/b", "cgroup.procs", "");
    write_limit(root.path(), "/a/b/c", "memory.max", "max\n");

    let limits = read_inherited_limits(root.path(), "/a/b/c", "/elsewhere").expect("the limits");

    assert_eq!(limits, InheritedLimits::default());
    assert!(limits.properties().is_empty(), "a property was made for a limit nobody set: {:?}", limits.properties());
}

/// `io.weight` has a `default` line and lines for single devices:
/// only the `default` line is the weight of the cgroup, and a weight at or above the kernel default is not carried.
#[test]
fn the_io_weight_is_the_default_line_and_never_a_device_line() {
    let cases: [(&str, Option<u64>); 4] = [
        ("default 5\n8:0 50\n", Some(5)),
        ("default 100\n8:0 5\n", None),
        ("default 500\n", None),
        ("8:0 5\ndefault 20\n", Some(20)),
    ];
    for (text, expected) in cases {
        let root = scratch();
        write_limit(root.path(), "/a", "io.weight", text);

        let limits = read_inherited_limits(root.path(), "/a", "/elsewhere").expect("the limits");

        assert_eq!(limits.io_weight, expected, "io.weight {text:?} (the kernel default is {KERNEL_DEFAULT_IO_WEIGHT})");
    }
}

/// A limit file that cannot be understood or read is an error that names the file, never "no limit":
/// reading it as absent could loosen the run.
#[test]
fn a_limit_file_that_cannot_be_read_is_an_error_that_names_it() {
    let garbage = scratch();
    write_limit(garbage.path(), "/a", "memory.max", "plenty\n");
    let unreadable = scratch();
    fs::create_dir_all(unreadable.path().join("a/pids.max")).expect("a directory where a file is expected");
    let no_default = scratch();
    write_limit(no_default.path(), "/a", "io.weight", "8:0 5\n");
    let zero_period = scratch();
    write_limit(zero_period.path(), "/a", "cpu.max", "100 0\n");

    for (root, name) in [(&garbage, "memory.max"), (&unreadable, "pids.max"), (&no_default, "io.weight"), (&zero_period, "cpu.max")] {
        let refused = read_inherited_limits(root.path(), "/a", "/elsewhere").expect_err(name);
        assert!(refused.to_string().contains(name), "{refused}");
    }
}

/// The text the limits are passed in is what `systemd-run --property=` reads:
/// the spelling and the order, with the quota as a percent rounded down and never to nothing.
#[test]
fn the_limits_are_spelled_as_properties_systemd_run_reads() {
    let limits = InheritedLimits { memory_max: Some(1_073_741_824), cpu_quota_per_sec_usec: Some(SECOND_IN_MICROSECONDS), tasks_max: Some(64), io_weight: Some(5) };
    assert_eq!(limits.properties(), ["MemoryMax=1073741824", "CPUQuota=100.00%", "TasksMax=64", "IOWeight=5"]);

    let quotas: [(u64, &str); 4] = [(1_234_567, "CPUQuota=123.45%"), (12_345, "CPUQuota=1.23%"), (50, "CPUQuota=0.01%"), (2_500_000, "CPUQuota=250.00%")];
    for (quota, expected) in quotas {
        let only_cpu = InheritedLimits { cpu_quota_per_sec_usec: Some(quota), ..InheritedLimits::default() };
        assert_eq!(only_cpu.properties(), [expected], "a quota of {quota} microseconds per second");
    }
}

/// The cgroup v2 path is the `0::` line of `/proc/<pid>/cgroup`; other lines and a missing line are not a path.
#[test]
fn the_own_cgroup_is_the_unified_line_of_the_proc_file() {
    assert_eq!(own_cgroup("0::/user.slice/run.scope\n").expect("a path"), "/user.slice/run.scope");
    assert_eq!(own_cgroup("1:name=systemd:/legacy\n0::/unified\n").expect("a path"), "/unified");
    assert_eq!(own_cgroup("0::/gone.scope (deleted)\n").expect("a path"), "/gone.scope");
    assert!(own_cgroup("1:name=systemd:/legacy\n").is_err(), "a v1-only file has no cgroup v2 path");
}

/// The caller's limits are read from the file that names its group and the slice the manager reported:
/// the groups below their common ancestor are read, and a path that leaves the mount is an error.
#[test]
fn the_callers_limits_come_from_its_cgroup_file_and_the_slice() {
    let root = scratch();
    let own = root.path().join("own-cgroup");
    fs::write(&own, "0::/user.slice/caller.scope\n").expect("the cgroup file");
    write_all(root.path(), "/user.slice", "1\n", "max 100000\n", "1\n", "default 100\n");
    write_all(root.path(), "/user.slice/app.slice", "max\n", "max 100000\n", "max\n", "default 100\n");
    write_all(root.path(), "/user.slice/caller.scope", "1073741824\n", "100000 100000\n", "64\n", "default 5\n");

    let limits = caller_limits(&own, root.path(), &root.path().join("user.slice/app.slice")).expect("the limits");

    assert_eq!(limits, InheritedLimits { memory_max: Some(1_073_741_824), cpu_quota_per_sec_usec: Some(SECOND_IN_MICROSECONDS), tasks_max: Some(64), io_weight: Some(5) });

    fs::write(&own, "0::/../escaped\n").expect("the cgroup file");
    assert!(caller_limits(&own, root.path(), &root.path().join("user.slice/app.slice")).is_err(), "a path above the mount was read");
}

/// A route whose scopes carry the given limits, for the launcher tests.
fn route_with(limits: InheritedLimits) -> Route {
    return Route { systemd_run: "systemd-run".to_string(), slice: "app.slice".to_string(), slice_dir: "/sys/fs/cgroup/app.slice".into(), namespace: 1, slice_view: crate::control_group::View { mount: 1, inode: 1 }, limits: Arc::new(limits) };
}

/// The arguments of a launcher, as text.
fn arguments_of(command: &Command) -> Vec<String> {
    return command.get_args().map(|argument| { return argument.to_string_lossy().to_string(); }).collect();
}

/// The launcher passes exactly the caller's limits as properties, after `Delegate=yes` and before the unit and the program.
#[test]
fn the_launcher_carries_exactly_the_callers_limits_as_properties() {
    let limits = InheritedLimits { memory_max: Some(1_073_741_824), cpu_quota_per_sec_usec: Some(SECOND_IN_MICROSECONDS), tasks_max: Some(64), io_weight: Some(5) };
    let route = route_with(limits);
    let planned = ControlGroup::plan(&route).expect("a scope name");

    let launch = planned.launcher(&route, Command::new("true").arg("--flag"), false);

    let name = planned.directory().file_stem().expect("the unit name").to_string_lossy().to_string();
    assert_eq!(
        arguments_of(&launch),
        [
            "--user".to_string(),
            "--scope".to_string(),
            "--quiet".to_string(),
            "--collect".to_string(),
            "--property=Delegate=yes".to_string(),
            "--property=MemoryMax=1073741824".to_string(),
            "--property=CPUQuota=100.00%".to_string(),
            "--property=TasksMax=64".to_string(),
            "--property=IOWeight=5".to_string(),
            format!("--unit={name}"),
            "--slice=app.slice".to_string(),
            "--".to_string(),
            "true".to_string(),
            "--flag".to_string(),
        ]
    );
}

/// A caller with no limits gets the argument list it always had: no property is invented.
#[test]
fn a_caller_without_limits_gets_no_limit_properties() {
    let route = route_with(InheritedLimits::default());
    let planned = ControlGroup::plan(&route).expect("a scope name");

    let launch = planned.launcher(&route, &Command::new("true"), false);

    let properties: Vec<String> = arguments_of(&launch).into_iter().filter(|argument| { return argument.starts_with("--property="); }).collect();
    assert_eq!(properties, ["--property=Delegate=yes"]);
}

/// The discovery of the route refuses a caller whose limits cannot be read (the run then takes the weaker path, which stays in the
/// caller's own group, so its limits still apply), and names the limits in the reason.
#[test]
fn a_route_is_refused_when_the_callers_limits_cannot_be_read() {
    if delegated_route().is_none() {
        return;
    }
    let root = scratch();
    let own = root.path().join("own-cgroup");
    fs::write(&own, "no unified line here\n").expect("the cgroup file");

    let refused = detect_route(&RouteSettings { own_cgroup_file: own, ..RouteSettings::default() }).expect_err("an unreadable caller group is a missing capability");

    assert!(refused.to_string().contains("resource limits"), "{refused}");
}

/// The values of the real-scope test, each named so no number stands alone.
const SCOPE_MEMORY_MAX: u64 = 1_073_741_824;
/// Half a CPU, in microseconds per second.
const SCOPE_CPU_QUOTA: u64 = SECOND_IN_MICROSECONDS / 2;
/// A task limit well above what the one probe process needs.
const SCOPE_TASKS_MAX: u64 = 64;
/// The period, in microseconds, that the manager gives a `CPUQuota=` in `cpu.max`.
const SCOPE_CPU_PERIOD: u64 = 100_000;
/// An I/O weight below the kernel default.
const SCOPE_IO_WEIGHT: u64 = 7;

/// On a host with a user manager, a scope started through the launcher carries the limits of its route:
/// the manager accepts every spelling and the scope's own files read back the same values.
/// The program of the scope prints its own `memory.max`, `cpu.max`, `pids.max` and `io.weight`.
#[test]
fn a_real_scope_carries_the_limits_of_its_route() {
    let Some(found) = delegated_route() else {
        return;
    };
    let route = Route {
        limits: Arc::new(InheritedLimits { memory_max: Some(SCOPE_MEMORY_MAX), cpu_quota_per_sec_usec: Some(SCOPE_CPU_QUOTA), tasks_max: Some(SCOPE_TASKS_MAX), io_weight: Some(SCOPE_IO_WEIGHT) }),
        ..found
    };
    let planned = ControlGroup::plan(&route).expect("a scope name");
    let mut program = Command::new("sh");
    program.arg("-c").arg("d=/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup); for f in memory.max cpu.max pids.max io.weight; do echo \"$f=$(cat $d/$f | tr '\\n' ' ')\"; done");

    let output = planned.launcher(&route, &program, false).output().expect("the launcher runs");

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(output.status.success(), "the launcher failed: {}", String::from_utf8_lossy(&output.stderr));
    // The manager gives a `CPUQuota=` the period it has chosen (one tenth of a second, measured 2026-11-07), so the quota per period is a tenth.
    let expected = [
        format!("memory.max={SCOPE_MEMORY_MAX}"),
        format!("cpu.max={} {SCOPE_CPU_PERIOD}", SCOPE_CPU_QUOTA * SCOPE_CPU_PERIOD / SECOND_IN_MICROSECONDS),
        format!("pids.max={SCOPE_TASKS_MAX}"),
        format!("io.weight=default {SCOPE_IO_WEIGHT}"),
    ];
    for line in expected {
        assert!(text.lines().any(|found_line| { return found_line.trim_end() == line; }), "{line:?} is not in the scope's files:\n{text}");
    }
}
