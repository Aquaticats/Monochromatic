//! Controls for the Git LFS upload decision:
//! which configurations count as LFS,
//! the switch-off reading,
//! the skip
//! variable,
//! the `PATH` lookup and the explanation texts.

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

/// `git lfs install` in an account writes the filter to the global file,
/// which says nothing about one repository.
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

/// Other keys,
/// near misses in the section or subsection name,
/// and an empty configuration are not LFS.
#[test]
fn other_configuration_does_not_mark_the_repository_as_using_lfs() {
    assert!(!uses_lfs(&snapshot(&[])));
    assert!(!uses_lfs(&snapshot(&[("local", "core.hookspath", "hooks")])));
    assert!(!uses_lfs(&snapshot(&[("local", "hook.lfs.event", "pre-push")])));
    assert!(!uses_lfs(&snapshot(&[("local", "filter.lfsx.process", "x")])));
    assert!(!uses_lfs(&snapshot(&[("local", "lfsextra.url", "x")])));
    assert!(!uses_lfs(&snapshot(&[("local", "filter.other.clean", "x")])));
}

/// The last `-c` that sets the hook's `enabled` decides,
/// in the spellings Git reads as false.
#[test]
fn the_callers_switch_off_is_read_from_the_last_setting() {
    let prefix = |words: &[&str]| -> Vec<OsString> { return words.iter().map(OsString::from).collect() };
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=false"]), &[]));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=no"]), &[]));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=0"]), &[]));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=off"]), &[]));
    assert!(hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled="]), &[]));
    assert!(!hook_switched_off(&prefix(&[
        "-c",
        "hook.cli-git-lfs-pre-push.enabled=false",
        "-c",
        "hook.cli-git-lfs-pre-push.enabled=true"
    ]), &[]));
    assert!(!hook_switched_off(&prefix(&["-c", "hook.other.enabled=false"]), &[]));
    assert!(hook_switched_off(&prefix(&["-c", "HOOK.cli-git-lfs-pre-push.ENABLED=false"]), &[]), "section and variable ignore case");
    assert!(!hook_switched_off(&prefix(&["-c", "hook.CLI-GIT-LFS-PRE-PUSH.enabled=false"]), &[]), "the subsection is exact");
    assert!(
        !hook_switched_off(&prefix(&["-c", "hook.cli-git-lfs-pre-push.enabled=false", "-c", "hook.cli-git-lfs-pre-push.enabled"]), &[]),
        "a later name without a value is the boolean true"
    );
    assert!(!hook_switched_off(&prefix(&["-C", "dir"]), &[]));
    assert!(!hook_switched_off(&prefix(&["-c"]), &[]));
    assert!(!hook_switched_off(&[], &[]));
}

/// What:
///  The switch given through `--config-env` is read as the `-c` spelling is,
/// in both spellings Git accepts,
/// and the last setting decides whichever spelling the caller used.
/// Why:
///   Reading `-c` alone ran the upload the caller had disabled:
/// a failing upload stopped a push real Git would have
///  published,
/// and a successful one sent the objects the caller had switched off.
#[test]
fn the_switch_the_environment_names_is_read_as_the_callers_own() {
    let prefix = |words: &[&str]| -> Vec<OsString> { return words.iter().map(OsString::from).collect() };
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("DISABLE_UPLOAD"), OsString::from("false")),
        (OsString::from("ENABLE_UPLOAD"), OsString::from("true")),
    ];
    for words in [
        vec!["--config-env=hook.cli-git-lfs-pre-push.enabled=DISABLE_UPLOAD"],
        vec!["--config-env", "hook.cli-git-lfs-pre-push.enabled=DISABLE_UPLOAD"],
    ] {
        assert!(hook_switched_off(prefix(words.as_slice()).as_slice(), environment.as_slice()), "{words:?}");
    }
    assert!(
        !hook_switched_off(
            prefix(&["--config-env=hook.cli-git-lfs-pre-push.enabled=ENABLE_UPLOAD"]).as_slice(),
            environment.as_slice()
        ),
        "a variable holding true keeps the upload"
    );
    assert!(
        !hook_switched_off(
            prefix(&[
                "--config-env=hook.cli-git-lfs-pre-push.enabled=DISABLE_UPLOAD",
                "-c",
                "hook.cli-git-lfs-pre-push.enabled=true"
            ])
            .as_slice(),
            environment.as_slice()
        ),
        "a later -c keeps the upload"
    );
    assert!(
        hook_switched_off(
            prefix(&[
                "-c",
                "hook.cli-git-lfs-pre-push.enabled=true",
                "--config-env=hook.cli-git-lfs-pre-push.enabled=DISABLE_UPLOAD"
            ])
            .as_slice(),
            environment.as_slice()
        ),
        "a later --config-env switches the upload off"
    );
    // Git refuses a `--config-env` the environment cannot answer, so the push never runs and no setting of the switch was made.
    for words in [
        vec!["--config-env=hook.cli-git-lfs-pre-push.enabled=MISSING"],
        vec!["--config-env=hook.cli-git-lfs-pre-push.enabled"],
    ] {
        assert!(!hook_switched_off(prefix(words.as_slice()).as_slice(), environment.as_slice()), "{words:?}");
    }
    // A switch the numbered pairs set still stands: the refused command line ends the push before anything is uploaded.
    let numbered: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_KEY_0"), OsString::from("hook.cli-git-lfs-pre-push.enabled")),
        (OsString::from("GIT_CONFIG_VALUE_0"), OsString::from("false")),
    ];
    assert!(
        hook_switched_off(
            prefix(&["--config-env=hook.cli-git-lfs-pre-push.enabled=MISSING"]).as_slice(),
            numbered.as_slice()
        ),
        "the numbered pair still switches the upload off"
    );
}

/// The lookup finds an executable file in any `PATH` directory and refuses a directory,
/// a plain file and nothing.
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
        "`git-lfs push` exited with status 2",
        "this push was not started and nothing was published",
        "`lfs.url`",
        "git lfs push --all <remote>",
        "git push --no-verify",
        "GIT_LFS_SKIP_PUSH=1",
    ] {
        assert!(message.contains(needle), "{needle}: {message}");
    }
}

/// `GIT_LFS_SKIP_PUSH` set to a true value asks to skip the upload;
/// absent or false values do not.
#[test]
fn the_skip_variable_is_read_as_a_boolean() {
    let with = |value: &str| -> Vec<(OsString, OsString)> {
        return vec![(OsString::from("GIT_LFS_SKIP_PUSH"), OsString::from(value))];
    };
    for truthy in ["1", "t", "T", "true", "TRUE", "y", "yes", "on"] {
        assert!(skip_requested(with(truthy).as_slice()), "{truthy}");
    }
    for other in ["0", "false", "", "no", "off", "2"] {
        assert!(!skip_requested(with(other).as_slice()), "{other:?}");
    }
    assert!(!skip_requested(&[]));
}

//region OpenSSH's words for a host key this machine does not know, and for one that changed

/// OpenSSH's own words for a host key this machine does not know,
/// under the account's default `StrictHostKeyChecking=ask`,
/// measured against a throwaway `sshd` on `127.0.0.1` with OpenSSH 10.6p2 and the wrapper's two variables
/// (`SSH_ASKPASS=false`,
/// `SSH_ASKPASS_REQUIRE=force`):
/// the yes/no question goes to a program that answers nothing,
/// and an empty answer is not "yes".
const AN_UNKNOWN_HOST_KEY: &str = "Host key verification failed.\n";

/// The words for a host whose key this machine's `known_hosts` holds as a different one:
/// OpenSSH warns that someone may be attacking the connection,
/// and then prints the same last line an unknown key gets.
const A_CHANGED_HOST_KEY: &str = "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @\n\
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!\n\
Someone could be eavesdropping on you right now (man-in-the-middle attack)!\n\
It is also possible that a host key has just been changed.\n\
The fingerprint for the ED25519 key sent by the remote host is\n\
SHA256:ZaVZ8S9YV0dQ4EG1Jc1iNoB8wqvbUpvB0GbUlZrKz3E.\n\
Please contact your system administrator.\n\
Add correct host key in a known_hosts file to get rid of this message.\n\
Offending ED25519 key in a known_hosts file:1\n\
Host key for [127.0.0.1]:41783 has changed and you have requested strict checking.\n\
Host key verification failed.\n";

/// The words for a host this machine does not know while the account asks for strict checking:
/// the refusal is still an unknown key,
/// and every way out the wrapper prints still applies.
const AN_UNKNOWN_HOST_KEY_UNDER_STRICT_CHECKING: &str =
    "No ED25519 host key is known for [127.0.0.1]:41783 and you have requested strict checking.\nHost key verification failed.\n";

/// What:
///  A host key this machine does not know is read,
///  whether the account leaves the question open or asks for strict checking.
/// Why:
///   Both are openSSH's refusal to connect to a host whose key this machine has none of,
///  which is the refusal the no-prompt rule turns into an empty answer,
///  so both get the sentence that names the two ways to add the key.
#[test]
fn a_host_key_this_machine_does_not_know_is_read() {
    for text in [AN_UNKNOWN_HOST_KEY, AN_UNKNOWN_HOST_KEY_UNDER_STRICT_CHECKING] {
        assert!(reads_as_unknown_host_key(text), "{text}");
    }
}

/// What:
///  A host key that changed is not read as an unknown one.
/// Why:
///   OpenSSH has just said that this machine knows a key for the host and that the key it was given differs,
///  and that someone may be attacking the connection;
///  telling the person the key is unknown and to add the new one would answer that warning with the opposite advice.
///  OpenSSH's own words stay,
/// as they do on `main`.
#[test]
fn a_changed_host_key_keeps_opensshs_own_warning_and_is_no_unknown_key() {
    assert!(
        !reads_as_unknown_host_key(A_CHANGED_HOST_KEY),
        "a changed key is not an unknown one, and its transcript is OpenSSH's warning of a possible attack"
    );
}

//endregion

/// A relative `PATH` entry is resolved where the lookup happens,
/// so the file found is the file that runs even when the
///  child starts in another directory (`-C`).
#[test]
fn a_relative_path_entry_is_found_as_an_absolute_file() {
    let root = fixture("lfs-relative-path");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir(&bin).expect("bin directory");
    executable(bin.join("git-lfs").as_path(), b"#!/bin/sh\n");
    std::env::set_current_dir(&root).expect("the fixture is the lookup directory");
    let found: PathBuf = find_program(OsStr::new("bin")).expect("found through the relative entry");
    assert!(found.is_absolute(), "{found:?}");
    assert_eq!(found, root.join("bin/git-lfs"));
    assert!(program_on_path(OsStr::new("bin")));
    remove(root.as_path());
}
