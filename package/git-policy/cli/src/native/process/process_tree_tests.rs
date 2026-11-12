//! Controls for reading the processes below one process and signalling them safely.

use super::*;
use crate::member_signalled_through_its_pidfd::{Delivered, kill_members, signal_members, terminate_members};
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
    let delivered: Vec<Delivered> = signal_members(&found, libc::SIGKILL);
    // The shell's `wait` returns when the sleeper has ended, so this is the event the checks below need.
    let status: Option<ExitStatus> = status_within_bound(&mut shell);
    let sleeper_ended: bool = !still_running(member);
    reap(&mut shell);
    assert_eq!(delivered, vec![Delivered::Sent; found.len()], "a member that is still the process that was found is signalled through its descriptor: {found:?}");
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
    for (name, end) in [("terminate_members", terminate_members as fn(&[TreeMember]) -> Vec<Delivered>), ("kill_members", kill_members)] {
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

/// How long a control waits for a process it did not signal to still be there,
/// so a signal that is on its way is seen before the answer is "survived".
///  Longer than the kernel takes to end a `SIGKILL`ed process,
/// and only the control pays it.
const STILL_THERE_BOUND: Duration = Duration::from_millis(500);

/// What:
///  A number that was reused before its signal went out never reaches the process that took it.
/// Why:
///   Reading a process's start time and sending its signal are two operations,
/// and the process the number named can end and its number pass to another command between them:
/// the review of 2026-11-09 put Git's exit and reaping inside that interval,
/// after which the signal ends an unrelated process of the person's.
///  A control cannot reuse a number on demand,
/// so the reuse is put behind one identity read (`reused_after_the_next_read`):
/// the number named the member up to that read,
/// and the sleeping child after it.
#[cfg(target_os = "linux")]
#[test]
fn a_number_reused_before_the_signal_is_not_signalled() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("sleep starts");
    let pid: i32 = i32::try_from(sleeper.id()).expect("a process number");
    let (started, _zombie): (u64, bool) = identity_of(pid).expect("the sleeper is read");
    // The member a look read a moment earlier: the same number, and the start time of the process that has since ended.
    let found: TreeMember = TreeMember { pid, started: started.wrapping_add(1) };
    reused_after_the_next_read(pid, found.started);
    let delivered: Vec<Delivered> = signal_members(&[found], libc::SIGKILL);
    let deadline: Instant = Instant::now() + STILL_THERE_BOUND;
    let mut killed: Option<ExitStatus> = sleeper.try_wait().expect("the sleeper is polled");
    while killed.is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        killed = sleeper.try_wait().expect("the sleeper is polled");
    }
    if killed.is_none() {
        sleeper.kill().expect("cleanup");
        sleeper.wait().expect("reaped");
    }
    assert!(killed.is_none(), "the process that took the reused number was killed ({killed:?}) by a signal meant for the process that ended");
    assert_eq!(delivered, vec![Delivered::Gone], "the signal reported itself as sent to a number that names another process");
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

/// What:
///  A helper that detaches twice is still a descendant of the child that started it,
/// so the sweep of a stopped run ends it,
/// and a child started without the reaper attribute loses it to init.
/// Why:
///   A stopped upload ended the tree it could still find,
/// and a transfer helper that called `setsid` and was reparented escaped that tree and ran on
/// (audit `alpha-12-lfs`).
/// The two arms are one control:
/// the first proves the escape this fixes,
/// and the second proves the fix,
/// so a change that drops either is caught.
#[test]
fn a_helper_that_detaches_twice_stays_a_descendant_of_the_child_that_started_it() {
    let root: std::path::PathBuf = crate::test_support::fixture("process-subreaper");
    let recorded: std::path::PathBuf = root.join("helper.pid");
    // The child starts a launcher in a session of its own, the launcher starts a helper in a session of its own and
    //  exits, so the helper is reparented to the nearest living ancestor that reaps orphans.
    let script: String = format!(
        "setsid sh -c 'setsid sleep 30 & echo $! > {recorded}' ; sleep 30",
        recorded = recorded.display()
    );
    for owner in [false, true] {
        let _absent: std::io::Result<()> = std::fs::remove_file(recorded.as_path());
        let mut command: Command = Command::new("sh");
        command.arg("-c").arg(script.as_str()).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        if owner {
            own_the_descendants_it_starts(&mut command);
        }
        let mut child: Child = crate::test_support::spawn_owned(&mut command).expect("the child starts");
        let began: Instant = Instant::now();
        while !recorded.exists() {
            assert!(began.elapsed() < ENDING_BOUND, "the detached helper never recorded its number");
            std::thread::sleep(Duration::from_millis(5));
        }
        let helper: i32 = std::fs::read_to_string(recorded.as_path())
            .expect("the recorded number")
            .trim()
            .parse()
            .expect("a process number");
        // The launcher has exited by now, so the helper has been reparented.
        std::thread::sleep(Duration::from_millis(100));
        let found: bool = descendants_of(i32::try_from(child.id()).expect("a pid"))
            .iter()
            .any(|member: &TreeMember| { return member.pid == helper; });
        // SAFETY: `kill` takes a process number and a signal and touches no memory.
        unsafe {
            libc::kill(helper, libc::SIGKILL);
        }
        let _killed = child.kill();
        let _waited = child.wait();
        assert_eq!(
            found, owner,
            "with the reaper attribute {owner}, the detached helper {helper} is {} among the child's descendants",
            if found { "found" } else { "not found" }
        );
    }
    crate::test_support::remove(root.as_path());
}
