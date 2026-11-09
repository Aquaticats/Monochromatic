//! Controls for reading the processes below one process and signalling them safely.

use super::*;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// How long a test waits for a process to end before it reports that the process was not stopped.
///  Generous,
/// since only a function that signals nothing ever uses all of it.
const ENDING_BOUND: Duration = Duration::from_secs(10);

/// A shell,
/// in a process group of its own,
/// that starts a sleeping child,
/// prints its number,
/// and waits for it.
fn shell_with_a_sleeper() -> (Child, i32) {
    let mut shell: Child = Command::new("sh")
        .args(["-c", "sleep 600 & echo $!; wait"])
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the shell starts");
    let mut line: String = String::new();
    BufReader::new(shell.stdout.take().expect("piped stdout"))
        .read_line(&mut line)
        .expect("the sleeper's number");
    let sleeper: i32 = line.trim().parse().expect("a process number");
    return (shell, sleeper);
}

/// End the shell and the sleeper under it through the shell's process group,
/// and collect the shell.
///  The tests below check `signal_members`,
/// so their cleanup must not depend on it:
/// a version of it that signals
///  nothing would otherwise leave the sleeper running for its whole life and the shell's wait blocked on it.
fn reap(shell: &mut Child) {
    let group: i32 = i32::try_from(shell.id()).expect("a process number");
    // A shell that was already collected has nothing left, and its group number may by now belong to another group.
    if shell.try_wait().expect("the shell is polled").is_none() {
        // SAFETY: `kill` takes no pointers. The shell is this test's own child and leads its own group; a negative
        // number names that group, which stays valid until the shell is collected, and the shell was not collected
        // when it was just polled; a failure only means everything in the group ended meanwhile.
        unsafe { libc::kill(-group, libc::SIGKILL) };
    }
    shell.wait().expect("the shell is collected");
}

/// The shell's status once it ends,
/// or nothing when it is still running after the bound.
///  Its `wait` ends with the sleeper under it,
/// so a sleeper that was not stopped shows as a shell that does not end.
fn status_within_bound(shell: &mut Child) -> Option<ExitStatus> {
    let deadline: Instant = Instant::now() + ENDING_BOUND;
    loop {
        if let Some(status) = shell.try_wait().expect("the shell is polled") {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The command name sits in parentheses and may hold spaces and parentheses;
/// the fields are read after the last one.
#[cfg(target_os = "linux")]
#[test]
fn a_stat_line_is_read_after_the_last_parenthesis() {
    let line: &str = "1234 (we ) ird) S 99 1 1 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 555 0 0";
    assert_eq!(parse_stat(line), Some((99, 555, false)));
    let zombie: &str = "1234 (cat) Z 7 1 1 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 9 0 0";
    assert_eq!(parse_stat(zombie), Some((7, 9, true)));
    assert_eq!(parse_stat("1234 (cat) S 7"), None);
    assert_eq!(parse_stat("no parenthesis"), None);
}

/// A grandchild is found below its grandparent,
/// and signalling the found members ends it.
#[cfg(target_os = "linux")]
#[test]
fn a_descendant_is_found_and_signalled() {
    let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
    let root: i32 = i32::try_from(shell.id()).expect("a process number");
    let found: Vec<TreeMember> = descendants_of(root);
    let member: &TreeMember = found.iter().find(|candidate| { return candidate.pid == sleeper; }).expect("the sleeper is below the shell");
    assert!(still_running(member));
    signal_members(&found, libc::SIGKILL);
    // The shell's `wait` returns when the sleeper has ended, so this is the event the checks below need.
    let status: Option<ExitStatus> = status_within_bound(&mut shell);
    let sleeper_ended: bool = !still_running(member);
    reap(&mut shell);
    assert!(status.is_some_and(|ended| { return ended.success(); }), "the shell ended once the sleeper did: {status:?}");
    assert!(sleeper_ended, "the sleeper ended");
}

/// Ending the found members with the named wrapper reaches the process:
/// `SIGTERM` for `terminate_members`,
/// `SIGKILL`
/// for `kill_members`.
#[cfg(target_os = "linux")]
#[test]
fn the_named_wrappers_end_the_found_members() {
    for (name, end) in [("terminate_members", terminate_members as fn(&[TreeMember])), ("kill_members", kill_members)] {
        let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
        let root: i32 = i32::try_from(shell.id()).expect("a process number");
        let found: Vec<TreeMember> = descendants_of(root);
        let member: &TreeMember = found.iter().find(|candidate| { return candidate.pid == sleeper; }).expect("the sleeper is below the shell");
        end(&found);
        // Bounded poll: a wrapper that signals nothing leaves the sleeper running for its whole lifetime.
        let deadline: Instant = Instant::now() + ENDING_BOUND;
        while still_running(member) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let ended: bool = !still_running(member);
        // Reaps the shell whatever happened.
        reap(&mut shell);
        assert!(ended, "{name} ended the sleeper");
    }
}

/// A number whose start time differs is another process:
/// it is not signalled.
#[cfg(target_os = "linux")]
#[test]
fn a_reused_number_is_not_signalled() {
    let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
    let root: i32 = i32::try_from(shell.id()).expect("a process number");
    let found: Vec<TreeMember> = descendants_of(root);
    let real: &TreeMember = found.iter().find(|candidate| { return candidate.pid == sleeper; }).expect("the sleeper is below the shell");
    let stale: TreeMember = TreeMember { pid: real.pid, started: real.started + 1 };
    assert!(!still_running(&stale));
    signal_members(&[stale], libc::SIGKILL);
    let left_alone: bool = still_running(real);
    reap(&mut shell);
    assert!(left_alone, "the process with the other start time was left alone");
}

/// A process with no children has no descendants.
#[cfg(target_os = "linux")]
#[test]
fn a_leaf_has_no_descendants() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("sleep starts");
    assert_eq!(descendants_of(i32::try_from(sleeper.id()).expect("a process number")), Vec::<TreeMember>::new());
    sleeper.kill().expect("sleep is killed");
    sleeper.wait().expect("sleep is collected");
}

/// What:
///  The cheap walk finds the same grandchild the pass over `/proc` finds,
/// and `use_of` names it and holds its state and processor time.
/// Why:
///   The notice of the scratch `git commit` looks many times while one step runs,
/// and a look at this walk costs 19.6 microseconds against 17.6 ms for the pass (`descendant_numbers_from_thread_lists`).
///  A walk that found nothing would leave the step with no ceiling at all,
/// so the control reads the same child both ways.
#[cfg(target_os = "linux")]
#[test]
fn the_cheap_walk_finds_the_descendants_the_pass_finds() {
    let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
    let root: i32 = i32::try_from(shell.id()).expect("a process number");
    let by_pass: Vec<i32> = descendants_of(root).iter().map(|member| { return member.pid; }).collect();
    let by_walk: Vec<i32> = descendant_numbers_from_thread_lists(root).expect("this kernel has the per-thread child lists");
    reap(&mut shell);
    assert!(by_pass.contains(&sleeper), "the pass found the sleeper: {by_pass:?}");
    assert!(by_walk.contains(&sleeper), "the walk found the sleeper: {by_walk:?}");
    assert_eq!(by_walk.len(), by_pass.len(), "both find the same number of processes: {by_walk:?} against {by_pass:?}");
}

/// `use_of` names a running process
/// and holds the state the kernel gives it.
#[cfg(target_os = "linux")]
#[test]
fn use_of_names_a_process_and_holds_what_it_uses() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("sleep starts");
    let pid: i32 = i32::try_from(sleeper.id()).expect("a process number");
    let used: ProcessUse = use_of(pid).expect("a running process is read");
    sleeper.kill().expect("sleep is killed");
    sleeper.wait().expect("sleep is collected");
    assert_eq!(used.name, "sleep");
    assert_ne!(used.state, 'Z', "a running process is not a zombie");
}
