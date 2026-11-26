//! Controls of one companion run:
//!  the pass on this host,
//!  the skip as a host answers it,
//!  the arm failure a defect answers,
//!  the deadline,
//!  the two runs that end before any record,
//!  the control arm that runs nothing,
//!  and the sentence a missing runtime folder prints.
//!
//! Each control runs the real parent over the real child in a run directory it
//! makes under a fixture root of its own,
//! and the pass control names the host it could not run on rather than passing
//! in silence.
//! No control reads or changes the environment of this process:
//! the runtime folder is the input the add route resolves,
//! so a control passes one of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runUnder(runtimeFolder('skip'), { kind: 'fixture', fixture: { step: 'uid-map', errno: 1 } }, ONE_SECOND).kind).toBe('skipped');
//! ```

use super::run_under;
use crate::account_companion::companion_buffers::Fixture;
use crate::account_companion::companion_deadline::COMPANION_DEADLINE;
use crate::account_companion::companion_lines::{
    CompanionVerdict,
    Refusal,
};
use crate::account_companion::companion_scratch::{
    RuntimeFolder,
    ScratchRefusal,
};
use crate::account_companion::companion_settings::user_namespace_entry_exists;
use crate::account_companion::companion_steps::CompanionStep;
use crate::account_companion::companion_test_switch::Switch;
use std::path::PathBuf;
use std::time::Duration;

/// The acknowledgement a host without user namespaces names,
/// which the
/// wrapper's own namespace control already reads.
const NO_USER_NAMESPACE_ACKNOWLEDGED: &str = "CLI_GIT_NO_USER_NAMESPACE_ACKNOWLEDGED";

/// What:
///  The runtime folder one control keeps its run under,
///  inside a fixture root the control owns.
/// Why:
///  The companion takes its runtime folder as an input,
///  so a control names a folder of its own and never the process's
///  `XDG_RUNTIME_DIR`:
///  the gate's container sets none,
///  and a control that read the variable would pass only on a host whose
///  session made one.
fn runtime_folder(name: &str) -> RuntimeFolder {
    let root: PathBuf = crate::test_support::fixture_with_prefix("account-companion", name);
    return RuntimeFolder::at(root.as_path()).expect("the fixture root is a directory");
}

/// What:
///  Every step passes on a host that gives this account a user namespace,
///  which is what makes the other controls' failures mean something.
#[test]
fn every_step_passes_on_a_host_that_gives_a_user_namespace() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("every-step-passes"),
        Switch::None,
        COMPANION_DEADLINE,
    );
    match &verdict {
        CompanionVerdict::Passed => assert!(verdict.lines().is_empty(), "a pass prints nothing"),
        CompanionVerdict::Skipped {
            skip,
        } => {
            assert!(
                std::env::var_os(NO_USER_NAMESPACE_ACKNOWLEDGED).is_some(),
                "this host refused the companion its user namespace ({} answered errno {}), so the pass control cannot run here; set {NO_USER_NAMESPACE_ACKNOWLEDGED} on such a host",
                skip.call.name(),
                skip.errno
            );
            eprintln!(
                "ACCOUNT COMPANION CONTROL NOT RUN: this host skips the pass arm ({} answered errno {})",
                skip.call.name(),
                skip.errno
            );
        }
        CompanionVerdict::Refused {
            refusal,
        } => panic!(
            "the companion failed on a host that gives this account a user namespace: {}",
            refusal.line()
        ),
        CompanionVerdict::NotRun => {
            panic!("this control runs the shipped check, so no switch may disable it")
        }
    }
}

/// What:
///  The pass control is able to fail:
///  a fixture that makes the walk answer `EPERM` must be a refused arm,
///  not a pass and not a skip.
#[test]
fn an_errno_the_skip_list_does_not_name_refuses_the_arm() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("walk-eperm"),
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::Walk,
                errno: libc::EPERM,
            },
        },
        COMPANION_DEADLINE,
    );
    match verdict {
        CompanionVerdict::Refused {
            refusal:
                Refusal::ArmFailed {
                    step,
                    failure,
                },
        } => {
            assert_eq!(step, CompanionStep::Walk);
            assert_eq!(failure.errno, libc::EPERM);
        }
        other => {
            panic!("a walk that answered EPERM must refuse the arm, and it answered {other:?}")
        }
    }
}

/// What:
///  A host that refuses the user namespace with `ENOSPC` prints the skip,
///  names `unshare` and the number,
/// and lets the add go ahead.
#[test]
fn a_refused_user_namespace_skips_and_the_add_goes_ahead() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("userns-enospc"),
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::UserNamespace,
                errno: libc::ENOSPC,
            },
        },
        COMPANION_DEADLINE,
    );
    let CompanionVerdict::Skipped {
        skip,
    } = &verdict
    else {
        panic!("a user namespace refused with ENOSPC must skip, and it answered {verdict:?}")
    };
    assert_eq!(skip.call.name(), "unshare");
    assert_eq!(skip.errno, libc::ENOSPC);
    assert!(
        verdict.the_managed_attach_goes_ahead(),
        "the project accepted that a skip lets the add go ahead"
    );
    let lines: Vec<String> = verdict.lines();
    assert!(
        lines[0].contains("unshare answered ENOSPC (errno 28)"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].starts_with("user.max_user_namespaces="),
        "{}",
        lines[1]
    );
}

/// What:
///  The first mapping write is the call Ubuntu 24.04's default profile denies,
///  so `EPERM` there is a skip and an `EACCES` at the scratch mount is one too.
#[test]
fn the_first_mapping_write_and_the_scratch_mount_both_skip() {
    let runtime: RuntimeFolder = runtime_folder("first-mapping-and-scratch-mount");
    let mapped: CompanionVerdict = run_under(
        &runtime,
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::UidMap,
                errno: libc::EPERM,
            },
        },
        COMPANION_DEADLINE,
    );
    assert!(
        matches!(mapped, CompanionVerdict::Skipped { .. }),
        "the first mapping write answering EPERM is the Ubuntu restriction: {mapped:?}"
    );
    let mounted: CompanionVerdict = run_under(
        &runtime,
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::ScratchTmpfs,
                errno: libc::EACCES,
            },
        },
        COMPANION_DEADLINE,
    );
    let CompanionVerdict::Skipped {
        skip,
    } = &mounted
    else {
        panic!(
            "a scratch mount refused with EACCES is mount mediation, and it answered {mounted:?}"
        )
    };
    assert_eq!(skip.call.name(), "mount tmpfs");
}

/// What:
///  The second mapping write answering `EPERM` is the companion's own order
///  rather than the host's policy,
///  so it refuses the arm and names that write.
#[test]
fn the_second_mapping_write_refuses_the_arm_and_names_that_write() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("gid-map-eperm"),
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::GidMap,
                errno: libc::EPERM,
            },
        },
        COMPANION_DEADLINE,
    );
    match verdict {
        CompanionVerdict::Refused {
            refusal:
                Refusal::ArmFailed {
                    step,
                    failure,
                },
        } => {
            assert_eq!(step, CompanionStep::GidMap);
            assert_eq!(failure.call.name(), "write /proc/self/gid_map");
        }
        other => panic!(
            "the second mapping write answering EPERM must refuse the arm, and it answered {other:?}"
        ),
    }
}

/// What:
///  An `fsmount` answering `EPERM` is the arm M11's fixture checks:
///  the mount the account is refused after the setup,
///  which refuses the managed attach and takes the fallback route.
#[test]
fn an_fsmount_refused_after_the_setup_refuses_the_managed_attach() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("fsmount-eperm"),
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::Attach,
                errno: libc::EPERM,
            },
        },
        COMPANION_DEADLINE,
    );
    match &verdict {
        CompanionVerdict::Refused {
            refusal:
                Refusal::ArmFailed {
                    step,
                    failure,
                },
        } => {
            assert_eq!(*step, CompanionStep::Attach);
            assert_eq!(failure.call.name(), "fsmount");
        }
        other => {
            panic!("an fsmount refused with EPERM must refuse the arm, and it answered {other:?}")
        }
    }
    assert!(!verdict.the_managed_attach_goes_ahead());
    let lines: Vec<String> = verdict.lines();
    assert!(
        lines[0].contains("this slot takes the reflink or ordinary Git route"),
        "{}",
        lines[0]
    );
}

/// What:
///  `EINVAL` at a setup call is a defect where this host has a user-namespace
///  entry,
///  which is the other half of the line M11 asks for.
#[test]
fn einval_at_the_setup_refuses_the_arm_where_the_entry_exists() {
    if !user_namespace_entry_exists() {
        eprintln!(
            "ACCOUNT COMPANION CONTROL NOT RUN: this host has no /proc/self/ns/user, so EINVAL is its skip"
        );
        return;
    }
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("userns-einval"),
        Switch::Fixture {
            fixture: Fixture {
                step: CompanionStep::UserNamespace,
                errno: libc::EINVAL,
            },
        },
        COMPANION_DEADLINE,
    );
    assert!(
        matches!(
            verdict,
            CompanionVerdict::Refused {
                refusal: Refusal::ArmFailed { .. }
            }
        ),
        "EINVAL with the entry present is a malformed request: {verdict:?}"
    );
    assert!(
        verdict.lines()[0].contains("managed add refused"),
        "{}",
        verdict.lines()[0]
    );
}

/// What:
///  A child held past the deadline is killed and named,
///  and the managed attach is refused.
#[test]
fn a_child_that_outlasts_the_deadline_is_killed_and_named() {
    let verdict: CompanionVerdict = run_under(
        &runtime_folder("held-past-the-deadline"),
        Switch::Hold {
            step: CompanionStep::Walk,
        },
        Duration::from_millis(30),
    );
    match verdict {
        CompanionVerdict::Refused {
            refusal: Refusal::Deadline {
                deadline,
            },
        } => assert_eq!(deadline, Duration::from_millis(30)),
        other => panic!(
            "a child held after the walk must miss a thirty millisecond deadline, and it answered {other:?}"
        ),
    }
    assert!(!verdict.the_managed_attach_goes_ahead());
}

/// What:
///  A child that ends before its first record is the status pipe's own failure,
///  and a child that kills itself names the signal.
#[test]
fn a_child_that_writes_no_record_and_one_that_dies_are_each_named() {
    let runtime: RuntimeFolder = runtime_folder("no-record-and-killed");
    let silent: CompanionVerdict = run_under(&runtime, Switch::NoStatus, COMPANION_DEADLINE);
    match silent {
        CompanionVerdict::Refused {
            refusal: Refusal::NoStatusLine {
                step_in_flight,
            },
        } => assert_eq!(step_in_flight, CompanionStep::UserNamespace),
        other => panic!(
            "a child that wrote no record is a missing status line, and it answered {other:?}"
        ),
    }
    let killed: CompanionVerdict = run_under(
        &runtime,
        Switch::Signal {
            number: libc::SIGKILL,
        },
        COMPANION_DEADLINE,
    );
    match killed {
        CompanionVerdict::Refused {
            refusal:
                Refusal::ChildSignaled {
                    step_in_flight,
                    signal,
                },
        } => {
            assert_eq!(step_in_flight, CompanionStep::UserNamespace);
            assert_eq!(signal, libc::SIGKILL);
        }
        other => panic!(
            "a child that killed itself before its first record names the signal, and it answered {other:?}"
        ),
    }
}

/// What:
///  The control arm runs nothing,
///  so the timing measurement has a second arm on the same binary.
#[test]
fn the_control_arm_runs_nothing_and_lets_the_add_go_ahead() {
    assert_eq!(
        run_under(
            &runtime_folder("control-arm"),
            Switch::Disabled,
            COMPANION_DEADLINE
        ),
        CompanionVerdict::NotRun
    );
}

/// What:
///  An environment that names no runtime folder refuses the managed add in the
///  sentence a person read before the folder became an input,
///  word for word.
/// Why:
///  The add route resolves the folder and this module writes the line,
///  so a reader of the gate's container log keeps meeting the same sentence:
///  a container sets no `XDG_RUNTIME_DIR`,
///  and nothing about that fact changed when the folder became an input.
#[test]
fn a_variable_that_names_no_folder_refuses_the_managed_add_in_the_sentence_a_person_reads() {
    let refusal: ScratchRefusal = RuntimeFolder::from_name(None)
        .expect_err("a variable that names nothing resolves to no folder");
    let verdict: CompanionVerdict = CompanionVerdict::Refused {
        refusal: Refusal::Scratch {
            message: refusal.message,
        },
    };
    let expected: String = String::from(
        "managed add refused: the account companion's scratch failed: XDG_RUNTIME_DIR is not set, so the account companion has no runtime directory to keep its scratch in. Ways forward: log in again, or set XDG_RUNTIME_DIR to this account's runtime directory, then run the add again",
    );
    assert_eq!(verdict.lines(), vec![expected]);
    assert!(!verdict.the_managed_attach_goes_ahead());
}
