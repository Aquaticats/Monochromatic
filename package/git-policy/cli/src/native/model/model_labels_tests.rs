//! What:
//!  A guard on the evidence labels of the model modules: every `model_*.rs` module is either one of the revision 13 modules (the only
//!  approval evidence for revision 13) or opens with the label `Historical control, not approval evidence for revision 13`.
//! Why:
//!  The third models review asked that retired or narrowly scoped greens be labelled out of the approval evidence. A new model file that
//!  is neither listed nor labelled fails here, so an old green cannot be read as current evidence by omission.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const file of readdirSync(dir).filter(isModel)) expect(approval.has(file) || firstLine(file).startsWith(LABEL)).toBe(true);
//! ```

use std::fs;
use std::path::PathBuf;

/// The label every historical module opens with.
const LABEL: &str = "//! Historical control, not approval evidence for revision 13:";

/// The modules that are evidence for revision 13, and the tools and test files that are not models.
const CURRENT: [&str; 35] = [
    "model_explore.rs",
    "model_fixture.rs",
    "model_w1_r13.rs",
    "model_register_r13.rs",
    "model_barrier_r13.rs",
    "model_capture_r13.rs",
    "model_watch_r13.rs",
    "model_pipe_r13.rs",
    "model_m1_split_r13.rs",
    "model_discharge_r14.rs",
    "model_bound_r14.rs",
    "model_submission_r15.rs",
    "model_submission_r15_tests.rs",
    "model_merge_r14.rs",
    "model_merge_r14_tests.rs",
    "model_bound_r14_tests.rs",
    "model_discharge_r14_tests.rs",
    "model_m1_split_r13_tests.rs",
    "mod.rs",
    "model_labels_tests.rs",
    "model_w1_r13_tests.rs",
    "model_register_r13_tests.rs",
    "model_barrier_r13_tests.rs",
    "model_capture_r13_tests.rs",
    "model_watch_r13_tests.rs",
    "model_pipe_r13_tests.rs",
    "model_support_r18.rs",
    "model_support_r18_tests.rs",
    "model_receipt_r18.rs",
    "model_receipt_r18_tests.rs",
    "model_target_tests.rs",
    "model_fresh_attempt_r19.rs",
    "model_fresh_attempt_r19_tests.rs",
    "model_m1_reclaim_r19.rs",
    "model_m1_reclaim_r19_tests.rs",
];

/// Every model module is current or labelled historical.
#[test]
fn every_model_module_is_current_or_labelled_historical() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native").join("model");
    let mut checked: usize = 0;
    for entry in fs::read_dir(&directory).expect("the source directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let is_model: bool = name.starts_with("model_") && name.ends_with(".rs") && !name.ends_with("_tests.rs");
        if !is_model || CURRENT.contains(&name.as_str()) {
            continue;
        }
        let text: String = fs::read_to_string(entry.path()).expect("the module source");
        assert!(text.starts_with(LABEL), "{} is neither a revision 13 module nor labelled historical", name);
        checked += 1;
    }
    assert!(checked > 0, "no historical module was found, so the scan did not run");
}

/// A revision 13 module never carries the historical label (the label would hide current evidence).
#[test]
fn the_current_modules_do_not_carry_the_historical_label() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native").join("model");
    for name in CURRENT {
        let text: String = fs::read_to_string(directory.join(name)).expect("the module source");
        assert!(!text.starts_with(LABEL), "{} is current evidence", name);
    }
}

/// The acceptance manifest for revision 14: each of the thirteenth review's twenty-two controls and the test that carries it, so that a
/// whole-target green is not read as acceptance and a missing or renamed control is found here. Historical results are not listed.
const ACCEPTANCE: [(u8, &str, &str); 25] = [
    (1, "model_w1_r13_tests.rs", "success_then_owner_death_then_reset_does_not_publish_again_under_fourteen"),
    (2, "model_bound_r14_tests.rs", "exposure_before_the_client_lock_lets_the_owner_decide_a_live_command_as_clientless"),
    (3, "model_bound_r14_tests.rs", "fourteen_takes_the_lock_before_the_link_and_exits_on_both_sides_are_clean"),
    (4, "model_barrier_r13_tests.rs", "thirteen_a_timed_departure_can_lose_only_the_mark"),
    (5, "model_barrier_r13_tests.rs", "fifteen_the_barrier_runs_to_completion_and_every_exit_keeps_its_stop"),
    (6, "model_pipe_r13_tests.rs", "no_writer_between_instructions_aborts_the_transaction_unless_the_owner_keeps_the_end"),
    (7, "model_pipe_r13_tests.rs", "failed_closing_writes_queue_nothing_and_an_abort_may_not_be_enqueued"),
    (8, "model_discharge_r14_tests.rs", "a_reclaimed_request_does_not_drop_the_later_adoption"),
    (9, "model_discharge_r14_tests.rs", "a_checkpoint_survives_the_deletion_of_its_predecessor"),
    (10, "commit_state_log_tests.rs", "one_writer_owns_the_state_and_a_checkpoint_does_not_lose_later_appends"),
    (11, "commit_state_log_tests.rs", "a_sync_acknowledges_only_the_captured_prefix"),
    (12, "model_discharge_r14_tests.rs", "a_checkpoint_before_its_live_dependency_is_not_a_root_and_the_predecessor_remains"),
    (13, "model_capture_r13_tests.rs", "a_mark_that_appears_between_the_samples_repeats_the_capture"),
    (14, "model_w1_r13_tests.rs", "a_late_positive_failure_after_the_unresolved_exit_records_withdrawn"),
    (15, "model_merge_r14_tests.rs", "a_plain_commit_of_an_old_based_edit_keeps_the_merge_change"),
    (16, "model_merge_r14_tests.rs", "a_stat_match_with_e_is_not_a_proof_that_the_file_is_o"),
    (17, "model_merge_r14_tests.rs", "overlapping_staged_and_worktree_versions_are_all_kept"),
    (18, "model_watch_r13_tests.rs", "fifteen_has_the_accepted_concession_and_nothing_else"),
    (19, "model_register_r13_tests.rs", "thirteen_has_no_bound_for_the_stop_receipt_and_for_materialization"),
    (20, "model_w1_r13_tests.rs", "a_planned_shutdown_with_an_attached_client_aborts_as_class_f_and_decides_nothing"),
    (21, "model_barrier_r13_tests.rs", "thirteen_a_named_commit_whose_parent_was_not_forced_is_lost"),
    (22, "model_w1_r13_tests.rs", "a_query_during_the_unresolved_mark_never_says_nothing_can_land"),
    (0, "model_watch_r13_tests.rs", "the_earlier_watches_are_red_over_every_schedule"),
    (0, "model_submission_r15_tests.rs", "the_submission_window_is_red_under_fourteen_and_green_under_fifteen"),
    (0, "model_register_r13_tests.rs", "fifteen_every_entry_is_bounded_except_the_final_save_which_is_exempt_by_decision"),
];

/// Without the seams feature the state-log controls would be listed and never run, so the manifest refuses that configuration.
#[cfg(not(feature = "test-seams"))]
#[test]
fn the_acceptance_manifest_needs_the_seams_feature() {
    panic!("the acceptance manifest is checked only in the configuration that runs its controls (--features test-seams)");
}

/// Every control of the manifest has a test of that name in the file named (control 0 rows are the owner-question matrices).
#[test]
fn every_acceptance_control_has_its_test() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native").join("model");
    for (number, file, name) in ACCEPTANCE {
        let place: PathBuf = if file.starts_with("commit_") { directory.join("..").join("commit").join(file) } else { directory.join(file) };
        let text: String = fs::read_to_string(place).expect("the test file");
        assert!(text.contains(&format!("fn {}(", name)), "control {} has no test `{}` in {}", number, name, file);
    }
    let numbered: std::collections::BTreeSet<u8> = ACCEPTANCE.iter().map(|row| { return row.0; }).filter(|number| { return *number != 0; }).collect();
    assert_eq!(numbered.len(), 22, "the thirteenth review lists twenty-two controls");
}

/// The sixteenth review's controls (revision 17's list) that these models and components carry, with the test for each. Controls not listed
/// here are not covered yet and are queued in the branch notes.
#[cfg(feature = "test-seams")]
const ACCEPTANCE_R17: [(u8, &str, &str); 8] = [
    (13, "commit_state_log_tests.rs", "p1_a_background_synced_candidate_with_a_lost_dependency_loses_to_its_predecessor"),
    (13, "commit_state_log_tests.rs", "p1_a_torn_envelope_named_as_a_dependency_is_not_a_satisfied_dependency"),
    (13, "commit_state_log_tests.rs", "p1_a_dependency_under_an_unsynced_name_chain_is_forced_through_the_anchor"),
    (14, "commit_inbox_tests.rs", "p1_a_departure_for_a_withdrawal_forces_the_mark_and_cannot_omit_it"),
    (15, "commit_inbox_tests.rs", "p1_a_quarantine_never_moves_a_file_it_did_not_inspect"),
    (17, "model_register_r13_tests.rs", "p1_an_ordinary_wait_completion_cannot_stand_in_for_the_final_save"),
    (17, "model_w1_r13_tests.rs", "a_planned_exit_is_a_drain_that_never_ends_a_protected_hook"),
    (18, "model_discharge_r14_tests.rs", "p1_a_checkpoint_never_erases_a_still_live_request"),
];

/// Every covered control of the sixteenth review has its test.
#[cfg(feature = "test-seams")]
#[test]
fn every_covered_sixteenth_review_control_has_its_test() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    for (number, file, name) in ACCEPTANCE_R17 {
        let place: PathBuf = if file.starts_with("commit_") { directory.join("commit").join(file) } else { directory.join("model").join(file) };
        let text: String = fs::read_to_string(place).expect("the test file");
        assert!(text.contains(&format!("fn {}(", name)), "control {} has no test `{}` in {}", number, name, file);
    }
}
