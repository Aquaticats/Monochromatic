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
    let mut environment: Vec<(OsString, OsString)> = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        environment.push((OsString::from(*name), OsString::from(*value)));
    }
    return environment;
}

/// Global options as the wrapper receives them.
fn globals(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
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
    return git_runs_external_with_globals(directory, pairs, &[], word);
}

/// What:
///  Whether real Git runs `git-<word>` for this environment and these global options,
///  read from whether the printed marker appears.
/// Why:
///  A global such as `--exec-path=` decides the search as much as the environment does,
///  so a control of that reading compares
///  the same invocation Git receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitOutput(directory, [...globals, word]).stdout.includes('ran')
/// ```
fn git_runs_external_with_globals(directory: &Path, pairs: &[(&str, &str)], globals: &[&str], word: &str) -> bool {
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
    let output: std::process::Output = command
        .args(globals)
        .arg(word)
        .output()
        .expect("native Git fixture command");
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
        Some(root.as_path()),
        OsStr::new("audit-external"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-external"));
    assert!(answer, "the program on PATH was not found");
    remove(root.as_path());
}

/// What:
///  The caller's `--exec-path` decides the search,
///  over a `GIT_EXEC_PATH` the environment also names.
/// Why:
///  `--exec-path=<dir>` sets Git's exec path
///  (`git.c:178` to `git.c:181`),
///  and `git_exec_path` answers with that setting before it reads the environment
///  (`exec-cmd.c:313` to `exec-cmd.c:323`),
///  so a reading that read the environment first would find a program Git never runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { GIT_EXEC_PATH: old }, globals: [`--exec-path=${new}`], word: 'audit-external' })
/// ```
#[test]
fn the_exec_path_option_decides_the_search_over_the_environment() {
    let root: PathBuf = scratch("external-exec-path-option");
    let old_exec: PathBuf = root.join("old-exec");
    let new_exec: PathBuf = root.join("new-exec");
    std::fs::create_dir(&old_exec).expect("old exec directory");
    std::fs::create_dir(&new_exec).expect("new exec directory");
    program(old_exec.join("git-audit-external").as_path());
    let old_text: String = old_exec.display().to_string();
    let new_text: String = new_exec.display().to_string();
    let pairs: Vec<(&str, &str)> = vec![("GIT_EXEC_PATH", old_text.as_str()), ("PATH", "/usr/bin:/bin")];
    let option: String = format!("--exec-path={new_text}");
    let given: [&str; 1] = [option.as_str()];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        globals(given.as_slice()).as_slice(),
        Some(root.as_path()),
        OsStr::new("audit-external"),
    );
    assert_eq!(
        answer,
        git_runs_external_with_globals(root.as_path(), pairs.as_slice(), given.as_slice(), "audit-external")
    );
    assert!(!answer, "a program only the environment's exec path names was taken as the command");
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
        Some(root.as_path()),
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
        Some(root.as_path()),
        OsStr::new("audit-external"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-external"));
    assert!(answer, "the program in the exec path was not found");
    remove(root.as_path());
}

/// What:
///  A relative `GIT_EXEC_PATH` is searched from the directory Git runs in,
///  so a program there is what Git runs.
/// Why:
///  Git makes a relative exec path absolute against its own directory before it adds it to `PATH`
///  (`exec-cmd.c:325` to `exec-cmd.c:340`),
///  so the search has an answer;
///  a reading that called the relative path uncertain left the alias to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { GIT_EXEC_PATH: 'exec' }, currentDirectory: root, word: 'audit-external' })
/// ```
#[test]
fn a_relative_exec_path_is_searched_where_git_searches_it() {
    let root: PathBuf = scratch("external-relative-exec-path");
    let exec: PathBuf = root.join("exec");
    std::fs::create_dir(&exec).expect("exec path directory");
    program(exec.join("git-audit-external").as_path());
    let pairs: Vec<(&str, &str)> = vec![("GIT_EXEC_PATH", "exec"), ("PATH", "/usr/bin:/bin")];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        OsStr::new("audit-external"),
    );
    assert_eq!(answer, git_runs_external(root.as_path(), pairs.as_slice(), "audit-external"));
    assert!(answer, "the program in the relative exec path was not found");
    remove(root.as_path());
}

/// What:
///  A `PATH` entry without a leading separator is read from the directory Git runs in,
///  which the caller's `-C` options move.
/// Why:
///  `-C` changes Git's directory before it looks a command up
///  (`git.c:316` to `git.c:328`,
///  then `git.c:988`),
///  and `locate_in_PATH` joins each entry to the file as it stands
///  (`run-command.c:203` to `:218`),
///  so a program only the caller's own directory holds is one Git never runs,
///  and
///  a reading that resolved the entry against the caller's directory found it and left the alias to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { PATH: '../programs/bin:/usr/bin:/bin' }, globals: ['-C', '../B'], currentDirectory: caller })
/// ```
#[test]
fn a_relative_path_entry_is_read_from_the_directory_git_changed_into() {
    let root: PathBuf = scratch("external-relative-path-entry");
    let caller: PathBuf = root.join("A");
    let moved: PathBuf = root.join("B");
    let programs: PathBuf = root.join("programs/bin");
    std::fs::create_dir_all(&caller).expect("caller directory");
    std::fs::create_dir(&moved).expect("moved directory");
    std::fs::create_dir_all(&programs).expect("program directory");
    program(programs.join("git-audit-relative").as_path());
    let pairs: Vec<(&str, &str)> = vec![("PATH", "../programs/bin:/usr/bin:/bin")];
    let given: Vec<OsString> = globals(&["-C", "../B"]);
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        given.as_slice(),
        Some(caller.as_path()),
        OsStr::new("audit-relative"),
    );
    assert_eq!(
        answer,
        git_runs_external_with_globals(caller.as_path(), pairs.as_slice(), &["-C", "../B"], "audit-relative")
    );
    assert!(answer, "the program the moved entry names was not found");
    // A program only the directory the caller stood in holds is one Git never runs after `-C`.
    let caller_programs: PathBuf = caller.join("bin");
    std::fs::create_dir(caller_programs.as_path()).expect("caller program directory");
    program(caller_programs.join("git-audit-here").as_path());
    let here: Vec<(&str, &str)> = vec![("PATH", "bin:/usr/bin:/bin")];
    let answer_here: bool = dashed_external_selected(
        &context(),
        environment(here.as_slice()).as_slice(),
        given.as_slice(),
        Some(caller.as_path()),
        OsStr::new("audit-here"),
    );
    assert_eq!(
        answer_here,
        git_runs_external_with_globals(caller.as_path(), here.as_slice(), &["-C", "../B"], "audit-here")
    );
    assert!(!answer_here, "a program only the caller's own directory holds was taken as the command");
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
        Some(root.as_path()),
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
    let reported: Option<OsString> = printed_exec_path(&context(), &[], root.as_path());
    let expected: String = String::from_utf8_lossy(&git_output(root.as_path(), &["--exec-path"]).stdout)
        .trim_end_matches('\n')
        .to_owned();
    assert_eq!(reported, Some(OsString::from(expected)));
    // `match` picks by variant, so the reading names the present case instead of shadowing the option.
    match reported {
        Some(path) => assert!(!path.is_empty(), "Git reported an empty exec path"),
        None => panic!("Git reported no exec path"),
    }
    remove(root.as_path());
}
