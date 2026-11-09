//! What:
//!  Process controls for starting real Git and reporting how it ended.
//! Why:
//!  Arguments must arrive byte for byte,
//!  and exit codes and terminating signals
//!      must be reported the way Git itself reports its children.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(exitCode({ signaled: 15 })).toBe(143);
//! ```
#![cfg(unix)]

/// Import the forwarding functions under test and shared fixtures.
use super::{ChildOutcome, STOP_GRACE_FOR_THIS_THREAD, exit_code, git_command, replace_process_with_real_git, run_real_git, run_real_git_until};
use crate::test_support::{executable, fixture, remove};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Exit codes pass through;
///  a terminating signal becomes 128 plus its number,
///  as in Git's run-command.
#[test]
fn exit_codes_follow_git_child_convention() {
    for (outcome, expected) in [
        (ChildOutcome::Exited(0), 0),
        (ChildOutcome::Exited(1), 1),
        (ChildOutcome::Exited(128), 128),
        (ChildOutcome::Exited(255), 255),
        (ChildOutcome::Signaled(2), 130),
        (ChildOutcome::Signaled(9), 137),
        (ChildOutcome::Signaled(13), 141),
        (ChildOutcome::Signaled(15), 143),
    ] {
        assert_eq!(exit_code(outcome), expected, "{outcome:?}");
    }
}

/// The command carries the exact program,
///  each argument as its own unchanged element,
///  and only the overlay.
#[test]
fn command_preserves_program_arguments_and_overlay() {
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        OsString::from_vec(b"dir-\xff\xfe".to_vec()),
        OsString::new(),
        OsString::from("a b"),
        OsString::from("--"),
        OsString::from("$(touch x); 'q' \"d\" \\ \n*"),
        OsString::from("--"),
    ];
    let overlay: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")),
        (
            OsString::from("MARKER"),
            OsString::from_vec(b"/opt/\xff/git".to_vec()),
        ),
    ];
    let command: Command = git_command(
        Path::new("/opt/real git/git"),
        arguments.as_slice(),
        overlay.as_slice(),
    );
    assert_eq!(command.get_program(), OsStr::new("/opt/real git/git"));
    let mut seen_arguments: Vec<OsString> = Vec::<OsString>::new();
    for argument in command.get_args() {
        seen_arguments.push(argument.to_os_string());
    }
    // The hooks-off option is added only once the process has switched it on (`hooks_off::activate`).
    assert_eq!(seen_arguments, arguments);
    let mut seen_environment: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
    let mut start_marker: Option<OsString> = None;
    for (name, found) in command.get_envs() {
        let text: OsString = found.expect("overlay sets, never removes").to_os_string();
        // Every real-Git start carries the marker that says which wrapper started it as Git.
        if name == crate::started_as_git::STARTED_AS_GIT_VARIABLE {
            start_marker = Some(text);
            continue;
        }
        seen_environment.push((name.to_os_string(), text));
    }
    seen_environment.sort();
    assert_eq!(seen_environment, overlay);
    let marker = crate::started_as_git::parse_marker(start_marker.expect("the start carries the marker").as_os_str())
        .expect("a well-formed marker");
    assert_eq!(marker.starter_pid, std::process::id());
    assert_eq!(marker.mode, crate::started_as_git::StartMode::Child);
    // No overlay means the child environment is exactly the inherited one plus the marker.
    let bare: Command = git_command(Path::new("git"), &[], &[]);
    assert_eq!(bare.get_envs().count(), 1);
    assert_eq!(bare.get_envs().next().map(|(name, _value)| { return name.to_os_string(); }), Some(OsString::from(crate::started_as_git::STARTED_AS_GIT_VARIABLE)));
}

/// A child's own exit code and its terminating signal are both observed exactly.
#[test]
fn run_reports_exit_codes_and_signals() {
    let root: PathBuf = fixture("forward-run");
    for (name, script, expected) in [
        ("zero", "#!/bin/sh\nexit 0\n", ChildOutcome::Exited(0)),
        ("seven", "#!/bin/sh\nexit 7\n", ChildOutcome::Exited(7)),
        (
            "git-128",
            "#!/bin/sh\nexit 128\n",
            ChildOutcome::Exited(128),
        ),
        (
            "term",
            "#!/bin/sh\nkill -TERM $$\n",
            ChildOutcome::Signaled(15),
        ),
        (
            "kill",
            "#!/bin/sh\nkill -KILL $$\n",
            ChildOutcome::Signaled(9),
        ),
        // The child sees each argument unchanged and the overlay variable.
        (
            "argv",
            "#!/bin/sh\n[ \"$#\" = 3 ] && [ \"$1\" = 'a b' ] && [ \"$2\" = '' ] && [ \"$3\" = '--' ] && [ \"$MARKER\" = 'm v' ] && exit 42\nexit 1\n",
            ChildOutcome::Exited(42),
        ),
    ] {
        let program: PathBuf = root.join(name);
        executable(program.as_path(), script.as_bytes());
        let outcome: ChildOutcome = run_real_git(
            program.as_path(),
            &[OsString::from("a b"), OsString::new(), OsString::from("--")],
            &[(OsString::from("MARKER"), OsString::from("m v"))],
        )
        .expect("child starts");
        assert_eq!(outcome, expected, "{name}");
    }
    remove(root.as_path());
}

/// A program that cannot be started is an error for both forwarding forms,
///  never a fake exit code.
#[test]
fn unstartable_program_is_an_error() {
    let missing: &Path = Path::new("/nonexistent-directory/git");
    assert_eq!(
        run_real_git(missing, &[], &[])
            .expect_err("missing program")
            .kind(),
        std::io::ErrorKind::NotFound
    );
    // Process replacement returns only on failure, leaving this test process running.
    assert_eq!(
        replace_process_with_real_git(missing, &[], &[]).kind(),
        std::io::ErrorKind::NotFound
    );
}

/// What:
///  Whether a process with this number is still there,
/// asked for up to two seconds so a just-killed one is not counted by its zombie.
/// Why:
///   A stopped step must leave nothing behind that writes into the worktree afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let tries = 0; tries < 40; tries += 1) { if (!isAlive(pid)) return false; await delay(50); } return true;
/// ```
fn still_alive(pid: i32) -> bool {
    for _try in 0..40 {
        // SAFETY: `kill` with signal 0 probes a process number and touches no memory.
        if unsafe { libc::kill(pid, 0) } != 0 {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    return true;
}

/// The number a child wrote into a file the fixture names.
fn number_in(path: &Path) -> i32 {
    let text: String = std::fs::read_to_string(path).expect("the child wrote its number");
    return text.trim().parse::<i32>().expect("a process number");
}

/// What:
///  A run with a ceiling ends the child and everything the child started at that ceiling,
///  and says what happened.
/// Why:
///   The ladder's own steps run the checkout's conversion filters:
/// a smudge filter or a Git LFS process that never
///  answers must not hold the add,
/// and stopping only the leader would leave that filter writing into the
///  worktree beside a cleanup that removes what it writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const error = await awaitReject(runRealGitUntil(program, [], [], 300))
/// expect(error.kind).toBe('TimedOut'); expect(isAlive(grandchild)).toBe(false)
/// ```
#[test]
fn a_run_with_a_ceiling_ends_the_child_and_what_it_started() {
    let root: PathBuf = fixture("forward-ceiling");
    let child_number: PathBuf = root.join("child.pid");
    let grandchild_number: PathBuf = root.join("grandchild.pid");
    let program: PathBuf = root.join("slow-git");
    let script: String = format!(
        "#!/bin/sh\nsleep 600 &\necho $! > {}\necho $$ > {}\nsleep 600\n",
        grandchild_number.display(),
        child_number.display()
    );
    executable(program.as_path(), script.as_bytes());
    let began: std::time::Instant = std::time::Instant::now();
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_millis(300)).expect_err("the ceiling ends the step");
    let waited: std::time::Duration = began.elapsed();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "{error}");
    assert!(error.to_string().contains("300ms") && error.to_string().contains("was ended"), "the error names the ceiling and what happened: {error}");
    assert!(waited < std::time::Duration::from_secs(20), "the ceiling ends it near its number, not when the child would have finished: {waited:?}");
    assert!(!still_alive(number_in(child_number.as_path())), "the leader is gone");
    assert!(!still_alive(number_in(grandchild_number.as_path())), "the process the step started is gone too");
    remove(root.as_path());
}

/// A step that ends before its ceiling reports its own outcome,
/// and one that cannot start is the start error rather than a ceiling.
#[test]
fn a_run_inside_its_ceiling_reports_what_the_child_did() {
    let root: PathBuf = fixture("forward-under-ceiling");
    let program: PathBuf = root.join("quick-git");
    executable(program.as_path(), b"#!/bin/sh\nexit 3\n");
    assert_eq!(run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(30)).expect("it ends at once"), ChildOutcome::Exited(3));
    let missing: PathBuf = root.join("no-git");
    assert_eq!(run_real_git_until(missing.as_path(), &[], &[], std::time::Duration::from_secs(30)).expect_err("nothing to start").kind(), std::io::ErrorKind::NotFound);
    remove(root.as_path());
}

/// What:
///  A ceiling whose child cannot be collected inside the stop grace still returns,
/// and its error says the child has not
///  exited.
/// Why:
///   `SIGKILL` cannot be ignored,
/// so a child still running after the grace is inside a system call the kernel cannot
///  interrupt (a hung filesystem,
/// a dead FUSE server);
/// a plain `wait` there would wait for exactly the thing the
///  ceiling exists to end (owner: "nothing should ever wait forever or indefinitely").
/// The grace is set to zero for
///  this thread,
/// so the leaving path is what the control reaches,
/// and the leader's number proves the kill itself was sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortenStopGrace(0); const error = await awaitReject(runRealGitUntil(program, [], [], 300));
/// expect(error.message).toContain('has not exited'); expect(isAlive(leader)).toBe(false);
/// ```
#[test]
fn a_ceiling_whose_child_cannot_be_collected_returns_and_says_so() {
    let root: PathBuf = fixture("forward-ceiling-uncollected");
    let child_number: PathBuf = root.join("child.pid");
    let program: PathBuf = root.join("uninterruptible-git");
    let script: String = format!("#!/bin/sh\necho $$ > {}\nsleep 600\n", child_number.display());
    executable(program.as_path(), script.as_bytes());
    STOP_GRACE_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<std::time::Duration>>| slot.set(Some(std::time::Duration::ZERO)));
    let began: std::time::Instant = std::time::Instant::now();
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_millis(300)).expect_err("the ceiling ends the step");
    let waited: std::time::Duration = began.elapsed();
    STOP_GRACE_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<std::time::Duration>>| slot.set(None));
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "{error}");
    assert!(error.to_string().contains("300ms") && error.to_string().contains("has not exited"), "the error names the ceiling and what it saw: {error}");
    assert!(waited < std::time::Duration::from_secs(20), "it does not wait for a child that cannot be stopped: {waited:?}");
    assert!(collected_here(number_in(child_number.as_path())), "the kill ended the child, and only its collection was left to this process");
    remove(root.as_path());
}

/// What:
///  Whether a child of this process has ended,
/// asking without waiting,
/// and collecting it when it has.
/// Why:
///   A leader the wrapper left unreaped is a zombie,
/// and a zombie answers `kill(pid, 0)`,
/// so the proof that the
///  kill reached it is a `waitpid` that reports it ended;
/// asking without waiting keeps the control bounded when the
///  proof would fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let tries = 0; tries < 40; tries += 1) { if (waitpid(pid, WNOHANG) === pid) return true; await delay(50); } return false;
/// ```
fn collected_here(pid: i32) -> bool {
    for _try in 0..40 {
        let mut status: libc::c_int = 0;
        // SAFETY: `waitpid` takes the number of a child of this process and a status integer this function owns, and it
        // touches no other memory.
        let collected: libc::pid_t = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if collected == pid {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    return false;
}
