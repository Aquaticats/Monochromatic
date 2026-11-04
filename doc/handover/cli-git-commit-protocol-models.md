# Commit protocol models and components: open work

State of branch `feat/git-policy-cli-commit-protocol-models` after round 13 (the round-12 review findings and
the revision 18 acceptance target). Nothing here is wired into the live wrapper; the reviewers' line stands that
component and model work can continue and that the checkpoint reclamation and quarantine APIs are a foundation
for live integration only after a review accepts them.

## Closed in round 13

Components review 9 (`commit-components-sol-9.txt`), group "Checkpoint selection and cutover" and group "Rule V
departure":

- The replacement authority is no longer caller-asserted. `ClosureEntry::capture` derives the kind from the
  content (an envelope is validated as one whatever the caller declared), and the live requests of a record are
  the envelopes directly under `inbox/` in its closure. Red:
  `p9_a_torn_envelope_captured_as_opaque_is_not_replacement_authority`. Green: same test.
- Later appends cannot escape the dependency check. A record declares its live requests; recovery takes the
  longest prefix whose records are each eligible; an append may drop a live request only when the ledger holds
  its line. Red: `p9_a_later_append_cannot_drop_a_live_request_of_the_checkpoint`. Green: same test.
- Completion cannot delete a dependency it validated and forced: a closure entry inside the state directory is
  refused at begin and at completion. Red and green:
  `p9_completion_cannot_delete_a_dependency_it_validated_and_forced`.
- A mutable dependency has a lifetime contract: `EntryKind::AppendPrefix` identifies a prefix, and immutable
  content is made read-only at completion. Red and green:
  `p9_an_append_to_a_captured_ledger_does_not_invalidate_the_root`.
- Departure validates the promise and not only the presence of an argument. Red and green:
  `p9_a_withdrawal_with_an_unrelated_stop_file_is_refused`,
  `p9_a_terminal_departure_needs_the_terminal_line_of_its_request`,
  `p9_a_departure_forces_the_whole_closure_of_the_state_it_acknowledges`, and (stop failure still attempts the
  other saves) `p9_a_stop_failure_still_attempts_the_remaining_saves`.
- An established root with a lost dependency fails closed:
  `p9_an_established_root_with_a_lost_dependency_fails_closed` (a coverage test; the behaviour already held and
  was not red first).

Models review 7 (`commit-models-sol-7.txt`), group "L1 to L7, L5, X1, service drain" and the register's final
save:

- P1, ordinary successful completion bypassed the final save. The ordinary end of a wait in FOREGROUND and
  COMMITTING is now one of two typed completions: the request was published (nothing is owed) or it reached a
  terminal outcome (its ledger line is forced by the barrier before any normal return). Red:
  `p7_ordinary_completion_of_foreground_and_committing_is_published_or_saved`. Green: same test, plus
  `p7_a_terminal_outcome_that_skips_its_final_save_is_red_and_the_barrier_route_is_green` for the control.
- P2, the shutdown monitor was unreachable. Kill and parent-death are distinct steps in every variant;
  parent-death follows the owner's death whatever the reason; a ghost `departed_by_plan` keeps the attribution
  per incarnation. Red:
  `p7_parent_death_follows_owner_death_whatever_its_reason_and_the_monitor_names_a_planned_exit`. Green and
  mutation: `p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive`.
- P2, `FifteenDropsLive` also removed R2's durability guard. Red:
  `p7_the_drop_control_keeps_the_durability_guard_and_is_red_for_the_drop_alone`. Green: same test, and
  `p1_a_checkpoint_never_erases_a_still_live_request` now asserts the specific diagnostic.

Revision 18 models, built to the acceptance target:

- Effective ordered base (`model/model_support_r18.rs`): the P/Q/A/D schedule is red under revision 17 and green
  under revision 18, with single-defect controls and a sequential reference interpreter. The interpreter's
  reference world is the literal tail and tip; the rules differ only in D's derived decision, which is what the
  model compares.
- Receipt responsibility (`model/model_receipt_r18.rs`): receipt directory as registration, seal before final
  sweep, one-shot flusher with lock and recheck sweep, startup sweep, and L4 dispatching without foreground
  forcing.

## Observations for the design reviewer

- A planned shutdown whose final sweep cannot force (the disk stalls) keeps the lane held, so an unrelated
  command waits for the lane in its acquire entry. The model treats that wait as bounded by the register's own
  deadline and does not fail on it; the rule text does not say it.
- A receipt created after a forcing error ended a flusher is registered and unforced; only the next owner's
  startup sweep covers it, and the error record names the receipts that were in the flusher's snapshot, not this
  one. The model excuses the window as an outside fault until a next owner starts; the notice text may need to
  say it.

- The receipt model has one flusher actor. A client that finds the lock free in the window where the holder has released
  it and will sweep once more is treated as covered by that sweep; two flushers contending for the lock in that window
  are not modeled. This is a named simplification, and the design reviewer may ask for the second actor.
- `LIVE_FORCE_PASSES` in `commit/commit_inbox.rs` bounds how often a departure re-forces the live closure when the
  state grows while it is forced. It turns an endless appender into a named refusal; the value is not measured.
- The sequential reference interpreter of the support model (`reference_landing`) is the single-path definition: D's
  staged version is diffed against the tip D lands on. It is not an execution of the whole request sequence by a second
  implementation; the literal tail and tip are shared by the model and the reference.

## Coverage of the revision 18 acceptance target

Each row names the test that shows the property or control, says that the test shows a narrower control and what it leaves unshown, or says
what it needs. A build test (`model/model_target_tests.rs`) checks that every named test exists. Counts: 70 covered by a current test, 11
covered only by a narrower control (the row says what remains), 9 shown only by a labelled historical control, 17 not yet. Models review 8
(`commit-models-sol-8.txt`, section 2) is the source of the narrower rows; the map checks inventory, not semantic coverage, and the
classification is the author's until a review accepts it.

### L1 to L7, L5, X1, service drain

- Historical only, control: retained-descriptor deadlock (scope never empties):
  `revision_six_retained_descriptor_blocks_the_next_owner` in `model_f2_tests.rs` (a labelled historical
  control, not approval evidence).
- Historical only, control: restart versus hook admission:
  `revision_six_restart_ends_a_hook_dispatched_after_the_observation` in `model_f3_tests.rs` (a labelled
  historical control, not approval evidence).
- Historical only, control: replacement-incarnation targeting:
  `candidate_restarts_only_a_quiet_owner_and_only_the_observed_incarnation` in `model_f3_tests.rs` (a labelled
  historical control, not approval evidence).
- Covered, control: helper parent exit (the one-shot flusher):
  `a_flusher_killed_or_reaped_mid_sync_leaves_its_receipts_to_the_startup_sweep` in
  `model_receipt_r18_tests.rs`.
- Not yet, control: helper parent exit (every other helper: syncer, level-2 helper). Needs: a model of each
  helper with its parent and scope; none of those helpers exists as a component yet.
- Covered, control: hook outlasting an upgrade caller, the drain never ends the hook:
  `a_planned_exit_is_a_drain_that_never_ends_a_protected_hook` in `model_w1_r13_tests.rs`.
- Not yet, control: hook outlasting an upgrade caller, the caller returns within its bound. Needs: a register
  entry for the drain wait (WAIT_DRAIN) and its bound check.
- Covered, property: normal lifecycle never cancels a protected execution (premature drain is red):
  `p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive` in
  `model_w1_r13_tests.rs`.
- Narrower, obligation: helpers retain no lock, gate, client-lock or pipe descriptor:
  `a_lane_lock_the_launcher_releases_is_free_while_the_helper_lives` in `commit_helper_launch_tests.rs`. Not
  shown: the other helpers (the syncer, a level-2 helper) are not components yet, and the manager-started
  transient unit route is not built; the launch shown is the double-forked fallback, with a descriptor inventory
  (a_launched_helper_inherits_no_descriptor_of_its_launcher) and the lock test.
- Narrower, obligation: launch is non-recursive and resource-bounded:
  `a_launched_entry_is_a_function_in_a_new_session_and_not_a_second_start_of_the_program` in
  `commit_helper_launch_tests.rs`. Not shown: refusal by a process-count or memory cap is not injected (the
  launch returns LaunchError::Refused, untested); the transient unit route is not built.

### C0 to C5, S1, C3 repair

- Covered, control: interrupted exposure: `a_torn_or_damaged_inbox_file_is_treated_as_never_written` in
  `commit_inbox_tests.rs`.
- Covered, control: concurrent resend: `two_clients_sending_the_same_bytes_end_with_one_file_and_a_resend` in
  `commit_inbox_tests.rs`.
- Covered, control: marker and state races: `a_mark_that_appears_between_the_samples_repeats_the_capture` in
  `model_capture_r13_tests.rs`.
- Covered, control: failed initial read:
  `an_early_read_failure_never_reaches_the_link_and_a_fresh_acceptance_does` in `commit_inbox_tests.rs`.
- Narrower, control: competing repairers: `p1_a_quarantine_never_moves_a_file_it_did_not_inspect` in
  `commit_inbox_tests.rs`. Not shown: two legitimate repairers contending for authority beyond the one
  non-blocking lock refusal the test asserts; the test replaces the file by raw writes while authority is held
  and so shows stale-object detection.
- Not yet, control: preservation-boundary crashes. Needs: a crash injected at every step of repair and
  quarantine, and the three-file conflict-order example.
- Covered, property: unknown status never authorizes native fallback:
  `a_link_that_finds_the_name_and_cannot_read_it_reports_an_unknown_status` in `commit_inbox_tests.rs`.
- Covered, property: repair preserves inspected content and unsupported formats:
  `repair_is_owner_only_keeps_unsupported_versions_and_forces_the_archive` in `commit_inbox_tests.rs`.

### C1, B1 to B3, Q1

- Covered, control: the P/Q/A/D counterexample: `pqad_is_red_under_seventeen_and_green_under_eighteen` in
  `model_support_r18_tests.rs`.
- Covered, control: repeated supporting-owner removals:
  `repeated_removals_of_supporting_owners_follow_the_fold` in `model_support_r18_tests.rs`.
- Covered, control: replay changing support:
  `a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection` in
  `model_support_r18_tests.rs`.
- Covered, control: checkpoint and recovery between rebuild steps:
  `a_crash_between_a_checkpoint_and_a_rebuild_is_rebuilt_from_the_tail` in `model_support_r18_tests.rs`.
- Covered, control: held requests: `a_held_head_blocks_publication_and_leaves_the_decision_alone` in
  `model_support_r18_tests.rs`.
- Covered, control: disjoint requests: `a_disjoint_request_neither_supports_nor_overrides` in
  `model_support_r18_tests.rs`.
- Covered, control: overlapping requests:
  `an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing` in `model_support_r18_tests.rs`.
- Covered, control: superseded requests: `a_superseding_version_changes_support_only_through_its_own_entry` in
  `model_support_r18_tests.rs`.
- Covered, control: comparison with a sequential reference interpreter:
  `the_reference_world_follows_every_operation_through_its_own_code` in `model_support_r18_tests.rs`.
- Covered, control: the whole request family against the reference (every start, red under revision 17):
  `every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen` in `model_support_r18_tests.rs`.
- Not yet, property: held-version completion cannot affect its successor. Needs: a model step where a
  superseded version's hook completes and a control that lets it publish.
- Not yet, property: retained bytes suffice for recomputation (ledger never compacted). Needs: a component
  that recomputes the augmented delta from the request bytes and the ledger's removals.

### P1 to P4, E1, W1 and W2, recovery classes

- Covered, control: death at each pipe write:
  `no_writer_between_instructions_aborts_the_transaction_unless_the_owner_keeps_the_end` in
  `model_pipe_r13_tests.rs`.
- Covered, control: death at the client-lock handoff:
  `exposure_before_the_client_lock_lets_the_owner_decide_a_live_command_as_clientless` in
  `model_bound_r14_tests.rs`.
- Covered, control: closed-nosubmit takeover:
  `a_client_killed_before_it_wrote_leaves_the_decision_to_the_owner` in `model_w1_r13_tests.rs`.
- Covered, control: a full commit write after takeover then death is class U:
  `a_full_commit_write_then_client_death_is_unknown_and_not_failed` in `model_submission_r15_tests.rs`.
- Covered, control: buffered commit then abort: `a_buffered_commit_is_read_before_end_of_input` in
  `model_pipe_r13_tests.rs`.
- Covered, control: missing diagnostics: `classify_separates_the_negative_inference_from_unresolved` in
  `model_w1_r13_tests.rs`.
- Covered, control: rewritten publication:
  `a_reset_before_the_child_dies_is_unresolved_or_rewritten_never_failed` in `model_w1_r13_tests.rs`.
- Covered, property: only a positive failure authorizes retry:
  `only_a_positive_failure_allows_the_next_attempt` in `model_w1_r13_tests.rs`.
- Covered, property: unresolved departure permits only the submitted attempt:
  `a_query_during_the_unresolved_mark_never_says_nothing_can_land` in `model_w1_r13_tests.rs`.
- Narrower, control: disposable Git experiments (actual FIFO, hook phases, diagnostics, backend):
  `a_fifo_transaction_prepares_holds_through_a_pause_and_commits_on_commit` in `model_git_experiments_tests.rs`.
  Not shown: every supported invocation beyond `update-ref --stdin` with an update (creation, deletion, symbolic
  refs, the side-ref transaction), packed refs, linked worktrees, and Git versions other than the installed
  2.56.0; the neighbouring tests cover abort, end of input, buffered commit, ENXIO, hook phases, a hook refusal
  and two diagnostics on both the files and reftable backends.

### P2c, receipt forcing, L2 and L4, shutdown

- Covered, control: delayed client after the final sweep:
  `a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen` in
  `model_receipt_r18_tests.rs`.
- Covered, control: enqueue rejection by a running service:
  `an_enqueue_rejected_by_a_running_service_falls_back_to_a_flusher` in `model_receipt_r18_tests.rs`.
- Covered, control: a ring to a service that has gone is lost (the delayed client):
  `a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker launch refused by a cap:
  `a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker error (every unforced receipt recorded, joiners included, each retried by the next
  owner): `a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker error on the actual lane files (the joiner is recorded, the forced receipt is not):
  `a_forcing_error_records_the_failed_receipt_and_the_joiner_but_not_the_forced_one` in
  `commit_flusher_tests.rs`.
- Covered, control: worker exit (one-shot, lock released and recheck sweep):
  `a_launch_in_the_release_window_starts_a_second_flusher_that_takes_the_lock` in `model_receipt_r18_tests.rs`.
- Covered, control: worker exit on the actual lock (the recheck sweep forces a receipt created in the release
  window): `a_receipt_created_in_the_release_window_is_forced_by_the_recheck_sweep` in
  `commit_flusher_tests.rs`.
- Narrower, control: reap mid-sync:
  `a_flusher_killed_or_reaped_mid_sync_leaves_its_receipts_to_the_startup_sweep` in
  `model_receipt_r18_tests.rs`. Not shown: the real reap of an incarnation scope by the service (the model
  places the flusher outside or inside the scope as a rule choice).
- Covered, control: replacement startup:
  `a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each` in
  `model_receipt_r18_tests.rs`.
- Covered, control: many outstanding receipts:
  `many_outstanding_receipts_keep_one_flusher_and_the_lock_free_control_runs_two` in
  `model_receipt_r18_tests.rs`.
- Covered, control: the actual flusher lock admits one holder and a second flusher forces nothing:
  `a_second_flusher_finds_the_lock_held_and_forces_nothing` in `commit_flusher_tests.rs`.
- Covered, control: launch in the release window of the flusher lock:
  `a_launch_in_the_release_window_starts_a_second_flusher_that_takes_the_lock` in `model_receipt_r18_tests.rs`.
- Covered, control: receipt forcing paused, unrelated dispatch bounded:
  `unrelated_dispatch_does_not_wait_for_receipt_forcing` in `model_receipt_r18_tests.rs`.
- Covered, property: seal before final sweep: `sealing_after_the_final_sweep_loses_the_delayed_receipt` in
  `model_receipt_r18_tests.rs`.
- Covered, property: outside-fault exits are reported honestly:
  `p1_a_failed_mark_creation_never_returns_normally` in `model_register_r13_tests.rs`.

### Rule V departure and optional level 2

- Covered, control: omitted stop or namespace (one stop cannot stand for two requests; each needs its own):
  `r14_each_departing_request_needs_its_own_stop_and_its_own_adoption` in `commit_inbox_tests.rs`.
- Covered, control: one stop receipt cannot acknowledge two withdrawals:
  `r14_one_stop_receipt_cannot_acknowledge_the_withdrawal_of_two_requests` in `commit_inbox_tests.rs`.
- Covered, control: insufficient acknowledgment (content-bound, still covering after a later append):
  `r14_a_departures_acknowledgment_still_covers_after_a_later_append` in `commit_inbox_tests.rs`.
- Covered, control: terminal departure without its ledger line (a valid ledger of another request):
  `r14_a_valid_ledger_of_another_request_does_not_satisfy_a_terminal_departure` in `commit_inbox_tests.rs`.
- Covered, control: the acknowledged state's whole closure is forced (saved content of the candidate):
  `r14_a_departure_forces_the_saved_content_of_the_candidate_it_acknowledges` in `commit_inbox_tests.rs`.
- Covered, control: stop failure still attempts the remaining saves (a stop error and a saved-content error):
  `r14_every_save_is_attempted_after_a_stop_error_and_a_saved_content_error` in `commit_inbox_tests.rs`.
- Covered, control: ordinary completion cannot stand in for the final save:
  `p7_a_terminal_outcome_that_skips_its_final_save_is_red_and_the_barrier_route_is_green` in
  `model_register_r13_tests.rs`.
- Narrower, property: captured authority valid through completion or verified substitution:
  `r14_an_acknowledgment_survives_completion_and_not_a_substitute_that_drops_its_support` in
  `commit_state_log_tests.rs`. Not shown: survival under a concurrent reclaimer (the reclamation-during-capture
  control is not yet); the older test p9_completion_cannot_delete_a_dependency_it_validated_and_forced shows
  only the state-directory refusal.
- Not yet, control: reclamation during capture. Needs: a model of capture against a concurrent reclaimer.
- Not yet, control: every forcing error. Needs: fault injection at each forcing call of the departure, with
  the outside-fault exit checked.
- Not yet, control: cross-filesystem refs (level 2). Needs: level 2 eligibility checks; level 2 may stay
  disabled.
- Not yet, control: repack overlap (level 2). Needs: level 2 registry observation checks.
- Not yet, control: verification timeout followed by disposal (level 2). Needs: level 2 helper abandonment
  model.
- Not yet, property: exact-commit claims need verified durable closure, hook-free execution, fenced late
  effects. Needs: level 2 models; the level may remain disabled.

### Checkpoint selection and cutover, T0, T1, G1 to G4

- Covered, control: non-drop counterexample: `p1_a_checkpoint_never_erases_a_still_live_request` in
  `model_discharge_r14_tests.rs`.
- Covered, control: non-drop for later appends (a record cannot omit what its predecessor retained):
  `r14_a_record_cannot_omit_what_the_previous_record_retained` in `commit_state_log_tests.rs`.
- Covered, control: retirement discharge forces the ledger prefix that proves it:
  `r14_a_retirement_forces_the_ledger_prefix_that_discharges_it` in `commit_state_log_tests.rs`.
- Covered, control: a later record retaining new saved content is not selected without it:
  `r14_a_later_record_that_retains_new_saved_content_is_not_selected_without_it` in `commit_state_log_tests.rs`.
- Covered, control: drop control isolated from the durability guard:
  `p7_the_drop_control_keeps_the_durability_guard_and_is_red_for_the_drop_alone` in
  `model_discharge_r14_tests.rs`.
- Covered, control: torn dependencies (a prefix entry under the inbox cannot validate a torn envelope):
  `r14_a_prefix_entry_under_the_inbox_cannot_validate_a_torn_envelope` in `commit_state_log_tests.rs`.
- Covered, control: missing dependencies:
  `a_checkpoint_validates_and_forces_its_closure_first_and_refuses_a_missing_entry` in
  `commit_state_log_tests.rs`.
- Covered, control: reordered persistence:
  `a_checkpoint_before_its_live_dependency_is_not_a_root_and_the_predecessor_remains` in
  `model_discharge_r14_tests.rs`.
- Covered, control: candidate activation:
  `p1_a_background_synced_candidate_with_a_lost_dependency_loses_to_its_predecessor` in
  `commit_state_log_tests.rs`.
- Covered, control: predecessor deletion: `a_checkpoint_survives_the_deletion_of_its_predecessor` in
  `model_discharge_r14_tests.rs`.
- Covered, control: mutable dependency (an existing ledger writer after capture):
  `r14_an_existing_ledger_writer_cannot_invalidate_a_root_that_captured_the_ledger` in
  `commit_state_log_tests.rs`.
- Covered, property: maximum-generation conflicts are rejected:
  `two_documents_at_one_generation_are_a_conflict_not_a_root` in `commit_state_log_tests.rs`.
- Narrower, property: damaged established roots fail closed:
  `p9_an_established_root_with_a_lost_dependency_fails_closed` in `commit_state_log_tests.rs`. Not shown:
  telling an established damaged root from an incomplete candidate when older segments are usable; selection
  records no establishment, and the test has no other usable segment.
- Historical only, control: registry admission:
  `nine_withdraws_a_release_when_a_run_appears_and_loses_nothing_otherwise` in `model_f8_tests.rs` (a labelled
  historical control, not approval evidence).
- Historical only, control: check and release races:
  `eight_releases_after_a_run_was_admitted_between_the_observation_and_the_release` in `model_f8_tests.rs` (a
  labelled historical control, not approval evidence).
- Historical only, control: cross-worktree maintenance: `revision_six_other_worktree_releases_during_a_prune`
  in `model_f8_tests.rs` (a labelled historical control, not approval evidence).
- Historical only, property: maintenance cannot outlive its recovery protection:
  `ten_roots_the_closure_until_the_release_is_done` in `model_f8_tests.rs` (a labelled historical control, not
  approval evidence).
- Not yet, property: saved content survives until authorized disposal. Needs: a disposal model: the disposed
  ledger line durable before the saved content is deleted.
- Narrower, control: push exclusion and frozen updates against actual Git:
  `a_negative_refspec_keeps_the_internal_namespace_off_the_remote_and_a_named_object_is_pushed_as_given` in
  `model_git_experiments_tests.rs`. Not shown: the marker the lane acknowledges and the stacked push path; the
  test pushes to a local bare remote with plain `git push`.

### M1 to M3, MG1 to MG3, MG6c

- Covered, control: concurrent staging: `a_later_staging_is_replaced_under_twelve_and_left_under_thirteen` in
  `model_capture_r13_tests.rs`.
- Covered, control: conditional installation keeps displaced versions:
  `overlapping_staged_and_worktree_versions_are_all_kept` in `model_merge_r14_tests.rs`.
- Covered, control: no foreign lock removal: `a_lock_without_a_live_creator_is_never_removed` in
  `model_m1_split_r13_tests.rs`.
- Covered, control: interrupted installation and reboot:
  `a_cleanup_begun_before_the_span_ends_finishes_after_it_and_no_new_one_starts` in `model_watch_r13_tests.rs`.
- Historical only, control: HEAD changes: `revision_seven_first_try_path_stages_on_the_switched_branch` in
  `model_f5_tests.rs` (a labelled historical control, not approval evidence).
- Historical only, control: crash at every carry step: `nine_recovers_every_kill_point_by_identity` in
  `model_m1_crash_tests.rs` (a labelled historical control, not approval evidence).
- Not yet, control: open-descriptor writes. Needs: a model of a writer holding the file open across the
  install.
- Not yet, control: atomic saves. Needs: a model of rename-over saves against the install.
- Not yet, control: oversized and raced reads. Needs: inline-bound checks on actual reads and whole-index
  work.
- Not yet, control: expiry between clock check and marker (a resumed install does not restart the window).
  Needs: the watch model with the lock before expiry and the marker after it.

### Rule B, client results, notices and costs

- Narrower, control: whole-command timing and sync instrumentation:
  `the_waiting_path_of_a_plain_commit_makes_no_sync` in `commit_inbox_tests.rs`. Not shown: whole-process
  latency and every sync the wrapper makes; the test counts the inbox waiting path only.
- Covered, control: notice dismissal keeps saved copies:
  `dismissing_the_reminder_deletes_no_saved_copy_and_the_restore_command_still_works` in
  `model_watch_r13_tests.rs`.
- Not yet, control: no-manager preparation timeout. Needs: a register entry and fixture for the no-manager
  preparation wait.
- Narrower, control: recovery conflict, refusal and signing failure notices: `the_notice_follows_the_outcome`
  in `model_fresh_attempt_r19_tests.rs`. Not shown: the notice texts of the conflict, refusal and
  signing-failure outcomes; the model checks that the stored notice follows the outcome of the fresh attempt.
- Narrower, control: notice crash and reclamation:
  `recovery_data_is_kept_until_the_outcome_notice_is_delivered` in `model_fresh_attempt_r19_tests.rs`. Not
  shown: crash points of every notice kind; the model covers the fresh-attempt notice only (its second-crash and
  printed-then-crash controls are the neighbouring tests).
- Covered, property: decision 19, fresh attempt with a stored notice when no decisive positive authority
  survives: `decision_nineteen_is_green_over_every_schedule` in `model_fresh_attempt_r19_tests.rs`.
- Covered, control: the notice is forced before the attempt starts:
  `the_notice_is_forced_before_the_attempt_starts` in `model_fresh_attempt_r19_tests.rs`.
- Covered, control: a durable receipt settles the request and the attempt-again control is red:
  `a_durable_receipt_settles_the_request_and_the_control_that_ignores_it_runs_a_second_attempt` in
  `model_fresh_attempt_r19_tests.rs`.

## Still queued (P2 and P3 from the earlier reviews)

Components review 8 (`commit-components-sol-8.txt`):

- 3.1 P2: conflict detection depends on scan order (the three-file conflict-order example).
- 3.2 P2: `StatusUnknown` does not cover initial-read ambiguity beyond the cases tested.
- 3.3 P2: quarantine accepts paths outside the inbox (the identity check is in; the review's remaining cases are
  not).
- 3.4 P2: the departure anchor does not cover inbox or state ancestry.
- 3.5 P2: selector and reclaimer, and prefix-lifetime, races.
- 3.6 P2 and P3: remaining namespace and cost problems.
- 3.7: departure ordering and foreground forcing beyond the stop failure case.
- Section 4: the missing behavioural reds, and the working-directory test that does not reliably test the fixed
  bug.

Components review 9 (`commit-components-sol-9.txt`): everything it reported as P1 is closed above; its P2 and P3
items are not yet mapped.

Models review 6a, 6b and 7: the P2 and P3 items other than those closed above remain (the vacuity audit of the
no-timeout test, the acceptance manifest checking a configuration prerequisite, the watch excluding
unauthorized-overwrite schedules, the bound model's asserted departure barrier, and the buffered-commit schedule
that the shutdown green does not assert it reaches).

## Round 14 state

Round 14 follows the review in `commit-models-sol-8.txt` (section 4 is the path). All six items of the path are worked; the review
itself has not run on them. Nothing here is wired into the live wrapper.

Done:

- Item 1 and item 2 (components), commit `ecc36c5216`: complete document-bound retained sets, path-derived kinds, content-addressed
  saved content, ledger prefix as retirement proof, one stop per request, whole retained state forced, every save attempted after errors,
  content-bound acknowledgment (`state_covers`, `Acknowledgment::still_covers`). Red tests have the `r14_` prefix.
- Item 3 (models): support model with stale, rebuild, checkpoint and crash steps and a separately written reference interpreter; receipt
  model with two flusher slots, enqueue rejection, per-receipt error recording, kill and reap controls, startup retry of recorded errors.
- Item 4 (map): `TARGET` rewritten row by row from the review's section 2 with a `Narrower` status that carries what the test leaves
  unshown; renamed tests fixed; the question 8 row replaced by the decision 19 rows. The map test was red on the renamed names
  (`/var/home/user/temp/agent/r14/target1.log` and `target2.log`) and is green (`target3.log`, and `target4.log` after the item 5 rows). The coverage section above is
  generated from the map by `/var/home/user/temp/agent/scripts/gen-notes-r14.py`.
- Item 5 (current evidence):
  - `commit/commit_helper_launch.rs` (two forks, `setsid`, standard streams on `/dev/null`, `close_range` with a `/proc/self/fd`
    fallback, a bounded wait for the first child) and `commit/commit_flusher.rs` (the internal flush entry point under
    `lane/flusher.lock`, recheck sweep, `receipt-errors` record). Written against stubs first: with an inline launch and a lock that
    always succeeds, 7 of the 10 new component tests were red (`a_launched_helper_inherits_no_descriptor_of_its_launcher`,
    `a_lane_lock_the_launcher_releases_is_free_while_the_helper_lives`,
    `a_launched_entry_is_a_function_in_a_new_session_and_not_a_second_start_of_the_program`,
    `the_flusher_lock_admits_one_holder_at_a_time`, `a_second_flusher_finds_the_lock_held_and_forces_nothing`,
    `a_receipt_created_in_the_release_window_is_forced_by_the_recheck_sweep`,
    `a_recheck_that_finds_work_and_the_lock_taken_hands_over`) and green with the real code. The three error-record and healthy-lane tests
    passed under the stubs because the stubs did not change that behaviour; they were not red first. A one-off control that left out
    `close_range` made the lock and descriptor tests red again.
  - `model/model_git_experiments_tests.rs`: real Git 2.56.0, files and reftable backends, FIFO `update-ref --stdin` built as the pipe
    bootstrap specifies, hook phases, diagnostics, push exclusion. Two tests were red first because the author's assumption was wrong
    (Git runs the hook in `preparing` before `prepared`; the design already says so at the P2 text); the rest recorded Git's behaviour
    and passed on first run.
- Item 6: `model/model_fresh_attempt_r19.rs` and its tests (decision 19), passing; the map rows name them.

Gates at the tip: the full `native:test:model` run is 411 passed, 0 failed (`/var/home/user/temp/agent/r14/full-model2.log`); both
`native:clippy` and `native:clippy:seams` pass; the `source_guard` tests of the library pass (39 of 39).

Findings and limits for the reviewer (none adds a row):

- The launch component implements only the double-forked fallback; the manager-started transient unit route is not built, so the
  "helpers" obligations are `Narrower`. The fork-refused path (`LaunchError::Refused`) is not injected.
- `flusher.lock` is an exclusive `flock` through `File::try_lock`, taken without waiting. The flusher records a forcing error for every
  receipt not yet forced, and forces the receipt directory only after a sweep that forced files.
- The helper runs Rust code after a bare `fork` (no `exec`). The design allows the fork-only fallback ("a double-forked child in its own
  session"), which is how it stays non-recursive; the safety argument is that the wrapper's launching process is single-threaded at that
  point, which the component cannot enforce.
- The Git experiments cover one Git version and two backends only; packed refs, linked worktrees and the side-ref transaction are listed
  as unshown in the P1 row.

Left:

- The review of these inputs (the coordinator launches it).
- The `NotYet` rows of the map, which need components or models that do not exist yet (see the coverage section).
- The queued P2 and P3 items listed under "Still queued".
