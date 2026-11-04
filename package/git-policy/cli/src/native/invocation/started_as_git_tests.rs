//! What:
//!  Controls for the marker every real-Git start carries and for the check that stops a wrapper started as Git.
//! Why:
//!  The check decides from made-up process numbers and birth identities,
//! so each way a start can look (the starter
//!  replaced itself,
//! the starter is the parent,
//! something else stands in between,
//! a number was reused) is shown without
//!  starting a process;
//! one control then reads the marker the real builders put on a command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(startedAsGit({ starterPid: 10, mode: 'child' }, { ownPid: 12, parentPid: 10 })).toBe(true);
//! ```
#![cfg(unix)]

/// The marker,
/// its parser,
/// the check and the builders under test.
use super::{
    ProcessFacts, STARTED_AS_GIT_VARIABLE, StartMarker, StartMode, marker_of, marker_value, mark_real_git_start,
    parse_marker, refusal_text, refusal_when_started_as_git, started_as_git,
};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// The identity a control gives the starter.
const STARTER_IDENTITY: &str = "linux:1000";

/// A marker of a start by process `pid` in the given mode,
/// with the starter's identity.
fn marker(pid: u32, mode: StartMode) -> StartMarker {
    return StartMarker { starter_pid: pid, starter_identity: STARTER_IDENTITY.to_string(), mode };
}

/// Whether the check judges a process with the numbers `own` and `parent` to be the start `marker` names,
/// when the
///  identity of the candidate process reads as `identity_of_candidate` (`None`:
/// unreadable).
fn started(marker: &StartMarker, own: u32, parent: u32, identity_of_candidate: Option<&str>) -> bool {
    let answer = |_pid: u32| -> Option<String> { return identity_of_candidate.map(str::to_string); };
    let facts: ProcessFacts<'_> = ProcessFacts { own_pid: own, parent_pid: parent, identity_of: &answer };
    return started_as_git(marker, &facts);
}

/// The marker value round-trips through its parser for both modes,
/// and text that is not a marker is none.
#[test]
fn a_marker_value_is_read_back_and_other_text_is_no_marker() {
    for mode in [StartMode::Replace, StartMode::Child] {
        let value: OsString = marker_value(mode, 4242, STARTER_IDENTITY);
        assert_eq!(parse_marker(value.as_os_str()), Some(marker(4242, mode)), "{mode:?}");
    }
    let empty_identity: OsString = marker_value(StartMode::Child, 7, "");
    assert_eq!(parse_marker(empty_identity.as_os_str()).map(|read| { return read.starter_identity; }), Some(String::new()));
    for other in ["", "1234", "abc\u{1f}x\u{1f}child", "12\u{1f}x\u{1f}sideways", "12\u{1f}x", "12\u{1f}x\u{1f}child\u{1f}extra", "-5\u{1f}x\u{1f}child"] {
        assert_eq!(parse_marker(OsString::from(other).as_os_str()), None, "{other:?}");
    }
}

/// A start that replaced its starter is the starter's own process:
/// it matches by the program's own number,
/// and a
///  parent with that number alone does not make it one.
#[test]
fn a_replaced_start_is_the_process_the_marker_names() {
    let replace: StartMarker = marker(100, StartMode::Replace);
    assert!(started(&replace, 100, 1, Some(STARTER_IDENTITY)), "same process");
    assert!(!started(&replace, 101, 100, Some(STARTER_IDENTITY)), "a child of the real Git that replaced the wrapper is a hook, not the start");
    assert!(!started(&replace, 101, 1, Some(STARTER_IDENTITY)), "another process");
}

/// A child start is the starter's child:
/// it matches by the parent's number,
/// and the starter's own number alone does not
///  (a child never has the starter's number).
#[test]
fn a_child_start_is_a_child_of_the_process_the_marker_names() {
    let child: StartMarker = marker(100, StartMode::Child);
    assert!(started(&child, 200, 100, Some(STARTER_IDENTITY)), "child of the starter");
    assert!(!started(&child, 200, 150, Some(STARTER_IDENTITY)), "something stands between: a hook or an alias of the real Git");
    assert!(!started(&child, 100, 1, Some(STARTER_IDENTITY)), "the number alone is not the child start");
}

/// A number that a later process reuses is not the starter:
/// its identity differs;
/// an identity that cannot be read does
///  not contradict the marker,
/// and a marker without an identity decides by the numbers.
#[test]
fn a_reused_number_is_not_the_starter() {
    let child: StartMarker = marker(100, StartMode::Child);
    assert!(!started(&child, 200, 100, Some("linux:2000")), "another process holds the number now");
    assert!(started(&child, 200, 100, None), "unreadable identity decides by the numbers");
    let without: StartMarker = StartMarker { starter_pid: 100, starter_identity: String::new(), mode: StartMode::Child };
    assert!(started(&without, 200, 100, Some("linux:2000")), "no recorded identity to compare");
}

/// The environment carries the marker only under its own name,
/// and only a well-formed value counts.
#[test]
fn the_environment_marker_is_read_by_name() {
    let value: OsString = marker_value(StartMode::Child, 9, STARTER_IDENTITY);
    let present: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), value.clone())];
    assert_eq!(marker_of(present.as_slice()), Some(marker(9, StartMode::Child)));
    let misnamed: Vec<(OsString, OsString)> = vec![(OsString::from("CLI_GIT_STARTED_AS_GIT_NOT"), value)];
    assert_eq!(marker_of(misnamed.as_slice()), None);
    let garbled: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), OsString::from("1"))];
    assert_eq!(marker_of(garbled.as_slice()), None);
    assert_eq!(marker_of(&[]), None);
}

/// This test process is neither the starter nor its child,
/// so a marker naming other processes does not stop it,
/// and a
///  marker naming this very process as a replacing starter does.
#[test]
fn the_live_check_reads_this_process() {
    let own_pid: u32 = std::process::id();
    let identity: String = crate::process_identity::current_birth_identity().expect("this process has an identity");
    let own_marker = |pid: u32, mode: StartMode| -> Vec<(OsString, OsString)> {
        return vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(mode, pid, identity.as_str()))];
    };
    let elsewhere: u32 = own_pid.wrapping_add(7);
    assert_eq!(refusal_when_started_as_git(own_marker(elsewhere, StartMode::Replace).as_slice(), Path::new("/wrapper")), None);
    assert_eq!(refusal_when_started_as_git(own_marker(elsewhere, StartMode::Child).as_slice(), Path::new("/wrapper")), None);
    let said: String = refusal_when_started_as_git(own_marker(own_pid, StartMode::Replace).as_slice(), Path::new("/wrapper"))
        .expect("this process is the replaced starter");
    assert!(said.contains("started as Git"), "{said}");
    assert!(said.contains("/wrapper"), "{said}");
    // The parent of this test process is the process that started it; a child marker naming it matches.
    // SAFETY: `getppid` takes no argument, reads no memory and cannot fail.
    let parent: u32 = u32::try_from(unsafe { libc::getppid() }).expect("a parent number");
    let parent_identity: String = crate::process_identity::process_birth_identity(i64::from(parent))
        .expect("readable").expect("parent runs");
    let named_parent: Vec<(OsString, OsString)> =
        vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, parent, parent_identity.as_str()))];
    assert!(refusal_when_started_as_git(named_parent.as_slice(), Path::new("/wrapper")).is_some());
}

/// The message names the executable,
/// the starter,
/// the way it started,
/// the Git it chose and the PATH,
/// and every way out.
#[test]
fn the_refusal_names_what_a_person_needs() {
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("CLI_GIT_NATIVE_FORWARD_TARGET"), OsString::from("/opt/launchers/git")),
        (OsString::from("PATH"), OsString::from("/opt/launchers")),
    ];
    let said: String = refusal_text(&marker(321, StartMode::Child), Path::new("/opt/other-release/cli-git-native"), environment.as_slice());
    for needle in [
        "/opt/other-release/cli-git-native",
        "was started as Git by a cli-git wrapper",
        "process 321",
        "waits for its answer",
        "/opt/launchers/git",
        "PATH here is /opt/launchers",
        "never runs itself as Git",
        "(for example /usr/bin)",
        "remove the entry that leads back to cli-git",
    ] {
        assert!(said.contains(needle), "{needle}: {said}");
    }
    let replaced: String = refusal_text(&marker(321, StartMode::Replace), Path::new("/x"), &[]);
    assert!(replaced.contains("replaced with this program"), "{replaced}");
    assert!(replaced.contains("PATH is not set here"), "{replaced}");
}

/// The builders put the marker of the right mode on a command,
/// naming this process,
/// and a later mark overrides the
///  earlier one (the exec path marks "replace" over the shared builder's "child").
#[test]
fn marking_a_command_names_this_process_and_the_last_mark_wins() {
    let mut command: Command = Command::new("/bin/true");
    mark_real_git_start(&mut command, StartMode::Child);
    mark_real_git_start(&mut command, StartMode::Replace);
    let value: OsString = command
        .get_envs()
        .find(|(name, _value)| { return *name == STARTED_AS_GIT_VARIABLE; })
        .and_then(|(_name, value)| { return value.map(OsString::from); })
        .expect("the command carries the marker");
    let read: StartMarker = parse_marker(value.as_os_str()).expect("a marker");
    assert_eq!(read.starter_pid, std::process::id());
    assert_eq!(read.mode, StartMode::Replace);
    assert_eq!(command.get_envs().filter(|(name, _value)| { return *name == STARTED_AS_GIT_VARIABLE; }).count(), 1);
}
