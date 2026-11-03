//! Controls for the unit templates the wrapper provisions.

use super::{arguments_after_subcommand, switched_branch_name};
use crate::service_provision::{OWN_MARKERS, SERVICE_UNIT, SOCKET_UNIT};
use std::ffi::OsString;
use crate::service_instance::SOCKET_DIRECTORY;


/// The socket template listens where the wrapper looks, one socket per instance and privately, and the service template is
///  bounded, ordered after its own instance's socket, and told which worktree it serves.
#[test]
fn the_activation_units_match_the_wrapper_and_the_resource_rules() {
    assert!(SOCKET_UNIT.lines().any(|line| { return line == format!("ListenStream=%t/{SOCKET_DIRECTORY}/%i.sock"); }), "{SOCKET_UNIT}");
    assert!(SOCKET_UNIT.contains("SocketMode=0600") && SOCKET_UNIT.contains("DirectoryMode=0700"));
    assert!(SERVICE_UNIT.contains("Requires=cli-git@%i.socket") && SERVICE_UNIT.contains("After=cli-git@%i.socket"));
    assert!(SERVICE_UNIT.contains("--instance %i"), "the service learns which worktree it is for");
    assert!(SERVICE_UNIT.contains("--journal %S/cli-git/service/%i"), "each instance keeps its own journal");
    for property in ["TasksMax=", "CPUQuota=", "IOWeight="] {
        assert!(SERVICE_UNIT.lines().any(|line| { return line.starts_with(property); }), "{property}");
    }
    let weight: u32 = SERVICE_UNIT
        .lines()
        .find_map(|line| { return line.strip_prefix("IOWeight="); })
        .and_then(|value| { return value.trim().parse().ok(); })
        .expect("a numeric IOWeight");
    assert!(weight <= 50, "the owner's rule: never above 50");
    assert!(SERVICE_UNIT.contains("@SERVICE_BINARY@") && SERVICE_UNIT.contains("@GIT@"), "the installer fills these in");
}

/// What:     a test that the service template sets no `MemoryMax=` line, so systemd puts no memory cap of the
///           wrapper's own on the service or on the Git processes it starts.
/// Why:      the cap of 1 GiB had no measurement behind it; the service's Git children read the repository's packs and
///           index through the page cache, which a cap charges to the unit, so on the main checkout the unit hit the
///           cap 274,736 times (`memory.events` max) with 16,125,924 major page faults by 2026-11-03T08:40:51-05:00,
///           and a commit's `git reset` took 59 s that plain Git does in milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the service unit sets no memory cap of its own', () => {
///   assert(!SERVICE_UNIT.split('\n').some((line) => line.startsWith('MemoryMax=')));
/// });
/// ```
#[test]
fn the_service_unit_sets_no_memory_cap_of_its_own() {
    // What:     `.lines()` walks the template line by line; `.any(|line| ...)` is `Array.prototype.some` with a Rust
    //           closure (`|line| { ... }` is `(line) => { ... }`); `!` negates the result.
    // Why:      the template must not carry the line at all, whatever value it would hold.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(!SERVICE_UNIT.split('\n').some((line) => line.startsWith('MemoryMax=')), SERVICE_UNIT);
    // ```
    assert!(!SERVICE_UNIT.lines().any(|line| { return line.starts_with("MemoryMax="); }), "{SERVICE_UNIT}");
}

/// Both templates carry the marker that lets the wrapper recognize its own units, on the first line.
#[test]
fn the_templates_carry_the_marker_on_the_first_line() {
    for text in [SOCKET_UNIT, SERVICE_UNIT] {
        let first: &str = text.lines().next().expect("a first line");
        assert!(first.starts_with('#') && first.contains(OWN_MARKERS[0]), "{first}");
    }
}

/// The command's own words start after the subcommand,
/// whatever stands before it,
/// and a command line that ends at the
/// subcommand,
/// or at which the subcommand is not,
/// has none.
#[test]
fn the_words_of_the_command_start_after_the_subcommand() {
    let line: Vec<OsString> = ["-C", "somewhere", "commit", "-m", "message", "--", "file"].iter().map(OsString::from).collect();
    let after: Vec<OsString> = ["-m", "message", "--", "file"].iter().map(OsString::from).collect();
    assert_eq!(arguments_after_subcommand(line.as_slice(), 2), after.as_slice());
    assert_eq!(arguments_after_subcommand(line.as_slice(), 6), &[] as &[OsString], "the subcommand is the last word");
    assert_eq!(arguments_after_subcommand(line.as_slice(), 40), &[] as &[OsString], "no such word");
    let bare: Vec<OsString> = vec![OsString::from("commit"), OsString::from("-a")];
    assert_eq!(arguments_after_subcommand(bare.as_slice(), 0), &bare[1..], "a subcommand in first place leaves everything after it, and not itself");
}

/// A branch that switched under a commit is named as the person knows it:
/// a detached head by those words,
/// a branch without the
/// `refs/heads/` of its reference,
/// and any other reference as it is.
#[test]
fn a_switched_branch_is_named_as_the_person_knows_it() {
    assert_eq!(switched_branch_name("HEAD"), "a detached head");
    assert_eq!(switched_branch_name("refs/heads/main"), "main");
    assert_eq!(switched_branch_name("refs/heads/topic/deep"), "topic/deep");
    assert_eq!(switched_branch_name("refs/tags/v1"), "refs/tags/v1");
    assert_ne!(switched_branch_name("HEADS"), "a detached head", "only the word HEAD is a detached head");
}
