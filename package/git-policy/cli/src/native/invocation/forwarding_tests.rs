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
use crate::hooks_off::StartOwner;
use crate::abandoned_locks::{candidates_of_git_directory, clear_abandoned_locks};
use crate::lock_holder_records::hold_before_reading_a_childs_identity_test_support::hold as hold_the_identity_read;
use crate::lock_holder_records::{SELF_KIND, remember_repository, this_process_record_path};
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
        StartOwner::Wrapper,
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
    let bare: Command = git_command(Path::new("git"), &[], &[], StartOwner::Wrapper);
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
            StartOwner::Wrapper,
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
        run_real_git(missing, &[], &[], StartOwner::Wrapper)
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
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING, StartOwner::Wrapper).expect_err("the ceiling ends the step");
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
    assert_eq!(run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(30), StartOwner::Wrapper).expect("it ends at once"), ChildOutcome::Exited(3));
    let missing: PathBuf = root.join("no-git");
    assert_eq!(run_real_git_until(missing.as_path(), &[], &[], std::time::Duration::from_secs(30), StartOwner::Wrapper).expect_err("nothing to start").kind(), std::io::ErrorKind::NotFound);
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
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING, StartOwner::Wrapper).expect_err("the ceiling ends the step");
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
    let outcome: ChildOutcome = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING, StartOwner::Wrapper).expect("the script ends at once");
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
    let error: std::io::Error = run_real_git_until(program.as_path(), &[], &[], ENDED_CHILD_CEILING, StartOwner::Wrapper).expect_err("the ceiling ends the step");
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

/// What:
///  A step with a ceiling names the Git it starts in the repository's record,
///  so a lock that Git was holding when the ceiling killed it has a proof a later command can use.
/// Why:
///  The ceiling ends Git with `SIGKILL`, and the index lock it held then stands on disk.
///  Unlike `run_real_git_naming`, this start once named nothing,
///  so the lock it left could never be proved abandoned and a later write failed on it
///  (a review of `4e9b9fff1d`,
/// finding 4).
///  The record is kept by this process's own line and a child line for the Git the ceiling ended,
///  which is exactly what the proof reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberRepository(common); awaitReject(runRealGitUntil(slow, [], [], 200ms));
/// expect(readFileSync(thisProcessRecordPath(common))).toContain('child ');
/// ```
#[test]
fn a_step_with_a_ceiling_names_the_git_it_starts_in_the_record() {
    let root: PathBuf = fixture("forward-ceiling-record");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the Git directory");
    // The repository this process works in is a value for the whole process, so this control's fixture is named for
    //  itself and nothing else in this binary sets it.
    remember_repository(common.as_path());
    let program: PathBuf = root.join("slow-git");
    executable(program.as_path(), b"#!/bin/sh\nexec sleep 600\n");
    let _ended = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_millis(200), StartOwner::Wrapper)
        .expect_err("the ceiling ends the step");
    let record: PathBuf = this_process_record_path(common.as_path()).expect("this process's record path");
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record the start wrote");
    assert!(text.starts_with("monochromatic-git-policy-cli lock-holder-record v1"), "a record starts with its header: {text:?}");
    assert!(text.contains(format!("{SELF_KIND} {} ", std::process::id()).as_str()), "the record names this process: {text:?}");
    // What: The Git the ceiling ended is named by the line it wrote itself and by the parent's full identity.
    // Why:  The child writes its line before it can take a lock, and the parent adds the identity with the boot, the PID
    //       namespace and the machine beside it; both name one process.
    assert!(text.matches("child ").count() >= 1, "the Git the ceiling ended is named: {text:?}");
    assert!(
        text.lines().any(|line| { return line.starts_with("child ") && line.matches('#').count() == 3; }),
        "and by the full identity the parent adds: {text:?}"
    );
    remove(root.as_path());
}

/// What:
///  A Git that takes the index lock and dies before the wrapper has read its identity is still named in the record,
///  so the next pass clears the lock it left.
/// Why:
///  A start that names its child after the child may already be running loses the record of a Git that dies at once:
///  the identity cannot be read, the record is not written, and the next commit fails on the standing lock
///  (a review of `429503238b`,
/// finding 1).
///  The window is made certain by a hold before the identity is read,
///  which is the delay the review's arm made with `strace`:
///  a start that holds its child until it is named passes,
///  and one that does not finds the child gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdBeforeReadingAChildsIdentity(300); const lock = join(common, 'index.lock');
/// await runRealGitUntil(scriptThatLocksAndDies, [], [], 5s); expect(recordLines(common)).toContain('child ');
/// expect(clearAbandonedLocks(...).cleared).toHaveLength(1); expect(existsSync(lock)).toBe(false);
/// ```
#[test]
fn a_git_that_dies_before_its_identity_is_read_is_still_named_and_its_lock_is_cleared() {
    let root: PathBuf = fixture("forward-fast-death-record");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("objects").as_path()).expect("the object directory");
    std::fs::create_dir_all(common.join("refs/heads").as_path()).expect("the references");
    std::fs::write(common.join("HEAD"), b"ref: refs/heads/main\n").expect("HEAD");
    remember_repository(common.as_path());
    let lock: PathBuf = common.join("index.lock");
    let pid_file: PathBuf = common.join("index~pid.lock");
    let program: PathBuf = root.join("dying-git");
    // The lock and its PID file are written first, and the Git then ends itself before the wrapper can read its identity.
    let script: String = format!(
        "#!/bin/sh\n: > '{0}'\nsleep 0.02\necho pid $$ > '{1}'\nkill -9 $$\n",
        lock.display(),
        pid_file.display()
    );
    executable(program.as_path(), script.as_bytes());
    hold_the_identity_read(Some(std::time::Duration::from_millis(300)));
    let ended = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(5), StartOwner::Wrapper);
    hold_the_identity_read(None);
    assert!(matches!(ended, Ok(ChildOutcome::Signaled(9))), "the stand-in Git ended itself: {ended:?}");
    let record: PathBuf = this_process_record_path(common.as_path()).expect("this process's record path");
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record the start wrote, though the Git died at once");
    // What: The child is named, and by the full identity the parent adds as well as by the line it wrote itself.
    // Why:  The child writes its own line between `fork` and `exec`, and the parent adds the identity with the boot,
    //       the PID namespace and the machine beside it; both name one process and the proof judges both.
    assert!(text.matches("child ").count() >= 1, "the Git that died is named: {text:?}");
    assert!(
        text.lines().any(|line| { return line.starts_with("child ") && line.matches('#').count() == 3; }),
        "the full identity of that Git is named too: {text:?}"
    );
    assert!(lock.exists() && pid_file.exists(), "the lock and its PID file are the ones the stand-in left");
    // The PID file names the shell that made it, and the record names the same process: the proof is the one a later
    //  command has, and it is enough.
    let (proven, candidates) = candidates_of_git_directory(common.as_path(), common.as_path()).expect("the fixture is a Git directory");
    let outcome = clear_abandoned_locks(&proven, candidates.as_slice());
    assert_eq!(outcome.cleared.len(), 1, "the next pass clears the lock on the record the start wrote");
    assert!(!lock.exists(), "the lock the dead Git left is gone");
    remove(root.as_path());
}

/// The variable that makes this control's own test binary the wrapper a control kills.
const KILLED_WRAPPER_FIXTURE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_KILLED_WRAPPER_FIXTURE";

/// What:
///  A wrapper killed between starting a Git and naming it leaves that Git named all the same,
///  because the Git wrote its own line before it could take a lock,
///  and the next pass clears the lock it left.
/// Why:
///  The record was written by the parent after the start, so a kill of the wrapper in that window left a lock nothing
///  could ever explain, and the next commit failed on `File exists`
///  (a review of `3ac1254a73`,
/// finding 3).
///  The kill is placed by the change's own test-only hold, before the identity is read, so this control is about that
///  instant and not about a race;
///  the positive control runs the same arm without a kill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(currentTestBinary, [thisTest, '--exact'], { env: { ...env, KILLED_WRAPPER_FIXTURE_VARIABLE: fixture } });
/// await waitFor(existsSync(join(common, 'index~pid.lock'))); child.kill('SIGKILL');
/// expect(recordLines(common).some((line) => line.startsWith('child '))).toBe(true);
/// expect(clearAbandonedLocks(...).cleared).toHaveLength(1);
/// ```
#[test]
fn a_wrapper_killed_between_starting_a_git_and_naming_it_leaves_that_git_named() {
    // What: The arm that runs as the "wrapper" is this very test, started again by the control that kills it.
    // Why:  The kill must hit a process whose only job is to start that Git, and the test binary is the library under test.
    if let Some(fixture) = std::env::var_os(KILLED_WRAPPER_FIXTURE_VARIABLE) {
        let root: PathBuf = PathBuf::from(fixture);
        let common: PathBuf = root.join(".git");
        remember_repository(common.as_path());
        let program: PathBuf = root.join("slow-git");
        let script: String = format!(
            "#!/bin/sh\n: > '{0}/index.lock'\nsleep 0.02\necho pid $$ > '{0}/index~pid.lock'\nkill -9 $$\n",
            common.display()
        );
        executable(program.as_path(), script.as_bytes());
        hold_the_identity_read(Some(std::time::Duration::from_millis(600)));
        let _ran = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(60), StartOwner::Wrapper);
        return;
    }
    let root: PathBuf = fixture("forward-killed-wrapper");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("objects").as_path()).expect("the object directory");
    std::fs::create_dir_all(common.join("refs/heads").as_path()).expect("the references");
    std::fs::write(common.join("HEAD"), b"ref: refs/heads/main\n").expect("HEAD");
    let lock: PathBuf = common.join("index.lock");
    let pid_file: PathBuf = common.join("index~pid.lock");
    // What: This control runs the test binary again, as the wrapper whose kill it measures.
    // Why:  The kill must hit a process whose only job is to start that Git; the executable here is the test binary, not a
    //       copy the wrapper started from its own code (`doc/planning/cli-git-no-self-invocation.md`, "The rule").
    #[allow(clippy::disallowed_methods, reason = "the control starts this test binary as the wrapper it kills; the wrapper's own code starts no copy of itself")]
    let arm_binary: PathBuf = std::env::current_exe().expect("this test binary");
    let mut arm: std::process::Command = std::process::Command::new(arm_binary.as_path());
    arm.args(["--exact", "invocation::forwarding::tests::a_wrapper_killed_between_starting_a_git_and_naming_it_leaves_that_git_named", "--nocapture", "--test-threads=1"])
        .env(KILLED_WRAPPER_FIXTURE_VARIABLE, root.as_os_str())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child: std::process::Child = arm.spawn().expect("the arm starts");
    // The arm is killed once its Git holds the lock and names itself, which is the window the record is written in.
    let began: std::time::Instant = std::time::Instant::now();
    while !pid_file.exists() {
        assert!(began.elapsed() < std::time::Duration::from_secs(20), "the arm's Git took the lock in time");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let _killed = child.kill();
    let _waited = child.wait().expect("the arm ends");
    let folder: PathBuf = crate::lock_holder_records::folder_of(common.as_path());
    let mut names_the_git: bool = false;
    if let Ok(listed) = std::fs::read_dir(folder.as_path()) {
        for entry in listed.flatten() {
            let text: String = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if text.lines().any(|line| { return line.starts_with("child "); }) {
                names_the_git = true;
            }
        }
    }
    assert!(names_the_git, "the Git the killed wrapper started is named in the record it wrote itself");
    assert!(lock.exists(), "the lock the killed Git left stands");
    let (proven, candidates) = candidates_of_git_directory(common.as_path(), common.as_path()).expect("the fixture is a Git directory");
    let outcome = crate::abandoned_locks::clear_abandoned_locks(&proven, candidates.as_slice());
    assert_eq!(outcome.cleared.len(), 1, "the next pass clears the lock on the record the Git wrote");
    assert!(!lock.exists(), "the lock is gone");
    remove(root.as_path());
}

/// What:
///  A child that has become Git is not stopped and does not fail the command when its fuller identity cannot be
///  written:
///  the command ends with Git's own status.
/// Why:
///  A successful `spawn` means the child passed `exec`,
///  so Git may already have finished a commit and moved the branch;
///  stopping it then or returning the wrapper's error tells the caller that a landed commit failed,
///  and a retry lands a second one
///  (a review of `58cd4d9f7c`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failTheNextChildNaming(); const child = spawnNamingItsChild(command); expect(await child.wait()).toBe(0);
/// expect(existsSync(marker)).toBe(true)
/// ```
#[test]
fn a_record_that_cannot_name_a_git_that_already_became_it_does_not_fail_the_command() {
    let root: PathBuf = fixture("forward-name-failure-after-exec");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("objects").as_path()).expect("the object directory");
    std::fs::create_dir_all(common.join("refs/heads").as_path()).expect("the references");
    std::fs::write(common.join("HEAD"), b"ref: refs/heads/main\n").expect("HEAD");
    remember_repository(common.as_path());
    let marker: PathBuf = root.join("the-git-ran");
    let program: PathBuf = root.join("quick-git");
    let script: String = format!("#!/bin/sh\n: > '{0}'\nexit 0\n", marker.display());
    executable(program.as_path(), script.as_bytes());
    crate::lock_holder_records::hold_before_reading_a_childs_identity_test_support::fail_the_next_child_naming();
    let ended: std::io::Result<ChildOutcome> = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(20), StartOwner::Wrapper);
    assert!(matches!(ended, Ok(ChildOutcome::Exited(0))), "the command ends with Git's own status, not the wrapper's error: {ended:?}");
    assert!(marker.exists(), "and the Git the wrapper started ran to its end");
    remove(root.as_path());
}

/// What:
///  A named pipe at this process's record path neither holds the command inside `spawn`
///  nor stops the Git it cannot record.
/// Why:
///  The the parent's own note once answered success for whatever stood at the path,
///  and the child's append then waited on the pipe with no reader,
///  so a Git command waited forever before Git started
///  (a review of `58cd4d9f7c`,
/// finding 3).
///  The pipe is opened with `O_NONBLOCK` and never waited on,
///  the line is dropped,
///  and the command runs as it would with no record
///  (a review of `551084db3c`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(thisProcessRecordPath(common)); const began = Date.now(); const child = spawnNamingItsChild(command);
/// expect(Date.now() - began).toBeLessThan(5000); expect(child.status).toBe(0); expect(existsSync(marker)).toBe(true)
/// ```
#[test]
fn a_named_pipe_at_the_record_path_neither_holds_the_start_nor_stops_the_git() {
    let root: PathBuf = fixture("forward-record-pipe");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(crate::lock_holder_records::folder_of(common.as_path()).as_path()).expect("the record folder");
    remember_repository(common.as_path());
    let record: PathBuf = this_process_record_path(common.as_path()).expect("this process's record path");
    let made = std::process::Command::new("mkfifo").arg(record.as_path()).status().expect("mkfifo runs");
    assert!(made.success(), "the control made a named pipe at the record's path");
    let marker: PathBuf = root.join("the-git-ran");
    let program: PathBuf = root.join("quick-git");
    let script: String = format!("#!/bin/sh\n: > '{0}'\nexit 0\n", marker.display());
    executable(program.as_path(), script.as_bytes());
    let began: std::time::Instant = std::time::Instant::now();
    let started = crate::forwarding::spawn_naming_its_child(&mut git_command(program.as_path(), &[], &[], StartOwner::Wrapper));
    assert!(began.elapsed() < std::time::Duration::from_secs(5), "the start answers at once instead of waiting for a reader: {began:?}");
    let mut child = started.expect("a record path that holds a pipe never fails the start");
    let ended = child.wait().expect("the Git ends");
    assert!(ended.success(), "the Git runs and ends plainly: {ended:?}");
    assert!(marker.exists(), "and it really ran");
    remove(root.as_path());
}

/// What:
///  A link at this process's record path is never written through,
///  and the command it cannot record runs anyway.
/// Why:
///  The child's append open had no `O_NOFOLLOW`,
///  so the record line was added to whatever the link named
///  (a review of `58cd4d9f7c`,
/// finding 4).
///  What follows from the record's own rule is that the start does not fail:
///  the line is dropped,
///  a warning names the folder,
///  and Git runs
///  (a review of `551084db3c`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(personsFile, thisProcessRecordPath(common)); const child = spawnNamingItsChild(command);
/// expect(child.status).toBe(0); expect(readFileSync(personsFile)).toEqual(original)
/// ```
#[test]
fn a_link_at_the_record_path_is_never_written_through_and_the_command_runs() {
    let root: PathBuf = fixture("forward-record-link");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(crate::lock_holder_records::folder_of(common.as_path()).as_path()).expect("the record folder");
    remember_repository(common.as_path());
    let persons_file: PathBuf = root.join("a-person's-file");
    let persons_bytes: &[u8] = b"a line of a person's own\n";
    std::fs::write(persons_file.as_path(), persons_bytes).expect("the person's file");
    let record: PathBuf = this_process_record_path(common.as_path()).expect("this process's record path");
    std::os::unix::fs::symlink(persons_file.as_path(), record.as_path()).expect("the link at the record's path");
    let marker: PathBuf = root.join("the-git-ran");
    let program: PathBuf = root.join("quick-git");
    let script: String = format!("#!/bin/sh\n: > '{0}'\nexit 0\n", marker.display());
    executable(program.as_path(), script.as_bytes());
    let started = crate::forwarding::spawn_naming_its_child(&mut git_command(program.as_path(), &[], &[], StartOwner::Wrapper));
    let mut child = started.expect("a record path that is a link never fails the start");
    let ended = child.wait().expect("the Git ends");
    assert!(ended.success(), "the Git the person asked for runs and ends plainly: {ended:?}");
    assert!(marker.exists(), "and it really ran");
    assert_eq!(std::fs::read(persons_file.as_path()).expect("the person's file is still there"), persons_bytes, "the person's file is not written to");
    remove(root.as_path());
}

/// What:
///  A file standing where the record folder would be never fails the command:
///  the Git starts,
///  ends with its own status,
///  and the file is left exactly as it was.
/// Why:
///  The record is the wrapper's own bookkeeping,
///  and making its folder was done with `?` before the start,
///  so a wrapped commit failed with `File exists (os error 17)` and started no Git at all,
///  while plain Git committed the same change
///  (a review of `551084db3c`,
/// finding 1).
///  What stands at the folder's path is no record of this wrapper's:
///  it is taken for a person's file or for what another program left,
///  and it is never removed,
///  replaced or written through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(common, 'cli-git-lock-holders'), bytes); rememberRepository(common);
/// const ended = runRealGitUntil(standInThatTouchesItsMarker, [], [], 5s, StartWrapper);
/// expect(ended).toEqual({ exited: 0 }); expect(readFileSync(folder)).toEqual(bytes);
/// ```
#[test]
fn a_record_folder_that_cannot_be_written_never_fails_the_command() {
    let root: PathBuf = fixture("forward-record-path-is-a-file");
    let common: PathBuf = root.join(".git");
    crate::test_support::make_it_a_git_directory(common.as_path());
    remember_repository(common.as_path());
    // What: A regular file with a person's bytes stands at the folder's own path.
    // Why:  This is the state the review reproduced: `create_dir_all` answers `File exists` before any Git starts.
    let folder: PathBuf = crate::lock_holder_records::folder_of(common.as_path());
    let blocking: &[u8] = b"a person's file where the record folder would be\n";
    std::fs::write(folder.as_path(), blocking).expect("the file at the record folder's path");
    // The record attempt is the one the start makes, and it says its warning without failing anything.
    let mut command: Command = Command::new("/usr/bin/git");
    super::record_the_git_this_command_starts(&mut command);
    assert_eq!(std::fs::read(folder.as_path()).expect("the file is still there"), blocking, "the file at the folder's path is neither removed nor written");
    // The command itself still runs, and its own status is the wrapper's.
    let marker: PathBuf = root.join("the-git-ran");
    let program: PathBuf = root.join("stand-in-git");
    let script: String = format!("#!/bin/sh\n: > '{}'\nexit 0\n", marker.display());
    executable(program.as_path(), script.as_bytes());
    let ended = run_real_git_until(program.as_path(), &[], &[], std::time::Duration::from_secs(10), StartOwner::Wrapper);
    assert!(matches!(ended, Ok(ChildOutcome::Exited(0))), "the command runs as it would with no record: {ended:?}");
    assert!(marker.exists(), "the Git the person asked for really ran");
    assert_eq!(std::fs::read(folder.as_path()).expect("the file is still there"), blocking, "and the file at the folder's path is still exactly as it was");
    remove(root.as_path());
}
