//! What:
//!  The controls of the plan value:
//!  the first write,
//!  the merge rule,
//!  the closed history,
//!  and the fields the writer refuses to put past their bounds.
//! Why:
//!  The first write is the record of everything the proof verified,
//!  so a line more or a line less than the inventory is a record no later reader can trust,
//!  and a writer that published a field past its bound would leave a plan its own reader refuses
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "2. The fields" and "4. Progress, unknown outcomes and objects that are partly gone").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = DestructionPlan.firstWrite(facts, snapshots);
//! expect(plan.attempts.every(a => a.outcome === 'planned' && a.attempt === 0)).toBe(true);
//! ```

use crate::worktree_clone_removal_proof_and_plan::fixtures::{SAMPLE_CLONE_GUID, SAMPLE_OWN_GUID, guid, sample_bytes, sample_facts, sample_plan, sample_snapshots};
use crate::worktree_clone_removal_proof_and_plan::plan_parse::parse_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanAttempt, PlanFacts, PlanSnapshot, owner_text, target_hexadecimal};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::PlanRefusalCause;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::split_the_header;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};
use std::path::Path;

#[test]
fn the_first_write_records_every_verified_snapshot_once_at_attempt_zero() {
    let plan: DestructionPlan = sample_plan();
    assert_eq!(plan.head, HeadWord::Present);
    assert_eq!(plan.attempts.len(), sample_snapshots().len());
    for line in &plan.attempts {
        assert_eq!(line.attempt, 0);
        assert_eq!(line.outcome, SnapshotOutcome::Planned);
        assert_eq!(line.errno, None);
    }
    let names: Vec<&str> = plan.attempts.iter().map(|line| return line.snapshot.name.as_str()).collect();
    assert_eq!(names, vec!["tank/wt/fix-flaky-test@sanoid-hourly", "tank/wt/fix-flaky-test@worktree-own"], "the lines are in ascending name order");
}

#[test]
fn the_written_header_is_the_designs_own_lines() {
    let text: String = String::from_utf8(sample_bytes()).expect("a plan is text");
    assert!(text.starts_with("# monochromatic-git-policy-cli zfs worktree clone destruction plan, version 1\nrun=6f1c9d2a7b4e40a1b8c1d3e5f7091a2b\n"));
    assert!(text.contains("\nowner=48213:linux:9481733\n"));
    assert!(text.contains("\ncloneDataset=tank/wt/fix-flaky-test\n"));
    assert!(text.contains(&format!("\ncloneGuid={SAMPLE_CLONE_GUID}\n")));
    assert!(text.contains("\nunmountMountId=none\n"));
    assert!(text.contains("\nhead=present\nattempts=2\n"));
    assert!(text.contains("tuple=snapshot 0 planned 9332110456720018274 20 - name=tank/wt/fix-flaky-test@sanoid-hourly\n"));
    assert!(text.ends_with("tuple=snapshot 0 planned 17157080670377245572 18 - name=tank/wt/fix-flaky-test@worktree-own\n"), "the last line is the last name: {text}");
}

#[test]
fn the_bytes_read_back_as_the_value_they_were_written_from() {
    let plan: DestructionPlan = sample_plan();
    let bytes: Vec<u8> = plan.encode().expect("the sample is inside every bound");
    let (header, attempts) = split_the_header(bytes.as_slice()).expect("the sample has a complete header");
    let read_back: DestructionPlan = parse_plan(header, attempts, Some(SAMPLE_CLONE_GUID.to_string().as_str())).expect("the sample reads back");
    assert_eq!(read_back, plan);
}

#[test]
fn the_merge_rule_reads_a_names_highest_number_and_a_gap_as_a_lost_write() {
    let mut plan: DestructionPlan = sample_plan();
    let name: String = String::from("tank/wt/fix-flaky-test@worktree-own");
    let snapshot: PlanSnapshot = plan.attempts[1].snapshot.clone();
    // Attempt 1 landed, attempt 2 was lost before it landed, attempt 3 landed: the gap is not damage.
    plan.record_line(PlanAttempt { snapshot: snapshot.clone(), attempt: 1, outcome: SnapshotOutcome::Refused, errno: Some(16) }).expect("a first retry is recorded");
    plan.record_line(PlanAttempt { snapshot: snapshot.clone(), attempt: 3, outcome: SnapshotOutcome::Intact, errno: None }).expect("a later attempt is recorded");
    let current: &PlanAttempt = plan.current_state(name.as_str()).expect("the name is in the plan");
    assert_eq!(current.attempt, 3);
    assert_eq!(current.outcome, SnapshotOutcome::Intact);
    assert_eq!(plan.current_state("tank/wt/fix-flaky-test@absent"), None);
}

#[test]
fn a_destroyed_line_closes_a_names_history_and_the_writer_refuses_to_reopen_it() {
    let mut plan: DestructionPlan = sample_plan();
    let snapshot: PlanSnapshot = plan.attempts[1].snapshot.clone();
    plan.record_line(PlanAttempt { snapshot: snapshot.clone(), attempt: 1, outcome: SnapshotOutcome::Destroyed, errno: None }).expect("the end of a history is recorded");
    let refusal = plan.record_line(PlanAttempt { snapshot: snapshot.clone(), attempt: 2, outcome: SnapshotOutcome::Intact, errno: None }).expect_err("nothing follows a destroyed line");
    assert_eq!(refusal.cause, PlanRefusalCause::DuplicateAttempt, "the closed history refuses a later line");
    plan.attempts.push(PlanAttempt { snapshot, attempt: 2, outcome: SnapshotOutcome::Intact, errno: None });
    let reader_side = plan.encode().expect_err("a line after a destroyed one is not written");
    assert_eq!(reader_side.cause, PlanRefusalCause::DestroyedHistoryReopened);
}

#[test]
fn the_writer_refuses_two_lines_of_one_name_and_one_attempt_number() {
    let mut plan: DestructionPlan = sample_plan();
    let snapshot: PlanSnapshot = plan.attempts[0].snapshot.clone();
    plan.attempts.push(PlanAttempt { snapshot, attempt: 0, outcome: SnapshotOutcome::Intact, errno: None });
    let refusal = plan.encode().expect_err("one number is used once");
    assert_eq!(refusal.cause, PlanRefusalCause::DuplicateAttempt);
}

#[test]
fn a_value_past_its_own_bound_is_refused_before_it_is_published() {
    let long_path: std::path::PathBuf = Path::new("/").join("a".repeat(4_096));
    let mut facts: PlanFacts = sample_facts();
    facts.target = long_path;
    let path_refusal = DestructionPlan::first_write(facts, sample_snapshots().as_slice()).encode().expect_err("a path past 4,095 bytes cannot be recorded");
    assert_eq!(path_refusal.cause, PlanRefusalCause::FieldTooWide);
    assert_eq!(path_refusal.field.as_deref(), Some("targetHex"));

    let long_name: PlanSnapshot = PlanSnapshot { name: format!("{}{}", "tank/wt@", "a".repeat(256)), guid: guid(SAMPLE_OWN_GUID), createtxg: 18 };
    let name_refusal = DestructionPlan::first_write(sample_facts(), &[long_name]).encode().expect_err("a name past the kernel's bound cannot be recorded");
    assert_eq!(name_refusal.cause, PlanRefusalCause::FieldTooWide);
}

#[test]
fn a_moment_that_is_not_an_iso_second_with_an_offset_is_refused() {
    let mut facts: PlanFacts = sample_facts();
    facts.started = String::from("yesterday");
    let refusal = DestructionPlan::first_write(facts, sample_snapshots().as_slice()).encode().expect_err("a moment of another shape is not recorded");
    assert_eq!(refusal.cause, PlanRefusalCause::NotCanonical);
    assert_eq!(refusal.field.as_deref(), Some("started"));
}

#[test]
fn an_errno_is_written_only_on_a_refused_line_and_a_refusal_needs_one() {
    let mut plan: DestructionPlan = sample_plan();
    plan.attempts[0].errno = Some(16);
    let refusal = plan.encode().expect_err("an errno belongs to a refusal");
    assert_eq!(refusal.cause, PlanRefusalCause::ErrnoOnAnotherLine);
    plan.attempts[0].outcome = SnapshotOutcome::Refused;
    plan.attempts[0].errno = None;
    let missing_errno = plan.encode().expect_err("a refusal carries the errno it was refused with");
    assert_eq!(missing_errno.cause, PlanRefusalCause::MissingErrno);
    plan.attempts[0].errno = Some(16);
    let text: String = String::from_utf8(plan.encode().expect("a refusal with its errno is written")).expect("a plan is text");
    assert!(text.contains("tuple=snapshot 0 refused 9332110456720018274 20 16 name="), "the refusal's errno is written: {text}");
}

#[test]
fn the_owner_and_the_target_are_written_in_the_route_its_own_spellings() {
    assert_eq!(owner_text(48_213, "linux:9481733"), "48213:linux:9481733");
    assert_eq!(target_hexadecimal(Path::new("/repo/.claude/worktrees/fix")), "2f7265706f2f2e636c617564652f776f726b74726565732f666978");
}
