//! Controls for the executable Git runs before an alias:
//!  the search path it walks,
//!  the exec path Git reports,
//!  and the real-Git comparison of each answer.

use super::*;
use crate::test_support::{executable, fixture, git_output, remove};
use std::path::PathBuf;

/// A repository-less fixture directory,
///  where Git resolves commands without needing one.
fn scratch(name: &str) -> PathBuf {
    return fixture(name);
}

/// An environment naming only these variables.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    return pairs.iter().map(|(name, value)| (OsString::from(*name), OsString::from(*value))).collect();
}

/// The context a search runs in:
///  real Git,
///  and no configuration of its own.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// An executable file at `path` that prints one marker line.
fn program(path: &Path) -> PathBuf {
    executable(path, b"#!/bin/sh\necho ran\n");
    return path.to_path_buf();
}

/// What:
///  Whether real Git runs `git-<word>` for this environment,
///  read from whether the printed marker appears.
/// Why:
///  The control answers the same question the search answers,
///  so a wrong reading fails against Git and not
///  against another reading of the same rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitOutput(directory, ['word']).stdout.includes('ran')
/// ```
fn git_runs_external(directory: &Path, pairs: &[(&str, &str)], word: &str) -> bool {
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::REAL_GIT);
    command
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config");
    for (name, value) in pairs {
        command.env(name, value);
    }
    let output: std::process::Output = command.arg(word).output().expect("native Git fixture command");
    return String::from_utf8_lossy(&output.stdout).contains("ran");
}

/// An executable on a `PATH` entry is what Git runs for that word,
///  and the search answers it too.
#[test]
fn a_program_on_the_path_is_the_command_git_runs() {
    let root: PathBuf = scratch("external-path");
    let directory: PathBuf = root.join("bin");
    std::fs::create_dir(&directory).expect("program directory");
    program(directory.join("git-audit-external").as_path());
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let pairs: Vec<(&str, &str)> = vec![("PATH", path.as_str())];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        OsStr::new("audit-external"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-external"));
    assert!(answer, "the program on PATH was not found");
    remove(root.as_path());
}

/// A word nothing on the search path names is no external command,
///  so Git reaches the alias.
#[test]
fn a_word_no_program_names_is_not_an_external_command() {
    let root: PathBuf = scratch("external-absent");
    let pairs: Vec<(&str, &str)> = vec![("PATH", "/usr/bin:/bin")];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        OsStr::new("audit-absent"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-absent"));
    assert!(!answer, "a program nothing provides was found");
    remove(root.as_path());
}

/// A `GIT_EXEC_PATH` entry is searched before `PATH`,
///  so a program there is what Git runs,
///  as the audit's fixture found.
#[test]
fn a_program_in_the_exec_path_is_the_command_git_runs() {
    let root: PathBuf = scratch("external-exec-path");
    let exec: PathBuf = root.join("exec");
    std::fs::create_dir(&exec).expect("exec path directory");
    program(exec.join("git-audit-external").as_path());
    let exec_path: String = exec.display().to_string();
    let pairs: Vec<(&str, &str)> = vec![("GIT_EXEC_PATH", exec_path.as_str()), ("PATH", "/usr/bin:/bin")];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        OsStr::new("audit-external"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-external"));
    assert!(answer, "the program in the exec path was not found");
    remove(root.as_path());
}

/// A file without the owner execute bit is skipped,
///  as Git skips it,
///  and the exec path Git reports is the one its own search uses.
#[test]
fn a_file_without_the_owner_execute_bit_is_no_command() {
    let root: PathBuf = scratch("external-not-executable");
    let directory: PathBuf = root.join("bin");
    std::fs::create_dir(&directory).expect("program directory");
    let plain: PathBuf = directory.join("git-audit-plain");
    std::fs::write(&plain, b"#!/bin/sh\necho ran\n").expect("plain file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).expect("plain mode");
    }
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let pairs: Vec<(&str, &str)> = vec![("PATH", path.as_str())];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        OsStr::new("audit-plain"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-plain"));
    assert!(!answer, "a file Git cannot execute was taken as a command");
    remove(root.as_path());
}

/// The exec path Git reports ends in one newline that the reading removes,
///  and a Windows answer ends in a carriage return before it.
#[test]
fn the_reported_exec_path_loses_only_its_record_terminator() {
    assert_eq!(without_record_terminator(b"/usr/lib/git-core\n"), OsString::from("/usr/lib/git-core"));
    assert_eq!(without_record_terminator(b"/usr/lib/git-core\r\n"), OsString::from("/usr/lib/git-core"));
    assert_eq!(without_record_terminator(b"/usr/lib/git-core"), OsString::from("/usr/lib/git-core"));
    // A path whose last byte is a carriage return keeps it when no newline ended the answer.
    assert_eq!(without_record_terminator(b"/x\r"), OsString::from("/x\r"));
    assert_eq!(without_record_terminator(b""), OsString::new());
}

/// Git reports an exec path,
///  and the answer is the directory it keeps its commands in.
#[test]
fn git_reports_the_exec_path_the_search_uses() {
    let root: PathBuf = scratch("external-exec-path-query");
    let reported: Option<OsString> = printed_exec_path(&context(), &[]);
    let expected: String = String::from_utf8_lossy(&git_output(root.as_path(), &["--exec-path"]).stdout)
        .trim_end_matches('\n')
        .to_owned();
    assert_eq!(reported, Some(OsString::from(expected)));
    assert!(reported.is_some_and(|path| !path.is_empty()), "Git reported no exec path");
    remove(root.as_path());
}
