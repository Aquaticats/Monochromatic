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

use super::{EXIT_DONE, EXIT_REFUSED, run_behind};
use crate::zfs_compatibility_gate::gate_verdict::{Facts, Verdict};
use crate::zfs_compatibility_gate::{BeforeRequests, gate_words::TESTED_MODULE_VERSION};
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
