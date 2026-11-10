//! What:
//!  The gate's own control:
//! a command that never asks Git for the merge attribute leaves the control woken by the command's end,
//!  never waiting for a pass that will not come.
//! Why:
//!  The two controls of the second pass wait on the pass,
//!  and a wait that only the pass can end would hang the whole control when a change stops the pass
//!  from reaching its attribute query.
//!  The command's own end is the gate's other word,
//!  so this control proves it is written and heard,
//!  and it needs no clock to do it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [observed, control] = passGate(fixture).heldRun(fixture, repo, ['rev-parse', '--git-dir'], () => { throw new Error('never') });
//! expect(await control).toBe(false);  // woken by the command's end, not by the pass
//! ```

use super::support::{Fixture, fixture, remove, repository};
use super::word_merge_pass_gate::PassGate;
use std::ffi::OsStr;
use std::path::PathBuf;

/// What:
///  A wrapped command that starts the stand-in Git but never asks for the merge attribute ends,
/// and the control that watches for the pass says it never got there.
/// Why:
///   Without the command's own end as a word,
/// a control would wait for ever whenever the pass does not reach its query,
///  which is a hang and not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [observed, control] = gate.heldRun(fixture, repo, ['rev-parse', '--git-dir'], when);
/// expect(observed.code).toBe(0); expect(await control).toBe(false);
/// ```
#[test]
fn a_command_that_never_asks_for_the_attribute_wakes_the_control() {
    let fixture: Fixture = fixture("word-merge-gate-never-asked");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let gate: PassGate = PassGate::prepare(&fixture);
    let (observed, control) = gate.held_run(&fixture, repo.as_path(), &["rev-parse", "--git-dir"], || {
        panic!("the command never asks for the merge attribute, so the control must not act");
    });
    assert_eq!(observed.code, Some(0), "the command itself ran: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
    let starts: Vec<String> = gate.starts();
    assert!(starts.iter().any(|line| { return line.contains("rev-parse"); }), "the stand-in Git was the run's Git, so the query could have come to it: {starts:?}");
    assert!(!gate.reached_marker(), "the command asked for no merge attribute, so the gate never opened");
    assert!(!control.join().expect("the control thread"), "the control was woken by the command's own end, not left waiting for a pass that never came");
    remove(&fixture);
}
