//! Controls of the enhancement:
//! the gate that decides whether a service manager may be asked at all,
//! and the bound on a manager that does not answer.
//! Why:
//!  An enhancement that can fail a commit is not one;
//! what is proven here is that a bus of another session, a folder this program fell back to, and a manager that
//!  does not answer all leave the base's start untouched, and that a manager of this session is asked in
//!  exactly the words `systemd-run` takes.

use super::*;
use std::os::unix::fs::PermissionsExt;

/// A folder of this test's own, short enough for a socket address where one is needed.
fn folder(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture("manager-gate").join(name);
    crate::test_support::track_for_this_thread(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("fixture folder");
    return root;
}

/// A stand-in for `systemd-run` that records the words it was given and then exits `status` after `sleep_seconds`.
fn stand_in(root: &Path, status: i32, sleep_seconds: &str) -> PathBuf {
    let programme: PathBuf = root.join("systemd-run");
    std::fs::write(
        programme.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{record}'\nsleep {sleep_seconds}\nexit {status}\n",
            record = root.join("calls.txt").display()
        ),
    )
    .expect("the stand-in is written");
    std::fs::set_permissions(programme.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the stand-in runs");
    return programme;
}

/// A host whose runtime directory the environment named, and whose bus is inside it.
fn host(runtime: &Path, bus_address: Option<String>) -> ServiceHost {
    return ServiceHost {
        runtime_dir: runtime.to_path_buf(),
        state_dir: runtime.join("state"),
        service_binary: PathBuf::from("/bin/git"),
        git: PathBuf::from("/usr/bin/git"),
        instance: String::from("3f9a"),
        git_dir: PathBuf::from("/repo/.git"),
        named_runtime_dir: Some(runtime.to_path_buf()),
        bus_address,
    };
}

/// What:
///  A manager is asked only for a session this command belongs to.
/// Why:
///  A folder the program fell back to (`~/.run`) is no session, a bus elsewhere is another session's manager, and without
///  the manager's program there is nothing to ask; each answer is the base's start, with nothing a person must act on.
#[test]
fn a_manager_is_asked_only_for_this_commands_own_session() {
    let root: PathBuf = folder("session");
    let _stand_in: PathBuf = stand_in(root.as_path(), 0, "0");
    // The program is found on PATH, which this control pins to the folder holding the stand-in.
    let held: Option<std::ffi::OsString> = std::env::var_os("PATH");
    // SAFETY: this control is the only one in this process that changes PATH, and it restores it before it ends.
    let previous: Option<std::ffi::OsString> = held.clone();
    unsafe { std::env::set_var("PATH", root.as_os_str()) };
    let ours: ServiceHost = host(root.as_path(), Some(format!("unix:path={}", root.join("bus").display())));
    assert!(ManagerStart::in_this_session(&ours).is_some(), "our own session's bus and the program are both there");
    let elsewhere: ServiceHost = host(root.as_path(), Some(String::from("unix:path=/run/user/9999/bus")));
    assert!(ManagerStart::in_this_session(&elsewhere).is_none(), "a bus of another session is never asked");
    let no_bus: ServiceHost = host(root.as_path(), None);
    assert!(ManagerStart::in_this_session(&no_bus).is_none(), "no bus names no manager");
    let mut fell_back: ServiceHost = host(root.as_path(), Some(format!("unix:path={}", root.join("bus").display())));
    fell_back.named_runtime_dir = None;
    assert!(ManagerStart::in_this_session(&fell_back).is_none(), "a folder this program fell back to is no session");
    match previous {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }
}

/// What:
///  A manager that does not answer within the probe's bound is a manager this command does without,
///  and one that answers well is asked.
/// Why:
///  A manager re-reading its units answers after the reload (139,770 ms was measured once), and a commit may not pay that:
///  the probe is the only wait the enhancement adds, and it ends at `PROBE_WAIT`.
#[test]
fn a_probe_that_does_not_answer_in_time_is_no_manager() {
    let root: PathBuf = folder("probe");
    std::fs::create_dir_all(root.join("quick").as_path()).expect("folder");
    let quick: ManagerStart = ManagerStart { program: stand_in(root.join("quick").as_path(), 0, "0") };
    let began: std::time::Instant = std::time::Instant::now();
    assert!(quick.probe(), "a manager that answers at once is asked");
    assert!(began.elapsed() < PROBE_WAIT, "and the answer costs less than the bound");

    std::fs::create_dir_all(root.join("slow").as_path()).expect("folder");
    let slow: ManagerStart = ManagerStart { program: stand_in(root.join("slow").as_path(), 0, "30") };
    let slow_began: std::time::Instant = std::time::Instant::now();
    assert!(!slow.probe(), "a manager that has not answered by the bound is not there for this command");
    let waited: std::time::Duration = slow_began.elapsed();
    assert!(waited >= PROBE_WAIT && waited < PROBE_WAIT + Duration::from_secs(2), "the bound is the bound: {waited:?}");

    std::fs::create_dir_all(root.join("refused").as_path()).expect("folder");
    let refused: ManagerStart = ManagerStart { program: stand_in(root.join("refused").as_path(), 1, "0") };
    assert!(!refused.probe(), "a manager that refuses the probe is not asked again");
}

/// What:
///  A probe child the kernel does not let go is never waited for:
///  the probe answers at its bound plus the stop's grace,
///  and the child is left to the thread that collects it when the kernel releases it.
/// Why:
///  The fourth review of issue 6293 (finding 2):
/// the probe killed its child and then waited for it with no bound of its own,
/// so a probe child in an uninterruptible call held the start path of a commit for as long as the kernel took.
/// No control on this host can hold a process in such a call (measured attempts under finding 1's report),
/// so the kill is what this control refuses:
///  a child that is not ended by its signal is the same state from this wait's side,
/// and it ends by itself once the control lets it.
#[test]
fn a_probe_child_the_kernel_does_not_let_go_is_left_to_a_reaper() {
    let root: PathBuf = folder("unreapable");
    std::fs::create_dir_all(root.join("held").as_path()).expect("folder");
    // Six seconds: past the bound and the grace with room to see the difference, and short enough for the reaper to collect it here.
    let programme: PathBuf = stand_in(root.join("held").as_path(), 0, "6");
    let manager: ManagerStart = ManagerStart { program: programme };
    crate::child_waited_for_within_a_ceiling::a_control_refuses_the_next_kill();
    let began: std::time::Instant = std::time::Instant::now();
    let answered: bool = manager.probe();
    let waited: Duration = began.elapsed();
    assert!(!answered, "a manager whose probe could not be ended has not answered");
    assert!(waited >= PROBE_WAIT, "the probe waited for its bound: {waited:?}");
    assert!(
        waited < PROBE_WAIT + REAP_WAIT + Duration::from_secs(1),
        "and no longer than the stop's grace past it: {waited:?}"
    );
}

/// What:
///  The start through the manager is `systemd-run --user --scope -- <program> <the service's own words>`.
/// Why:
///  `--scope` is what makes the run a cgroup of its own with the caller's environment (identical to the base's), and the
///  service's words after `--` are exactly the base's, so the service a manager starts is the service the base starts.
#[test]
fn the_start_through_the_manager_wraps_the_bases_own_words() {
    let root: PathBuf = folder("words");
    std::fs::create_dir_all(root.join("run").as_path()).expect("folder");
    let recording: PathBuf = stand_in(root.join("run").as_path(), 0, "0");
    let manager: ManagerStart = ManagerStart { program: recording };
    let server: ServerToStart = ServerToStart {
        socket: root.join("run/3f9a.sock"),
        instance: String::from("3f9a"),
        journal: root.join("run/journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: PathBuf::from("/bin/git"),
        manager: None,
    };
    let started: Option<i32> = manager.start(&server).expect("the stand-in runs");
    assert!(started.is_some_and(|process| { return process > 0; }), "the leader's number is reported");
    // The stand-in writes the words it was given before it exits; it exits at once, so the file is there.
    let mut words: String = String::new();
    for _try in 1..200 {
        words = std::fs::read_to_string(root.join("run/calls.txt")).unwrap_or_default();
        if !words.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    // The manager's own words, then the program, then the service's own words after it.
    assert!(words.starts_with("--user --scope --quiet -- /bin/git monochromatic-git-policy-cli service run --path "), "{words}");
    assert!(words.contains("--instance 3f9a"), "the service is told which worktree it serves: {words}");
}
