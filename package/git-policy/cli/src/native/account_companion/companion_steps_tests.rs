//! Controls of the skip's boundary:
//!  every call and errno the design names is a skip,
//!  every other answer is an arm failure,
//!  and nothing after the scratch tmpfs is ever a skip.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(skipsAt({ step: 'uid-map', errno: 'EPERM' })).toBe(true);
//! expect(skipsAt({ step: 'attach', errno: 'EPERM' })).toBe(false);
//! ```

use super::{
    CompanionStep,
    STEP_COUNT,
    STEPS_IN_ORDER,
    is_skip,
};
use crate::account_companion::companion_calls::CompanionCall;

/// The entry present on both hosts these controls run on.
const ENTRY_PRESENT: bool = true;
/// A host whose kernel was built without user namespaces.
const ENTRY_ABSENT: bool = false;

/// What:
///  `unshare(CLONE_NEWUSER)` is a skip for the three refusals the design names,
///  and for `EINVAL` only where the host has no user-namespace entry.
#[test]
fn a_refused_user_namespace_is_a_skip_for_the_named_errnos_only() {
    let step = CompanionStep::UserNamespace;
    let call = CompanionCall::UnshareUserNamespace;
    for errno in [libc::ENOSPC, libc::EPERM, libc::EACCES] {
        assert!(
            is_skip(step, call, errno, ENTRY_PRESENT),
            "errno {errno} is on the skip's list with the entry present"
        );
        assert!(
            is_skip(step, call, errno, ENTRY_ABSENT),
            "errno {errno} is on the skip's list with the entry absent"
        );
    }
    assert!(
        is_skip(step, call, libc::EINVAL, ENTRY_ABSENT),
        "EINVAL without a user-namespace entry is a host without the option"
    );
    assert!(
        !is_skip(step, call, libc::EINVAL, ENTRY_PRESENT),
        "EINVAL with the entry present is the companion's own malformed request"
    );
    assert!(
        !is_skip(step, call, libc::ENOMEM, ENTRY_PRESENT),
        "a memory refusal is nobody's namespace policy"
    );
}

/// What:
///  The first mapping write is a skip for `EPERM`,
///  which is the call Ubuntu 24.04's default profile denies,
///  and the second mapping write is never a skip.
#[test]
fn the_first_mapping_write_skips_and_the_second_does_not() {
    assert!(is_skip(
        CompanionStep::UidMap,
        CompanionCall::WriteUidMap,
        libc::EPERM,
        ENTRY_PRESENT
    ));
    assert!(
        !is_skip(
            CompanionStep::GidMap,
            CompanionCall::WriteGidMap,
            libc::EPERM,
            ENTRY_PRESENT
        ),
        "the second mapping's EPERM is a defect of the companion's order"
    );
}

/// What:
///  The mount namespace,
/// the propagation and the scratch mount are skips for
///  the answers the design names,
///  and the setgroups write never is.
#[test]
fn the_mount_namespace_and_the_scratch_mount_skip_their_named_answers() {
    assert!(is_skip(
        CompanionStep::MountNamespace,
        CompanionCall::UnshareMountNamespace,
        libc::EPERM,
        ENTRY_PRESENT
    ));
    assert!(is_skip(
        CompanionStep::PrivatePropagation,
        CompanionCall::MountSetattr,
        libc::EPERM,
        ENTRY_PRESENT
    ));
    assert!(is_skip(
        CompanionStep::PrivatePropagation,
        CompanionCall::MountPropagation,
        libc::EACCES,
        ENTRY_PRESENT
    ));
    assert!(is_skip(
        CompanionStep::ScratchTmpfs,
        CompanionCall::MountTmpfs,
        libc::EPERM,
        ENTRY_PRESENT
    ));
    assert!(is_skip(
        CompanionStep::ScratchTmpfs,
        CompanionCall::MountTmpfs,
        libc::EACCES,
        ENTRY_PRESENT
    ));
    assert!(
        !is_skip(
            CompanionStep::ScratchTmpfs,
            CompanionCall::MountTmpfs,
            libc::ENOMEM,
            ENTRY_PRESENT
        ),
        "a memory refusal at the scratch mount is a failure"
    );
    assert!(
        !is_skip(
            CompanionStep::Setgroups,
            CompanionCall::WriteSetgroups,
            libc::EPERM,
            ENTRY_PRESENT
        ),
        "the setgroups write asks for no capability"
    );
}

/// What:
///  Nothing at or after the walk is ever a skip,
///  whichever call and errno it answers.
#[test]
fn no_step_after_the_setup_is_a_skip() {
    let after: Vec<CompanionStep> = STEPS_IN_ORDER
        .iter()
        .copied()
        .filter(|step| return !step.is_setup())
        .collect();
    assert_eq!(
        after.len(),
        STEP_COUNT - 7,
        "the seven setup steps are the only ones the skip can cover"
    );
    for step in after {
        for errno in [
            libc::EPERM,
            libc::EACCES,
            libc::ENOSPC,
            libc::EINVAL,
            libc::ENOMEM,
        ] {
            assert!(
                !is_skip(step, CompanionCall::FsMount, errno, ENTRY_PRESENT),
                "{} must never skip",
                step.name()
            );
        }
    }
}

/// What:
///  The steps and their numbers agree,
///  so a record is never placed on the wrong step.
#[test]
fn every_step_has_a_number_and_a_name_that_names_it() {
    for (at, step) in STEPS_IN_ORDER.iter().enumerate() {
        assert_eq!(
            *step as usize,
            at,
            "{} is numbered by its place",
            step.name()
        );
        assert_eq!(
            CompanionStep::from_number(step_as_number(*step)),
            Some(*step)
        );
        assert_eq!(CompanionStep::from_name(step.name()), Some(*step));
    }
    assert_eq!(
        CompanionStep::from_number(u8::try_from(STEP_COUNT).unwrap_or(u8::MAX)),
        None,
        "a number past the last step names none"
    );
}

/// The number one step is written as.
fn step_as_number(step: CompanionStep) -> u8 {
    return step as u8;
}
