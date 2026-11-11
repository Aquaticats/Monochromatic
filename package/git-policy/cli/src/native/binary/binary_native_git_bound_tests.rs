//! What:
//!  The two ways a native Git command of this harness could outlive the bound the harness gives it:
//!  a command that carries no bound at all,
//!  and a read that keeps waiting for a captured stream after the bound ended Git.
//! Why:
//!  The oracle every landing control compares the service against is native Git,
//! and a Git waiting on something a control cannot see held the conclusion control
//! until the runner's five-minute stop ended the run,
//!  naming no state (2026-11-11).
//! `timeout` ends Git alone:
//!  a program that a hook of Git started inherits the captured stream,
//! outlives Git,
//!  and keeps the pipe open,
//! so a read that waits for that pipe outlives its own bound as well.
//! Every command of the harness therefore carries a bound,
//! and every read of native Git keeps the stream in a file rather than a pipe.

use super::service_landing_support::text_of_within;
use super::support::{Fixture, NATIVE_GIT_BOUND_SECONDS, REAL_GIT, direct, executable, fixture, git, native_git_within, repository};
use std::ffi::OsStr;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The bound a control gives a native Git command it means to see ended,
/// in seconds.
const READ_BOUND_SECONDS: &str = "1";

/// The longest a control waits for a command it bounded at `READ_BOUND_SECONDS`,
/// in seconds.
/// Why not the command's own bound:
///  the control asks whether the command ended near its bound,
/// and a host other sessions load stretches every step of setting that wait up.
const CONTROL_LIMIT_SECONDS: u64 = 20;

/// How long the stand-in for a Git that never answers keeps its streams open,
/// in seconds.
/// Why longer than every bound here:
///  a read that waits for the streams instead of for Git waits for this whole delay,
/// so the control's own limit ends that wait instead of the delay doing it.
const HOLD_SECONDS: u64 = 30;

/// How long the stand-in for a Git that never answers delays before it answers,
/// in seconds.
/// Why longer than `READ_BOUND_SECONDS` and shorter than `CONTROL_LIMIT_SECONDS`:
///  a read that its own bound does not end waits for this delay and then answers,
/// so a control that lost its bound fails saying that Git answered and left nothing behind for it to end.
const STAND_IN_DELAY_SECONDS: u64 = 8;

/// The words of `command`'s command line,
/// as the harness starts it.
fn words_of(command: &Command) -> Vec<String> {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
    return words;
}

/// The text of a panic a control caught,
/// whether the panic carried a `String` or a `&str`.
fn message_of(payload: Box<dyn std::any::Any + Send>) -> String {
    return match payload.downcast_ref::<String>() {
        Some(text) => text.clone(),
        None => match payload.downcast_ref::<&str>() {
            Some(text) => (*text).to_string(),
            None => String::from("(a panic with no text)"),
        },
    };
}

/// What:
///  A `core.fsmonitor` hook that never answers Git,
/// and a program it starts whose standard error is Git's own.
/// Why:
///  `timeout` ends Git and not the programs Git started,
/// so the hook and the program it starts stay alive holding the stream Git's caller reads:
/// that is the shape of Git that a bound cannot end.
/// The hook writes its own process number and the program's to `pids`,
///  so a control can free both after it judged the read.
fn never_answering_hook(fixture: &Fixture, name: &str, pids: &Path) -> PathBuf {
    let hook: PathBuf = fixture.root.join(name);
    executable(
        hook.as_path(),
        format!("#!/bin/sh\nsleep {HOLD_SECONDS} >/dev/null 2>&1 &\nprintf '%s %s' \"$$\" \"$!\" > '{}'\nwait\n", pids.display()).as_bytes(),
    );
    return hook;
}

/// Ends the hook whose process numbers are in `pids`,
/// and the program it started,
///  so a read that waited for their streams can end.
fn free_holders(pids: &Path) {
    let Ok(text) = std::fs::read_to_string(pids) else {
        return;
    };
    for field in text.split_whitespace() {
        if let Ok(pid) = field.parse::<i32>() {
            // SAFETY: `kill` takes two integers and touches no memory of this process; a process that is gone answers `ESRCH`.
            unsafe { libc::kill(pid, libc::SIGKILL) };
        }
    }
}

/// What:
///  Every native Git command a fixture starts carries a bound of its own.
/// Why:
///  A command without one waits for ever,
/// and the control that started it ends as the runner's five-minute stop,
///  naming nothing (2026-11-11).
#[test]
fn every_native_git_command_a_fixture_starts_carries_its_own_bound() {
    let fixture: Fixture = fixture("native-git-bound");
    let command: Command = direct(&fixture);
    let expected: Vec<String> = vec![String::from("/usr/bin/timeout"), String::from("--signal=KILL"), NATIVE_GIT_BOUND_SECONDS.to_string(), REAL_GIT.to_string()];
    let line: Vec<String> = words_of(&command);
    assert_eq!(
        line.as_slice(),
        expected.as_slice(),
        "a native Git command of a fixture starts under the harness's bound, and this one is {line:?}: a Git that never answers would hold its control until the runner ends the run"
    );
}

/// What:
///  A native Git command that never answers is ended at its bound,
/// and the failure names the command,
///  the directory and the bound.
/// Why:
///  The oracle every landing control compares the service against is native Git,
/// and the failure a control reads must say what it waited for instead of ending the run silently.
#[test]
fn a_native_git_command_that_never_answers_ends_at_its_bound_and_names_it() {
    let fixture: Fixture = fixture("native-git-never-answers");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let hook: PathBuf = fixture.root.join("never-answering-fsmonitor");
    // The stand-in needs no stream of its own: this control is about the command's bound,
    // and the read that waits for a stream is the subject of the control below.
    executable(hook.as_path(), format!("#!/bin/sh\nexec sleep {STAND_IN_DELAY_SECONDS} </dev/null >/dev/null 2>&1\n").as_bytes());
    let root: PathBuf = fixture.root.clone();
    let printed = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let (sender, receiver): (Sender<_>, Receiver<_>) = channel();
    let running = std::thread::spawn(move || {
        let setting: String = format!("core.fsmonitor={}", hook.display());
        let arguments: [&str; 5] = ["-c", setting.as_str(), "status", "--short", "--untracked-files=all"];
        let ended = catch_unwind(AssertUnwindSafe(|| {
            return native_git_within(&fixture, &repo, arguments.as_slice(), READ_BOUND_SECONDS);
        }));
        let _sent = sender.send(ended);
    });
    let arrived = receiver.recv_timeout(Duration::from_secs(CONTROL_LIMIT_SECONDS));
    std::panic::set_hook(printed);
    drop(running);
    let ended = arrived.unwrap_or_else(|_| {
        panic!("a native Git command bounded at {READ_BOUND_SECONDS} s was still running {CONTROL_LIMIT_SECONDS} s after it started, and its stand-in answers after {STAND_IN_DELAY_SECONDS} s: the harness waits for an oracle that its own bound does not end")
    });
    let message: String = match ended {
        Ok(output) => panic!("a Git command that never answers must be ended at its bound instead of answering: {:?}", output.status),
        Err(payload) => message_of(payload),
    };
    assert!(message.contains("status"), "the failure names the command it waited for: {message}");
    assert!(message.contains(root.display().to_string().as_str()), "the failure names the directory: {message}");
    assert!(message.contains(format!("{READ_BOUND_SECONDS} s bound").as_str()), "the failure names the bound: {message}");
}

/// What:
///  A native Git read that the harness's bound ends does not go on waiting for a program that a hook of Git left
///  holding the captured stream.
/// Why:
///  `Command::output` returns when the captured pipes reach end of file,
/// and the bound ends Git alone:
/// a program the hook started outlives Git and keeps the pipe open,
///  so the read waits for that program instead of ending at its bound,
///  and the diagnostic naming the command never arrives.
#[test]
fn a_native_git_read_ended_at_its_bound_does_not_wait_for_a_hook_s_descendant() {
    let fixture: Fixture = fixture("native-git-detached-reader");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a file to read");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    let pids: PathBuf = fixture.root.join("holders.pids");
    let hook: PathBuf = never_answering_hook(&fixture, "detaching-fsmonitor", pids.as_path());
    let root: PathBuf = fixture.root.clone();
    let printed = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let (sender, receiver): (Sender<_>, Receiver<_>) = channel();
    let reading = std::thread::spawn(move || {
        let setting: String = format!("core.fsmonitor={}", hook.display());
        let arguments: [&str; 5] = ["-c", setting.as_str(), "status", "--short", "--untracked-files=all"];
        let ended = catch_unwind(AssertUnwindSafe(|| {
            return text_of_within(&fixture, &repo, arguments.as_slice(), READ_BOUND_SECONDS);
        }));
        let _sent = sender.send(ended);
    });
    let arrived = receiver.recv_timeout(Duration::from_secs(CONTROL_LIMIT_SECONDS));
    // The streams are freed before the judgement, so a read that waited for them is ended however that judgement goes.
    free_holders(pids.as_path());
    std::panic::set_hook(printed);
    drop(reading);
    let ended = arrived.unwrap_or_else(|_| {
        panic!(
            "a native Git read bounded at {READ_BOUND_SECONDS} s was still waiting {CONTROL_LIMIT_SECONDS} s after it started: the bound ended Git, and the read went on waiting for the streams the programs of Git's hook hold (numbers in {})",
            pids.display()
        );
    });
    let message: String = match ended {
        Ok(text) => panic!("a read that never answers must be ended at its bound instead of answering after its holders were freed: {text}"),
        Err(payload) => message_of(payload),
    };
    assert!(message.contains("status"), "the failure names the command it waited for: {message}");
    assert!(message.contains(root.display().to_string().as_str()), "the failure names the directory: {message}");
    assert!(message.contains(format!("{READ_BOUND_SECONDS} s bound").as_str()), "the failure names the bound: {message}");
}
