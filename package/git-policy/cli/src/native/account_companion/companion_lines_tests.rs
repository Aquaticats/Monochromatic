//! Controls of the lines a person reads:
//!  a pass prints nothing,
//!  a skip prints its call,
//! its errno and the three settings,
//!  and every refusal prints the design's own sentence and says whether the
//!  add goes ahead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verdictOf({ kind: 'passed' }).lines()).toEqual([]);
//! expect(verdictOf({ kind: 'skipped', skip }).lines()[0]).toContain('user namespaces are unavailable here');
//! ```

use super::{
    CompanionVerdict,
    Refusal,
    Skip,
};
use crate::account_companion::companion_calls::{
    CompanionCall,
    Failure,
    NO_ERRNO,
};
use crate::account_companion::companion_settings::Settings;
use crate::account_companion::companion_steps::CompanionStep;
use std::time::Duration;

/// The settings a skip line prints.
fn settings() -> Settings {
    return Settings {
        max_user_namespaces: String::from("252369"),
        unprivileged_userns_clone: String::from("1"),
        apparmor_restriction: String::from("absent"),
    };
}

/// What:
///  A pass and the control arm print nothing,
///  and both let the managed attach go ahead.
#[test]
fn a_pass_prints_nothing_and_lets_the_managed_attach_go_ahead() {
    assert!(CompanionVerdict::Passed.lines().is_empty());
    assert!(CompanionVerdict::Passed.the_managed_attach_goes_ahead());
    assert!(
        CompanionVerdict::NotRun.lines().is_empty(),
        "the control arm prints nothing too"
    );
    assert!(CompanionVerdict::NotRun.the_managed_attach_goes_ahead());
}

/// What:
///  A skip prints the call,
/// the errno and every setting,
///  and the add goes ahead.
#[test]
fn a_skip_prints_its_call_and_settings_and_lets_the_add_go_ahead() {
    let skip: Skip = Skip {
        call: CompanionCall::WriteUidMap,
        errno: libc::EPERM,
        settings: settings(),
    };
    let verdict: CompanionVerdict = CompanionVerdict::Skipped {
        skip,
    };
    let lines: Vec<String> = verdict.lines();
    assert_eq!(lines.len(), 2, "the skip line and the settings line");
    assert!(lines[0].starts_with("account companion skipped: write /proc/self/uid_map answered EPERM (errno 1); user namespaces are unavailable here"), "{}", lines[0]);
    assert_eq!(
        lines[1],
        "user.max_user_namespaces=252369, kernel.unprivileged_userns_clone=1, kernel.apparmor_restrict_unprivileged_userns=absent"
    );
    assert!(
        verdict.the_managed_attach_goes_ahead(),
        "the project accepted that a skip lets the add go ahead"
    );
}

/// What:
///  A failed arm prints the step,
/// the call and the number,
///  and refuses the managed attach.
#[test]
fn a_failed_arm_refuses_the_managed_attach_and_names_its_call() {
    let verdict: CompanionVerdict = CompanionVerdict::Refused {
        refusal: Refusal::ArmFailed {
            step: CompanionStep::Attach,
            failure: Failure {
                call: CompanionCall::FsMount,
                errno: libc::EPERM,
            },
        },
    };
    let lines: Vec<String> = verdict.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0],
        "managed add refused: the account companion's attach failed: fsmount (EPERM, errno 1); this slot takes the reflink or ordinary Git route."
    );
    assert!(!verdict.the_managed_attach_goes_ahead());
}

/// What:
///  A deadline and a child's signal have their own sentences,
///  and a run that wrote no record is the status pipe's own.
#[test]
fn the_deadline_the_signal_and_the_missing_record_each_have_a_sentence() {
    let deadline: String = Refusal::Deadline {
        deadline: Duration::from_millis(1_500),
    }
    .line();
    assert_eq!(
        deadline,
        "managed add refused: the account companion did not answer within its deadline (1500 ms); this slot takes the reflink or ordinary Git route."
    );
    let signalled: String = Refusal::ChildSignaled {
        step_in_flight: CompanionStep::UserNamespace,
        signal: libc::SIGKILL,
    }
    .line();
    assert!(
        signalled.contains("user-namespace failed: the child (signal 9)"),
        "{signalled}"
    );
    let missing: String = Refusal::NoStatusLine {
        step_in_flight: CompanionStep::Walk,
    }
    .line();
    assert!(
        missing.contains("walk failed: the status pipe (no status line)"),
        "{missing}"
    );
}

/// What:
///  A comparison has no errno,
///  so its line says so rather than printing a zero.
#[test]
fn a_comparison_that_failed_prints_no_errno() {
    let verdict: CompanionVerdict = CompanionVerdict::Refused {
        refusal: Refusal::ArmFailed {
            step: CompanionStep::MountFacts,
            failure: Failure {
                call: CompanionCall::MountFactsCheck,
                errno: NO_ERRNO,
            },
        },
    };
    assert!(
        verdict.lines()[0].contains("the mount facts read back (no errno)"),
        "{}",
        verdict.lines()[0]
    );
}

/// What:
///  A refused fork prints the count of refusals,
///  and the scratch refusal prints the sentence it was given.
#[test]
fn the_start_failures_print_what_they_spent_and_what_to_do() {
    let refused: String = Refusal::ForkRefused {
        refusals: 5,
        errno: libc::EAGAIN,
    }
    .line();
    assert!(
        refused.contains("fork (errno 11) after 5 refusals"),
        "{refused}"
    );
    let scratch: String = Refusal::Scratch {
        message: String::from("XDG_RUNTIME_DIR is not set"),
    }
    .line();
    assert!(
        scratch.contains("the account companion's scratch failed: XDG_RUNTIME_DIR is not set"),
        "{scratch}"
    );
}
