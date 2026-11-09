//! What:
//!  Controls for the notices about reference locks and the quoting of the command that clears one.
//! Why:
//!  The command is meant to be pasted into a shell,
//! and the names in it come from the file system,
//! so the controls run the
//!  quoted words through a real shell and require that exactly the named file goes.
//! The wording is pinned as printed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(outcome.notice()).toBe("monochromatic-git-policy-cli: left the reference lock ... alone: ...\nmonochromatic-git-policy-cli:   rm -- '...'\n");
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

/// A finding with the facts the notice words,
/// and none that vary between runs.
fn found(lock: &str, verdict: Verdict, pid_file: PidFile, companions: &[&str]) -> Found {
    return Found {
        lock: PathBuf::from(lock),
        verdict,
        size: 0,
        born: None,
        pid_file,
        companions: companions.iter().map(PathBuf::from).collect(),
    };
}

/// Names that a shell or a terminal could misread,
/// each of which must survive a trip through `bash -c "rm -- <quoted>"`.
fn awkward_names() -> Vec<OsString> {
    return vec![
        OsString::from("plain.lock"),
        OsString::from("with space.lock"),
        OsString::from("it's$(touch${IFS}pwned);`id`&x|(y).lock"),
        OsString::from("new\nline.lock"),
        OsString::from("escape\u{1b}[31mred.lock"),
        OsString::from("tab\there\\back.lock"),
        OsString::from_vec(vec![b'n', b'o', b't', 0xff, 0xfe, b'.', b'l', b'o', b'c', b'k']),
        OsString::from("-leading-dash.lock"),
    ];
}

/// Every awkward name,
/// quoted,
/// clears exactly its own file in a real shell and runs nothing.
#[test]
fn quoted_names_name_exactly_their_files_in_a_real_shell() {
    let root: PathBuf = fixture("stale-message-quoting");
    for name in awkward_names() {
        let directory: PathBuf = root.join("d");
        std::fs::create_dir_all(directory.as_path()).expect("directory");
        let target: PathBuf = directory.join(&name);
        let bystander: PathBuf = directory.join("bystander");
        std::fs::write(target.as_path(), "").expect("target");
        std::fs::write(bystander.as_path(), "").expect("bystander");
        let command: String = format!("cd \"$1\" && rm -- {}", shell_quote(Path::new(&name)));
        let ran = Command::new("bash").arg("-c").arg(command.as_str()).arg("bash").arg(directory.as_os_str()).output().expect("start bash");
        assert!(ran.status.success(), "{command}: {}", String::from_utf8_lossy(&ran.stderr));
        assert!(!target.exists() && bystander.exists(), "{command}");
        assert!(!directory.join("pwned").exists() && !root.join("pwned").exists(), "{command} ran something");
        assert!(!shell_quote(Path::new(&name)).contains('\u{1b}'), "a terminal escape is printed raw");
        remove(directory.as_path());
    }
    remove(root.as_path());
}

/// A plain name is not quoted,
/// a name with a single quote has it written `'\''`,
/// and an empty name is quoted.
#[test]
fn quoting_leaves_plain_names_alone_and_escapes_single_quotes() {
    assert_eq!(shell_quote(Path::new("/repo/.git/refs/heads/main.lock")), "/repo/.git/refs/heads/main.lock");
    assert_eq!(shell_quote(Path::new("/r/it's.lock")), "'/r/it'\\''s.lock'");
    assert_eq!(shell_quote(Path::new("")), "''");
}

/// The notice for a lock that nothing proves stale,
/// as printed,
/// with the command that clears it and its companions.
#[test]
fn an_unproven_lock_is_described_with_the_command_that_clears_it() {
    let outcome: Outcome = Outcome {
        left: vec![found("/r/.git/refs/remotes/origin/main.lock", Verdict::Unproven(String::from("no pid file beside it names a holder")), PidFile::Absent, &[])],
        ..Outcome::default()
    };
    let expected: &str = "monochromatic-git-policy-cli: left the reference lock /r/.git/refs/remotes/origin/main.lock alone: no pid file beside it names a holder.\n\
monochromatic-git-policy-cli: it is 0 bytes. If no Git process is running in this repository, clear it with:\n\
monochromatic-git-policy-cli:   rm -- /r/.git/refs/remotes/origin/main.lock\n\
monochromatic-git-policy-cli: Git writes a process number beside its locks only when `core.lockfilePid` is on. The wrapper turns it on for the Git it starts; `git config --global core.lockfilePid true` does it for the rest, so the next lock can be proven.\n";
    assert_eq!(outcome.notice(), expected);
}

/// The packed-references lock names its pid file and staging file in the command,
/// and a lock of an adversarial name is quoted.
#[test]
fn the_command_names_the_companions_and_quotes_an_odd_reference() {
    let outcome: Outcome = Outcome {
        left: vec![
            found("/r/.git/packed-refs.lock", Verdict::Unproven(String::from("why")), PidFile::Older(7), &["/r/.git/packed-refs~pid.lock", "/r/.git/packed-refs.new"]),
            found("/r/.git/refs/heads/it's$(x).lock", Verdict::Unproven(String::from("why")), PidFile::Older(7), &[]),
        ],
        ..Outcome::default()
    };
    let text: String = outcome.notice();
    assert!(text.contains("monochromatic-git-policy-cli:   rm -- /r/.git/packed-refs.lock '/r/.git/packed-refs~pid.lock' /r/.git/packed-refs.new\n"), "{text}");
    assert!(text.contains("monochromatic-git-policy-cli:   rm -- '/r/.git/refs/heads/it'\\''s$(x).lock'\n"), "{text}");
    assert!(!text.contains("core.lockfilePid"), "the hint is only for locks without a pid file: {text}");
}

/// The notice for a removed lock,
/// and for a lock a running process holds,
/// as printed.
#[test]
fn a_removed_lock_and_a_held_lock_are_worded_as_printed() {
    let outcome: Outcome = Outcome {
        cleared: vec![
            found("/r/.git/packed-refs.new", Verdict::Stale(Proof::OrphanStaging), PidFile::Absent, &[]),
        ],
        left: vec![found("/r/.git/refs/heads/d.lock", Verdict::Held(String::from("process 43 (git), which the pid file beside it names, is running")), PidFile::Names(43), &[])],
        displaced: vec![PathBuf::from("/r/.git/cli-git-stale-lock-1-0-ff")],
    };
    let expected: &str = "monochromatic-git-policy-cli: removed the packed-references staging file /r/.git/packed-refs.new, left by a Git that is gone: Git makes this staging file only while it holds `packed-refs.lock`, and that lock is gone.\n\
monochromatic-git-policy-cli: the reference lock /r/.git/refs/heads/d.lock is in use: process 43 (git), which the pid file beside it names, is running; it was left alone.\n\
monochromatic-git-policy-cli: a lock file was set aside while a new lock took its place, and is left at /r/.git/cli-git-stale-lock-1-0-ff; look at it before removing it.\n";
    assert_eq!(outcome.notice(), expected);
}

/// A pass that found nothing to say prints nothing.
#[test]
fn a_pass_with_nothing_to_say_prints_nothing() {
    assert_eq!(Outcome::default().notice(), "");
}
