//! Controls for the Git LFS pre-push hook: which configurations count as LFS, the hook definition tokens and their
//! shell quoting, the entry words, and the hook entry's pass-through of arguments, input and status.

use super::*;
use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;

/// A snapshot of `git config --list -z --show-scope` built from `(scope, key, value)` triples.
fn snapshot(entries: &[(&str, &str, &str)]) -> ConfigSnapshot {
    let mut listing: Vec<u8> = Vec::new();
    for (scope, key, value) in entries {
        listing.extend_from_slice(scope.as_bytes());
        listing.push(0);
        listing.extend_from_slice(key.as_bytes());
        listing.push(b'\n');
        listing.extend_from_slice(value.as_bytes());
        listing.push(0);
    }
    return ConfigSnapshot::parse(listing.as_slice(), true).expect("listing parses");
}

/// The keys the filter's first run and a server URL write mark a repository as using LFS in its own scopes.
#[test]
fn lfs_keys_of_the_repository_mark_it_as_using_lfs() {
    assert!(uses_lfs(&snapshot(&[("local", "lfs.repositoryformatversion", "0")])));
    assert!(uses_lfs(&snapshot(&[("local", "lfs.https://example.invalid/x.git/info/lfs.access", "basic")])));
    assert!(uses_lfs(&snapshot(&[("local", "filter.lfs.process", "git-lfs filter-process")])));
    assert!(uses_lfs(&snapshot(&[("worktree", "lfs.url", "http://127.0.0.1:1")])));
    assert!(uses_lfs(&snapshot(&[("command", "lfs.url", "http://127.0.0.1:1")])));
    assert!(uses_lfs(&snapshot(&[("system", "core.autocrlf", "false"), ("local", "lfs.url", "x")])));
}

/// `git lfs install` in an account writes the filter to the global file, which says nothing about one repository.
#[test]
fn lfs_keys_of_the_machine_or_account_do_not_mark_a_repository_as_using_lfs() {
    assert!(!uses_lfs(&snapshot(&[("global", "filter.lfs.process", "git-lfs filter-process")])));
    assert!(!uses_lfs(&snapshot(&[("global", "lfs.customtransfer.xet.path", "git-xet")])));
    assert!(!uses_lfs(&snapshot(&[("system", "filter.lfs.clean", "git-lfs clean")])));
    assert!(!uses_lfs(&snapshot(&[
        ("global", "filter.lfs.smudge", "git-lfs smudge"),
        ("local", "core.bare", "false"),
    ])));
}

/// Other keys, near misses in the section or subsection name, and an empty configuration are not LFS.
#[test]
fn other_configuration_does_not_mark_the_repository_as_using_lfs() {
    assert!(!uses_lfs(&snapshot(&[])));
    assert!(!uses_lfs(&snapshot(&[("local", "core.hookspath", "hooks")])));
    assert!(!uses_lfs(&snapshot(&[("local", "hook.lfs.event", "pre-push")])));
    assert!(!uses_lfs(&snapshot(&[("local", "filter.lfsx.process", "x")])));
    assert!(!uses_lfs(&snapshot(&[("local", "lfsextra.url", "x")])));
    assert!(!uses_lfs(&snapshot(&[("local", "filter.other.clean", "x")])));
}

/// Quoting keeps every byte literal: plain text, spaces, an embedded single quote, and the empty string.
#[test]
fn shell_quoting_keeps_one_literal_word() {
    assert_eq!(shell_quote(OsStr::new("/usr/bin/x")), OsString::from("'/usr/bin/x'"));
    assert_eq!(shell_quote(OsStr::new("/a b/x")), OsString::from("'/a b/x'"));
    assert_eq!(shell_quote(OsStr::new("it's")), OsString::from("'it'\\''s'"));
    assert_eq!(shell_quote(OsStr::new("")), OsString::from("''"));
}

/// The hook definition is two `-c` options: the event, then the command that names the executable and the entry.
#[test]
fn the_hook_definition_names_the_event_and_the_wrapper_entry() {
    let tokens: Vec<OsString> = hook_arguments(Path::new("/opt/cli git/cli-git-native"));
    assert_eq!(
        tokens,
        vec![
            OsString::from("-c"),
            OsString::from("hook.cli-git-lfs-pre-push.event=pre-push"),
            OsString::from("-c"),
            OsString::from("hook.cli-git-lfs-pre-push.command='/opt/cli git/cli-git-native' cli-git lfs-pre-push"),
        ]
    );
}

/// The hook goes before the caller's global options, so a later `-c` of the caller still wins.
#[test]
fn the_hook_precedes_the_callers_arguments() {
    let combined: Vec<OsString> = with_lfs_hook(
        &[OsString::from("-c"), OsString::from("a=b"), OsString::from("push")],
        Path::new("/x"),
    );
    assert_eq!(combined.len(), 7);
    assert_eq!(combined[0], OsString::from("-c"));
    assert_eq!(combined[3].to_string_lossy(), "hook.cli-git-lfs-pre-push.command='/x' cli-git lfs-pre-push");
    assert_eq!(&combined[4..], &[OsString::from("-c"), OsString::from("a=b"), OsString::from("push")]);
}

/// Only the exact management words start the hook entry; the rest of the arguments belong to the hook.
#[test]
fn only_the_exact_entry_words_start_the_hook_entry() {
    let entry: Vec<OsString> = ["cli-git", "lfs-pre-push", "origin", "https://example.invalid/r.git"]
        .iter()
        .map(OsString::from)
        .collect();
    assert_eq!(
        hook_entry_arguments(entry.as_slice()),
        Some(&entry[2..])
    );
    let bare: Vec<OsString> = ["cli-git", "lfs-pre-push"].iter().map(OsString::from).collect();
    assert_eq!(hook_entry_arguments(bare.as_slice()), Some(&bare[2..]));
    for refused in [
        vec!["cli-git"],
        vec!["cli-git", "doctor"],
        vec!["-c", "x=y", "cli-git", "lfs-pre-push"],
        vec!["lfs-pre-push", "cli-git"],
        vec!["push", "origin"],
        vec![],
    ] {
        let arguments: Vec<OsString> = refused.iter().map(OsString::from).collect();
        assert_eq!(hook_entry_arguments(arguments.as_slice()), None, "{refused:?}");
    }
}

/// The missing-program refusal names the cause and each way forward.
#[test]
fn the_missing_program_message_names_the_cause_and_every_way_forward() {
    let message: String = missing_program_message(&std::io::Error::from(std::io::ErrorKind::NotFound));
    for needle in [
        "stopped before it sent anything",
        "`git-lfs` could not be started",
        "install Git LFS",
        "git lfs uninstall",
        "git push --no-verify",
        "git lfs push --all <remote>",
    ] {
        assert!(message.contains(needle), "{needle}: {message}");
    }
    assert!(!message.contains('\u{2014}') && !message.contains('\u{2013}'));
}

/// The failure explanation says nothing was published and names each way forward.
#[test]
fn the_failure_message_says_nothing_was_published_and_names_every_way_forward() {
    let message: String = failed_message("exited with status 2");
    for needle in [
        "`git-lfs pre-push` exited with status 2",
        "before any ref changed and nothing was published",
        "`lfs.url`",
        "git lfs push --all <remote>",
        "git push --no-verify",
        "GIT_LFS_SKIP_PUSH=1",
    ] {
        assert!(message.contains(needle), "{needle}: {message}");
    }
}

/// A stand-in `git-lfs` that records its arguments and standard input, then exits with `status`.
fn recording_program(root: &Path, status: i32) -> PathBuf {
    let program: PathBuf = root.join("fake-git-lfs");
    let source: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{root}/arguments.txt'\ncat > '{root}/input.txt'\nexit {status}\n",
        root = root.display()
    );
    executable(program.as_path(), source.as_bytes());
    return program;
}

/// Run the entry against a stand-in program, feeding no input, and return the status and diagnostics.
fn run_entry(program: &Path, arguments: &[&str]) -> (i32, String) {
    let owned: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let mut diagnostics: Vec<u8> = Vec::new();
    let status: i32 = run_with_program(program.as_os_str(), owned.as_slice(), &mut diagnostics);
    return (status, String::from_utf8_lossy(diagnostics.as_slice()).into_owned());
}

/// The program receives `pre-push` and then the hook's arguments; a zero status is silent success.
#[test]
fn the_entry_passes_the_hook_arguments_to_pre_push_and_succeeds_silently() {
    let root: PathBuf = fixture("lfs-entry-pass");
    let program: PathBuf = recording_program(root.as_path(), 0);
    let (status, diagnostics) = run_entry(program.as_path(), &["origin", "https://example.invalid/r.git"]);
    assert_eq!(status, 0);
    assert_eq!(diagnostics, "");
    let recorded: String = std::fs::read_to_string(root.join("arguments.txt")).expect("arguments");
    assert_eq!(recorded, "pre-push\norigin\nhttps://example.invalid/r.git\n");
    remove(root.as_path());
}

/// A non-zero status of the program is the hook's status, and the explanation follows the program's own output.
#[test]
fn the_entry_returns_the_programs_status_and_explains_a_failure() {
    let root: PathBuf = fixture("lfs-entry-fail");
    let program: PathBuf = recording_program(root.as_path(), 7);
    let (status, diagnostics) = run_entry(program.as_path(), &["origin", "u"]);
    assert_eq!(status, 7);
    assert!(diagnostics.contains("`git-lfs pre-push` exited with status 7"), "{diagnostics}");
    assert!(diagnostics.contains("nothing was published"), "{diagnostics}");
    remove(root.as_path());
}

/// A program that cannot be started blocks the push with status 1 and the install-or-remove explanation.
#[test]
fn the_entry_blocks_when_the_program_cannot_be_started() {
    let root: PathBuf = fixture("lfs-entry-missing");
    let (status, diagnostics) = run_entry(root.join("absent").as_path(), &["origin", "u"]);
    assert_eq!(status, 1);
    assert!(diagnostics.contains("could not be started"), "{diagnostics}");
    assert!(diagnostics.contains("install Git LFS"), "{diagnostics}");
    remove(root.as_path());
}

/// A program ended by a signal blocks too, and the explanation names the signal.
#[test]
fn the_entry_blocks_when_the_program_is_ended_by_a_signal() {
    let root: PathBuf = fixture("lfs-entry-signal");
    let program: PathBuf = root.join("killed");
    executable(program.as_path(), b"#!/bin/sh\nkill -TERM $$\nsleep 5\n");
    let (status, diagnostics) = run_entry(program.as_path(), &["origin", "u"]);
    assert_eq!(status, 128 + 15);
    assert!(diagnostics.contains("was ended by signal 15"), "{diagnostics}");
    remove(root.as_path());
}

/// The quoted hook command, run by a shell as Git runs it, reaches an executable whose path has a space and a quote.
#[test]
fn the_hook_command_survives_a_shell_with_an_awkward_executable_path() {
    let root: PathBuf = fixture("lfs-entry-quoting");
    let directory: PathBuf = root.join("it's a dir");
    std::fs::create_dir(&directory).expect("directory");
    let program: PathBuf = directory.join("cli git");
    let record: PathBuf = root.join("record.txt");
    let source: String = format!("#!/bin/sh\nprintf '%s|' \"$@\" > '{}'\n", record.display());
    executable(program.as_path(), source.as_bytes());
    let tokens: Vec<OsString> = hook_arguments(program.as_path());
    let command: String = tokens[3].to_string_lossy().replacen("hook.cli-git-lfs-pre-push.command=", "", 1);
    // Git appends the hook's own arguments to a shell command as `"$@"`.
    let status = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("{command} \"$@\""))
        .arg("sh")
        .arg("origin")
        .arg("a url")
        .status()
        .expect("sh starts");
    assert!(status.success());
    assert_eq!(std::fs::read_to_string(record).expect("record"), "cli-git|lfs-pre-push|origin|a url|");
    remove(root.as_path());
}
