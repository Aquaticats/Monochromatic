//! What:
//!  The coverage map of the revision 18 "Acceptance target": for each required property and showing control, the test that shows it
//!  (`Covered`), a test that shows a narrower control than the property with what remains unshown (`Narrower`), a historical control that is
//!  not approval evidence (`Historical`), or `NotYet` with what it needs. A test checks that every
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
    /// A current test shows a narrower control than the property the target states; the row says what remains unshown.
    Narrower,
    /// Only a historical control (a model module labelled as not approval evidence) shows it.
    Historical,
    /// No test shows it yet; the row says what it needs.
    NotYet,
}

use Status::{Covered, Historical, Narrower, NotYet};

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
    /// For `Narrower`: what the test does not show (empty otherwise).
    pub remains: &'static str,
}

const fn row(group: &'static str, item: &'static str, status: Status, file: &'static str, test: &'static str) -> Row {
    return Row { group, item, status, file, test, remains: "" };
}

const fn narrower(group: &'static str, item: &'static str, file: &'static str, test: &'static str, remains: &'static str) -> Row {
    return Row { group, item, status: Narrower, file, test, remains };
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
pub const TARGET: &[Row] = &[
    row(L, "control: retained-descriptor deadlock (scope never empties)", Historical, "model_f2_tests.rs", "revision_six_retained_descriptor_blocks_the_next_owner"),
    row(L, "control: restart versus hook admission", Historical, "model_f3_tests.rs", "revision_six_restart_ends_a_hook_dispatched_after_the_observation"),
    row(L, "control: replacement-incarnation targeting", Historical, "model_f3_tests.rs", "candidate_restarts_only_a_quiet_owner_and_only_the_observed_incarnation"),
    row(L, "control: helper parent exit (the one-shot flusher)", Covered, "model_receipt_r18_tests.rs", "a_flusher_killed_or_reaped_mid_sync_leaves_its_receipts_to_the_startup_sweep"),
    row(L, "control: helper parent exit (every other helper: syncer, level-2 helper)", NotYet, "", "a model of each helper with its parent and scope; none of those helpers exists as a component yet"),
    row(L, "control: hook outlasting an upgrade caller, the drain never ends the hook", Covered, "model_w1_r13_tests.rs", "a_planned_exit_is_a_drain_that_never_ends_a_protected_hook"),
    row(L, "control: hook outlasting an upgrade caller, the caller returns within its bound", NotYet, "", "a register entry for the drain wait (WAIT_DRAIN) and its bound check"),
    row(L, "property: normal lifecycle never cancels a protected execution (premature drain is red)", Covered, "model_w1_r13_tests.rs", "p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive"),
    narrower(L, "obligation: helpers retain no lock, gate, client-lock or pipe descriptor", "commit_helper_launch_tests.rs", "a_lane_lock_the_launcher_releases_is_free_while_the_helper_lives", "the other helpers (the syncer, a level-2 helper) are not components yet, and the manager-started transient unit route is not built; the launch shown is the double-forked fallback, with a descriptor inventory (a_launched_helper_inherits_no_descriptor_of_its_launcher), the lock test, and the flusher helper that keeps only the lock its launcher handed it: the launcher closes its own copy before the helper is told to start (a_client_that_joins_while_the_launcher_still_holds_its_copy_has_an_executor), the lock and the pipes are moved above the standard streams (a_launch_with_descriptor_zero_closed_leaves_the_helper_holding_the_lock), and an intermediate child that does not leave is killed and reaped (a_ready_helper_does_not_make_the_launch_succeed_while_the_intermediate_child_is_unreaped); a descriptor a raw creator keeps outside this component is not visible to it"),
    narrower(L, "obligation: launch is non-recursive and resource-bounded", "commit_helper_launch_tests.rs", "a_launched_entry_is_a_function_in_a_new_session_and_not_a_second_start_of_the_program", "the transient unit route is not built; a refused fork is injected by a real process limit and a memory cap is not; a process with more than one thread is refused, shown on test copies of a process and not at real call sites, which do not exist yet; the bound on helpers is one lock holder (clients_that_find_the_flusher_lock_held_start_no_helper, many_clients_start_one_helper_and_it_holds_the_lock) and NOT a bound on live helpers: a helper that released the lock and is in its recheck or exit step holds nothing, a launch in that step starts another, and processes stopped from outside can pile up (options in the handover, round 16); a launch that fails records every receipt of the lane, joiners included (a_client_that_joined_a_launch_whose_setup_failed_is_recorded)"),
    row(C0, "control: interrupted exposure", Covered, "commit_inbox_tests.rs", "a_torn_or_damaged_inbox_file_is_treated_as_never_written"),
    row(C0, "control: concurrent resend", Covered, "commit_inbox_tests.rs", "two_clients_sending_the_same_bytes_end_with_one_file_and_a_resend"),
    row(C0, "control: marker and state races", Covered, "model_capture_r13_tests.rs", "a_mark_that_appears_between_the_samples_repeats_the_capture"),
    row(C0, "control: failed initial read", Covered, "commit_inbox_tests.rs", "an_early_read_failure_never_reaches_the_link_and_a_fresh_acceptance_does"),
    narrower(C0, "control: competing repairers", "commit_inbox_tests.rs", "p1_a_quarantine_never_moves_a_file_it_did_not_inspect", "two legitimate repairers contending for authority beyond the one non-blocking lock refusal the test asserts; the test replaces the file by raw writes while authority is held and so shows stale-object detection"),
    row(C0, "control: preservation-boundary crashes", NotYet, "", "a crash injected at every step of repair and quarantine, and the three-file conflict-order example"),
    row(C0, "property: unknown status never authorizes native fallback", Covered, "commit_inbox_tests.rs", "a_link_that_finds_the_name_and_cannot_read_it_reports_an_unknown_status"),
    row(C0, "property: repair preserves inspected content and unsupported formats", Covered, "commit_inbox_tests.rs", "repair_is_owner_only_keeps_unsupported_versions_and_forces_the_archive"),
    row(C1, "control: the P/Q/A/D counterexample", Covered, "model_support_r18_tests.rs", "pqad_is_red_under_seventeen_and_green_under_eighteen"),
    row(C1, "control: repeated supporting-owner removals", Covered, "model_support_r18_tests.rs", "repeated_removals_of_supporting_owners_follow_the_fold"),
    row(C1, "control: replay changing support", Covered, "model_support_r18_tests.rs", "a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection"),
    narrower(C1, "control: checkpoint and recovery between rebuild steps", "model_support_r18_tests.rs", "a_crash_between_a_checkpoint_and_a_rebuild_is_rebuilt_from_the_tail", "persistence of the retained inputs and a complete checkpoint protocol: the model checks derived-cache recovery assuming the current tip and tail survive every modeled crash unchanged, and a checkpoint stores one decision bit (retained-byte sufficiency is the NotYet row below)"),
    row(C1, "control: held requests", Covered, "model_support_r18_tests.rs", "a_held_head_blocks_publication_and_leaves_the_decision_alone"),
    row(C1, "control: disjoint requests", Covered, "model_support_r18_tests.rs", "a_disjoint_request_neither_supports_nor_overrides"),
    row(C1, "control: overlapping requests", Covered, "model_support_r18_tests.rs", "an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing"),
    row(C1, "control: superseded requests", Covered, "model_support_r18_tests.rs", "a_superseding_version_changes_support_only_through_its_own_entry"),
    row(C1, "control: comparison with a sequential reference interpreter", Covered, "model_support_r18_tests.rs", "the_reference_world_follows_every_operation_through_its_own_code"),
    row(C1, "control: the whole request family against the reference (all two-entry combinations and four selected three-entry starts, tip fixed at zero, selections 1 or 2; red under revision 17)", Covered, "model_support_r18_tests.rs", "every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen"),
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
    narrower(P1, "control: disposable Git experiments (actual FIFO, hook phases, diagnostics, backend)", "model_git_experiments_tests.rs", "a_fifo_transaction_prepares_holds_through_a_pause_and_commits_on_commit", "every supported invocation beyond `update-ref --stdin` with an update (creation, deletion, symbolic refs, the side-ref transaction), packed refs, linked worktrees, and Git versions other than the installed 2.56.0; the neighbouring tests cover abort, end of input, buffered commit, ENXIO, hook phases, a hook refusal and two diagnostics on both the files and reftable backends"),
    row(P2C, "control: delayed client after the final sweep", Covered, "model_receipt_r18_tests.rs", "a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen"),
    row(P2C, "control: enqueue rejection by a running service", Covered, "model_receipt_r18_tests.rs", "an_enqueue_rejected_by_a_running_service_falls_back_to_a_flusher"),
    row(P2C, "control: a ring to a service that has gone is lost (the delayed client)", Covered, "model_receipt_r18_tests.rs", "a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen"),
    row(P2C, "control: worker launch refused by a cap", Covered, "model_receipt_r18_tests.rs", "a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused"),
    narrower(P2C, "control: worker error (every unforced receipt recorded, joiners included, each retried by the next owner)", "model_receipt_r18_tests.rs", "a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each", "the error path is now four observable phases (list, record, release, list again; a_receipt_that_joins_between_the_error_listing_and_the_release_is_recorded_by_the_second_listing shows the joiner), with one forcing error per run and one erroring flusher; a failure of the recording itself, repeated errors and the real error file are not modeled here (the file is shown by the component tests); a record written after the last startup sweep while an owner runs is retried by that owner's syncer (coordinator pick of round 15; a_running_owner_retries_an_error_recorded_after_its_startup_sweep, red for the control whose syncer never retries)"),
    row(P2C, "control: worker error on the actual lane files (every receipt whose name is not forced is recorded, the joiner too, also one that joins after the listing)", Covered, "commit_flusher_window_tests.rs", "a_receipt_that_joins_after_the_error_listing_is_recorded"),
    row(P2C, "control: worker exit (one-shot, lock released and recheck sweep)", Covered, "model_receipt_r18_tests.rs", "a_launch_in_the_release_window_starts_a_second_flusher_that_takes_the_lock"),
    row(P2C, "control: worker exit on the actual lock (the recheck sweep forces a receipt created in the release window)", Covered, "commit_flusher_tests.rs", "a_receipt_created_in_the_release_window_is_forced_by_the_recheck_sweep"),
    narrower(P2C, "control: reap mid-sync", "model_receipt_r18_tests.rs", "a_flusher_killed_or_reaped_mid_sync_leaves_its_receipts_to_the_startup_sweep", "the real reap of an incarnation scope by the service (the model places the flusher outside or inside the scope as a rule choice)"),
    row(P2C, "control: replacement startup", Covered, "model_receipt_r18_tests.rs", "a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each"),
    row(P2C, "control: many outstanding receipts", Covered, "model_receipt_r18_tests.rs", "many_outstanding_receipts_keep_one_flusher_and_the_lock_free_control_runs_two"),
    row(P2C, "control: the actual flusher lock admits one holder and a second flusher forces nothing", Covered, "commit_flusher_tests.rs", "a_second_flusher_finds_the_lock_held_and_forces_nothing"),
    row(P2C, "control: launch in the release window of the flusher lock", Covered, "model_receipt_r18_tests.rs", "a_launch_in_the_release_window_starts_a_second_flusher_that_takes_the_lock"),
    row(P2C, "control: receipt forcing paused, unrelated dispatch bounded", Covered, "model_receipt_r18_tests.rs", "unrelated_dispatch_does_not_wait_for_receipt_forcing"),
    row(P2C, "property: seal before final sweep", Covered, "model_receipt_r18_tests.rs", "sealing_after_the_final_sweep_loses_the_delayed_receipt"),
    row(P2C, "property: outside-fault exits are reported honestly", Covered, "model_register_r13_tests.rs", "p1_a_failed_mark_creation_never_returns_normally"),
    row(V, "control: omitted stop or namespace (one stop cannot stand for two requests; each needs its own)", Covered, "commit_inbox_tests.rs", "r14_each_departing_request_needs_its_own_stop_and_its_own_adoption"),
    row(V, "control: one stop receipt cannot acknowledge two withdrawals", Covered, "commit_inbox_tests.rs", "r14_one_stop_receipt_cannot_acknowledge_the_withdrawal_of_two_requests"),
    row(V, "control: insufficient acknowledgment (content-bound, still covering after a later append)", Covered, "commit_inbox_tests.rs", "r14_a_departures_acknowledgment_still_covers_after_a_later_append"),
    row(V, "control: terminal departure without its ledger line (a valid ledger of another request)", Covered, "commit_inbox_tests.rs", "r14_a_valid_ledger_of_another_request_does_not_satisfy_a_terminal_departure"),
    row(V, "control: the acknowledged state's whole closure is forced (saved content of the candidate)", Covered, "commit_inbox_tests.rs", "r14_a_departure_forces_the_saved_content_of_the_candidate_it_acknowledges"),
    row(V, "control: stop failure still attempts the remaining saves (a stop error and a saved-content error)", Covered, "commit_inbox_tests.rs", "r14_every_save_is_attempted_after_a_stop_error_and_a_saved_content_error"),
    row(V, "control: ordinary completion cannot stand in for the final save", Covered, "model_register_r13_tests.rs", "p7_a_terminal_outcome_that_skips_its_final_save_is_red_and_the_barrier_route_is_green"),
    narrower(V, "property: captured authority valid through completion or verified substitution", "commit_state_log_tests.rs", "r14_an_acknowledgment_survives_completion_and_not_a_substitute_that_drops_its_support", "survival under a concurrent reclaimer (the reclamation-during-capture control is not yet); across segments only a compaction of the head covers an acknowledgment (r16_a_checkpoint_that_changes_the_document_does_not_cover_an_earlier_acknowledgment, r15_an_unrelated_document_that_retains_the_same_items_does_not_cover_the_acknowledgment): a checkpoint that changes the document records no lineage, so an acknowledgment before it is not covered and the caller forces again; one hop only, so an acknowledgment older than the previous checkpoint is not covered; the older test p9_completion_cannot_delete_a_dependency_it_validated_and_forced shows only the state-directory refusal"),
    row(V, "control: reclamation during capture", NotYet, "", "a model of capture against a concurrent reclaimer"),
    row(V, "control: every forcing error", NotYet, "", "fault injection at each forcing call of the departure, with the outside-fault exit checked"),
    row(V, "control: cross-filesystem refs (level 2)", NotYet, "", "level 2 eligibility checks; level 2 may stay disabled"),
    row(V, "control: repack overlap (level 2)", NotYet, "", "level 2 registry observation checks"),
    row(V, "control: verification timeout followed by disposal (level 2)", NotYet, "", "level 2 helper abandonment model"),
    row(V, "property: exact-commit claims need verified durable closure, hook-free execution, fenced late effects", NotYet, "", "level 2 models; the level may remain disabled"),
    row(CK, "control: non-drop counterexample", Covered, "model_discharge_r14_tests.rs", "p1_a_checkpoint_never_erases_a_still_live_request"),
    narrower(CK, "control: non-drop for later appends (a record cannot omit what its predecessor retained)", "commit_state_log_tests.rs", "r14_a_record_cannot_omit_what_the_previous_record_retained", "complete closure: a reclaiming checkpoint derives its obligations from the lane (every request in inbox/, every saved content, the ledger up to its length now) and refuses a closure that omits one, whatever its document declares (r16_a_reclaiming_checkpoint_must_retain_what_the_lane_holds_whatever_its_document_declares); a dependency outside those places, such as an order file, is declared by needs lines (r15_a_document_that_names_a_dependency_cannot_be_checkpointed_or_appended_without_it) and a writer that omits that line is still accepted; a plain append reclaims nothing and keeps only the non-drop check; the alternative for such documents is a non-reclaiming checkpoint (handover, round 16)"),
    row(CK, "control: retirement discharge forces the ledger prefix that proves it", Covered, "commit_state_log_tests.rs", "r14_a_retirement_forces_the_ledger_prefix_that_discharges_it"),
    row(CK, "control: a later record retaining new saved content is not selected without it", Covered, "commit_state_log_tests.rs", "r14_a_later_record_that_retains_new_saved_content_is_not_selected_without_it"),
    row(CK, "control: drop control isolated from the durability guard", Covered, "model_discharge_r14_tests.rs", "p7_the_drop_control_keeps_the_durability_guard_and_is_red_for_the_drop_alone"),
    row(CK, "control: torn dependencies (a prefix entry under the inbox cannot validate a torn envelope)", Covered, "commit_state_log_tests.rs", "r14_a_prefix_entry_under_the_inbox_cannot_validate_a_torn_envelope"),
    row(CK, "control: missing dependencies", Covered, "commit_state_log_tests.rs", "a_checkpoint_validates_and_forces_its_closure_first_and_refuses_a_missing_entry"),
    row(CK, "control: reordered persistence", Covered, "model_discharge_r14_tests.rs", "a_checkpoint_before_its_live_dependency_is_not_a_root_and_the_predecessor_remains"),
    row(CK, "control: candidate activation", Covered, "commit_state_log_tests.rs", "p1_a_background_synced_candidate_with_a_lost_dependency_loses_to_its_predecessor"),
    row(CK, "control: predecessor deletion", Covered, "model_discharge_r14_tests.rs", "a_checkpoint_survives_the_deletion_of_its_predecessor"),
    narrower(CK, "control: mutable dependency (an existing ledger writer after capture)", "commit_state_log_tests.rs", "r14_an_existing_ledger_writer_cannot_invalidate_a_root_that_captured_the_ledger", "saved content enters the store only through create_saved_content and an existing file is never adopted: capture of saved content is refused for any existing file and creation installs its own fresh inode over the name (r16_an_existing_read_only_inode_at_the_name_is_replaced_and_never_adopted, r15_a_raw_file_in_the_saved_directory_is_not_saved_content_and_the_component_made_one_cannot_be_reopened_for_writing); a read lease is an extra check where granted and never a refusal (r15_a_file_system_that_refuses_read_leases_still_creates_and_checkpoints_saved_content); the premise is that nobody with the lane's account chmods and rewrites sealed content; a privileged writer or a writable parent directory can still replace it, which the checksum name detects and does not prevent"),
    row(CK, "property: maximum-generation conflicts are rejected", Covered, "commit_state_log_tests.rs", "two_documents_at_one_generation_are_a_conflict_not_a_root"),
    narrower(CK, "property: damaged established roots fail closed", "commit_state_log_tests.rs", "p9_an_established_root_with_a_lost_dependency_fails_closed", "telling an established damaged root from an incomplete candidate when older segments are usable; selection records no establishment, and the test has no other usable segment"),
    row(CK, "control: registry admission", Historical, "model_f8_tests.rs", "nine_withdraws_a_release_when_a_run_appears_and_loses_nothing_otherwise"),
    row(CK, "control: check and release races", Historical, "model_f8_tests.rs", "eight_releases_after_a_run_was_admitted_between_the_observation_and_the_release"),
    row(CK, "control: cross-worktree maintenance", Historical, "model_f8_tests.rs", "revision_six_other_worktree_releases_during_a_prune"),
    row(CK, "property: maintenance cannot outlive its recovery protection", Historical, "model_f8_tests.rs", "ten_roots_the_closure_until_the_release_is_done"),
    row(CK, "property: saved content survives until authorized disposal", NotYet, "", "a disposal model: the disposed ledger line durable before the saved content is deleted"),
    narrower(CK, "control: push exclusion and frozen updates against actual Git", "model_git_experiments_tests.rs", "a_negative_refspec_keeps_the_internal_namespace_off_the_remote_and_a_named_object_is_pushed_as_given", "the marker the lane acknowledges and the stacked push path; the test pushes to a local bare remote with plain `git push`"),
    row(M1, "control: concurrent staging", Covered, "model_capture_r13_tests.rs", "a_later_staging_is_replaced_under_twelve_and_left_under_thirteen"),
    row(M1, "control: conditional installation keeps displaced versions", Covered, "model_merge_r14_tests.rs", "overlapping_staged_and_worktree_versions_are_all_kept"),
    narrower(M1, "control: no foreign lock removal, no live lock removed, and the lane reclaims a proven-stale lock of its own dead incarnation (owner decision of 2026-11-04; claim files on every file system)", "model_m1_reclaim_r19_tests.rs", "a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_claim_chain_and_a_dead_claim_is_passed_over", "the claim chain is the current policy and this is its evidence; the flock hold is comparison evidence (a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_flock) and the mixed-mount control (the_mixed_mount_residue_is_reachable_and_is_the_stated_one) shows why mixed exclusion is unsuitable and is not an accepted residue; the model has two one-shot reclaimers and at most one death, scans for a slot and places the claim in one step, clears the claims in one step and has no retry, so the green is not unconditional eventual reclamation (a stale lock may stay after a death) and says little about slot reuse and cleanup interleavings; the slot protocol and its leftover handling are the unreviewed branch fix/git-policy-cli-index-lock-reclaim-without-flock at b071e48f74; a person removing the lock by hand between the check and the unlink is the labelled premise break; no component calls the mechanism yet"),
    row(M1, "control: interrupted installation and reboot", Covered, "model_watch_r13_tests.rs", "a_cleanup_begun_before_the_span_ends_finishes_after_it_and_no_new_one_starts"),
    row(M1, "control: HEAD changes", Historical, "model_f5_tests.rs", "revision_seven_first_try_path_stages_on_the_switched_branch"),
    row(M1, "control: crash at every carry step", Historical, "model_m1_crash_tests.rs", "nine_recovers_every_kill_point_by_identity"),
    row(M1, "control: open-descriptor writes", NotYet, "", "a model of a writer holding the file open across the install"),
    row(M1, "control: atomic saves", NotYet, "", "a model of rename-over saves against the install"),
    row(M1, "control: oversized and raced reads", NotYet, "", "inline-bound checks on actual reads and whole-index work"),
    row(M1, "control: expiry between clock check and marker (a resumed install does not restart the window)", NotYet, "", "the watch model with the lock before expiry and the marker after it"),
    narrower(RB, "control: whole-command timing and sync instrumentation", "commit_inbox_tests.rs", "the_waiting_path_of_a_plain_commit_makes_no_sync", "whole-process latency and every sync the wrapper makes; the test counts the inbox waiting path only"),
    row(RB, "control: notice dismissal keeps saved copies", Covered, "model_watch_r13_tests.rs", "dismissing_the_reminder_deletes_no_saved_copy_and_the_restore_command_still_works"),
    row(RB, "control: no-manager preparation timeout", NotYet, "", "a register entry and fixture for the no-manager preparation wait"),
    narrower(RB, "control: recovery conflict, refusal and signing failure notices", "model_fresh_attempt_r19_tests.rs", "the_notice_follows_the_outcome", "the notice texts of the conflict, refusal and signing-failure outcomes, which the model keeps as one Failed symbol without outcome detail or commit identity; the stored notice carries the attempt that produced it, and its print, removal and delivery are bound to that tag (an_old_notice_printed_after_a_new_attempt_was_chosen_does_not_deliver_the_new_attempt, with a monitor that reads the stored tag and not the owner's bookkeeping); after a crash with surviving positive authority the model reconciles a notice that still says scheduled (a_settled_recovery_reconciles_a_notice_that_still_says_scheduled); the first version of this row overstated both, the reviewer's schedule is now a regression"),
    narrower(RB, "control: notice crash and reclamation", "model_fresh_attempt_r19_tests.rs", "recovery_data_is_kept_until_the_outcome_notice_is_delivered", "crash points of every notice kind; the model covers the fresh-attempt notice only, for one request with at most two power losses; reclamation needs the forced removal of the current attempt's own outcome notice, tracked by the stored tag, with print, remove and force-removal as separate steps (an_earlier_attempts_delivery_does_not_authorize_reclaiming_a_later_attempt, an_old_notice_printed_after_a_new_attempt_was_chosen_does_not_deliver_the_new_attempt); the straight-line test named here shows the plain case only"),
    narrower(RB, "property: decision 19, fresh attempt with a stored notice when no decisive positive authority survives", "model_fresh_attempt_r19_tests.rs", "decision_nineteen_is_green_over_every_schedule", "one request and at most two power losses; one Failed symbol for conflict, refusal and signing failure, with no outcome detail and no commit identity; the notice file is a visible and a durable content with a producing-attempt tag, not the real file; no component implements the recovery; the green is exhaustive within these bounds and was false twice before (a settled notice left scheduled, delivery of an earlier attempt, an old notice retagged at print), each now a regression with its own control"),
    row(RB, "control: the notice is forced before the attempt starts", Covered, "model_fresh_attempt_r19_tests.rs", "the_notice_is_forced_before_the_attempt_starts"),
    row(RB, "control: a durable receipt settles the request and the attempt-again control is red", Covered, "model_fresh_attempt_r19_tests.rs", "a_durable_receipt_settles_the_request_and_the_control_that_ignores_it_runs_a_second_attempt"),
];

/// Every named test exists in the file named; every missing one is reported together.
#[test]
fn every_covered_or_historical_row_has_its_test() {
    let native: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    let mut missing: Vec<String> = Vec::new();
    for row in TARGET {
        assert!(!row.group.is_empty(), "{}: every row belongs to a group of the target", row.item);
        if row.status == NotYet {
            assert!(row.file.is_empty() && !row.test.is_empty() && row.remains.is_empty(), "{}: a not-yet row names what it needs", row.item);
            continue;
        }
        assert_eq!(row.status == Narrower, !row.remains.is_empty(), "{}: a narrower row says what remains, and no other row does", row.item);
        let place: PathBuf = if row.file.starts_with("commit_") { native.join("commit").join(row.file) } else { native.join("model").join(row.file) };
        match fs::read_to_string(&place) {
            Ok(text) => {
                if !text.contains(&format!("fn {}(", row.test)) {
                    missing.push(format!("{}: no test `{}` in {}", row.item, row.test, row.file));
                }
            }
            Err(error) => missing.push(format!("{}: {}: {}", row.item, place.display(), error)),
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// The counts the branch notes report; changing a row changes them on purpose.
#[test]
fn the_counts_of_the_map_are_the_ones_reported() {
    let count = |status: Status| -> usize { return TARGET.iter().filter(|row| { return row.status == status; }).count(); };
    assert_eq!((count(Covered), count(Narrower), count(Historical), count(NotYet)), (COVERED, NARROWER, HISTORICAL, NOT_YET));
}

/// Items carried by a current test.
pub const COVERED: usize = 64;
/// Items carried by a current test that shows less than the property states.
pub const NARROWER: usize = 17;
/// Items carried only by a historical control.
pub const HISTORICAL: usize = 9;
/// Items with no test yet.
pub const NOT_YET: usize = 17;
