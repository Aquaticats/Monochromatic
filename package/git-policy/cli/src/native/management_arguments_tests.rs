//! What:
//!  Grammar controls for `git cli-git` arguments.
//! Why:
//!  A malformed invocation must be refused with the right remedy,
//!  a pathspec must
//!      never be mistaken for an option or the reverse,
//!  and retired commands must
//!      still parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseManagementArgs(['check', '--all'])).toEqual({ command: 'check', all: true, ... });
//! ```

/// Import the grammar under test.
use super::{
    MANAGEMENT_HELP, MANAGEMENT_USAGE, ManagementAction, ManagementRefusal, RetiredCommand,
    parse_management_arguments,
};
use std::ffi::OsString;

/// Parse text arguments.
fn parse(values: &[&str]) -> Result<ManagementAction, ManagementRefusal> {
    let mut arguments: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        arguments.push(OsString::from(value));
    }
    return parse_management_arguments(arguments.as_slice());
}

/// Build the expected direct action from text parts.
fn direct(fix: bool, all: bool, policies: &[&str], pathspecs: &[&str]) -> ManagementAction {
    let mut owned_policies: Vec<String> = Vec::<String>::new();
    for policy in policies {
        owned_policies.push(String::from(*policy));
    }
    let mut owned_pathspecs: Vec<OsString> = Vec::<OsString>::new();
    for pathspec in pathspecs {
        owned_pathspecs.push(OsString::from(pathspec));
    }
    return ManagementAction::Direct {
        fix,
        all,
        policies: owned_policies,
        pathspecs: owned_pathspecs,
    };
}

/// Help is `--help` or `-h` alone.
#[test]
fn help_takes_no_further_arguments() {
    assert_eq!(parse(&["--help"]), Ok(ManagementAction::Help));
    assert_eq!(parse(&["-h"]), Ok(ManagementAction::Help));
    for arguments in [
        vec!["--help", "check"],
        vec!["-h", "--all"],
        vec!["--help", "--help"],
        vec!["help"],
        vec!["--HELP"],
    ] {
        assert_eq!(
            parse(arguments.as_slice()),
            Err(ManagementRefusal::Usage),
            "{arguments:?}"
        );
    }
    assert!(MANAGEMENT_HELP.starts_with("Usage: git cli-git <command> [options]\n"));
    assert!(MANAGEMENT_HELP.contains("The trust, untrust and status commands are retired"));
    assert!(MANAGEMENT_HELP.ends_with("so there is nothing to approve.\n"));
    assert!(MANAGEMENT_USAGE.starts_with("Usage: git cli-git check (--all | -- <pathspec>...)"));
    assert!(MANAGEMENT_USAGE.ends_with("git cli-git --help\n"));
}

/// Retired commands accept exactly the forms they had.
#[test]
fn retired_commands_accept_their_former_forms() {
    for (arguments, command, help) in [
        (vec!["trust"], RetiredCommand::Trust, false),
        (vec!["trust", "--yes"], RetiredCommand::Trust, false),
        (
            vec!["trust", "--yes", "--yes"],
            RetiredCommand::Trust,
            false,
        ),
        (vec!["trust", "--"], RetiredCommand::Trust, false),
        (vec!["trust", "--help"], RetiredCommand::Trust, true),
        (vec!["trust", "-h"], RetiredCommand::Trust, true),
        (
            vec!["trust", "--yes", "--help"],
            RetiredCommand::Trust,
            true,
        ),
        (
            vec!["trust", "--help", "--yes"],
            RetiredCommand::Trust,
            true,
        ),
        (vec!["untrust"], RetiredCommand::Untrust, false),
        (vec!["untrust", "--"], RetiredCommand::Untrust, false),
        (vec!["status"], RetiredCommand::Status, false),
        (vec!["status", "--"], RetiredCommand::Status, false),
    ] {
        assert_eq!(
            parse(arguments.as_slice()),
            Ok(ManagementAction::Retired { command, help }),
            "{arguments:?}"
        );
    }
}

/// Unknown commands,
///  options and stray arguments are usage refusals.
#[test]
fn unknown_forms_are_usage_refusals() {
    for arguments in [
        vec![],
        vec![""],
        vec!["Check"],
        vec!["checks"],
        vec!["trust", "now"],
        vec!["trust", "--yes=1"],
        vec!["trust", "--no"],
        vec!["trust", "-"],
        vec!["trust", "--", "--yes"],
        vec!["trust", "--", "--"],
        vec!["untrust", "--yes"],
        vec!["untrust", "--help"],
        vec!["untrust", "x"],
        vec!["status", "--yes"],
        vec!["status", "-h"],
        vec!["status", "--", "x"],
        vec!["check", "--al"],
        vec!["check", "--all=1"],
        vec!["check", "-a"],
        vec!["check", "--policy"],
        vec!["check", "--all", "--policy"],
        vec!["check", "--policies", "x", "--all"],
        vec!["check", "--policyx=y", "--all"],
        vec!["check", "--help"],
        vec!["fix", "--yes", "--all"],
        // An unknown option wins over a misplaced pathspec.
        vec!["fix", "a.txt", "--unknown"],
        vec!["--version"],
        vec!["--", "check", "--all"],
    ] {
        assert_eq!(
            parse(arguments.as_slice()),
            Err(ManagementRefusal::Usage),
            "{arguments:?}"
        );
    }
}

/// `check` and `fix` take exactly one scope,
///  with `--policy` values in first-occurrence order.
#[test]
fn direct_commands_parse_scope_and_policies() {
    assert_eq!(
        parse(&["check", "--all"]),
        Ok(direct(false, true, &[], &[]))
    );
    assert_eq!(parse(&["fix", "--all"]), Ok(direct(true, true, &[], &[])));
    assert_eq!(
        parse(&["check", "--all", "--all"]),
        Ok(direct(false, true, &[], &[]))
    );
    assert_eq!(
        parse(&["check", "--all", "--"]),
        Ok(direct(false, true, &[], &[]))
    );
    assert_eq!(
        parse(&["check", "--", "a.txt", "dir/"]),
        Ok(direct(false, false, &[], &["a.txt", "dir/"]))
    );
    assert_eq!(
        parse(&["fix", "--policy", "final-newline", "--", "a.txt"]),
        Ok(direct(true, false, &["final-newline"], &["a.txt"]))
    );
    assert_eq!(
        parse(&[
            "check",
            "--policy=b",
            "--policy",
            "a",
            "--all",
            "--policy",
            "b",
            "--policy=a",
            "--policy=",
        ]),
        Ok(direct(false, true, &["b", "a", ""], &[]))
    );
    // A dash-led value, including the separator spelling, is the option's value.
    assert_eq!(
        parse(&["check", "--policy", "--all", "--all"]),
        Ok(direct(false, true, &["--all"], &[]))
    );
    assert_eq!(
        parse(&["check", "--policy", "--", "--all"]),
        Ok(direct(false, true, &["--"], &[]))
    );
    // Everything after the separator is a pathspec, whatever it looks like.
    assert_eq!(
        parse(&["check", "--", "--all", "--policy", "-", "--", ""]),
        Ok(direct(
            false,
            false,
            &[],
            &["--all", "--policy", "-", "--", ""]
        ))
    );
}

/// A pathspec before `--` and a missing or doubled scope each have their own refusal.
#[test]
fn scope_mistakes_have_specific_refusals() {
    for (arguments, fix) in [
        (vec!["check", "a.txt"], false),
        (vec!["check", "a.txt", "--", "b.txt"], false),
        (vec!["check", "--all", "a.txt"], false),
        (vec!["fix", "-", "--all"], true),
        (vec!["fix", "--policy", "x", "y", "--all"], true),
    ] {
        assert_eq!(
            parse(arguments.as_slice()),
            Err(ManagementRefusal::PathspecsBeforeSeparator { fix }),
            "{arguments:?}"
        );
    }
    for (arguments, fix) in [
        (vec!["check"], false),
        (vec!["check", "--"], false),
        (vec!["check", "--policy", "x"], false),
        (vec!["check", "--all", "--", "a.txt"], false),
        (vec!["fix"], true),
        (vec!["fix", "--all", "--", ""], true),
    ] {
        assert_eq!(
            parse(arguments.as_slice()),
            Err(ManagementRefusal::ScopeRequired { fix }),
            "{arguments:?}"
        );
    }
}

/// Pathspecs keep their exact bytes;
///  a policy ID that is not UTF-8 is refused.
#[cfg(unix)]
#[test]
fn native_bytes_are_kept_for_pathspecs_and_refused_for_policy_ids() {
    use std::os::unix::ffi::OsStringExt;
    let raw: OsString = OsString::from_vec(b"dir/\xff\xfe.txt".to_vec());
    assert_eq!(
        parse_management_arguments(&[OsString::from("check"), OsString::from("--"), raw.clone(),]),
        Ok(ManagementAction::Direct {
            fix: false,
            all: false,
            policies: Vec::<String>::new(),
            pathspecs: vec![raw.clone()],
        })
    );
    for policy_form in [
        vec![
            OsString::from("check"),
            OsString::from("--policy"),
            raw.clone(),
        ],
        vec![
            OsString::from("check"),
            OsString::from_vec(b"--policy=\xff".to_vec()),
        ],
    ] {
        assert_eq!(
            parse_management_arguments(policy_form.as_slice()),
            Err(ManagementRefusal::Usage)
        );
    }
}

/// `doctor` takes no arguments, and the usage and help name it.
#[test]
fn doctor_takes_no_arguments_and_is_documented() {
    assert_eq!(parse(&["doctor"]), Ok(ManagementAction::Doctor));
    for arguments in [vec!["doctor", "--all"], vec!["doctor", "--help"], vec!["doctor", "--", "a.txt"], vec!["doctor", "x"]] {
        assert_eq!(parse(arguments.as_slice()), Err(ManagementRefusal::Usage), "{arguments:?}");
    }
    assert!(MANAGEMENT_USAGE.contains("git cli-git doctor"), "{MANAGEMENT_USAGE}");
    assert!(MANAGEMENT_HELP.contains("doctor"), "{MANAGEMENT_HELP}");
}

/// The help describes the `doctor` exit statuses the way `SPEC.md` ("Doctor") defines them:
///  0 for no hooks or only git-lfs shims, 1 for any other hook (even beside shims), 2 outside a repository.
#[test]
fn doctor_help_states_the_exit_statuses() {
    let sentence: &str = concat!(
        "  doctor   List traditional hooks (unsupported). Exit 0 when none or only git-lfs shims are found,\n",
        "           1 when any other hook is found, 2 outside a repository.\n",
    );
    assert!(MANAGEMENT_HELP.contains(sentence), "{MANAGEMENT_HELP}");
    assert!(!MANAGEMENT_HELP.contains("exit 1 when any are found"), "{MANAGEMENT_HELP}");
}
