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
use super::{ChildOutcome, STOP_OUTCOME_FOR_THIS_THREAD, StopOutcome, exit_code, git_command, replace_process_with_real_git, run_real_git, run_real_git_until};
use crate::test_support::{executable, fixture, remove};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::process::ExitStatusExt;
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
///  The ceiling the two controls that end a child use.
/// Why:
///   Each control reads its children's numbers from files the children write,
/// so a ceiling shorter than the host's
///  scheduling delay would end the child before it wrote anything,
/// and the control would fail on a missing file
///  rather than on what it measures.
/// Measured:
/// these controls failed together inside a 98-test run under a load average of
///  57 (0.45 s,
/// `the child wrote its number`),
/// and passed alone and in the same run afterwards;
/// five
///  seconds is more than ten times the delay a busy host added there.
const ENDED_CHILD_CEILING: std::time::Duration = std::time::Duration::from_secs(5);

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
///  A run with a ceiling ends Git at that ceiling,
/// leaves a helper Git started to run,
/// and says which of the
///  two is known about Git.
/// Why:
///   Git keeps this wrapper's process group and terminal,
/// which is what a conversion filter reading `/dev/tty`
///  needs,
/// so only Git is signalled when the ceiling is reached,
/// exactly as a Git the person starts themselves leaves its
///  helpers behind when it is killed.
/// The line names what was seen,
/// so it does not claim the helper ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const error = await awaitReject(runRealGitUntil(program, [], [], endedChildCeiling))
/// expect(error.message).toContain('Git ended'); expect(isAlive(leader)).toBe(false); expect(isAlive(helper)).toBe(true)
/// ```
#[test]
fn a_run_with_a_ceiling_ends_git_and_leaves_its_helpers_running() {
    let root: PathBuf = fixture("forward-ceiling");
    let child_number: PathBuf = root.join("child.pid");
    let helper_number: PathBuf = root.join("helper.pid");
    let program: PathBuf = root.join("slow-git");
    // `exec` at the end makes the leader the process that hangs, so the only helper left after the kill is the one this
    //  control reads and ends itself.
    let script: String = format!(
        "#!/bin/sh\nsleep 600 &\necho $! > {}\necho $$ > {}\nexec sleep 600\n",
        helper_number.display(),
        child_number.display()
    );
    executable(program.as_path(), script.as_bytes());
    let began: std::time::Instant = std::time::Instant::now();
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING).expect_err("the ceiling ends the step");
    let waited: std::time::Duration = began.elapsed();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "{error}");
    let named: String = format!("{ENDED_CHILD_CEILING:?}");
    let said: String = error.to_string();
    assert!(said.contains(named.as_str()) && said.contains("Git ended"), "the error names the ceiling and what it knows about Git: {said}");
    assert!(said.contains("a filter or helper Git started may still be running"), "the error does not claim the helper ended: {said}");
    assert!(waited < std::time::Duration::from_secs(20), "the ceiling ends it near its number, not when the child would have finished: {waited:?}");
    assert!(!still_alive(number_in(child_number.as_path())), "Git is gone");
    let helper: i32 = number_in(helper_number.as_path());
    assert!(still_alive(helper), "the helper Git started is not signalled, as under native Git: {said}");
    // SAFETY: `kill` with SIGKILL takes a process number this control started through the runner and touches no memory.
    unsafe { libc::kill(helper, libc::SIGKILL) };
    assert!(!still_alive(helper), "the control ends the helper it let run, so nothing outlives it");
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
///  A ceiling whose Git does not exit inside the stop grace still returns,
/// and its error says Git has not
///  exited.
/// Why:
///   `SIGKILL` cannot be ignored,
/// so a Git still running after the grace is inside a system call the kernel cannot
///  interrupt (a hung filesystem,
/// a dead FUSE server);
/// a plain `wait` there would wait for exactly the thing the
///  ceiling exists to end,
/// and the owner forbade that ("nothing should ever wait forever or indefinitely").
/// The outcome is stated by a seam and not by a shorter grace:
/// a grace of zero does not force an
///  uncollected child (one that exits before the next look is collected),
/// so the old control could fail under a valid
///  scheduling order and without any defect.
/// The kill itself is real,
/// and the control reaps Git itself,
/// which proves the
///  kill was sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateStopOutcome('left'); const error = await awaitReject(runRealGitUntil(program, [], [], endedChildCeiling));
/// expect(error.message).toContain('has not exited'); expect(isAlive(leader)).toBe(false);
/// ```
#[test]
fn a_ceiling_whose_child_cannot_be_collected_returns_and_says_so() {
    let root: PathBuf = fixture("forward-ceiling-uncollected");
    let child_number: PathBuf = root.join("child.pid");
    let program: PathBuf = root.join("uninterruptible-git");
    // `exec` makes the leader the process that hangs, so the kill leaves no helper of the control's own behind.
    let script: String = format!("#!/bin/sh\necho $$ > {}\nexec sleep 600\n", child_number.display());
    executable(program.as_path(), script.as_bytes());
    STOP_OUTCOME_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<StopOutcome>>| slot.set(Some(StopOutcome::Left)));
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING).expect_err("the ceiling ends the step");
    STOP_OUTCOME_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<StopOutcome>>| slot.set(None));
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "{error}");
    let named: String = format!("{ENDED_CHILD_CEILING:?}");
    let said: String = error.to_string();
    assert!(said.contains(named.as_str()) && said.contains("has not exited"), "the error names the ceiling and what it saw: {said}");
    assert!(said.contains("a filter or helper Git started may still be running"), "and what it cannot know about the helpers: {said}");
    assert!(collected_here(number_in(child_number.as_path())), "the kill reached Git, and only its collection was left to this process");
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

/// What:
///  A step under a ceiling keeps this process's group,
/// so a conversion filter that reads the terminal is a foreground
///  reader.
/// Why:
///   The line discipline stops a process group that is not the terminal's foreground group when it reads,
/// with
///  `SIGTTIN`,
/// so a group of its own turns a filter that asks for a credential or a confirmation on `/dev/tty`
///  into a stopped process and the add fails at its ceiling.
/// A Git the person starts themselves keeps their shell's group,
/// and the
///  step keeps it too;
/// the child writes the group the kernel gave it,
/// and this control compares it with its own.
/// Nothing here waits on a terminal,
/// so the answer does not depend on how the host schedules the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const group = numberIn(await boundedStep(program, [], [], endedChildCeiling));
/// expect(group).toBe(process.getpgrp());
/// ```
#[test]
fn a_bounded_step_keeps_the_process_group_of_this_one() {
    let root: PathBuf = fixture("forward-ceiling-group");
    let group_file: PathBuf = root.join("group.txt");
    let program: PathBuf = root.join("printing-git");
    let script: String = format!("#!/bin/sh\nread -r line < /proc/self/stat\nset -- $line\nprintf '%s\\n' \"$5\" > {}\n", group_file.display());
    executable(program.as_path(), script.as_bytes());
    let outcome: ChildOutcome = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING).expect("the script ends at once");
    assert_eq!(outcome, ChildOutcome::Exited(0), "the script ran to its end");
    // SAFETY: `getpgrp` takes no argument and touches no memory.
    let mine: i32 = unsafe { libc::getpgrp() };
    assert_eq!(number_in(group_file.as_path()), mine, "the step runs in this process's group, which is the terminal's foreground group");
    remove(root.as_path());
}

/// The variable that marks the child of the blocked-signal control.
const BLOCKED_SIGNAL_CHILD_VARIABLE: &str = "CLI_GIT_END_AS_GIT_ENDED_BLOCKED_CHILD";

/// The status the child exits with when the signal could not end it:
/// no program of the wrapper uses it.
const SIGNAL_NOT_DELIVERED_CODE: i32 = 97;

/// What:
///  A terminating signal this process blocks still ends it by that signal.
/// Why:
///  A route that keeps a process around Git reports the death as a value and ends by it afterwards
/// (`forwarding::end_as_git_ended`);
/// a signal the person's own start left blocked,
/// or one a parent blocked
///  before the wrapper ran,
/// stays pending when raised,
/// so the wrapper exited normally with `128 + the signal` and its caller saw a
///  normal exit where native Git's caller sees `WIFSIGNALED`
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 5 of the round-3 review).
/// The body runs in a child,
/// because the death is the assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // in a child: block(SIGTERM); endAsGitEnded(SIGTERM); expect(parent).toSeeSignal('SIGTERM');
/// ```
#[test]
fn a_blocked_terminating_signal_still_ends_by_it() {
    if std::env::var_os(BLOCKED_SIGNAL_CHILD_VARIABLE).is_some() {
        block(super::SIGNAL_TO_END_BY);
        let _status: i32 = super::end_as_git_ended(super::SIGNAL_TO_END_BY);
        // Only reached when the signal could not end this process: the parent names that status.
        std::process::exit(SIGNAL_NOT_DELIVERED_CODE);
    }
    let binary: PathBuf = crate::test_support::this_test_binary().expect("the test binary");
    let output: std::process::Output = Command::new(binary)
        .args([
            "--exact",
            "invocation::forwarding::tests::a_blocked_terminating_signal_still_ends_by_it",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(BLOCKED_SIGNAL_CHILD_VARIABLE, "1")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child test runs");
    assert_eq!(
        output.status.signal(),
        Some(super::SIGNAL_TO_END_BY),
        "the child ended with {:?} instead of the death by signal {}: {}",
        output.status.code(),
        super::SIGNAL_TO_END_BY,
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

/// Blocks `signal` in this thread,
/// so raising it leaves it pending.
fn block(signal: i32) {
    // SAFETY: `sigemptyset` and `sigaddset` write only this local set, and `pthread_sigmask` reads it and touches no other
    //  memory of this process.
    unsafe {
        let mut blocked: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut blocked);
        libc::sigaddset(&mut blocked, signal);
        let _masked = libc::pthread_sigmask(libc::SIG_BLOCK, &blocked, std::ptr::null_mut());
    }
}

/// What:
///  The line a ceiling writes says what a caller that reads standard error to its end sees.
/// Why:
///   A helper Git started inherits this command's standard error,
/// and the wrapper cannot close another process's descriptor:
/// while that helper lives,
/// a caller that captures the stream and waits for end of file
///  (`Command::output`,
/// the wrapper's own forwarded child)
/// keeps waiting,
/// the ceiling ends Git and not the caller's
///  wait,
/// and the person needs to know that from the line rather than from a hang.
/// The line names the helper it
///  cannot name,
/// because nothing reports which process a filter started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(error.message).toContain('a caller reading this command\'s standard error to its end waits until that helper closes it')
/// ```
#[test]
fn a_ceiling_says_what_a_caller_reading_the_stream_to_its_end_sees() {
    let root: PathBuf = fixture("forward-ceiling-eof");
    let helper_number: PathBuf = root.join("helper.pid");
    let program: PathBuf = root.join("filter-git");
    let script: String = format!("#!/bin/sh\nsleep 600 &\necho $! > {}\nexec sleep 600\n", helper_number.display());
    executable(program.as_path(), script.as_bytes());
    STOP_OUTCOME_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<StopOutcome>>| slot.set(Some(StopOutcome::Collected)));
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING).expect_err("the ceiling ends the step");
    STOP_OUTCOME_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<StopOutcome>>| slot.set(None));
    let said: String = error.to_string();
    // The helper holds the stream open exactly as the line says, so this control ends it here.
    let helper: i32 = number_in(helper_number.as_path());
    // SAFETY: `kill` with SIGKILL takes a process number this control started through the runner and touches no memory.
    unsafe { libc::kill(helper, libc::SIGKILL) };
    assert!(!still_alive(helper), "the control ends the helper it let run");
    assert!(said.contains("a caller reading this command's standard error to its end waits until that helper closes it"), "the line says what the caller's end of file means: {said}");
    assert!(said.contains("and Git ended"), "and what is known about Git: {said}");
    remove(root.as_path());
}
