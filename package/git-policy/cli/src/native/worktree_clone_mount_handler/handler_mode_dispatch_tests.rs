//! What:
//!  Controls of the command line's statuses:
//!  the help and version words answer zero,
//!  every unreadable invocation and every unbuilt mode answers the refusal status,
//!  and no refusal reaches zero.
//! Why:
//!  A boot unit's `ExecStart` and `ExecStopPost` read this status,
//!  so a mode that refuses while answering zero would report a reconciled boot that never happened
//!  (`handler_mode_dispatch`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(run(['--help'])).toBe(0); expect(run([])).toBe(2);
//! ```

use super::{
    EXIT_DONE, EXIT_REFUSED, attaching_sweep_mode, behind_the_gate, behind_the_gate_for_the_probe, boot_reconcile_mode, run_behind,
    shutdown_unmount_mode,
};
use crate::start_probe::RecordPlace;
use crate::testing::test_support::fixture;
use super::handler_mode_arguments::{self, HandlerMode};
use crate::zfs_compatibility_gate::gate_verdict::{Facts, Verdict};
use crate::zfs_compatibility_gate::{BeforeRequests, gate_words::TESTED_MODULE_VERSION};
use std::cell::Cell;
use std::ffi::OsString;

/// The gate a control hands in:
/// compatible,
/// with the tested module's release text,
/// so a mode's own answer is what the control reads.
struct PassingGate;

impl BeforeRequests for PassingGate {
    fn verdict(&self) -> Verdict {
        return Verdict::Compatible(Facts {
            target: "x86_64",
            version: Some(String::from(TESTED_MODULE_VERSION)),
            absent: Vec::new(),
            base_btf_bytes: 10,
            module_btf_bytes: 10,
            header_bytes: (32, 32),
        });
    }
}

/// The gate that refuses,
/// as a machine with no device answers.
struct RefusingGate;

impl BeforeRequests for RefusingGate {
    fn verdict(&self) -> Verdict {
        return Verdict::Refused(monochromatic_zfs_request_to_openzfs_kernel_device::CompatibilityRefusal {
            step: monochromatic_zfs_request_to_openzfs_kernel_device::CompatibilityStep::Device,
            file: None,
            detail: String::from("it is absent"),
        });
    }
}

/// One invocation through the passing gate.
fn run(words: &[OsString]) -> i32 {
    return run_behind(&PassingGate, words);
}

/// The arguments of one invocation,
/// as the kernel hands them over.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(|word| return OsString::from(*word)).collect();
}

/// One line reaches the stream with the newline that ends it,
/// so no two lines run together.
#[test]
fn one_line_carries_its_own_newline() {
    assert_eq!(super::line_to_write("mount-git-worktree-clone-datasets 0.6.1-alpha.44"), "mount-git-worktree-clone-datasets 0.6.1-alpha.44\n");
}

/// The two statuses are zero and the wrapper's own refusal status.
#[test]
fn the_two_statuses_are_zero_and_the_refusal_status() {
    assert_eq!(EXIT_DONE, 0);
    assert_eq!(EXIT_REFUSED, 2);
}

/// The words that ask what this build is answer zero.
#[test]
fn the_help_and_version_words_answer_zero() {
    assert_eq!(run(arguments(&["--help"]).as_slice()), EXIT_DONE);
    assert_eq!(run(arguments(&["--version"]).as_slice()), EXIT_DONE);
}

/// Every unbuilt mode refuses,
/// and the default invocation is one of them.
#[test]
fn every_unbuilt_mode_answers_the_refusal_status() {
    assert_eq!(run(arguments(&[]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["boot-reconcile"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["shutdown-unmount"]).as_slice()), EXIT_REFUSED);
}

/// An unreadable invocation is a refusal,
/// never a mode taken by default.
#[test]
fn an_unreadable_invocation_answers_the_refusal_status() {
    assert_eq!(run(arguments(&["sweep"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["boot-reconcile", "--force"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["adopt-worktree-parent"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["adopt-worktree-parent", "--checkout"]).as_slice()), EXIT_REFUSED);
}

/// A refused gate never calls the mode's own work,
/// and the line it answers says so.
#[test]
fn a_refused_gate_keeps_the_mode_unentered_and_says_so() {
    let ran: Cell<bool> = Cell::new(false);
    let answer: Result<String, String> = behind_the_gate(&RefusingGate, || {
        ran.set(true);
        return Ok(String::from("the mode ran"));
    });
    let Err(line): Result<String, String> = answer else {
        panic!("a refused gate ran the mode");
    };
    assert!(!ran.get(), "the refused gate called the mode");
    assert!(line.contains("no mode was entered"), "{line}");
    assert!(!line.contains("the mode ran"), "{line}");
    let passed: Result<String, String> = behind_the_gate(&PassingGate, || {
        ran.set(true);
        return Ok(String::from("the mode ran"));
    });
    assert!(passed.is_ok(), "a passing gate refused the mode");
    assert!(ran.get(), "a passing gate did not call the mode");
}

/// Each unbuilt mode names the plan step that lands it,
/// so a passing gate still leaves the step a person can act on.
///
/// Each constant is paired with the invocation that reaches its mode,
/// and the gate's line is the one a person reads:
/// the message the mode itself answers,
/// never the awaiting value's own words alone.
#[test]
fn each_unbuilt_mode_names_its_plan_step() {
    for (awaiting, words, mode) in [(shutdown_unmount_mode::AWAITING, vec!["shutdown-unmount"], HandlerMode::ShutdownUnmount)] {
        let invocation: Vec<OsString> = arguments(words.as_slice());
        assert_eq!(
            handler_mode_arguments::parse(invocation.as_slice()),
            Ok(mode.clone()),
            "{words:?} does not reach {mode:?}"
        );
        let message: String = awaiting.message();
        assert!(
            message.contains(format!("plan step {}", awaiting.step).as_str()),
            "{message}"
        );
        let answer: Result<String, String> = behind_the_gate(&PassingGate, || {
            return match mode {
                HandlerMode::ShutdownUnmount => shutdown_unmount_mode::run(),
                _ => unreachable!("the table names only the unbuilt modes"),
            }
            .map_err(|refused| return refused.message());
        });
        let Err(line): Result<String, String> = answer else {
            panic!("{words:?} answered a passing gate");
        };
        assert!(line.contains(message.as_str()), "{line}");
    }
}

/// What:
///  The sweep and the boot reconciliation are built,
///  so a passing gate enters them instead of a step's refusal,
///  and each answers the mode's own word.
/// Why:
///  The two modes plan step 8 builds must not answer as a step that has not
///  landed,
///  and this host has no installed configuration,
///  so each answers the configuration's own refusal naming the key to write.
#[test]
fn the_two_built_modes_answer_their_own_work() {
    for words in [Vec::<&str>::new(), vec!["boot-reconcile"]] {
        let answer: Result<String, String> = behind_the_gate(&PassingGate, || {
            return if words.is_empty() { attaching_sweep_mode::run() } else { boot_reconcile_mode::run() };
        });
        let Err(line): Result<String, String> = answer else {
            panic!("{words:?} answered a passing gate on a host with no installed configuration");
        };
        assert!(line.contains("git-worktree-clone-mounts.jsonc") || line.contains("start probe"), "the mode entered its own work and answered its own refusal: {line}");
        assert!(!line.contains("is not built yet"), "a built mode must not answer as one whose step has not landed: {line}");
    }
}

/// The probe's own gate refusal writes a skipped record in the place it is
/// given,
/// names the gate's step,
/// and never reaches the mode's own work.
#[test]
fn the_probes_gate_refusal_writes_a_skipped_record_in_the_given_place() {
    let root: std::path::PathBuf = fixture("start-probe-dispatch-skip");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    let configuration: std::path::PathBuf = root.join("handler.jsonc");
    std::fs::write(&configuration, "{}").expect("the fixture configuration is written");
    let answer: Result<String, String> = behind_the_gate_for_the_probe(&RefusingGate, &place, configuration.as_path());
    let line: String = answer.expect("a skipped probe is a verdict, not a refusal");
    assert!(line.contains("start probe skipped"), "{line}");
    assert!(line.contains("device"), "{line}");
    let text: String = std::fs::read_to_string(place.result()).expect("the skipped record is written");
    assert!(text.contains("verdict skipped"), "{text}");
    assert!(text.contains("skip-reason device"), "{text}");
}

/// A gate that refuses keeps the mode unentered,
/// so nothing that mode owns is opened or changed.
#[test]
fn a_refused_gate_answers_the_refusal_status_and_runs_no_mode() {
    for words in [Vec::<&str>::new(), vec!["boot-reconcile"], vec!["shutdown-unmount"]] {
        let status: i32 = run_behind(&RefusingGate, arguments(words.as_slice()).as_slice());
        assert_eq!(status, EXIT_REFUSED, "{words:?}");
    }
    assert_eq!(
        run_behind(&RefusingGate, arguments(&["--help"]).as_slice()),
        EXIT_DONE,
        "the words that ask what this build is need no gate"
    );
}
