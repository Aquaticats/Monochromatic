//! What:
//!  Controls for the Git child environment additions,
//!  checked against real Git.
//! Why:
//!  The overlay must append after exactly the entries Git 2.56.0 reads and must
//!      never change what Git reports for a caller's malformed variables.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(lockfilePidOverlay([])).toEqual([['GIT_CONFIG_COUNT', '1'], ...]);
//! ```

/// Import the functions under test and native string types.
use super::{
    ASKPASS_VARIABLE, CHILD_SET_PREFIX, COUNT_VARIABLE, FORWARD_TARGET_VARIABLE, TERMINAL_PROMPT_VARIABLE, child_environment_overlay,
    environment_with_overlay, invocation_overlay, lockfile_pid_overlay, never_prompt_overlay, prompting_disabled, service_child_overlay,
};
/// The answer about the process's controlling terminal the invocation overlay is built from.
use crate::control_terminal::Terminal;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// What:
///  Build an owned environment list from text pairs.
/// Why:
///  `pub(super)` lets the sibling count controls in `child_environment_count_tests.rs` reuse it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function environment(pairs: [string, string][]): [string, string][] { return pairs; }
/// ```
pub(super) fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut result: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
    for (name, value) in pairs {
        result.push((OsString::from(name), OsString::from(value)));
    }
    return result;
}

/// The three variables appended at one index.
fn appended(index: u64) -> Vec<(OsString, OsString)> {
    return environment(&[
        (COUNT_VARIABLE, (index + 1).to_string().as_str()),
        (
            format!("GIT_CONFIG_KEY_{index}").as_str(),
            "core.lockfilePid",
        ),
        (format!("GIT_CONFIG_VALUE_{index}").as_str(), "true"),
    ]);
}

/// With no numbered configuration the entry is appended at index zero.
#[test]
fn overlay_appends_after_existing_entries() {
    assert_eq!(lockfile_pid_overlay(&[]), appended(0));
    assert_eq!(
        lockfile_pid_overlay(environment(&[(COUNT_VARIABLE, "")]).as_slice()),
        appended(0)
    );
    assert_eq!(
        lockfile_pid_overlay(
            environment(&[
                (COUNT_VARIABLE, " 2"),
                ("GIT_CONFIG_KEY_0", "user.name"),
                ("GIT_CONFIG_VALUE_0", "A"),
                ("GIT_CONFIG_KEY_1", "core.lockfilePid"),
                ("GIT_CONFIG_VALUE_1", "false"),
            ])
            .as_slice()
        ),
        appended(2)
    );
}

/// A last entry that already reads true,
///  in any key or value case,
///  adds nothing.
#[test]
fn overlay_is_idempotent_when_the_last_entry_is_true() {
    for (key, value) in [
        ("core.lockfilePid", "true"),
        ("CORE.LOCKFILEPID", "TRUE"),
        ("core.lockfilepid", "Yes"),
        ("Core.LockfilePid", "on"),
        ("core.lockfilePid", "1"),
    ] {
        let pairs: Vec<(OsString, OsString)> = environment(&[
            (COUNT_VARIABLE, "2"),
            ("GIT_CONFIG_KEY_0", "core.lockfilePid"),
            ("GIT_CONFIG_VALUE_0", "false"),
            ("GIT_CONFIG_KEY_1", key),
            ("GIT_CONFIG_VALUE_1", value),
        ]);
        assert_eq!(
            lockfile_pid_overlay(pairs.as_slice()),
            Vec::<(OsString, OsString)>::new(),
            "{key}={value}"
        );
    }
    let nested: Vec<(OsString, OsString)> = appended(0);
    assert_eq!(
        lockfile_pid_overlay(nested.as_slice()),
        Vec::<(OsString, OsString)>::new()
    );
}

/// Only the last matching entry decides;
///  other keys and non-true spellings do not suppress the append.
#[test]
fn overlay_appends_when_the_effective_entry_is_not_true() {
    for (first, last) in [
        ("true", "false"),
        ("true", "no"),
        ("true", "0"),
        ("true", ""),
        ("true", "truthy"),
        ("true", "2"),
    ] {
        let pairs: Vec<(OsString, OsString)> = environment(&[
            (COUNT_VARIABLE, "2"),
            ("GIT_CONFIG_KEY_0", "core.lockfilePid"),
            ("GIT_CONFIG_VALUE_0", first),
            ("GIT_CONFIG_KEY_1", "core.lockfilePid"),
            ("GIT_CONFIG_VALUE_1", last),
        ]);
        assert_eq!(
            lockfile_pid_overlay(pairs.as_slice()),
            appended(2),
            "{last:?}"
        );
    }
    let other_key: Vec<(OsString, OsString)> = environment(&[
        (COUNT_VARIABLE, "1"),
        ("GIT_CONFIG_KEY_0", "core.lockfilePidX"),
        ("GIT_CONFIG_VALUE_0", "true"),
    ]);
    assert_eq!(lockfile_pid_overlay(other_key.as_slice()), appended(1));
    // An entry beyond the declared count is not read by Git and must not count.
    let beyond: Vec<(OsString, OsString)> = environment(&[
        (COUNT_VARIABLE, "0"),
        ("GIT_CONFIG_KEY_0", "core.lockfilePid"),
        ("GIT_CONFIG_VALUE_0", "true"),
    ]);
    assert_eq!(lockfile_pid_overlay(beyond.as_slice()), appended(0));
}

/// Values Git itself rejects are left for Git to report:
///  nothing is added.
#[test]
fn overlay_leaves_malformed_numbered_configuration_untouched() {
    for pairs in [
        environment(&[(COUNT_VARIABLE, "x")]),
        environment(&[(COUNT_VARIABLE, "-1")]),
        environment(&[(COUNT_VARIABLE, "2147483648")]),
        // Declared entries that are missing make Git fail with "missing config key/value".
        environment(&[(COUNT_VARIABLE, "1")]),
        environment(&[(COUNT_VARIABLE, "1"), ("GIT_CONFIG_KEY_0", "a.b")]),
        environment(&[(COUNT_VARIABLE, "1"), ("GIT_CONFIG_VALUE_0", "v")]),
        environment(&[
            (COUNT_VARIABLE, "2147483647"),
            ("GIT_CONFIG_KEY_0", "core.lockfilePid"),
            ("GIT_CONFIG_VALUE_0", "false"),
        ]),
    ] {
        assert_eq!(
            lockfile_pid_overlay(pairs.as_slice()),
            Vec::<(OsString, OsString)>::new(),
            "{pairs:?}"
        );
    }
}

/// The complete overlay ends with the forward-target marker carrying the exact path bytes.
#[test]
fn child_overlay_adds_the_forward_target_marker() {
    let overlay: Vec<(OsString, OsString)> =
        child_environment_overlay(&[], Path::new("/usr/bin/git"));
    let mut expected: Vec<(OsString, OsString)> = appended(0);
    expected.push((
        OsString::from(FORWARD_TARGET_VARIABLE),
        OsString::from("/usr/bin/git"),
    ));
    assert_eq!(overlay, expected);
    let already_true: Vec<(OsString, OsString)> = appended(0);
    assert_eq!(
        child_environment_overlay(already_true.as_slice(), Path::new("relative/git")),
        vec![(
            OsString::from(FORWARD_TARGET_VARIABLE),
            OsString::from("relative/git")
        )]
    );
}

/// A non-UTF-8 real-Git path is carried into the marker without any decoding.
#[cfg(unix)]
#[test]
fn forward_target_marker_preserves_non_utf8_paths() {
    use std::os::unix::ffi::OsStringExt;
    let raw: OsString = OsString::from_vec(b"/opt/g\xff/git".to_vec());
    let overlay: Vec<(OsString, OsString)> =
        child_environment_overlay(appended(0).as_slice(), Path::new(&raw));
    assert_eq!(
        overlay,
        vec![(OsString::from(FORWARD_TARGET_VARIABLE), raw)]
    );
}

/// Real Git applies the overlay,
///  and an explicit caller `-c` still wins over it.
#[test]
fn native_git_reads_the_overlay_and_explicit_configuration_wins() {
    let inherited: Vec<(OsString, OsString)> = environment(&[
        (COUNT_VARIABLE, "1"),
        ("GIT_CONFIG_KEY_0", "core.lockfilePid"),
        ("GIT_CONFIG_VALUE_0", "false"),
    ]);
    let overlay: Vec<(OsString, OsString)> = lockfile_pid_overlay(inherited.as_slice());
    for (extra, expected) in [
        (Vec::<&str>::new(), "true\n"),
        (vec!["-c", "core.lockfilePid=false"], "false\n"),
    ] {
        let mut command: Command = Command::new("/usr/bin/git");
        command
            .env_clear()
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .current_dir("/");
        for (name, value) in &inherited {
            command.env(name, value);
        }
        for (name, value) in &overlay {
            command.env(name, value);
        }
        let output = command
            .args(extra)
            .args(["config", "--get", "core.lockfilepid"])
            .output()
            .expect("native Git overlay probe");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

/// Every caller variable survives,
/// in order,
/// ahead of the overlay's entries.
#[test]
fn an_overlaid_environment_keeps_the_callers_variables_and_appends_the_overlay() {
    let caller = environment(&[("HOME", "/home/user"), ("PATH", "/usr/bin"), ("SSH_AUTH_SOCK", "/run/agent")]);
    let overlay = environment(&[("CLI_GIT_NATIVE_FORWARD_TARGET", "/usr/bin/git")]);
    assert_eq!(
        environment_with_overlay(caller.as_slice(), overlay.as_slice()),
        environment(&[
            ("HOME", "/home/user"),
            ("PATH", "/usr/bin"),
            ("SSH_AUTH_SOCK", "/run/agent"),
            ("CLI_GIT_NATIVE_FORWARD_TARGET", "/usr/bin/git"),
        ])
    );
}

/// A name the overlay sets keeps only the overlay's value,
/// wherever the caller had it.
#[test]
fn an_overlaid_environment_replaces_a_name_the_overlay_sets() {
    let caller = environment(&[("GIT_CONFIG_COUNT", "1"), ("HOME", "/home/user"), ("GIT_CONFIG_COUNT", "7")]);
    let overlay = environment(&[("GIT_CONFIG_COUNT", "2"), ("GIT_CONFIG_KEY_1", "core.lockfilePid")]);
    assert_eq!(
        environment_with_overlay(caller.as_slice(), overlay.as_slice()),
        environment(&[
            ("HOME", "/home/user"),
            ("GIT_CONFIG_COUNT", "2"),
            ("GIT_CONFIG_KEY_1", "core.lockfilePid"),
        ])
    );
}

/// Without an overlay the caller's environment is returned unchanged;
/// without a caller environment,
/// the overlay alone.
#[test]
fn an_overlaid_environment_with_an_empty_side_is_the_other_side() {
    let caller = environment(&[("HOME", "/home/user")]);
    assert_eq!(environment_with_overlay(caller.as_slice(), &[]), caller);
    assert_eq!(environment_with_overlay(&[], caller.as_slice()), caller);
}

/// The pair that ends a login prompt at once:
/// an empty askpass program and no terminal prompt.
#[test]
fn never_prompt_overlay_sets_an_empty_askpass_and_no_terminal_prompt() {
    assert_eq!(
        never_prompt_overlay(),
        vec![
            (OsString::from(ASKPASS_VARIABLE), OsString::new()),
            (OsString::from(TERMINAL_PROMPT_VARIABLE), OsString::from("0")),
        ]
    );
}

/// What the commit service adds for its own Git children is the lock PID injection and then the never-prompt pair.
#[test]
fn the_service_overlay_is_the_lockfile_overlay_then_the_never_prompt_pair() {
    let inherited = environment(&[("HOME", "/home/someone")]);
    let mut expected = lockfile_pid_overlay(inherited.as_slice());
    expected.extend(never_prompt_overlay());
    assert_eq!(service_child_overlay(inherited.as_slice()), expected);
}

/// A command the person typed keeps Git's own prompting:
/// the overlay of forwarded commands holds neither variable.
#[test]
fn the_overlay_of_a_forwarded_command_leaves_prompting_alone() {
    let overlay = child_environment_overlay(environment(&[]).as_slice(), Path::new("/usr/bin/git"));
    for (name, _value) in &overlay {
        assert_ne!(name, ASKPASS_VARIABLE);
        assert_ne!(name, TERMINAL_PROMPT_VARIABLE);
    }
}

/// The host's askpass settings do not survive the service overlay:
/// an inherited `GIT_ASKPASS` and `GIT_TERMINAL_PROMPT` are
///  replaced,
/// and `SSH_ASKPASS` stays for OpenSSH.
#[test]
fn the_service_overlay_replaces_inherited_prompt_settings_and_keeps_ssh_askpass() {
    let inherited = environment(&[("GIT_ASKPASS", "/usr/bin/other"), ("GIT_TERMINAL_PROMPT", "1"), ("SSH_ASKPASS", "/usr/bin/ksshaskpass")]);
    let complete = environment_with_overlay(inherited.as_slice(), service_child_overlay(inherited.as_slice()).as_slice());
    let value = |name: &str| -> Vec<OsString> { return complete.iter().filter(|(key, _)| { return key == name; }).map(|(_, found)| { return found.clone(); }).collect(); };
    assert_eq!(value("GIT_ASKPASS"), vec![OsString::new()]);
    assert_eq!(value("GIT_TERMINAL_PROMPT"), vec![OsString::from("0")]);
    assert_eq!(value("SSH_ASKPASS"), vec![OsString::from("/usr/bin/ksshaskpass")]);
}

/// What:
///  The overlay of a command whose process has no controlling terminal carries the never-prompt pair,
/// and the one of a command at a terminal does not.
/// Why:
///  This is the whole difference between a push a daemon started and a push a person typed;
/// the pair is what
///  keeps the desktop password dialog from opening on a machine whose `SSH_ASKPASS` names one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(invocationOverlay(env, git, 'absent')).toContainEqual(['GIT_ASKPASS', '']);
/// expect(invocationOverlay(env, git, 'present')).not.toContainEqual(['GIT_ASKPASS', '']);
/// ```
#[test]
fn the_invocation_overlay_never_prompts_only_without_a_controlling_terminal() {
    let inherited = environment(&[("HOME", "/home/someone")]);
    let mut expected = child_environment_overlay(inherited.as_slice(), Path::new("/usr/bin/git"));
    assert_eq!(invocation_overlay(inherited.as_slice(), Path::new("/usr/bin/git"), Terminal::Present), expected);
    expected.extend(never_prompt_overlay());
    assert_eq!(invocation_overlay(inherited.as_slice(), Path::new("/usr/bin/git"), Terminal::Absent), expected);
}

/// A variable a caller set for its own Git children cannot reopen the prompt of an unattended command:
/// the never-prompt pair comes after it.
#[test]
fn the_never_prompt_pair_wins_over_a_caller_set_child_variable() {
    let inherited = environment(&[(format!("{CHILD_SET_PREFIX}{ASKPASS_VARIABLE}").as_str(), "/usr/bin/ksshaskpass")]);
    let overlay = invocation_overlay(inherited.as_slice(), Path::new("/usr/bin/git"), Terminal::Absent);
    let askpass: Vec<&OsString> = overlay.iter().filter(|(name, _value)| { return name == ASKPASS_VARIABLE; }).map(|(_name, value)| { return value; }).collect();
    assert_eq!(askpass.last(), Some(&&OsString::new()), "the last entry wins, as an environment block reads it: {askpass:?}");
}

/// What:
///  The reading of whether a list of variables turns prompting off wants both halves of the never-prompt pair,
/// and each half alone is not the pair.
/// Why:
///   The explanation after a failed login says what the wrapper did about prompting,
///  so a list that carries only one of the two variables must not be read as "the prompt was disabled".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(promptingDisabled([])).toBe(false); expect(promptingDisabled([['GIT_ASKPASS', ''], ['GIT_TERMINAL_PROMPT', '0']])).toBe(true);
/// ```
#[test]
fn prompting_is_disabled_only_by_both_halves_of_the_never_prompt_pair() {
    assert!(!prompting_disabled(&environment(&[])), "an empty list turns nothing off");
    assert!(!prompting_disabled(&environment(&[(ASKPASS_VARIABLE, "")])), "an empty askpass alone does not stop a terminal prompt");
    assert!(!prompting_disabled(&environment(&[(TERMINAL_PROMPT_VARIABLE, "0")])), "a terminal refusal alone does not stop the askpass program");
    assert!(
        !prompting_disabled(&environment(&[(ASKPASS_VARIABLE, "/usr/bin/other"), (TERMINAL_PROMPT_VARIABLE, "0")])),
        "an askpass program of the caller's own still answers"
    );
    assert!(
        prompting_disabled(&environment(&[(ASKPASS_VARIABLE, ""), (TERMINAL_PROMPT_VARIABLE, "0")])),
        "the pair itself turns prompting off"
    );
}
