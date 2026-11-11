//! Verifies the `busctl` arguments,
//! the choice between creating a scope and attaching to it,
//! the confirmation and failure paths,
//! and the stop of a scope that stayed empty,
//! against a fake manager that changes a fake procfs and a fake cgroup tree,
//! and the real manager in an ignored test.

/// The limits a scope carries.
use crate::inherited_limits::{DeviceLimits, InheritedLimits};
/// Adoption under test.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter, BusctlAdopter, Joined, LimitRequest};
/// The arguments under test.
use crate::scope_calls::{attach_arguments, create_arguments, ensure_slice_arguments, set_properties_arguments, stop_arguments};
/// Why a process matched,
/// which the scope description names.
use crate::update_entry_points::MatchReason;
/// Where a scope lives.
use crate::scope_naming::scope_cgroup_directory;
/// The client the adopter runs.
use crate::user_client::UserClient;
/// Executable mode for the fake client.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Slice used by the tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";
/// Scope used by the tests.
const SCOPE: &str = "wg-quicker-update-app.slice-claude\\x2dcode\\x2dbash.scope";
/// Origin cgroup used by the tests.
const ORIGIN: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";

/// The limits of the cgroup the processes left,
/// all of the kinds a scope carries.
fn all_limits() -> InheritedLimits {
    return InheritedLimits {
        cpu_quota_per_sec_usec: Some(8_000_000),
        memory_max: Some(1_073_741_824),
        tasks_max: Some(500),
        io_weight: Some(10),
        ..InheritedLimits::default()
    };
}

/// A new scope is created delegated,
/// collected when it stops,
/// and with the limits as typed properties that the property count covers.
#[test]
fn the_create_arguments_follow_busctl_syntax() {
    let arguments = create_arguments(SCOPE, SLICE, &[10, 11], &InheritedLimits::default(), "wg-quicker: a description of the test scope");
    assert_eq!(
        arguments,
        [
            "--user",
            "call",
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "StartTransientUnit",
            "ssa(sv)a(sa(sv))",
            SCOPE,
            "fail",
            "5",
            "Slice",
            "s",
            SLICE,
            "PIDs",
            "au",
            "2",
            "10",
            "11",
            "Description",
            "s",
            "wg-quicker: a description of the test scope",
            "Delegate",
            "b",
            "true",
            "CollectMode",
            "s",
            "inactive-or-failed",
            "0",
        ]
    );
    let limited = create_arguments(SCOPE, SLICE, &[10], &all_limits(), "the test description");
    assert_eq!(limited[9], "9");
    for (name, value) in [("CPUQuotaPerSecUSec", "8000000"), ("MemoryMax", "1073741824"), ("TasksMax", "500"), ("IOWeight", "10")] {
        assert!(limited.windows(3).any(|window| return window == [name, "t", value]), "{name}");
    }
    assert_eq!(limited.last().map(String::as_str), Some("0"));
}

/// Attaching names the unit,
/// passes an empty subgroup,
/// and lists the pids.
#[test]
fn the_attach_arguments_name_the_unit_and_the_pids() {
    let arguments = attach_arguments(SCOPE, &[10, 11]);
    assert_eq!(arguments[5], "AttachProcessesToUnit");
    assert_eq!(arguments[6], "ssau");
    assert_eq!(arguments[7..], [SCOPE, "", "2", "10", "11"]);
}

/// A limit change sends every limit,
/// the unlimited and default ones as explicit values,
/// because a property left out would keep its old value on the live scope.
#[test]
fn the_limit_change_sends_every_limit_with_unlimited_for_the_absent_ones() {
    let arguments = set_properties_arguments(SCOPE, &InheritedLimits { cpu_quota_per_sec_usec: Some(250_000), ..InheritedLimits::default() });
    assert_eq!(arguments[5], "SetUnitProperties");
    assert_eq!(arguments[6], "sba(sv)");
    assert_eq!(arguments[7..10], [SCOPE, "true", "10"]);
    let unlimited = u64::MAX.to_string();
    for (name, value) in [
        ("CPUQuotaPerSecUSec", "250000"),
        ("MemoryMax", unlimited.as_str()),
        ("MemoryHigh", unlimited.as_str()),
        ("MemorySwapMax", unlimited.as_str()),
        ("TasksMax", unlimited.as_str()),
        ("IOWeight", "100"),
    ] {
        assert!(arguments.windows(3).any(|window| return window == [name, "t", value]), "{name}");
    }
}

/// The limits `io.max` sets per device are sent as arrays of device path and number,
/// with the property count covering each non-empty kind on a new scope,
/// and all four kinds on a live scope,
/// an empty one clearing that kind.
#[test]
fn per_device_limits_are_sent_as_arrays_of_device_paths_and_numbers() {
    let mut limits = InheritedLimits::default();
    limits.io_devices.insert("259:0".to_string(), DeviceLimits { read_bandwidth: Some(1_048_576), write_iops: Some(500), ..DeviceLimits::default() });
    limits.io_devices.insert("8:0".to_string(), DeviceLimits { read_bandwidth: Some(2_097_152), ..DeviceLimits::default() });
    let created = create_arguments(SCOPE, SLICE, &[10], &limits, "the test description");
    assert_eq!(created[9], "7", "five fixed properties and two kinds of device limit");
    let read_at = created.iter().position(|argument| return argument == "IOReadBandwidthMax").expect("the read bandwidth property");
    assert_eq!(created[read_at..read_at + 7], ["IOReadBandwidthMax", "a(st)", "2", "/dev/block/259:0", "1048576", "/dev/block/8:0", "2097152"]);
    let iops_at = created.iter().position(|argument| return argument == "IOWriteIOPSMax").expect("the write iops property");
    assert_eq!(created[iops_at..iops_at + 5], ["IOWriteIOPSMax", "a(st)", "1", "/dev/block/259:0", "500"]);
    assert!(!created.iter().any(|argument| return argument == "IOWriteBandwidthMax" || argument == "IOReadIOPSMax"), "a kind no device has is left out of a new scope");
    let live = set_properties_arguments(SCOPE, &limits);
    assert_eq!(live[9], "10");
    let live_at = live.iter().position(|argument| return argument == "IOWriteBandwidthMax").expect("the write bandwidth property");
    assert_eq!(live[live_at..live_at + 3], ["IOWriteBandwidthMax", "a(st)", "0"], "an empty array clears the kind on a live scope");
    let cleared = set_properties_arguments(SCOPE, &InheritedLimits::default());
    assert_eq!(cleared.iter().filter(|argument| return argument.as_str() == "a(st)").count(), 4);
}

/// Stopping names the unit and replaces a queued job.
#[test]
fn the_stop_arguments_name_the_unit() {
    let arguments = stop_arguments(SCOPE);
    assert_eq!(arguments[5], "StopUnit");
    assert_eq!(arguments[7..], [SCOPE, "replace"]);
}

/// The slice call has one property and no auxiliary units.
#[test]
fn the_slice_arguments_name_the_slice_with_one_property() {
    let arguments = ensure_slice_arguments(SLICE);
    assert_eq!(arguments[7], SLICE);
    assert_eq!(arguments[9], "1");
    assert_eq!(arguments.last().map(String::as_str), Some("0"));
    let property_count: usize = arguments[9].parse().expect("the property count");
    let first_property = arguments.iter().position(|argument| return argument.as_str() == "Description").expect("the property list");
    assert_eq!(arguments.len(), first_property + 3 * property_count + 1, "the property list and the aux unit count decide the rest");
}

/// What the fake manager does for each call it is given,
/// as shell snippets run after the call is logged.
struct Manager {
    /// For `StartTransientUnit`.
    create: String,
    /// For `AttachProcessesToUnit`.
    attach: String,
    /// For `SetUnitProperties`.
    set_properties: String,
}

/// Paths of the fake process and the fake scope that a behavior may change.
struct Paths {
    /// The fake directory of pid 77 in the fake procfs.
    process_directory: PathBuf,
    /// The fake cgroup file of pid 77.
    process_cgroup: PathBuf,
    /// The fake directory of the scope in the fake cgroup tree.
    scope_directory: PathBuf,
}

/// The default fake manager that accepts every call and changes nothing.
fn accepting() -> Manager {
    return Manager { create: "exit 0".to_string(), attach: "exit 0".to_string(), set_properties: "exit 0".to_string() };
}

/// A fake manager that accepts every call.
fn accepting_all(_paths: &Paths) -> Manager {
    return accepting();
}

/// Fixture of one test.
struct Place {
    /// The adopter under test.
    adopter: BusctlAdopter,
    /// The log of the calls the fake manager received.
    log: PathBuf,
    /// The paths a behavior may change.
    paths: Paths,
}

/// Shell that moves pid 77 into the slice.
fn moves(paths: &Paths) -> String {
    return format!(
        "printf '%s\\n' '0::/user.slice/user-1000.slice/user@1000.service/app.slice/{SLICE}/{SCOPE}' > '{}'",
        paths.process_cgroup.display()
    );
}

/// Shell that makes the scope's cgroup directory with the given populated state.
fn makes_scope(paths: &Paths, populated: bool) -> String {
    let flag = if populated { 1 } else { 0 };
    let directory = paths.scope_directory.display();
    return format!("mkdir -p '{directory}' && printf 'populated {flag}\\nfrozen 0\\n' > '{directory}/cgroup.events'");
}

/// Shell that makes an empty scope and moves pid 77 into it.
fn makes_scope_and_moves(paths: &Paths, populated: bool) -> String {
    return format!("{} && {}", makes_scope(paths, populated), moves(paths));
}

/// Builds the fixture:
/// a fake `busctl` that logs its arguments and then does what `manager` says,
/// a fake procfs with one live pid,
/// and an empty fake cgroup tree.
fn place(name: &str, manager: &dyn Fn(&Paths) -> Manager) -> Place {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-adopt-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    let proc_root = directory.join("proc");
    let cgroup_root = directory.join("cgroup");
    std::fs::create_dir_all(proc_root.join("77")).expect("make the fake procfs");
    std::fs::create_dir_all(&cgroup_root).expect("make the fake cgroup tree");
    let process_cgroup = proc_root.join("77/cgroup");
    std::fs::write(&process_cgroup, "0::/user.slice/user-1000.slice/user@1000.service/app.slice/old.scope\n").expect("write cgroup");
    std::fs::write(proc_root.join("77/stat"), stat_text("S", 0)).expect("write a live stat");
    let (uid, _gid) = own_ids();
    let paths = Paths {
        process_directory: proc_root.join("77"),
        process_cgroup,
        scope_directory: scope_cgroup_directory(&cgroup_root, uid, SLICE, SCOPE),
    };
    let behavior = manager(&paths);
    let log = directory.join("log");
    let body = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{log}'\nprintf 'env=%s\\n' \"$XDG_RUNTIME_DIR\" >> '{log}'\ncase \"$*\" in\n*StartTransientUnit*) {create} ;;\n*AttachProcessesToUnit*) {attach} ;;\n*SetUnitProperties*) {set_properties} ;;\nesac\nexit 0\n",
        log = log.display(),
        create = behavior.create,
        attach = behavior.attach,
        set_properties = behavior.set_properties,
    );
    let program = directory.join("busctl");
    std::fs::write(&program, body).expect("write the fake");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("make the fake executable");
    let adopter = BusctlAdopter {
        client: UserClient { program: program.to_string_lossy().into_owned(), timeout: Duration::from_secs(5) },
        proc_root,
        cgroup_root,
        confirmation_timeout: Duration::from_millis(150),
    };
    return Place { adopter, log, paths };
}

/// The calls the fake manager received,
/// as the text of its log.
fn calls(place: &Place) -> String {
    return std::fs::read_to_string(&place.log).unwrap_or_default();
}

/// Builds the text of a stat file with the given state letter and task flags.
fn stat_text(state: &str, flags: u64) -> String {
    let mut fields: Vec<String> = vec!["0".to_string(); 20];
    fields[0] = state.to_string();
    fields[1] = "1".to_string();
    fields[6] = flags.to_string();
    fields[19] = "99".to_string();
    return format!("77 (x) {}\n", fields.join(" "));
}

/// Our own uid and gid,
/// so the privilege drop is a no-op in an unprivileged test.
fn own_ids() -> (u32, u32) {
    // SAFETY: getuid and getgid have no preconditions.
    return unsafe { (libc::getuid(), libc::getgid()) };
}

/// Builds a request for the given pids and limits.
fn request<'a>(limits: &'a InheritedLimits, pids: &'a [i32]) -> AdoptRequest<'a> {
    let (uid, gid) = own_ids();
    return AdoptRequest { pids, uid, gid, slice: SLICE, limits, scope_name: SCOPE, origin: ORIGIN, reason: MatchReason::ToolchainPath, first_start_time: None };
}

/// A scope that an earlier build named with a second `.scope` is found before a new one is made,
/// so the origin's later processes join the running scope and share its cap.
#[test]
fn a_scope_named_by_an_earlier_build_is_found() {
    let place = place("legacy-name", &accepting_all);
    let (uid, _gid) = own_ids();
    let origin = "/user.slice/user-1000.slice/user@1000.service/app.slice/run-p1-i2.scope";
    assert_eq!(place.adopter.existing_scope_name(uid, SLICE, origin), None);
    let earlier = "wg-quicker-update-app.slice-run\\x2dp1\\x2di2.scope.scope";
    let directory = scope_cgroup_directory(&place.adopter.cgroup_root, uid, SLICE, earlier);
    std::fs::create_dir_all(&directory).expect("make the scope of the earlier build");
    assert_eq!(place.adopter.existing_scope_name(uid, SLICE, origin), Some(earlier.to_string()));
}

/// The manager that creates and populates the scope and moves pid 77 into it.
fn creating(paths: &Paths) -> Manager {
    return Manager { create: makes_scope_and_moves(paths, true), ..accepting() };
}

/// Makes the scope exist in the fake cgroup tree with the given populated state.
fn let_scope_exist(place: &Place, populated: bool) {
    std::fs::create_dir_all(&place.paths.scope_directory).expect("the scope exists");
    let flag = if populated { 1 } else { 0 };
    std::fs::write(place.paths.scope_directory.join("cgroup.events"), format!("populated {flag}\nfrozen 0\n")).expect("write the state");
}

/// A scope that does not exist is created with the limits,
/// delegated and collected,
/// and the move is confirmed.
#[test]
fn a_missing_scope_is_created_with_the_limits_and_the_move_is_confirmed() {
    let place = place("create", &creating);
    let (uid, _gid) = own_ids();
    let joined = place.adopter.adopt(&request(&all_limits(), &[77])).expect("the move is confirmed");
    assert_eq!(joined, Joined::Created);
    let log = calls(&place);
    assert!(log.contains("StartTransientUnit") && !log.contains("AttachProcessesToUnit"), "{log}");
    assert!(log.contains(&format!("{SCOPE}\nfail")), "{log}");
    assert!(log.contains("Delegate\nb\ntrue") && log.contains("CollectMode\ns\ninactive-or-failed"), "{log}");
    assert!(log.contains("MemoryMax\nt\n1073741824") && log.contains("IOWeight\nt\n10"), "{log}");
    assert!(log.contains(&format!("env=/run/user/{uid}")), "{log}");
    assert!(!log.contains("StopUnit"), "a populated scope is left running: {log}");
}

/// A scope that exists is attached to,
/// and no second scope is created for the same origin.
#[test]
fn an_existing_scope_is_attached_to_and_not_created_again() {
    let place = place("attach", &|paths| return Manager { attach: moves(paths), ..accepting() });
    let_scope_exist(&place, true);
    let joined = place.adopter.adopt(&request(&all_limits(), &[77, 78])).expect("the move is confirmed");
    assert_eq!(joined, Joined::Attached);
    let log = calls(&place);
    assert!(log.contains("AttachProcessesToUnit") && !log.contains("StartTransientUnit") && !log.contains("StopUnit"), "{log}");
    assert!(log.contains(&format!("{SCOPE}\n\n2\n77\n78")), "{log}");
}

/// A scope that ended while it was being attached to is created afresh,
/// so a move is not lost to the end of the scope that used to hold its origin.
#[test]
fn a_scope_that_ends_during_the_attach_is_created_afresh() {
    let place = place("ended", &|paths| {
        return Manager {
            attach: format!("rm -r '{}'; echo 'Unit not loaded.' >&2; exit 1", paths.scope_directory.display()),
            create: makes_scope_and_moves(paths, true),
            ..accepting()
        };
    });
    let_scope_exist(&place, true);
    assert_eq!(place.adopter.adopt(&request(&all_limits(), &[77])), Ok(Joined::Created));
    let log = calls(&place);
    assert!(log.contains("AttachProcessesToUnit") && log.contains("StartTransientUnit"), "{log}");
}

/// A scope that lives and refuses the attach is a failure that is retried later,
/// and no second scope is created beside it.
#[test]
fn an_attach_refused_by_a_living_scope_is_a_failure() {
    let place = place("refused", &|_paths| return Manager { attach: "echo 'boom' >&2; exit 1".to_string(), ..accepting() });
    let_scope_exist(&place, true);
    let error = place.adopter.adopt(&request(&all_limits(), &[77])).expect_err("refused");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("boom")), "{error}");
    assert!(!calls(&place).contains("StartTransientUnit"));
}

/// A scope that was handed only a process that is exiting is empty and never gets an empty event,
/// so it is stopped at once and the move counts as a vanish.
#[test]
fn a_new_scope_that_got_only_an_exiting_process_is_stopped() {
    let place = place("exiting", &|paths| return Manager { create: makes_scope(paths, false), ..accepting() });
    std::fs::write(place.adopter.proc_root.join("77/stat"), stat_text("R", 4)).expect("flag it exiting");
    let slow = BusctlAdopter { confirmation_timeout: Duration::from_secs(20), ..place.adopter };
    let started = Instant::now();
    assert_eq!(slow.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(started.elapsed() < Duration::from_secs(5), "the confirmation waited for its timeout");
    let log = std::fs::read_to_string(&place.log).unwrap_or_default();
    assert!(log.contains(&format!("StopUnit\nss\n{SCOPE}\nreplace")), "{log}");
}

/// A process that moved and exited before the check leaves a scope that reads `populated 0`,
/// which is stopped too,
/// and an attach that left an empty scope behind is stopped the same way.
#[test]
fn a_scope_left_empty_after_the_move_is_stopped() {
    let created = place("emptied", &|paths| return Manager { create: makes_scope_and_moves(paths, false), ..accepting() });
    assert_eq!(created.adopter.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(calls(&created).contains("StopUnit"));
    let attached = place("emptied-attach", &|paths| return Manager { attach: moves(paths), ..accepting() });
    let_scope_exist(&attached, false);
    assert_eq!(attached.adopter.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(calls(&attached).contains("StopUnit"));
}

/// A process that became a zombie after the manager accepted the job keeps its old cgroup path,
/// so it never shows in the slice;
/// it is a vanish and the wait does not run to its timeout.
#[test]
fn a_zombie_that_never_moves_is_vanished_without_waiting() {
    let place = place("zombie", &|paths| return Manager { create: makes_scope(paths, true), ..accepting() });
    std::fs::write(place.adopter.proc_root.join("77/stat"), stat_text("Z", 0)).expect("make it a zombie");
    let slow = BusctlAdopter { confirmation_timeout: Duration::from_secs(20), ..place.adopter };
    let started = Instant::now();
    assert_eq!(slow.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(started.elapsed() < Duration::from_secs(5), "the confirmation waited for its timeout");
}

/// A job the manager accepted but did not apply is reported as a failure.
#[test]
fn an_unconfirmed_move_is_an_error() {
    let place = place("unmoved", &|paths| return Manager { create: makes_scope(paths, true), ..accepting() });
    let error = place.adopter.adopt(&request(&all_limits(), &[77])).expect_err("not moved");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("is not in")), "{error}");
}

/// A client that fails reports its exit status and message,
/// and an empty request is refused.
#[test]
fn a_failing_client_is_an_error_naming_its_message() {
    let place = place("fails", &|_paths| return Manager { create: "echo 'boom' >&2; exit 1".to_string(), ..accepting() });
    let error = place.adopter.adopt(&request(&all_limits(), &[77])).expect_err("client fails");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("boom")), "{error}");
    assert!(matches!(place.adopter.adopt(&request(&all_limits(), &[])), Err(AdoptError::Failed(_))));
}

/// A process that exits while the manager call fails is a vanish and not a failure.
#[test]
fn a_process_that_exits_during_a_failed_call_is_vanished() {
    let place = place("gone-failed", &|paths| return Manager { create: format!("rm -r '{}'; exit 1", paths.process_directory.display()), ..accepting() });
    assert_eq!(place.adopter.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
}

/// The shell of a fake manager call that fails the way the real one does for a pid that no longer exists.
const NO_SUCH_PROCESS_CALL: &str = "echo 'Call failed: No such process' >&2; exit 1";

/// Adds a fake process with the given pid to the fake procfs.
/// `state` and `flags` become its stat file.
fn add_process(place: &Place, pid: i32, state: &str, flags: u64) {
    let directory = place.adopter.proc_root.join(pid.to_string());
    std::fs::create_dir_all(&directory).expect("make the fake process");
    std::fs::write(directory.join("cgroup"), "0::/user.slice/user-1000.slice/user@1000.service/app.slice/old.scope\n").expect("write cgroup");
    std::fs::write(directory.join("stat"), stat_text(state, flags)).expect("write stat");
}

/// A failed call for a process that is a zombie or flagged as exiting is a vanish,
/// as the real manager answers "No such process" for it while its `/proc` entry still exists.
#[test]
fn a_zombie_or_exiting_process_during_a_failed_call_is_vanished() {
    for (state, flags, call) in [
        ("Z", 0, NO_SUCH_PROCESS_CALL),
        ("R", 4, "echo 'Failed to set unit properties: No such process' >&2; exit 1"),
        ("Z", 0, "echo 'boom' >&2; exit 1"),
    ] {
        let place = place("zombie-failed", &|_paths| return Manager { create: call.to_string(), ..accepting() });
        std::fs::write(place.adopter.proc_root.join("77/stat"), stat_text(state, flags)).expect("make it exit");
        assert_eq!(place.adopter.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished), "{state} {flags} {call}");
    }
}

/// The same answer for an attach to a living scope is a vanish too,
/// and no scope is created beside it.
#[test]
fn a_zombie_during_a_failed_attach_is_vanished_and_creates_nothing() {
    let place = place("zombie-attach", &|_paths| return Manager { attach: NO_SUCH_PROCESS_CALL.to_string(), ..accepting() });
    let_scope_exist(&place, true);
    std::fs::write(place.adopter.proc_root.join("77/stat"), stat_text("Z", 0)).expect("make it a zombie");
    assert_eq!(place.adopter.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(!calls(&place).contains("StartTransientUnit"));
}

/// The manager's answer names no pid,
/// so a group member that is gone,
/// a zombie or exiting explains it,
/// while the matched process is still alive.
#[test]
fn a_gone_group_member_explains_the_no_such_process_answer() {
    for member in [None, Some(("Z", 0)), Some(("R", 4))] {
        let place = place("member-gone", &|_paths| return Manager { create: NO_SUCH_PROCESS_CALL.to_string(), ..accepting() });
        if let Some((state, flags)) = member {
            add_process(&place, 78, state, flags);
        }
        assert_eq!(place.adopter.adopt(&request(&all_limits(), &[77, 78])), Err(AdoptError::MemberVanished), "{member:?}");
    }
}

/// A process whose state cannot be established is never counted as gone:
/// a stat file that cannot be read or parsed leaves a failed move loud,
/// whatever the manager said,
/// and the wait for the move ends in a failure and not in a vanish.
#[test]
fn an_unreadable_or_unparseable_state_keeps_the_failure_loud() {
    for damage in ["unparseable", "unreadable"] {
        for call in ["echo 'Access denied' >&2; exit 1", NO_SUCH_PROCESS_CALL] {
            let place = place("unknown-state", &|_paths| return Manager { create: call.to_string(), ..accepting() });
            let stat = place.adopter.proc_root.join("77/stat");
            std::fs::remove_file(&stat).expect("remove the stat file");
            if damage == "unparseable" {
                std::fs::write(&stat, "not a stat line\n").expect("write garbage");
            } else {
                // Reading a directory fails with an error that is neither "absent" nor "no such process".
                std::fs::create_dir(&stat).expect("make the stat file unreadable");
            }
            let refused = place.adopter.adopt(&request(&all_limits(), &[77])).expect_err("the state is unknown");
            assert!(matches!(&refused, AdoptError::Failed(_)), "{damage} {call}: {refused}");
        }
        let waiting = place(
            "unknown-state-wait",
            &|paths| return Manager { create: makes_scope(paths, true), ..accepting() },
        );
        let stat = waiting.adopter.proc_root.join("77/stat");
        std::fs::remove_file(&stat).expect("remove the stat file");
        if damage == "unparseable" {
            std::fs::write(&stat, "not a stat line\n").expect("write garbage");
        } else {
            std::fs::create_dir(&stat).expect("make the stat file unreadable");
        }
        let unmoved = waiting.adopter.adopt(&request(&all_limits(), &[77])).expect_err("the move is not confirmed");
        assert!(matches!(&unmoved, AdoptError::Failed(text) if text.contains("is not in")), "{damage}: {unmoved}");
    }
}

/// A pid that another process has taken over since the match is gone:
/// the start time read at the match no longer fits.
/// The same pid with the matching start time stays a failure.
#[test]
fn a_pid_taken_over_by_another_process_is_vanished() {
    let place = place("reused", &|_paths| return Manager { create: "echo 'Access denied' >&2; exit 1".to_string(), ..accepting() });
    let (uid, gid) = own_ids();
    let limits = all_limits();
    let same = AdoptRequest { pids: &[77], uid, gid, slice: SLICE, limits: &limits, scope_name: SCOPE, origin: ORIGIN, reason: MatchReason::ToolchainPath, first_start_time: Some(99) };
    assert!(matches!(place.adopter.adopt(&same), Err(AdoptError::Failed(_))));
    let other = AdoptRequest { first_start_time: Some(100), ..same };
    assert_eq!(place.adopter.adopt(&other), Err(AdoptError::Vanished));
}

/// A living matched process whose move fails stays a failure,
/// whatever the answer says,
/// unless a member of the group is gone and the answer is "No such process".
#[test]
fn a_living_process_whose_move_fails_stays_a_failure() {
    let answer = place("live-answer", &|_paths| return Manager { create: NO_SUCH_PROCESS_CALL.to_string(), ..accepting() });
    let error = answer.adopter.adopt(&request(&all_limits(), &[77])).expect_err("the process lives");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("No such process")), "{error}");
    add_process(&answer, 78, "S", 0);
    assert!(matches!(answer.adopter.adopt(&request(&all_limits(), &[77, 78])), Err(AdoptError::Failed(_))), "every member lives");
    for other in ["echo 'boom' >&2; exit 1", "echo 'Call failed: No such file or directory' >&2; exit 1", "echo 'Access denied' >&2; exit 1"] {
        let place = place("live-other", &|_paths| return Manager { create: other.to_string(), ..accepting() });
        let refused = place.adopter.adopt(&request(&all_limits(), &[77, 78])).expect_err("another reason");
        assert!(matches!(&refused, AdoptError::Failed(_)), "{other}: {refused}");
    }
}

/// A process that exits after the manager accepted the job is noticed at once,
/// without waiting out the confirmation timeout.
#[test]
fn a_process_that_exits_after_an_accepted_call_is_vanished_without_waiting() {
    let place = place("gone-ok", &|paths| {
        return Manager { create: format!("{} && rm -r '{}'", makes_scope(paths, true), paths.process_directory.display()), ..accepting() };
    });
    let slow = BusctlAdopter { confirmation_timeout: Duration::from_secs(20), ..place.adopter };
    let started = Instant::now();
    assert_eq!(slow.adopt(&request(&all_limits(), &[77])), Err(AdoptError::Vanished));
    assert!(started.elapsed() < Duration::from_secs(5), "the confirmation waited for its timeout");
}

/// A client that outlives its timeout is a timeout failure,
/// which the policy treats as a possibly hung manager.
#[test]
fn a_hung_client_is_a_timeout_failure() {
    let place = place("hung", &creating);
    let sleeper = place.adopter.client.program.clone();
    std::fs::write(&sleeper, "#!/bin/sh\nexec /usr/bin/sleep 30\n").expect("rewrite the fake");
    let hung = BusctlAdopter { client: UserClient { program: sleeper, timeout: Duration::from_millis(200) }, ..place.adopter };
    let error = hung.adopt(&request(&all_limits(), &[77])).expect_err("the client hangs");
    assert!(matches!(error, AdoptError::TimedOut(_)), "{error}");
}

/// A missing client is an error,
/// not a silent skip.
#[test]
fn a_missing_client_is_an_error() {
    let adopter = BusctlAdopter {
        client: UserClient { program: "/nonexistent/busctl".to_string(), timeout: Duration::from_secs(1) },
        proc_root: PathBuf::from("/proc"),
        cgroup_root: PathBuf::from("/nonexistent/cgroup"),
        confirmation_timeout: Duration::from_millis(10),
    };
    assert!(matches!(adopter.adopt(&request(&all_limits(), &[1])), Err(AdoptError::Failed(_))));
}

/// Builds a limit request for the tests of `apply_limits`.
fn limit_request(limits: &InheritedLimits) -> LimitRequest<'_> {
    let (uid, gid) = own_ids();
    return LimitRequest { uid, gid, slice: SLICE, scope_name: SCOPE, limits };
}

/// A running scope is given the limits through `SetUnitProperties`.
#[test]
fn a_running_scope_is_given_new_limits() {
    let place = place("limits", &accepting_all);
    let_scope_exist(&place, true);
    place.adopter.apply_limits(&limit_request(&all_limits())).expect("applied");
    let log = calls(&place);
    assert!(log.contains("SetUnitProperties") && log.contains("CPUQuotaPerSecUSec\nt\n8000000"), "{log}");
}

/// A scope that is not there is reported as gone without a call,
/// and a scope that ends during the call is gone too,
/// while a scope that lives and refuses is a failure.
#[test]
fn limits_for_a_scope_that_is_gone_say_so() {
    let absent = place("limits-absent", &accepting_all);
    assert_eq!(absent.adopter.apply_limits(&limit_request(&all_limits())), Err(AdoptError::ScopeGone));
    assert!(!absent.log.exists(), "no call for a scope that is not there");
    let ending = place("limits-ending", &|paths| {
        return Manager { set_properties: format!("rm -r '{}'; echo 'not loaded' >&2; exit 1", paths.scope_directory.display()), ..accepting() };
    });
    let_scope_exist(&ending, true);
    assert_eq!(ending.adopter.apply_limits(&limit_request(&all_limits())), Err(AdoptError::ScopeGone));
    let refusing = place("limits-refused", &|_paths| return Manager { set_properties: "echo 'boom' >&2; exit 1".to_string(), ..accepting() });
    let_scope_exist(&refusing, true);
    let error = refusing.adopter.apply_limits(&limit_request(&all_limits())).expect_err("refused");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("boom")), "{error}");
}

/// An existing slice directory needs no call.
#[test]
fn an_existing_slice_is_not_requested_again() {
    let place = place("slice", &accepting_all);
    let (uid, gid) = own_ids();
    let directory = place.log.parent().expect("parent").to_path_buf();
    place.adopter.ensure_slice(uid, gid, SLICE, &directory).expect("an existing directory is enough");
    assert!(!place.log.exists());
    assert!(place.adopter.ensure_slice(uid, gid, SLICE, &directory.join("absent")).is_err());
}

/// The `major:minor` of the first whole block device of this machine,
/// the spelling `io.max` uses,
/// or `None` when it has none to limit.
fn a_whole_block_device() -> Option<String> {
    let mut names: Vec<String> = std::fs::read_dir("/dev/block").ok()?.flatten().filter_map(device_name).collect();
    names.sort();
    return names.into_iter().find(is_whole_device);
}

/// The file name of a directory entry as text.
fn device_name(entry: std::fs::DirEntry) -> Option<String> {
    return entry.file_name().into_string().ok();
}

/// Reports whether a `major:minor` names a disk and not one of its partitions.
fn is_whole_device(name: &String) -> bool {
    return !Path::new("/sys/dev/block").join(name).join("partition").exists();
}

/// Creates a scope for a real sleeping child in a throwaway slice of the user's own manager,
/// attaches a second child to it,
/// gives it new limits,
/// and checks what the cgroup files hold and that the scope ends with its last process.
/// It needs a user manager and its bus.
#[test]
#[ignore = "needs the user systemd manager"]
fn the_real_manager_creates_attaches_and_follows_limits() {
    let mut first = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the first child");
    let mut second = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the second child");
    let first_pid = i32::try_from(first.id()).expect("pid fits");
    let second_pid = i32::try_from(second.id()).expect("pid fits");
    let slice = "app-wgtest\\x2dshared.slice";
    let scope = format!("wg-quicker-update-wgtest-{first_pid}.scope");
    let (uid, gid) = own_ids();
    let origin = format!("/user.slice/user-{uid}.slice/user@{uid}.service/app.slice/wgtest-origin-{first_pid}.scope");
    let adopter = BusctlAdopter::new();
    let mut limits = InheritedLimits { memory_max: Some(1_073_741_824), cpu_quota_per_sec_usec: Some(2_000_000), tasks_max: Some(64), io_weight: Some(10), ..InheritedLimits::default() };
    let device = a_whole_block_device();
    if let Some(name) = &device {
        limits.io_devices.insert(name.clone(), DeviceLimits { read_bandwidth: Some(1_048_576), write_iops: Some(500), ..DeviceLimits::default() });
    }
    let created = adopter.adopt(&AdoptRequest { pids: &[first_pid], uid, gid, slice, limits: &limits, scope_name: &scope, origin: &origin, reason: MatchReason::ToolchainPath, first_start_time: None });
    let attached = adopter.adopt(&AdoptRequest { pids: &[second_pid], uid, gid, slice, limits: &limits, scope_name: &scope, origin: &origin, reason: MatchReason::ToolchainPath, first_start_time: None });
    let directory = scope_cgroup_directory(Path::new("/sys/fs/cgroup"), uid, slice, &scope);
    let read = |name: &str| return std::fs::read_to_string(directory.join(name)).unwrap_or_default().trim().replace('\n', " | ");
    let (created_cpu, created_memory, created_tasks, created_weight, created_bfq) = (read("cpu.max"), read("memory.max"), read("pids.max"), read("io.weight"), read("io.bfq.weight"));
    let created_io_max = read("io.max");
    let relaxed = InheritedLimits { cpu_quota_per_sec_usec: Some(500_000), ..InheritedLimits::default() };
    let applied = adopter.apply_limits(&LimitRequest { uid, gid, slice, scope_name: &scope, limits: &relaxed });
    let members = read("cgroup.procs");
    let (cpu, memory, tasks, weight, bfq) = (read("cpu.max"), read("memory.max"), read("pids.max"), read("io.weight"), read("io.bfq.weight"));
    let io_max = read("io.max");
    let _killed = first.kill();
    let _killed_second = second.kill();
    let _reaped = first.wait();
    let _reaped_second = second.wait();
    std::thread::sleep(Duration::from_millis(500));
    let ended = !directory.is_dir();
    let _stopped = std::process::Command::new("systemctl").args(["--user", "stop", slice]).status();
    assert_eq!(created, Ok(Joined::Created));
    assert_eq!(attached, Ok(Joined::Attached));
    assert_eq!(applied, Ok(()));
    println!("at creation: cpu.max {created_cpu}; memory.max {created_memory}; pids.max {created_tasks}; io.weight {created_weight}; io.bfq.weight {created_bfq}");
    println!("after the change: members {members}; cpu.max {cpu}; memory.max {memory}; pids.max {tasks}; io.weight {weight}; io.bfq.weight {bfq}");
    assert!(created_cpu.starts_with("200000 ") && created_memory == "1073741824" && created_tasks == "64", "{created_cpu} {created_memory} {created_tasks}");
    assert!(created_weight == "default 10" && created_bfq == "default 10", "{created_weight} {created_bfq}");
    println!("device {device:?}: io.max at creation {created_io_max:?}, after the change {io_max:?}");
    if let Some(name) = &device {
        assert_eq!(created_io_max, format!("{name} rbps=1048576 wbps=max riops=max wiops=500"));
    }
    assert_eq!(io_max, "", "the per-device limits were cleared when the origin had none");
    assert!(members.contains(&first_pid.to_string()) && members.contains(&second_pid.to_string()), "{members}");
    assert!(cpu.starts_with("50000 "), "{cpu}");
    assert_eq!(memory, "max");
    assert_eq!(tasks, "max");
    assert!(weight == "default 100" && bfq == "default 100", "{weight} {bfq}");
    assert!(ended, "the scope was collected when its last process exited");
}
