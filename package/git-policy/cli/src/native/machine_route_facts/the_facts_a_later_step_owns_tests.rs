//! What:
//!  The controls of the four facts plan steps 19 and 20 own:
//!  the covered entry and the boot setup answer unknown with the step named,
//!  the cancel command answers its own word and usage line,
//!  and Sanoid's coverage answers unknown.
//! Why:
//!  A report over these facts must not print a pass for a read nobody made,
//!  and the cancel command's line has to pass on a build that carries the word,
//!  so both arms are shown from one value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [, , cancel] = theFactsALaterStepOwns(1000);
//! expect(cancel.word).toBe(CANCEL_COMMAND_WORD);
//! ```

/// The one constructor a control reads.
use super::the_facts_a_later_step_owns;
/// The checks these four facts answer,
/// and the states they answer.
use crate::doctor_zfs_route::{
    BootProtectionFacts,
    BootSetupFacts,
    CancelCommandFacts,
    CheckOutcome,
    SanoidReport,
    check_boot_protection,
    check_cancel_command,
    check_service_unit,
};
/// A checkout path the plan is built for,
/// which is the design's own.
use std::path::Path;

/// The account a control states.
const AN_ACCOUNT: u32 = 1000;

/// What:
///  The covered entry,
///  the boot setup and the coverage comparison answer unknown,
///  each naming the plan step that owns it.
/// Why:
///  A pass on an entry nobody read would tell a person a checkout is protected
///  when nothing proved it,
///  and a finding would send them to repair a file nobody looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(checkBootProtection(protection).kind).toBe('unknown');
/// ```
#[test]
fn the_three_facts_of_later_steps_answer_unknown() {
    let (protection, setup, _cancel, sanoid): (
        BootProtectionFacts,
        BootSetupFacts,
        CancelCommandFacts,
        SanoidReport,
    ) = the_facts_a_later_step_owns(AN_ACCOUNT, Path::new("/home/user/Monochromatic"));
    let entry: CheckOutcome = check_boot_protection(&protection);
    let CheckOutcome::Unknown {
        reason,
    } = entry
    else {
        panic!("an entry nobody read is unknown: {entry:?}");
    };
    assert!(reason.contains("plan step 19"), "{reason}");
    let unit: CheckOutcome = check_service_unit(&setup);
    let CheckOutcome::Unknown {
        reason: unit_reason,
    } = unit
    else {
        panic!("a setup nobody read is unknown: {unit:?}");
    };
    assert!(unit_reason.contains("plan step 19"), "{unit_reason}");
    let CheckOutcome::Unknown {
        reason: coverage,
    } = sanoid.outcome
    else {
        panic!("a comparison nobody made is unknown: {:?}", sanoid.outcome);
    };
    assert!(coverage.contains("plan step 20"), "{coverage}");
    assert!(sanoid.lines.is_empty());
    assert_eq!(protection.wanted_owner, AN_ACCOUNT);
}

/// What:
///  The cancel command answers the word the build carries and the first line
///  of its own usage,
///  so its check passes on a build that has the command.
/// Why:
///  The line's only fact is the built command,
///  and a value that stated another word would report a command nobody ships.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(checkCancelCommand(cancel).kind).toBe('pass');
/// ```
#[test]
fn the_cancel_command_answers_its_own_grammar() {
    let (_protection, _setup, cancel, _sanoid) =
        the_facts_a_later_step_owns(AN_ACCOUNT, Path::new("/home/user/Monochromatic"));
    assert_eq!(
        cancel.word,
        crate::worktree_clone_removal_cancel::cancel_arguments::CANCEL_COMMAND_WORD
    );
    let usage: String = crate::worktree_clone_removal_cancel::cancel_arguments::usage();
    assert_eq!(
        cancel.usage_line,
        usage.lines().next().expect("the usage has a line")
    );
    let answer: CheckOutcome = check_cancel_command(&cancel);
    assert!(
        matches!(answer, CheckOutcome::Pass { .. }),
        "a build that carries the command passes: {answer:?}"
    );
}
