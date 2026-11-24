//! What:
//!  Controls of the command line's statuses:
//!  the help and version words answer zero,
//!  every gated mode word answers the refusal status,
//!  a refused gate never calls a mode's own work,
//!  and a probe caller that is not root is refused before the gate or any
//!  write.
//! Why:
//!  A boot unit's `ExecStart` and `ExecStopPost` read this status,
//!  so a mode that refuses while answering zero would report a reconciled boot that never happened
//!  (`handler_mode_dispatch`).
//!  Every mode runs from fixture places and behind a gate that answers without
//!  this machine,
//!  so a control reads the command line's own decision and never this host's
//!  configuration,
//!  store,
//!  mount table or `/dev/zfs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(run(['--help'])).toBe(0); expect(runBehindAt(gate, fixtures, [])).toBe(2);
//! ```

use super::{
    DispatchPlaces, EXIT_DONE, EXIT_REFUSED, attaching_sweep_mode, behind_the_gate, behind_the_gate_for_the_probe_as, boot_reconcile_mode, report,
    run_behind, run_behind_at, shutdown_unmount_mode,
};
use super::handler_mode_arguments::{self, HandlerMode};
use crate::start_probe::RecordPlace;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::ReadinessFile;
use crate::zfs_compatibility_gate::gate_verdict::{Facts, Verdict};
use crate::zfs_compatibility_gate::{BeforeRequests, gate_words::TESTED_MODULE_VERSION};
use rustix::process::geteuid;
use std::cell::Cell;
use std::ffi::OsString;
use std::path::PathBuf;

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

/// One invocation through the passing gate,
/// from this machine's installed places.
///
/// Only the words that read no place run this way:
/// `--help` and `--version` answer before any mode,
/// and an unreadable invocation never reaches one.
fn run(words: &[OsString]) -> i32 {
    return run_behind(&PassingGate, words);
}

/// The arguments of one invocation,
/// as the kernel hands them over.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(|word| return OsString::from(*word)).collect();
}

/// What:
///  One command line's places,
///  all of them in one fixture folder,
///  with a configuration file that is there and holds nothing.
/// Why:
///  An empty file is refused before any mode reaches the device,
///  the store or `/run`,
///  so each mode answers its own work without touching this machine,
///  and every place is under the folder the control removes.
fn fixture_places(label: &str) -> DispatchPlaces {
    let root: PathBuf = fixture(label);
    let configuration: PathBuf = root.join("git-worktree-clone-mounts.jsonc");
    std::fs::write(&configuration, "").expect("the fixture configuration is written");
    let store: PathBuf = root.join("store");
    let probe_record: PathBuf = root.join("probe-record");
    return DispatchPlaces {
        sweep: super::SweepPlaces::at(configuration.clone(), store.clone(), probe_record.clone()),
        reconcile: super::ReconcilePlaces::at(configuration.clone(), store, probe_record),
        readiness: ReadinessFile::at(root.join("readiness"), geteuid().as_raw()),
        probe: RecordPlace::at(root.join("records")),
        probe_configuration: configuration,
    };
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

/// Every gated mode word answers the refusal status,
/// from the places the control hands in.
#[test]
fn every_gated_mode_word_answers_the_refusal_status() {
    let places: DispatchPlaces = fixture_places("dispatch-gated-statuses");
    for words in [Vec::<&str>::new(), vec!["boot-reconcile"], vec!["shutdown-unmount"]] {
        assert_eq!(run_behind_at(&PassingGate, &places, arguments(words.as_slice()).as_slice()), EXIT_REFUSED, "{words:?}");
    }
}

/// An unreadable invocation is a refusal,
/// never a mode taken by default.
#[test]
fn an_unreadable_invocation_answers_the_refusal_status() {
    let places: DispatchPlaces = fixture_places("dispatch-unreadable");
    for words in [vec!["sweep"], vec!["boot-reconcile", "--force"], vec!["adopt-worktree-parent"], vec!["adopt-worktree-parent", "--checkout"]] {
        assert_eq!(run_behind_at(&PassingGate, &places, arguments(words.as_slice()).as_slice()), EXIT_REFUSED, "{words:?}");
    }
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

/// A refused gate answers the refusal status,
/// and the mode's own work is never called:
/// the closure that stands for a mode records that it ran,
/// so a build that entered it answers zero here.
#[test]
fn a_refused_gate_answers_the_refusal_status_and_runs_no_mode() {
    let ran: Cell<bool> = Cell::new(false);
    let outcome = || {
        ran.set(true);
        return Ok(String::new());
    };
    assert_eq!(report(behind_the_gate(&RefusingGate, outcome)), EXIT_REFUSED, "a refused gate answered a status other than the refusal status");
    assert!(!ran.get(), "the refused gate called the mode's own work");
    let passed = || {
        ran.set(true);
        return Ok(String::new());
    };
    assert_eq!(report(behind_the_gate(&PassingGate, passed)), EXIT_DONE, "a passing gate did not answer the mode's own status");
    assert!(ran.get(), "a passing gate did not call the mode's own work");
}

/// The one mode whose step has not landed names the unit that runs it,
/// so a passing gate still leaves a step a person can act on.
///
/// The constant is paired with the invocation that reaches its mode,
/// and the gate's line is the one a person reads:
/// the message the mode itself answers,
/// never the awaiting value's own words alone.
#[test]
fn the_unbuilt_mode_names_the_unit_that_runs_it() {
    let invocation: Vec<OsString> = arguments(&["shutdown-unmount"]);
    assert_eq!(handler_mode_arguments::parse(invocation.as_slice()), Ok(HandlerMode::ShutdownUnmount));
    let message: String = shutdown_unmount_mode::AWAITING.message();
    assert!(message.contains(shutdown_unmount_mode::AWAITING.arrives_with), "the refusal names the unit that runs the mode: {message}");
    let answer: Result<String, String> = behind_the_gate(&PassingGate, || {
        return shutdown_unmount_mode::run().map_err(|refused| return refused.message());
    });
    let Err(line): Result<String, String> = answer else {
        panic!("shutdown-unmount answered a passing gate");
    };
    assert!(line.contains(message.as_str()), "{line}");
}

/// What:
///  The sweep and the boot reconciliation are built,
///  so a passing gate enters them instead of a step's refusal,
///  and each answers the mode's own word.
/// Why:
///  The two modes plan step 8 builds must not answer as a step that has not
///  landed,
///  and a configuration file that is there and holds nothing is refused by the
///  mode's own read,
///  so neither opens the device the control's gate would have let it open.
#[test]
fn the_sweep_and_the_boot_reconciliation_answer_their_own_work_on_fixture_places() {
    let places: DispatchPlaces = fixture_places("dispatch-modes-own-work");
    let sweep: Result<String, String> = attaching_sweep_mode::run_at(&places.sweep);
    let Err(said): Result<String, String> = sweep else {
        panic!("the sweep answered a configuration that holds nothing");
    };
    assert!(said.contains("is empty"), "the sweep answered its own refusal: {said}");
    assert!(!said.contains("is not built yet"), "a built mode must not answer as one whose step has not landed: {said}");
    let boot: Result<String, String> = boot_reconcile_mode::run_at(&places.reconcile, &places.readiness);
    let Err(boot_said): Result<String, String> = boot else {
        panic!("the boot reconciliation answered a configuration that holds nothing");
    };
    assert!(boot_said.contains("is empty"), "the boot reconciliation answered its own refusal: {boot_said}");
    assert!(!boot_said.contains("is not built yet"), "a built mode must not answer as one whose step has not landed: {boot_said}");
}

/// The probe's own gate refusal writes a skipped record in the place it is
/// given,
/// names the gate's step,
/// and never reaches the mode's own work.
#[test]
fn the_probes_gate_refusal_writes_a_skipped_record_in_the_given_place() {
    let root: PathBuf = fixture("start-probe-dispatch-skip");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    let configuration: PathBuf = root.join("handler.jsonc");
    std::fs::write(&configuration, "{}").expect("the fixture configuration is written");
    let answer: Result<String, String> = behind_the_gate_for_the_probe_as(&RefusingGate, &place, configuration.as_path(), true, 0);
    let line: String = answer.expect("a skipped probe is a verdict, not a refusal");
    assert!(line.contains("start probe skipped"), "{line}");
    assert!(line.contains("device"), "{line}");
    let text: String = std::fs::read_to_string(place.result()).expect("the skipped record is written");
    assert!(text.contains("verdict skipped"), "{text}");
    assert!(text.contains("skip-reason device"), "{text}");
}

/// A caller that is not root is refused before the gate is asked and before
/// anything is written,
/// in the sentence the adoption answers with,
/// and the status is the refusal status.
///
/// The caller's answer is an argument because a control cannot change its own
/// effective user:
/// the command line passes the user the kernel reports.
#[test]
fn a_start_probe_that_is_not_root_is_refused_before_the_gate_or_any_write() {
    let root: PathBuf = fixture("start-probe-not-root");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    let configuration: PathBuf = root.join("handler.jsonc");
    std::fs::write(&configuration, "{}").expect("the fixture configuration is written");
    let answer: Result<String, String> = behind_the_gate_for_the_probe_as(&RefusingGate, &place, configuration.as_path(), false, 1000);
    let Err(line): Result<String, String> = answer else {
        panic!("a caller that is not root started the probe: {answer:?}");
    };
    assert!(line.contains("the start-probe mode runs as root"), "{line}");
    assert!(line.contains("effective user is 1000"), "{line}");
    assert!(!line.contains("compatibility gate"), "the refusal is the caller's, not the gate's: {line}");
    assert!(!place.result().exists(), "the refused caller wrote a record");
}
