//! What:
//!  The coverage map of the revision 18 "Acceptance target": for each required property and showing control, the test that shows it
//!  (`Covered`), a historical control that is not approval evidence (`Historical`), or `NotYet` with what it needs. A test checks that every
//!  named test exists in the file named, so a renamed or deleted control is found here.
//! Why:
//!  The reviewer's list is finite; a green run of the whole target says nothing about which items it carries. The map keeps the claim and
//!  the evidence in one place that the build checks, and the branch notes list the same rows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const row of TARGET) if (row.status !== 'NotYet') expect(read(row.file)).toContain(`fn ${row.test}(`);
//! ```

use std::fs;
use std::path::PathBuf;

/// How an item of the acceptance target is carried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// A current test shows it.
    Covered,
    /// Only a historical control (a model module labelled as not approval evidence) shows it.
    Historical,
    /// No test shows it yet; the row says what it needs.
    NotYet,
}

use Status::{Covered, Historical, NotYet};

/// One row: the target's group, the property or showing control, its status, and the test (file and name) or what it needs.
pub struct Row {
    /// The acceptance target heading.
    pub group: &'static str,
    /// The property or showing control.
    pub item: &'static str,
    /// How it is carried.
    pub status: Status,
    /// The file holding the test (empty for `NotYet`).
    pub file: &'static str,
    /// The test name, or for `NotYet` what the item needs.
    pub test: &'static str,
}

const fn row(group: &'static str, item: &'static str, status: Status, file: &'static str, test: &'static str) -> Row {
    return Row { group, item, status, file, test };
}

const L: &str = "L1 to L7, L5, X1, service drain";
const C0: &str = "C0 to C5, S1, C3 repair";
const C1: &str = "C1, B1 to B3, Q1";
const P1: &str = "P1 to P4, E1, W1 and W2, recovery classes";
const P2C: &str = "P2c, receipt forcing, L2 and L4, shutdown";
const V: &str = "Rule V departure and optional level 2";
const CK: &str = "Checkpoint selection and cutover, T0, T1, G1 to G4";
const M1: &str = "M1 to M3, MG1 to MG3, MG6c";
const RB: &str = "Rule B, client results, notices and costs";

/// The map. Counts are asserted below, so an added row is a deliberate edit.
pub const TARGET: [Row; 96] = [
    row(L, "control: retained-descriptor deadlock (scope never empties)", Historical, "model_f2_tests.rs", "revision_six_retained_descriptor_blocks_the_next_owner"),
    row(L, "control: restart versus hook admission", Historical, "model_f3_tests.rs", "revision_six_restart_ends_a_hook_dispatched_after_the_observation"),
    row(L, "control: replacement-incarnation targeting", Historical, "model_f3_tests.rs", "candidate_restarts_only_a_quiet_owner_and_only_the_observed_incarnation"),
    row(L, "control: helper parent exit (the one-shot flusher)", Covered, "model_receipt_r18_tests.rs", "a_flusher_in_the_owners_scope_is_ended_by_the_owner_exit"),
    row(L, "control: helper parent exit (every other helper: syncer, level-2 helper)", NotYet, "", "a model of each helper with its parent and scope; none of those helpers exists as a component yet"),
    row(L, "control: hook outlasting an upgrade caller, the drain never ends the hook", Covered, "model_w1_r13_tests.rs", "a_planned_exit_is_a_drain_that_never_ends_a_protected_hook"),
    row(L, "control: hook outlasting an upgrade caller, the caller returns within its bound", NotYet, "", "a register entry for the drain wait (WAIT_DRAIN) and its bound check"),
    row(L, "property: normal lifecycle never cancels a protected execution (premature drain is red)", Covered, "model_w1_r13_tests.rs", "p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive"),
    row(L, "obligation: helpers retain no lock, gate, client-lock or pipe descriptor", NotYet, "", "the internal flush entry point and a descriptor-inventory test of the launched process"),
    row(L, "obligation: launch is non-recursive and resource-bounded", NotYet, "", "the internal flush entry point and a test that the launch never uses the ordinary command line"),
    row(C0, "control: interrupted exposure", Covered, "commit_inbox_tests.rs", "a_torn_or_damaged_inbox_file_is_treated_as_never_written"),
    row(C0, "control: concurrent resend", Covered, "commit_inbox_tests.rs", "two_clients_sending_the_same_bytes_end_with_one_file_and_a_resend"),
    row(C0, "control: marker and state races", Covered, "model_capture_r13_tests.rs", "a_mark_that_appears_between_the_samples_repeats_the_capture"),
    row(C0, "control: failed initial read", Covered, "commit_inbox_tests.rs", "an_early_read_failure_never_reaches_the_link_and_a_fresh_acceptance_does"),
    row(C0, "control: competing repairers", Covered, "commit_inbox_tests.rs", "p1_a_quarantine_never_moves_a_file_it_did_not_inspect"),
    row(C0, "control: preservation-boundary crashes", NotYet, "", "a crash injected at every step of repair and quarantine, and the three-file conflict-order example"),
    row(C0, "property: unknown status never authorizes native fallback", Covered, "commit_inbox_tests.rs", "a_link_that_finds_the_name_and_cannot_read_it_reports_an_unknown_status"),
    row(C0, "property: repair preserves inspected content and unsupported formats", Covered, "commit_inbox_tests.rs", "repair_is_owner_only_keeps_unsupported_versions_and_forces_the_archive"),
    row(C1, "control: the P/Q/A/D counterexample", Covered, "model_support_r18_tests.rs", "pqad_is_red_under_seventeen_and_green_under_eighteen"),
    row(C1, "control: repeated supporting-owner removals", Covered, "model_support_r18_tests.rs", "repeated_removals_of_supporting_owners_follow_the_fold"),
    row(C1, "control: replay changing support", Covered, "model_support_r18_tests.rs", "a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection"),
    row(C1, "control: checkpoint and recovery between rebuild steps", Covered, "model_support_r18_tests.rs", "recovery_between_rebuild_steps_rebuilds_the_same_decision"),
    row(C1, "control: held requests", Covered, "model_support_r18_tests.rs", "a_held_head_blocks_publication_and_leaves_the_decision_alone"),
    row(C1, "control: disjoint requests", Covered, "model_support_r18_tests.rs", "a_disjoint_request_neither_supports_nor_overrides"),
    row(C1, "control: overlapping requests", Covered, "model_support_r18_tests.rs", "an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing"),
    row(C1, "control: superseded requests", Covered, "model_support_r18_tests.rs", "a_superseding_version_changes_support_only_through_its_own_entry"),
    row(C1, "control: comparison with a sequential reference interpreter", Covered, "model_support_r18_tests.rs", "pqad_is_red_under_seventeen_and_green_under_eighteen"),
    row(C1, "property: held-version completion cannot affect its successor", NotYet, "", "a model step where a superseded version's hook completes and a control that lets it publish"),
    row(C1, "property: retained bytes suffice for recomputation (ledger never compacted)", NotYet, "", "a component that recomputes the augmented delta from the request bytes and the ledger's removals"),
    row(P1, "control: death at each pipe write", Covered, "model_pipe_r13_tests.rs", "no_writer_between_instructions_aborts_the_transaction_unless_the_owner_keeps_the_end"),
    row(P1, "control: death at the client-lock handoff", Covered, "model_bound_r14_tests.rs", "exposure_before_the_client_lock_lets_the_owner_decide_a_live_command_as_clientless"),
    row(P1, "control: closed-nosubmit takeover", Covered, "model_w1_r13_tests.rs", "a_client_killed_before_it_wrote_leaves_the_decision_to_the_owner"),
    row(P1, "control: a full commit write after takeover then death is class U", Covered, "model_submission_r15_tests.rs", "a_full_commit_write_then_client_death_is_unknown_and_not_failed"),
    row(P1, "control: buffered commit then abort", Covered, "model_pipe_r13_tests.rs", "a_buffered_commit_is_read_before_end_of_input"),
    row(P1, "control: missing diagnostics", Covered, "model_w1_r13_tests.rs", "classify_separates_the_negative_inference_from_unresolved"),
    row(P1, "control: rewritten publication", Covered, "model_w1_r13_tests.rs", "a_reset_before_the_child_dies_is_unresolved_or_rewritten_never_failed"),
    row(P1, "property: only a positive failure authorizes retry", Covered, "model_w1_r13_tests.rs", "only_a_positive_failure_allows_the_next_attempt"),
    row(P1, "property: unresolved departure permits only the submitted attempt", Covered, "model_w1_r13_tests.rs", "a_query_during_the_unresolved_mark_never_says_nothing_can_land"),
    row(P1, "control: disposable Git experiments (actual FIFO, hook phases, diagnostics, backend)", NotYet, "", "fixtures that run the installed Git for every supported invocation; the process runner exists, the experiments do not"),
    row(P2C, "control: delayed client after the final sweep", Covered, "model_receipt_r18_tests.rs", "a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen"),
    row(P2C, "control: enqueue rejection, as the observable it has here: a ring to a service that has gone is lost (the model has no refusal step)", Covered, "model_receipt_r18_tests.rs", "a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen"),
    row(P2C, "control: worker launch refused by a cap", Covered, "model_receipt_r18_tests.rs", "a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused"),
    row(P2C, "control: worker error", Covered, "model_receipt_r18_tests.rs", "the_next_owner_sweeps_at_startup_and_retries_a_recorded_error"),
    row(P2C, "control: worker exit (one-shot, lock released and recheck sweep)", Covered, "model_receipt_r18_tests.rs", "a_flusher_that_does_not_sweep_once_more_after_releasing_its_lock_loses_a_wakeup"),
    row(P2C, "control: reap mid-sync", NotYet, "", "an L2 reap step for a flusher in its forcing phase, in a model that has the incarnation scope"),
    row(P2C, "control: replacement startup", Covered, "model_receipt_r18_tests.rs", "the_next_owner_sweeps_at_startup_and_retries_a_recorded_error"),
    row(P2C, "control: many outstanding receipts", Covered, "model_receipt_r18_tests.rs", "many_outstanding_receipts_start_one_flusher_and_the_lock_free_control_starts_more"),
    row(P2C, "control: receipt forcing paused, unrelated dispatch bounded", Covered, "model_receipt_r18_tests.rs", "unrelated_dispatch_does_not_wait_for_receipt_forcing"),
    row(P2C, "property: seal before final sweep", Covered, "model_receipt_r18_tests.rs", "sealing_after_the_final_sweep_loses_the_delayed_receipt"),
    row(P2C, "property: outside-fault exits are reported honestly", Covered, "model_register_r13_tests.rs", "p1_a_failed_mark_creation_never_returns_normally"),
    row(V, "control: omitted stop or namespace", Covered, "commit_inbox_tests.rs", "p9_a_withdrawal_with_an_unrelated_stop_file_is_refused"),
    row(V, "control: insufficient acknowledgment", Covered, "commit_inbox_tests.rs", "p1_a_departure_for_a_withdrawal_forces_the_mark_and_cannot_omit_it"),
    row(V, "control: terminal departure without its ledger line", Covered, "commit_inbox_tests.rs", "p9_a_terminal_departure_needs_the_terminal_line_of_its_request"),
    row(V, "control: the acknowledged state's whole closure is forced", Covered, "commit_inbox_tests.rs", "p9_a_departure_forces_the_whole_closure_of_the_state_it_acknowledges"),
    row(V, "control: stop failure still attempts the remaining saves", Covered, "commit_inbox_tests.rs", "p9_a_stop_failure_still_attempts_the_remaining_saves"),
    row(V, "control: ordinary completion cannot stand in for the final save", Covered, "model_register_r13_tests.rs", "p7_a_terminal_outcome_that_skips_its_final_save_is_red_and_the_barrier_route_is_green"),
    row(V, "property: captured authority valid through completion or verified substitution", Covered, "commit_state_log_tests.rs", "p9_completion_cannot_delete_a_dependency_it_validated_and_forced"),
    row(V, "control: reclamation during capture", NotYet, "", "a model of capture against a concurrent reclaimer"),
    row(V, "control: every forcing error", NotYet, "", "fault injection at each forcing call of the departure, with the outside-fault exit checked"),
    row(V, "control: cross-filesystem refs (level 2)", NotYet, "", "level 2 eligibility checks; level 2 may stay disabled"),
    row(V, "control: repack overlap (level 2)", NotYet, "", "level 2 registry observation checks"),
    row(V, "control: verification timeout followed by disposal (level 2)", NotYet, "", "level 2 helper abandonment model"),
    row(V, "property: exact-commit claims need verified durable closure, hook-free execution, fenced late effects", NotYet, "", "level 2 models; the level may remain disabled"),
    row(CK, "control: non-drop counterexample", Covered, "model_discharge_r14_tests.rs", "p1_a_checkpoint_never_erases_a_still_live_request"),
    row(CK, "control: non-drop for later appends", Covered, "commit_state_log_tests.rs", "p9_a_later_append_cannot_drop_a_live_request_of_the_checkpoint"),
    row(CK, "control: drop control isolated from the durability guard", Covered, "model_discharge_r14_tests.rs", "p7_the_drop_control_keeps_the_durability_guard_and_is_red_for_the_drop_alone"),
    row(CK, "control: torn dependencies", Covered, "commit_state_log_tests.rs", "p9_a_torn_envelope_captured_as_opaque_is_not_replacement_authority"),
    row(CK, "control: missing dependencies", Covered, "commit_state_log_tests.rs", "a_checkpoint_validates_and_forces_its_closure_first_and_refuses_a_missing_entry"),
    row(CK, "control: reordered persistence", Covered, "model_discharge_r14_tests.rs", "a_checkpoint_before_its_live_dependency_is_not_a_root_and_the_predecessor_remains"),
    row(CK, "control: candidate activation", Covered, "commit_state_log_tests.rs", "p1_a_background_synced_candidate_with_a_lost_dependency_loses_to_its_predecessor"),
    row(CK, "control: predecessor deletion", Covered, "model_discharge_r14_tests.rs", "a_checkpoint_survives_the_deletion_of_its_predecessor"),
    row(CK, "control: mutable dependency (an appended ledger)", Covered, "commit_state_log_tests.rs", "p9_an_append_to_a_captured_ledger_does_not_invalidate_the_root"),
    row(CK, "property: maximum-generation conflicts are rejected", Covered, "commit_state_log_tests.rs", "two_documents_at_one_generation_are_a_conflict_not_a_root"),
    row(CK, "property: damaged established roots fail closed", Covered, "commit_state_log_tests.rs", "p9_an_established_root_with_a_lost_dependency_fails_closed"),
    row(CK, "control: registry admission", Historical, "model_f8_tests.rs", "nine_withdraws_a_release_when_a_run_appears_and_loses_nothing_otherwise"),
    row(CK, "control: check and release races", Historical, "model_f8_tests.rs", "eight_releases_after_a_run_was_admitted_between_the_observation_and_the_release"),
    row(CK, "control: cross-worktree maintenance", Historical, "model_f8_tests.rs", "revision_six_other_worktree_releases_during_a_prune"),
    row(CK, "property: maintenance cannot outlive its recovery protection", Historical, "model_f8_tests.rs", "ten_roots_the_closure_until_the_release_is_done"),
    row(CK, "property: saved content survives until authorized disposal", NotYet, "", "a disposal model: the disposed ledger line durable before the saved content is deleted"),
    row(CK, "control: push exclusion and frozen updates against actual Git", NotYet, "", "disposable Git experiments"),
    row(M1, "control: concurrent staging", Covered, "model_capture_r13_tests.rs", "a_later_staging_is_replaced_under_twelve_and_left_under_thirteen"),
    row(M1, "control: conditional installation keeps displaced versions", Covered, "model_merge_r14_tests.rs", "overlapping_staged_and_worktree_versions_are_all_kept"),
    row(M1, "control: no foreign lock removal", Covered, "model_m1_split_r13_tests.rs", "a_lock_without_a_live_creator_is_never_removed"),
    row(M1, "control: interrupted installation and reboot", Covered, "model_watch_r13_tests.rs", "a_cleanup_begun_before_the_span_ends_finishes_after_it_and_no_new_one_starts"),
    row(M1, "control: HEAD changes", Historical, "model_f5_tests.rs", "revision_seven_first_try_path_stages_on_the_switched_branch"),
    row(M1, "control: crash at every carry step", Historical, "model_m1_crash_tests.rs", "nine_recovers_every_kill_point_by_identity"),
    row(M1, "control: open-descriptor writes", NotYet, "", "a model of a writer holding the file open across the install"),
    row(M1, "control: atomic saves", NotYet, "", "a model of rename-over saves against the install"),
    row(M1, "control: oversized and raced reads", NotYet, "", "inline-bound checks on actual reads and whole-index work"),
    row(M1, "control: expiry between clock check and marker (a resumed install does not restart the window)", NotYet, "", "the watch model with the lock before expiry and the marker after it"),
    row(RB, "control: whole-command timing and sync instrumentation (plain commit makes no sync)", Covered, "commit_inbox_tests.rs", "the_waiting_path_of_a_plain_commit_makes_no_sync"),
    row(RB, "control: notice dismissal keeps saved copies", Covered, "model_watch_r13_tests.rs", "dismissing_the_reminder_deletes_no_saved_copy_and_the_restore_command_still_works"),
    row(RB, "control: no-manager preparation timeout", NotYet, "", "a register entry and fixture for the no-manager preparation wait"),
    row(RB, "control: recovery conflict, refusal and signing failure notices", NotYet, "", "notice models per outcome"),
    row(RB, "control: notice crash and reclamation", NotYet, "", "notice delivery state with crash points"),
    row(RB, "property: question 8 recovery policy (open owner choice, no green by adopting the recommendation)", NotYet, "", "the owner's answer on corrected options"),
];

/// Every named test exists in the file named.
#[test]
fn every_covered_or_historical_row_has_its_test() {
    let native: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    for row in &TARGET {
        assert!(!row.group.is_empty(), "{}: every row belongs to a group of the target", row.item);
        if row.status == NotYet {
            assert!(row.file.is_empty() && !row.test.is_empty(), "{}: a not-yet row names what it needs", row.item);
            continue;
        }
        let place: PathBuf = if row.file.starts_with("commit_") { native.join("commit").join(row.file) } else { native.join("model").join(row.file) };
        let text: String = fs::read_to_string(&place).unwrap_or_else(|error| { panic!("{}: {}: {}", row.item, place.display(), error) });
        assert!(text.contains(&format!("fn {}(", row.test)), "{}: no test `{}` in {}", row.item, row.test, row.file);
    }
}

/// The counts the branch notes report; changing a row changes them on purpose.
#[test]
fn the_counts_of_the_map_are_the_ones_reported() {
    let count = |status: Status| -> usize { return TARGET.iter().filter(|row| { return row.status == status; }).count(); };
    assert_eq!((count(Covered), count(Historical), count(NotYet)), (COVERED, HISTORICAL, NOT_YET));
}

/// Items carried by a current test.
pub const COVERED: usize = 62;
/// Items carried only by a historical control.
pub const HISTORICAL: usize = 9;
/// Items with no test yet.
pub const NOT_YET: usize = 25;
