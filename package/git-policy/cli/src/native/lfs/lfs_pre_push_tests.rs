//! Controls for the Git LFS upload decision:
//! which configurations count as LFS,
//! the switch-off reading,
//! the skip
//! variable,
//! the `PATH` lookup and the explanation texts.

use super::*;
/// The list,
/// its sources and the refusal that names them,
/// which the sibling file holds.
use crate::lfs_program_search::{PUSH_FROM_SERVICE, PUSH_FROM_THE_REQUEST, ProgramSearch, missing_program_message};
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
    assert!(program_on_path(joined(&[&without, &with_program]).as_os_str(), std::path::Path::new("/")));
    assert!(!program_on_path(joined(&[&without, &plain, &directory_named_so]).as_os_str(), std::path::Path::new("/")));
    assert!(!program_on_path(OsStr::new(""), std::path::Path::new("/")));
    remove(root.as_path());
}

/// The missing-program refusal names the cause and each way forward.
#[test]
fn the_missing_program_message_names_the_cause_and_every_way_forward() {
    let search: ProgramSearch = ProgramSearch::of(&[(OsString::from("PATH"), OsString::from("/opt/lfs/bin:/usr/bin"))]);
    let message: String = missing_program_message(&search);
    for needle in [
        "stopped before it sent anything",
        "`git-lfs` was not found in the folders this push searches: /opt/lfs/bin, /usr/bin",
        "The list is this command's own `PATH`",
        "install Git LFS",
        "git lfs uninstall",
        "git push --no-verify",
        "git lfs push --all <remote>",
    ] {
        assert!(message.contains(needle), "{needle}: {message}");
    }
    assert!(!message.contains('\u{2014}') && !message.contains('\u{2013}'));
}

/// What:
///  A push the commit service runs says which list it searched and where that list came from,
///  so a person can tell the manager's list from their own.
/// Why:
///   A refusal that says only "not on PATH" leaves the reader unable to act:
/// the service's list is not the shell's,
///  and the shell's is the one they can change.
///  The remedy may promise only what a push from that shell really does,
/// because nothing of the committing shell travels with the service's push.
#[test]
fn the_missing_program_message_names_the_service_list_and_the_shell_it_offers() {
    let from_the_service: ProgramSearch = ProgramSearch::of(&[
        (OsString::from("PATH"), OsString::from("/usr/local/bin:/usr/bin")),
        (OsString::from(PUSH_FROM_SERVICE), OsString::from("1")),
    ]);
    let without_a_list: String = missing_program_message(&from_the_service);
    assert!(without_a_list.contains("the folders this push searches: /usr/local/bin, /usr/bin"), "{without_a_list}");
    assert!(without_a_list.contains("No list of the shell that made the commit came with this push"), "{without_a_list}");
    assert!(without_a_list.contains("the commit service's own `PATH`, which the manager that started the service fixed"), "{without_a_list}");
    assert!(
        without_a_list.contains("on the `PATH` of a shell and push by hand once from that shell (`git push`), which searches that shell's own `PATH`"),
        "{without_a_list}"
    );
    assert!(!without_a_list.contains("records that list"), "a typed push records nothing: {without_a_list}");
    assert!(!without_a_list.contains("make the commit again"), "nothing of the committing shell travels with the service's push: {without_a_list}");
    let from_this_command: ProgramSearch = ProgramSearch::of(&[(OsString::from("PATH"), OsString::from("/opt/lfs/bin"))]);
    let typed: String = missing_program_message(&from_this_command);
    assert!(typed.contains("The list is this command's own `PATH`"), "{typed}");
    assert!(typed.contains("on this command's `PATH` and push again"), "{typed}");
}

/// What:
///  A push the service runs with the list the command that asked for it gave names that command's list,
///  and remedies it with a commit made again,
///  never with a list the person cannot change.
/// Why:
///   The values travel with the request and live only in the service's memory (`service_push_programs_held_in_memory`),
/// so a refusal must name the list it really searched:
/// the shell's,
/// which a new commit hands over again,
///  rather than the manager's,
/// which no command of the person reaches.
#[test]
fn the_missing_program_message_names_the_list_the_command_that_asked_gave() {
    let from_the_request: ProgramSearch = ProgramSearch::of(&[
        (OsString::from("PATH"), OsString::from("/opt/lfs/bin:/usr/bin")),
        (OsString::from(PUSH_FROM_SERVICE), OsString::from("1")),
        (OsString::from(PUSH_FROM_THE_REQUEST), OsString::from("1")),
    ]);
    assert_eq!(from_the_request.source, crate::lfs_program_search::ProgramSearchSource::TheCommandThatAsked, "the request's list is the one named");
    let message: String = missing_program_message(&from_the_request);
    assert!(message.contains("the folders this push searches: /opt/lfs/bin, /usr/bin"), "{message}");
    assert!(message.contains("The list is the `PATH` of the shell that asked for this push"), "{message}");
    assert!(message.contains("on the `PATH` of the shell that commits, then make the commit again"), "{message}");
    assert!(!message.contains("the commit service's own `PATH`"), "the manager's list was not the one searched: {message}");
    assert!(!message.contains("records that list"), "no remedy promises a record: {message}");
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
/// measured against a throwaway `sshd` on `127.0.0.1` with OpenSSH 10.6p2 and the wrapper's two variables as they were
/// on that round (`SSH_ASKPASS=false`,
/// `SSH_ASKPASS_REQUIRE=force`;
/// the first holds the wrapper's sentence since 2026-11-24,
/// and these words without its line are what a hook or another program's push still leaves):
/// the yes/no question goes to a value that answers nothing,
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

/// The words for a host whose key this machine's `known_hosts` marks `@revoked`,
/// measured the same way as the two above:
/// OpenSSH warns that a stolen key may be in use,
/// says the key was revoked,
/// and then prints the same last line an unknown key gets.
const A_HOST_KEY_MARKED_REVOKED: &str = "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
@       WARNING: REVOKED HOST KEY DETECTED!               @\n\
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
The ED25519 host key for [127.0.0.1]:43055 is marked as revoked.\n\
This could mean that a stolen key is being used to\n\
impersonate this host.\n\
ED25519 host key for [127.0.0.1]:43055 was revoked and you have requested strict checking.\n\
Host key verification failed.\n";

/// The words for a host whose key a `RevokedHostKeys` file lists,
/// measured the same way:
/// OpenSSH names the file that revoked it,
/// and then prints the same last line an unknown key gets.
const A_HOST_KEY_REVOKED_BY_A_FILE: &str = "Host key ED25519 SHA256:hxi5iW5hp0qrgMwSe0Hxb1aY0bOOHuIp6qZxsZ+Prf4 revoked by file \
/home/someone/revoked-host-keys\n\
Host key verification failed.\n";

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

/// What:
///  A host key this machine's `known_hosts` marks as revoked is not read as an unknown one.
/// Why:
///   OpenSSH has just said that the key it was given is revoked and that a stolen key may be in use;
///  telling the person the key is unknown and to add it to `known_hosts` would answer that warning with advice to trust
///  the key a theft was suspected of,
///  and the push's record would promise a retry that adds nothing.
///  OpenSSH's own words stay,
/// as they do on `main`.
#[test]
fn a_host_key_marked_revoked_is_no_unknown_key() {
    assert!(
        !reads_as_unknown_host_key(A_HOST_KEY_MARKED_REVOKED),
        "a revoked key is not an unknown one, and its transcript is OpenSSH's warning that a stolen key may be in use"
    );
}

/// What:
///  A host key a `RevokedHostKeys` file lists is not read as an unknown one.
/// Why:
///   The account said the key is revoked and OpenSSH named the file that says so;
///  the way out above would send the person to add that very key,
///  and this file is where the account took it away.
#[test]
fn a_host_key_revoked_by_a_file_is_no_unknown_key() {
    assert!(
        !reads_as_unknown_host_key(A_HOST_KEY_REVOKED_BY_A_FILE),
        "a key revoked by a file is not an unknown one, and adding it is what the account forbade"
    );
}

/// The warning OpenSSH prints for a changed key
/// (`warn_changed_key`,
/// `sshconnect.c` at `V_10_6_P2` lines 1716 to 1726),
/// as a whole block and without the lines about the `known_hosts` entry,
/// with the last line every host-key refusal ends in.
/// The banner never comes without the sentence about the host key that changed,
/// because one function prints both.
const A_CHANGED_HOST_KEYS_WARNING_ALONE: &str = "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @\n\
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!\n\
Someone could be eavesdropping on you right now (man-in-the-middle attack)!\n\
It is also possible that a host key has just been changed.\n\
Host key verification failed.\n";

/// The line that ends OpenSSH's warning about a changed key,
/// alone with the same last line.
const A_CHANGED_HOST_KEYS_STRICT_CHECKING_LINE_ALONE: &str =
    "Host key for [127.0.0.1]:41783 has changed and you have requested strict checking.\nHost key verification failed.\n";

/// The banner OpenSSH prints for the key of a refused host that it will not trust,
/// alone with the same last line:
/// its capitals are what the reading folds.
const A_REVOKED_HOST_KEYS_BANNER_ALONE: &str = "@       WARNING: REVOKED HOST KEY DETECTED!               @\nHost key verification failed.\n";

/// The sentence that says the key is marked as revoked,
/// alone with the same last line.
const A_REVOKED_HOST_KEYS_MARKED_LINE_ALONE: &str =
    "The ED25519 host key for [127.0.0.1]:43055 is marked as revoked.\nHost key verification failed.\n";

/// The line with which OpenSSH refuses a certificate that does not fit the host
/// (`sshkey_cert_check_host`,
/// `sshkey.c` at `V_10_6_P2` lines 2480 to 2484),
/// alone with the same last line.
const A_CERTIFICATE_THAT_DOES_NOT_FIT: &str = "Certificate invalid: name is not a listed principal\nHost key verification failed.\n";

/// The words for a host key that differs from the one this machine knows for the host's IP address,
/// while the account asks for strict checking,
/// measured against a throwaway `sshd` on a private address inside a container,
/// with OpenSSH 10.6p2 and the wrapper's two variables:
/// an arm on `127.0.0.1` was allowed the connection instead,
/// because OpenSSH turns this check off for a loopback address
/// (`sshconnect.c` at `V_10_6_P2` lines 1043 to 1045).
const A_HOST_KEY_THAT_DIFFERS_FOR_THE_IP_ADDRESS_UNDER_STRICT_CHECKING: &str = "Warning: the ED25519 host key for \
'[container.invalid]:22022' differs from the key for the IP address '192.168.253.119:22022'\n\
Offending key for IP in /home/someone/.ssh/known_hosts:2\n\
Matching host key in /home/someone/.ssh/known_hosts:1\n\
Exiting, you have requested strict checking.\n\
Host key verification failed.\n";

/// The same refusal under the account's default `ask`,
/// measured the same way:
/// OpenSSH writes the warning into `confirm`'s question (`sshconnect.c` lines 1429 to 1447),
/// the askpass program gets the question,
/// and standard error holds the last line alone,
/// which is an unknown key's transcript exactly.
const A_HOST_KEY_THAT_DIFFERS_FOR_THE_IP_ADDRESS_UNDER_ASK: &str = "Host key verification failed.\n";

/// OpenSSH's own words for a host key this machine does not know while the account asks for debug output
/// (`LogLevel=DEBUG`),
/// measured against the same throwaway `sshd` with 10.6p2 and the wrapper's two variables:
/// two `debug1:` lines name the host key,
/// and the refusal's own words are still the last line alone.
const AN_UNKNOWN_HOST_KEY_WHILE_DEBUG_LINES_ARE_PRINTED: &str = "debug1: Authenticating to 127.0.0.1:PORT as 'user'\n\
debug1: kex: host key algorithm: ssh-ed25519\n\
debug1: Server host key: ssh-ed25519 SHA256:fmPLxcCMNbngQZqUl8uaZr+QD8f5lNeq/d8B3g3dRAM\n\
debug1: read_passphrase: requested to askpass\n\
Host key verification failed.\n";

/// The words of a push whose pre-push hook was refused for an unknown host key,
/// where the remote's URL names a certificate:
/// Git ends a push its hook refused with `failed to push some refs to '<url>'`,
/// and the URL is the only place the word sits.
const A_PUSH_REFUSED_FOR_AN_UNKNOWN_KEY_WHOSE_REMOTE_NAMES_A_CERTIFICATE: &str = "\
Host key verification failed.\n\
fatal: Could not read from remote repository.\n\
error: failed to push some refs to 'git@github.com:acme/certificate-manager.git'\n";

/// The same push in the spelling a German Git prints:
/// the planning record of 2026-11-19 measured that Git translates this line
/// (`doc/planning/cli-git-no-self-invocation.md`),
/// so a rule that skipped Git's English words would miss this one,
/// and Git 2.56.0 translates the prefix with the sentence (`usage.c` and `po/de.po`).
const THE_SAME_PUSH_IN_THE_SPELLING_A_GERMAN_GIT_PRINTS: &str = "Host key verification failed.\n\
Fehler: Fehler beim Versenden einiger Referenzen nach 'git@github.com:acme/certificate-manager.git'\n";

/// A connection that needed no host-key decision at all,
/// because the key it was given was already known.
const A_CONNECTION_THAT_NEEDED_NO_HOST_KEY_DECISION: &str =
    "Warning: Permanently added '127.0.0.1' (ED25519) to the list of known hosts.\n";

/// What:
///  Each sentence OpenSSH prints beside its last line for a refusal that is not an unknown key answers by itself:
///  a transcript holding that sentence and nothing else of a host key is no unknown key.
/// Why:
///   The reading holds only while every host-key line it meets is an unknown key's own,
///  so each phrase has to answer alone:
///  a transcript that also carried another refusal's words would pass whatever the reading did with either of them.
#[test]
fn each_other_refusals_sentence_alone_is_no_unknown_key() {
    for text in [
        A_CHANGED_HOST_KEYS_WARNING_ALONE,
        A_CHANGED_HOST_KEYS_STRICT_CHECKING_LINE_ALONE,
        A_REVOKED_HOST_KEYS_BANNER_ALONE,
        A_REVOKED_HOST_KEYS_MARKED_LINE_ALONE,
        A_CERTIFICATE_THAT_DOES_NOT_FIT,
        A_HOST_KEY_THAT_DIFFERS_FOR_THE_IP_ADDRESS_UNDER_STRICT_CHECKING,
    ] {
        assert!(!reads_as_unknown_host_key(text), "{text}");
    }
}

/// What:
///  A transcript that does not hold OpenSSH's last line is no unknown-key refusal,
///  whatever else it printed.
/// Why:
///   That line is the only mark that a host key stopped the connection,
///  so a connection that listed the host as known says nothing about an unknown key.
#[test]
fn a_transcript_without_the_last_line_is_no_unknown_key() {
    assert!(!reads_as_unknown_host_key(A_CONNECTION_THAT_NEEDED_NO_HOST_KEY_DECISION));
}

/// What:
///  The one refusal this reading cannot tell from an unknown key is read as an unknown key,
///  and the same refusal under strict checking is not.
/// Why:
///   With `CheckHostIP=yes` and a key this machine knows for the host beside a different one for its address,
///  OpenSSH writes the warning into `confirm`'s question,
///  so under `ask` standard error holds the last line alone and the transcript is an unknown key's exactly;
///  the sentence's way to add the key is then wrong for it and its way to connect once is right,
///  because the question a plain `ssh` asks is OpenSSH's own about the address.
///  This control holds that transcript to the limit the reading's doc states.
#[test]
fn a_host_key_that_differs_for_the_ip_address_is_read_by_whether_its_warning_was_printed() {
    assert!(
        reads_as_unknown_host_key(A_HOST_KEY_THAT_DIFFERS_FOR_THE_IP_ADDRESS_UNDER_ASK),
        "under ask the warning goes to the askpass program, so the transcript is an unknown key's"
    );
    assert!(
        !reads_as_unknown_host_key(A_HOST_KEY_THAT_DIFFERS_FOR_THE_IP_ADDRESS_UNDER_STRICT_CHECKING),
        "under strict checking OpenSSH prints the warning, so the refusal is told by its own words"
    );
}

/// What:
///  An unknown host key is read while the account asks OpenSSH for debug output.
/// Why:
///   `LogLevel=DEBUG` puts OpenSSH's own `debug1:` lines in the transcript,
///  and two of them name the host key;
///  no refusal is printed at a debug level,
///  so the refusal's own words are still the last line and the person still needs the sentence.
#[test]
fn an_unknown_host_key_reads_while_openssh_prints_its_debug_lines() {
    assert!(
        reads_as_unknown_host_key(AN_UNKNOWN_HOST_KEY_WHILE_DEBUG_LINES_ARE_PRINTED),
        "the debug lines are OpenSSH's own output and not a refusal's words"
    );
}

/// What:
///  An unknown host key is read while the remote's URL names a certificate,
///  in Git's own English spelling and in the German one this repository has measured.
/// Why:
///   Git ends every push a pre-push hook refused with `failed to push some refs to '<url>'`
///  (`doc/planning/cli-git-no-self-invocation.md`),
///  so a remote named for a certificate puts the word in the push's words;
///  the word is the URL's and not a refusal's,
///  and the push still needs the sentence and the promise.
#[test]
fn a_push_refused_for_an_unknown_host_key_reads_while_the_remote_url_names_a_certificate() {
    for text in [
        A_PUSH_REFUSED_FOR_AN_UNKNOWN_KEY_WHOSE_REMOTE_NAMES_A_CERTIFICATE,
        THE_SAME_PUSH_IN_THE_SPELLING_A_GERMAN_GIT_PRINTS,
    ] {
        assert!(reads_as_unknown_host_key(text), "the word belongs to the remote's name: {text}");
    }
}

/// The line OpenSSH prints when the value the no-prompt rule gives `SSH_ASKPASS` is not a program it can start,
/// with the refusal it precedes:
///  `ssh_askpass: exec(<the sentence>): File name too long`
///  (`readpass.c:77` and `:78` at `V_10_6_P2`),
/// then OpenSSH's own last line and Git's two advice lines.
fn an_unknown_host_key_while_the_askpass_value_is_the_wrappers_sentence() -> String {
    return format!(
        "ssh_askpass: exec({}): File name too long\nHost key verification failed.\nfatal: Could not read from remote repository.\n\n\
         Please make sure you have the correct access rights\nand the repository exists.\n",
        crate::child_environment::NO_ANSWER_SENTENCE
    );
}

/// What:
///  A refusal read from a transcript that holds the wrapper's own askpass line is still an unknown host key.
/// Why:
///   The no-prompt rule writes a sentence into `SSH_ASKPASS`,
///  and OpenSSH prints that sentence inside its own failed start before its refusal,
///  so every push and upload the wrapper runs carries this line in the words this reading judges;
///  a reading that took the line for another refusal's words would stop the upload's sentence and the service's record
///  from printing at the one refusal they exist for.
#[test]
fn an_unknown_host_key_reads_while_the_askpass_value_is_the_wrappers_sentence() {
    let text: String = an_unknown_host_key_while_the_askpass_value_is_the_wrappers_sentence();
    assert!(reads_as_unknown_host_key(text.as_str()), "the wrapper's own line is not another refusal's words: {text}");
}

/// What:
///  The sentence the no-prompt rule gives `SSH_ASKPASS` turns none of the three readings on.
/// Why:
///   The line prints on commands that succeed, and it reaches the readers that judge a failed push or upload,
///  so a sentence naming a host key or a certificate outside single quotes would stop the sentence those readers print
///  for a real refusal,
///  and one holding a login failure's words would make a successful command explain a login it never needed.
///  This is the wording's constraint written as a control:
///  a sentence edited to name a host key fails here, which is where the choice between that wording and a change to the
///  reading has to be made.
#[test]
fn the_wrappers_sentence_for_openssh_turns_no_reading_on() {
    let sentence: &str = crate::child_environment::NO_ANSWER_SENTENCE;
    assert!(
        !names_a_host_key_outside_quotes(sentence.to_ascii_lowercase().as_str()),
        "the sentence names no host key or certificate outside single quotes: {sentence}"
    );
    assert!(!reads_as_unknown_host_key(sentence), "no refusal's last line is in it: {sentence}");
    assert!(!reads_as_a_login_failure(sentence), "no login failure's words are in it: {sentence}");
    assert!(!reads_as_a_key_failure(sentence), "no server's refusal is in it: {sentence}");
    assert!(!sentence.contains('\n'), "it is one environment string: {sentence}");
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
    let found: PathBuf = find_program(OsStr::new("bin"), root.as_path()).expect("found through the relative entry");
    assert!(found.is_absolute(), "{found:?}");
    assert_eq!(found, root.join("bin/git-lfs"));
    assert!(program_on_path(OsStr::new("bin"), root.as_path()));
    remove(root.as_path());
}
