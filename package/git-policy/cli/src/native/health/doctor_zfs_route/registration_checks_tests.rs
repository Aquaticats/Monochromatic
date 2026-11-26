//! What:
//!  Controls of one slot's Git registration line:
//!  a complete registration,
//!  a record that names none yet,
//!  a pruned one,
//!  one that points at another path,
//!  a half-made one with no `gitdir`,
//!  and a path the look could not read.
//! Why:
//!  A person reads this line to learn whether `git worktree` still lists their
//!  worktree and which of the two repairs applies,
//!  so every arm is shown with the words that send them to the right place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkRegistration(facts)).toMatchObject({ kind: 'finding', field: 'registration' });
//! ```

use super::{
    RegistrationFacts,
    check_registration,
};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::handler_checks::PathState;
use std::path::PathBuf;

/// The slot every control here is about.
const SLOT: &str = "topic";

/// The slot's whole path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The registration the record names.
fn registration() -> PathBuf {
    return PathBuf::from("/repo/.git/worktrees/topic");
}

/// The facts of a registration directory that stands,
/// named by the slot's own `.git`.
fn standing() -> RegistrationFacts {
    return RegistrationFacts {
        slot: String::from(SLOT),
        recorded: Some(registration()),
        at_the_path: PathState::Directory {
            owner: 1000,
            mode: 0o755,
        },
        gitdir: Some(slot_path().join(".git")),
        slot_git: slot_path().join(".git"),
    };
}

/// A complete registration passes.
#[test]
fn a_complete_registration_passes() {
    let answer: CheckOutcome = check_registration(&standing());
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a complete registration passes: {answer:?}");
    };
    assert!(detail.contains("/repo/.git/worktrees/topic"), "{detail}");
    assert!(
        detail.contains("git worktree` lists this worktree"),
        "{detail}"
    );
}

/// A record that names no registration yet passes,
/// because the add has not reached that step.
#[test]
fn a_record_without_a_registration_passes() {
    let mut facts: RegistrationFacts = standing();
    facts.recorded = None;
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a record without a registration passes: {answer:?}");
    };
    assert!(detail.contains("names no Git registration yet"), "{detail}");
}

/// A registration that is gone is a finding naming it and the prune.
#[test]
fn a_pruned_registration_is_a_finding() {
    let mut facts: RegistrationFacts = standing();
    facts.at_the_path = PathState::Absent;
    facts.gitdir = None;
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a pruned registration is a finding: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("/repo/.git/worktrees/topic"), "{detail}");
    assert!(detail.contains("prune"), "{detail}");
    assert!(detail.contains("cannot rebuild"), "{detail}");
    assert!(detail.contains("do not run Git in the folder"), "{detail}");
}

/// A registration directory the look could not read is unknown,
/// never a finding and never a pass.
#[test]
fn an_unreadable_registration_is_unknown() {
    let mut facts: RegistrationFacts = standing();
    facts.at_the_path = PathState::Unreadable {
        why: String::from("permission denied"),
    };
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an unreadable registration is unknown: {answer:?}");
    };
    assert!(reason.contains("permission denied"), "{reason}");
}

/// A file at the registration's path is a finding.
#[test]
fn a_file_at_the_registration_path_is_a_finding() {
    let mut facts: RegistrationFacts = standing();
    facts.at_the_path = PathState::File {
        owner: 1000,
        mode: 0o644,
    };
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a file at the path is a finding: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("a file stands there"), "{detail}");
}

/// A registration with no `gitdir` is the state a kill inside Git's add
/// leaves,
/// and the finding says so.
#[test]
fn a_registration_without_a_gitdir_is_a_finding() {
    let mut facts: RegistrationFacts = standing();
    facts.gitdir = None;
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a registration without a gitdir is a finding: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("no `gitdir` file"), "{detail}");
    assert!(detail.contains("worktree add"), "{detail}");
}

/// A registration whose `gitdir` names another path is a finding naming both,
/// with the repair command.
#[test]
fn a_misdirected_registration_names_both_paths_and_the_repair() {
    let mut facts: RegistrationFacts = standing();
    facts.gitdir = Some(PathBuf::from("/repo/.claude/worktrees/other/.git"));
    let answer: CheckOutcome = check_registration(&facts);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a misdirected registration is a finding: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(
        detail.contains("/repo/.claude/worktrees/other/.git"),
        "{detail}"
    );
    assert!(
        detail.contains("/repo/.claude/worktrees/topic/.git"),
        "{detail}"
    );
    assert!(detail.contains("git worktree repair topic"), "{detail}");
}

/// A path written with a repeated or a dot separator is the same file,
/// so the line passes rather than reporting a damage nobody made.
#[test]
fn a_path_with_a_repeated_separator_is_the_same_path() {
    let mut facts: RegistrationFacts = standing();
    facts.gitdir = Some(PathBuf::from("/repo//.claude/./worktrees/topic/.git"));
    let answer: CheckOutcome = check_registration(&facts);
    assert!(matches!(answer, CheckOutcome::Pass { .. }), "{answer:?}");
}
