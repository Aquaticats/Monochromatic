//! Controls for the lines a person reads:
//!  the evidence names the record, the slot, the pool's answer, the cleanup and the plan,
//!  the removals are numbered in order,
//!  and the last line names what the wrapper can no longer prove.

use crate::testing::mounted_dataset_table::ScriptedMountTable;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_record_settling::settling_absence::{TheClonesPresence, the_clones_presence};
use crate::worktree_clone_record_settling::settling_fixtures::{CLONE_GUID, ScriptedProbe, SLOT, TheFixture, a_guid, publish_the_record, the_fixture, write_the_pointer};
use crate::worktree_clone_record_settling::settling_lines::{TheEvidence, evidence_lines, last_line, removal_lines};
use crate::worktree_clone_record_settling::settling_record::{TheRecord, read_the_record};
use crate::worktree_clone_record_settling::settling_slot::{TheSlot, read_the_slot};
use crate::worktree_clone_record_settling::settling_steps::WHEN_NO_CHECKOUT_HAS_BEGUN;

/// The clone's dataset name the fixture record carries.
const CLONE: &str = "tank/monochromatic-git-worktree-clone-datasets/topic";

/// The evidence of one healthy fixture whose clone the pool proved gone.
fn the_healthy_evidence(made: &TheFixture) -> (TheRecord, TheClonesPresence, TheSlot) {
    publish_the_record(made, Phase::Filled);
    write_the_pointer(made);
    let record: TheRecord = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let probe: ScriptedProbe = ScriptedProbe::empty();
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(made.slot_path.as_path(), made.registration.as_path(), record.recorded.registration_identity, &table).expect("the slot reads");
    return (record, presence, slot);
}

/// The evidence names the record, the confirmed slot, the absence, the empty cleanup and the empty plan.
#[test]
fn the_evidence_names_every_read() {
    let made: TheFixture = the_fixture("settling-lines-evidence");
    let (record, presence, slot) = the_healthy_evidence(&made);
    let evidence: TheEvidence<'_> = TheEvidence { record: &record, presence: &presence, slot: &slot, cleanup: None, plan: None };
    let lines: Vec<String> = evidence_lines(&evidence);
    assert!(lines[0].contains("phase=filled"), "{}", lines[0]);
    assert!(lines[0].contains("GUID `4821193048571903102`"), "{}", lines[0]);
    assert!(lines[1].contains("no mount stands at the slot"), "{}", lines[1]);
    assert!(lines[1].contains("no checkout has begun"), "{}", lines[1]);
    assert!(lines[2].contains("absence is proved"), "{}", lines[2]);
    assert!(lines[3].contains("no origin cleanup record stands"), "{}", lines[3]);
    assert!(lines[4].contains("no destruction plan stands"), "{}", lines[4]);
}

/// A checkout's line says which command removes what the command leaves.
#[test]
fn the_checkout_line_names_the_worktree_removal() {
    let made: TheFixture = the_fixture("settling-lines-checkout");
    let (record, presence, _unused) = the_healthy_evidence(&made);
    std::fs::write(made.slot_path.join("README.md"), b"# the person's file\n").expect("the file writes");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(made.slot_path.as_path(), made.registration.as_path(), record.recorded.registration_identity, &table).expect("the slot reads");
    let evidence: TheEvidence<'_> = TheEvidence { record: &record, presence: &presence, slot: &slot, cleanup: None, plan: None };
    let lines: Vec<String> = evidence_lines(&evidence);
    assert!(lines[1].contains("a checkout has begun"), "{}", lines[1]);
    assert!(lines[1].contains("removes only the records"), "{}", lines[1]);
}

/// The removals are numbered in the order the run makes them.
#[test]
fn the_removals_are_numbered_in_order() {
    let lines: Vec<String> = removal_lines(WHEN_NO_CHECKOUT_HAS_BEGUN.as_slice());
    assert!(lines[0].contains("in this order"), "{}", lines[0]);
    assert!(lines[1].trim_start().starts_with("1. the registration's lost lock"), "{}", lines[1]);
    assert!(lines[2].trim_start().starts_with("2. the registration's own files"), "{}", lines[2]);
    assert!(lines[7].trim_start().starts_with("7. the slot's claim"), "{}", lines[7]);
}

/// The last line names what the wrapper can no longer prove.
#[test]
fn the_last_line_names_the_cost() {
    let line: String = last_line(SLOT);
    assert!(line.contains("can no longer prove which clone"), "{line}");
    assert!(line.contains(SLOT), "{line}");
}
