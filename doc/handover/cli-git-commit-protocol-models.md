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

## Coverage of the revision 18 acceptance target

Each row names the test that shows the property or control, or says what it needs. A build test
(`model/model_target_tests.rs`) checks that every named test exists. Counts: 62 covered by a current test, 9
shown only by a labelled historical control, 25 not yet.

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
  `a_flusher_in_the_owners_scope_is_ended_by_the_owner_exit` in `model_receipt_r18_tests.rs`.
- Not yet, control: helper parent exit (every other helper: syncer, level-2 helper). Needs: a model of each
  helper with its parent and scope; none of those helpers exists as a component yet.
- Covered, control: hook outlasting an upgrade caller, the drain never ends the hook:
  `a_planned_exit_is_a_drain_that_never_ends_a_protected_hook` in `model_w1_r13_tests.rs`.
- Not yet, control: hook outlasting an upgrade caller, the caller returns within its bound. Needs: a register
  entry for the drain wait (WAIT_DRAIN) and its bound check.
- Covered, property: normal lifecycle never cancels a protected execution (premature drain is red):
  `p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive` in
  `model_w1_r13_tests.rs`.
- Not yet, obligation: helpers retain no lock, gate, client-lock or pipe descriptor. Needs: the internal flush
  entry point and a descriptor-inventory test of the launched process.
- Not yet, obligation: launch is non-recursive and resource-bounded. Needs: the internal flush entry point and a
  test that the launch never uses the ordinary command line.

### C0 to C5, S1, C3 repair

- Covered, control: interrupted exposure: `a_torn_or_damaged_inbox_file_is_treated_as_never_written` in
  `commit_inbox_tests.rs`.
- Covered, control: concurrent resend: `two_clients_sending_the_same_bytes_end_with_one_file_and_a_resend` in
  `commit_inbox_tests.rs`.
- Covered, control: marker and state races: `a_mark_that_appears_between_the_samples_repeats_the_capture` in
  `model_capture_r13_tests.rs`.
- Covered, control: failed initial read:
  `an_early_read_failure_never_reaches_the_link_and_a_fresh_acceptance_does` in `commit_inbox_tests.rs`.
- Covered, control: competing repairers: `p1_a_quarantine_never_moves_a_file_it_did_not_inspect` in
  `commit_inbox_tests.rs`.
- Not yet, control: preservation-boundary crashes. Needs: a crash injected at every step of repair and
  quarantine, and the three-file conflict-order example.
- Covered, property: unknown status never authorizes native fallback:
  `a_link_that_finds_the_name_and_cannot_read_it_reports_an_unknown_status` in `commit_inbox_tests.rs`.
- Covered, property: repair preserves inspected content and unsupported formats:
  `repair_is_owner_only_keeps_unsupported_versions_and_forces_the_archive` in `commit_inbox_tests.rs`.

### C1, B1 to B3, Q1

- Covered, control: the P/Q/A/D counterexample: `pqad_is_red_under_seventeen_and_green_under_eighteen` in
  `model_support_r18_tests.rs`.
- Covered, control: repeated supporting-owner removals: `repeated_removals_of_supporting_owners_follow_the_fold`
  in `model_support_r18_tests.rs`.
- Covered, control: replay changing support:
  `a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection` in
  `model_support_r18_tests.rs`.
- Covered, control: checkpoint and recovery between rebuild steps:
  `recovery_between_rebuild_steps_rebuilds_the_same_decision` in `model_support_r18_tests.rs`.
- Covered, control: held requests: `every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen`
  in `model_support_r18_tests.rs`.
- Covered, control: disjoint requests: `a_disjoint_request_neither_supports_nor_overrides` in
  `model_support_r18_tests.rs`.
- Covered, control: overlapping requests:
  `an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing` in `model_support_r18_tests.rs`.
- Covered, control: superseded requests:
  `every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen` in `model_support_r18_tests.rs`.
- Covered, control: comparison with a sequential reference interpreter:
  `pqad_is_red_under_seventeen_and_green_under_eighteen` in `model_support_r18_tests.rs`.
- Not yet, property: held-version completion cannot affect its successor. Needs: a model step where a superseded
  version's hook completes and a control that lets it publish.
- Not yet, property: retained bytes suffice for recomputation (ledger never compacted). Needs: a component that
  recomputes the augmented delta from the request bytes and the ledger's removals.

### P1 to P4, E1, W1 and W2, recovery classes

- Covered, control: death at each pipe write:
  `no_writer_between_instructions_aborts_the_transaction_unless_the_owner_keeps_the_end` in
  `model_pipe_r13_tests.rs`.
- Covered, control: death at the client-lock handoff:
  `exposure_before_the_client_lock_lets_the_owner_decide_a_live_command_as_clientless` in
  `model_bound_r14_tests.rs`.
- Covered, control: closed-nosubmit takeover: `a_client_killed_before_it_wrote_leaves_the_decision_to_the_owner`
  in `model_w1_r13_tests.rs`.
- Covered, control: a full commit write after takeover then death is class U:
  `a_full_commit_write_then_client_death_is_unknown_and_not_failed` in `model_submission_r15_tests.rs`.
- Covered, control: buffered commit then abort: `a_buffered_commit_is_read_before_end_of_input` in
  `model_pipe_r13_tests.rs`.
- Covered, control: missing diagnostics: `classify_separates_the_negative_inference_from_unresolved` in
  `model_w1_r13_tests.rs`.
- Covered, control: rewritten publication:
  `a_reset_before_the_child_dies_is_unresolved_or_rewritten_never_failed` in `model_w1_r13_tests.rs`.
- Covered, property: only a positive failure authorizes retry: `only_a_positive_failure_allows_the_next_attempt`
  in `model_w1_r13_tests.rs`.
- Covered, property: unresolved departure permits only the submitted attempt:
  `a_query_during_the_unresolved_mark_never_says_nothing_can_land` in `model_w1_r13_tests.rs`.
- Not yet, control: disposable Git experiments (actual FIFO, hook phases, diagnostics, backend). Needs: fixtures
  that run the installed Git for every supported invocation; the process runner exists, the experiments do not.

### P2c, receipt forcing, L2 and L4, shutdown

- Covered, control: delayed client after the final sweep:
  `a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen` in
  `model_receipt_r18_tests.rs`.
- Covered, control: enqueue rejection (a ring to a service that has gone):
  `a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker launch refused by a cap:
  `a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker error: `the_next_owner_sweeps_at_startup_and_retries_a_recorded_error` in
  `model_receipt_r18_tests.rs`.
- Covered, control: worker exit (one-shot, lock released and recheck sweep):
  `a_flusher_that_does_not_sweep_once_more_after_releasing_its_lock_loses_a_wakeup` in
  `model_receipt_r18_tests.rs`.
- Not yet, control: reap mid-sync. Needs: an L2 reap step for a flusher in its forcing phase, in a model that
  has the incarnation scope.
- Covered, control: replacement startup: `the_next_owner_sweeps_at_startup_and_retries_a_recorded_error` in
  `model_receipt_r18_tests.rs`.
- Covered, control: many outstanding receipts:
  `many_outstanding_receipts_start_one_flusher_and_the_lock_free_control_starts_more` in
  `model_receipt_r18_tests.rs`.
- Covered, control: receipt forcing paused, unrelated dispatch bounded:
  `unrelated_dispatch_does_not_wait_for_receipt_forcing` in `model_receipt_r18_tests.rs`.
- Covered, property: seal before final sweep: `sealing_after_the_final_sweep_loses_the_delayed_receipt` in
  `model_receipt_r18_tests.rs`.
- Covered, property: outside-fault exits are reported honestly:
  `p1_a_failed_mark_creation_never_returns_normally` in `model_register_r13_tests.rs`.

### Rule V departure and optional level 2

- Covered, control: omitted stop or namespace: `p9_a_withdrawal_with_an_unrelated_stop_file_is_refused` in
  `commit_inbox_tests.rs`.
- Covered, control: insufficient acknowledgment:
  `p1_a_departure_for_a_withdrawal_forces_the_mark_and_cannot_omit_it` in `commit_inbox_tests.rs`.
- Covered, control: terminal departure without its ledger line:
  `p9_a_terminal_departure_needs_the_terminal_line_of_its_request` in `commit_inbox_tests.rs`.
- Covered, control: the acknowledged state's whole closure is forced:
  `p9_a_departure_forces_the_whole_closure_of_the_state_it_acknowledges` in `commit_inbox_tests.rs`.
- Covered, control: stop failure still attempts the remaining saves:
  `p9_a_stop_failure_still_attempts_the_remaining_saves` in `commit_inbox_tests.rs`.
- Covered, control: ordinary completion cannot stand in for the final save:
  `p7_a_terminal_outcome_that_skips_its_final_save_is_red_and_the_barrier_route_is_green` in
  `model_register_r13_tests.rs`.
- Covered, property: captured authority valid through completion or verified substitution:
  `p9_completion_cannot_delete_a_dependency_it_validated_and_forced` in `commit_state_log_tests.rs`.
- Not yet, control: reclamation during capture. Needs: a model of capture against a concurrent reclaimer.
- Not yet, control: every forcing error. Needs: fault injection at each forcing call of the departure, with the
  outside-fault exit checked.
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
- Covered, control: non-drop for later appends: `p9_a_later_append_cannot_drop_a_live_request_of_the_checkpoint`
  in `commit_state_log_tests.rs`.
- Covered, control: drop control isolated from the durability guard:
  `p7_the_drop_control_keeps_the_durability_guard_and_is_red_for_the_drop_alone` in
  `model_discharge_r14_tests.rs`.
- Covered, control: torn dependencies: `p9_a_torn_envelope_captured_as_opaque_is_not_replacement_authority` in
  `commit_state_log_tests.rs`.
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
- Covered, control: mutable dependency (an appended ledger):
  `p9_an_append_to_a_captured_ledger_does_not_invalidate_the_root` in `commit_state_log_tests.rs`.
- Covered, property: maximum-generation conflicts are rejected:
  `two_documents_at_one_generation_are_a_conflict_not_a_root` in `commit_state_log_tests.rs`.
- Covered, property: damaged established roots fail closed:
  `p9_an_established_root_with_a_lost_dependency_fails_closed` in `commit_state_log_tests.rs`.
- Historical only, control: registry admission:
  `nine_withdraws_a_release_when_a_run_appears_and_loses_nothing_otherwise` in `model_f8_tests.rs` (a labelled
  historical control, not approval evidence).
- Historical only, control: check and release races:
  `eight_releases_after_a_run_was_admitted_between_the_observation_and_the_release` in `model_f8_tests.rs` (a
  labelled historical control, not approval evidence).
- Historical only, control: cross-worktree maintenance: `revision_six_other_worktree_releases_during_a_prune` in
  `model_f8_tests.rs` (a labelled historical control, not approval evidence).
- Historical only, property: maintenance cannot outlive its recovery protection:
  `ten_roots_the_closure_until_the_release_is_done` in `model_f8_tests.rs` (a labelled historical control, not
  approval evidence).
- Not yet, property: saved content survives until authorized disposal. Needs: a disposal model: the disposed
  ledger line durable before the saved content is deleted.
- Not yet, control: push exclusion and frozen updates against actual Git. Needs: disposable Git experiments.

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
- Not yet, control: open-descriptor writes. Needs: a model of a writer holding the file open across the install.
- Not yet, control: atomic saves. Needs: a model of rename-over saves against the install.
- Not yet, control: oversized and raced reads. Needs: inline-bound checks on actual reads and whole-index work.
- Not yet, control: expiry between clock check and marker (a resumed install does not restart the window).
  Needs: the watch model with the lock before expiry and the marker after it.

### Rule B, client results, notices and costs

- Covered, control: whole-command timing and sync instrumentation (plain commit makes no sync):
  `the_waiting_path_of_a_plain_commit_makes_no_sync` in `commit_inbox_tests.rs`.
- Covered, control: notice dismissal keeps saved copies:
  `dismissing_the_reminder_deletes_no_saved_copy_and_the_restore_command_still_works` in
  `model_watch_r13_tests.rs`.
- Not yet, control: no-manager preparation timeout. Needs: a register entry and fixture for the no-manager
  preparation wait.
- Not yet, control: recovery conflict, refusal and signing failure notices. Needs: notice models per outcome.
- Not yet, control: notice crash and reclamation. Needs: notice delivery state with crash points.
- Not yet, property: question 8 recovery policy (open owner choice, no green by adopting the recommendation).
  Needs: the owner's answer on corrected options.

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
