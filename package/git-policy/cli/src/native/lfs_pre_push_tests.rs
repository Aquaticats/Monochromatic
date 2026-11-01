//! Controls for the Git LFS pre-push hook: which configurations count as LFS, the hook definition tokens (plain words,
//! no shell syntax, no wrapper), the switch-off reading, the `PATH` lookup and the explanation texts.

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

/// The hook definition is two `-c` options: the event, then a command of two plain words that names no executable path.
#[test]
fn the_hook_definition_is_two_plain_words_with_no_shell_syntax_and_no_wrapper() {
    let tokens: Vec<OsString> = hook_arguments();
    assert_eq!(
        tokens,
        vec![
            OsString::from("-c"),
            OsString::from("hook.cli-git-lfs-pre-push.event=pre-push"),
            OsString::from("-c"),
            OsString::from("hook.cli-git-lfs-pre-push.command=git-lfs pre-push"),
        ]
    );
    for forbidden in ["'", "\"", "$", "`", ";", "|", "&", "<", ">", "(", ")", "{", "}", "\\", "*", "?", "\n", "cli-git-native", "cli-git lfs"] {
        assert!(!HOOK_COMMAND.contains(forbidden), "{forbidden:?} in {HOOK_COMMAND}");
    }
    assert_eq!(HOOK_COMMAND.split(' ').collect::<Vec<&str>>(), vec!["git-lfs", "pre-push"]);
}

/// The hook goes before the caller's global options, so a later `-c` of the caller still wins.
#[test]
fn the_hook_precedes_the_callers_arguments() {
    let combined: Vec<OsString> = with_lfs_hook(&[OsString::from("-c"), OsString::from("a=b"), OsString::from("push")]);
    let hook: Vec<OsString> = hook_arguments();
    assert_eq!(combined.len(), hook.len() + 3);
    assert_eq!(&combined[..hook.len()], hook.as_slice());
    assert_eq!(&combined[hook.len()..], &[OsString::from("-c"), OsString::from("a=b"), OsString::from("push")]);
}

/// The last `-c` that sets the hook's `enabled` decides, in the spellings Git reads as false.
#[test]
fn the_callers_switch_off_is_read_from_the_last_setting() {
    let prefix = |words: &[&str]| -> Vec<OsString> { return words.iter().map(OsString::from).collect() };
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=false"])));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=no"])));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=0"])));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=off"])));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled="])));
    assert!(!hook_switched_off(&prefix(&[
        "-c",
        "hook.cli-git-lfs-pre-push.enabled=false",
        "-c",
        "hook.cli-git-lfs-pre-push.enabled=true"
    ])));
    assert!(!hook_switched_off(&prefix(&["-c", "hook.other.enabled=false"])));
    assert!(!hook_switched_off(&prefix(&["-C", "dir"])));
    assert!(!hook_switched_off(&prefix(&["-c"])));
    assert!(!hook_switched_off(&[]));
}

/// The lookup finds an executable file in any `PATH` directory and refuses a directory, a plain file and nothing.
#[test]
fn the_lookup_finds_only_an_executable_file_in_a_path_directory() {
    let root: PathBuf = fixture("lfs-path-lookup");
    let with_program: PathBuf = root.join("with");
    let without: PathBuf = root.join("without");
    let plain: PathBuf = root.join("plain");
    let directory_named_so: PathBuf = root.join("dir");
    for directory in [&with_program, &without, &plain, &directory_named_so] {
        std::fs::create_dir(directory).expect("directory");
    }
    executable(with_program.join("git-lfs").as_path(), b"#!/bin/sh\n");
    std::fs::write(plain.join("git-lfs"), b"text").expect("plain file");
    std::fs::create_dir(directory_named_so.join("git-lfs")).expect("directory with the program's name");
    let joined = |directories: &[&PathBuf]| {
        return std::env::join_paths(directories).expect("joined");
    };
    assert!(program_on_path(joined(&[&without, &with_program]).as_os_str()));
    assert!(!program_on_path(joined(&[&without, &plain, &directory_named_so]).as_os_str()));
    assert!(!program_on_path(OsStr::new("")));
    remove(root.as_path());
}

/// The missing-program refusal names the cause and each way forward.
#[test]
fn the_missing_program_message_names_the_cause_and_every_way_forward() {
    let message: String = missing_program_message();
    for needle in [
        "stopped before it sent anything",
        "`git-lfs` was not found in any directory of PATH",
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

/// A plain status reads as an exit status; a status above 128 also names the signal it may stand for.
#[test]
fn a_status_above_128_names_the_signal_it_may_stand_for() {
    let terminated: i64 = SIGNAL_STATUS_BASE + i64::from(libc::SIGTERM);
    assert_eq!(status_description(1), "exited with status 1");
    assert_eq!(status_description(SIGNAL_STATUS_BASE), format!("exited with status {SIGNAL_STATUS_BASE}"));
    assert_eq!(
        status_description(terminated),
        format!("exited with status {terminated} (that is signal {} when it was ended by one)", libc::SIGTERM)
    );
    let beyond: i64 = SIGNAL_STATUS_BASE + MAX_SIGNAL + 1;
    assert_eq!(status_description(beyond), format!("exited with status {beyond}"));
}
