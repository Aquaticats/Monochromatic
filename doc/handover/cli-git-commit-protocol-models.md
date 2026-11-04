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
what it needs. A build test (`model/model_target_tests.rs`) checks that every named test exists. Counts: 64 covered by a current test, 17
covered only by a narrower control (the row says what remains), 9 shown only by a labelled historical control, 18 not yet. Models review 8
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
  (a_launched_helper_inherits_no_descriptor_of_its_launcher) and the lock test; the flusher helper takes the
  lock itself after its setup and the client takes no lock and looks at none (a probe held the lock without
  listing anything, and a client or helper that saw it held took that for an executor, review 14; looking
  without taking is impossible with flock), so every holder of the lock is a flusher (production acquisition
  sites, each followed by a locked sweep: the recheck's reacquisition, flush_lane_with, the helper's prepare;
  test fixtures that hold it without sweeping are not production holders). The argument for a helper that finds
  it held: its client's receipt was visible before the failed acquisition, so the holder's next locked sweep
  lists it, or, if that sweep is past, the holder's release and recheck lists it, takes the lock again or finds
  another flusher, and accounts for a forcing failure
  (a_receipt_exposed_after_the_last_empty_sweep_is_covered_by_the_holders_release_and_recheck: the holder
  acquired before the receipt existed). A launcher that dies, is slow or cannot reap its intermediate child
  leaves no copy and no false holder. The removed probe is reproduced by a test seam for negative controls that
  show the receipts lost, every coordination step asserted
  (the_removed_probe_loses_a_receipt_when_its_owner_dies_before_launching,
  the_removed_probe_loses_the_receipts_of_a_helper_that_lost_to_it_and_of_a_client_that_saw_it,
  the_removed_probe_makes_a_recheck_hand_a_receipt_over_to_a_holder_that_never_lists), and the fixed
  counterparts assert the receipts recorded
  (the_fixed_launch_gives_a_client_an_executor_whatever_another_client_does,
  the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock); the earlier round 20 controls did not
  establish their interleavings and are superseded; two limits of the controls: the negative controls observe
  the absence of a record for 1.5 s only (an executor acts within milliseconds, which is observed and not
  proven), and the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock does not force a failed
  recheck acquisition (whether the recheck wins or loses the lock depends on scheduling), so it is not
  deterministic evidence of that branch, which
  a_second_executor_pinned_across_the_recheck_decision_carries_the_receipt pins; the other controls (
  (a_slow_launcher_that_then_dies_leaves_every_receipt_with_an_executor,
  a_client_that_arrives_after_a_launch_that_could_not_reap_its_intermediate_child_has_an_executor,
  a_launcher_that_dies_right_after_the_helper_is_ready_leaves_no_receipt_without_an_executor,
  a_client_that_arrives_after_a_failed_launch_finds_the_lock_free_and_has_an_executor); the helper reports the
  result of taking the lock (a_helper_that_cannot_take_the_lock_is_a_failed_launch_with_every_receipt_recorded);
  the lock and the pipes are above the standard streams and every descriptor has an owner on every path, driven
  through EMFILE (a_launch_with_descriptor_zero_closed_leaves_the_helper_holding_the_lock,
  a_launch_whose_descriptor_normalization_fails_leaks_no_descriptor,
  a_lock_acquisition_whose_descriptor_move_fails_leaks_no_descriptor); a descriptor a raw creator keeps outside
  this component is not visible to it.
- Narrower, obligation: launch is non-recursive and resource-bounded:
  `a_launched_entry_is_a_function_in_a_new_session_and_not_a_second_start_of_the_program` in
  `commit_helper_launch_tests.rs`. Not shown: OWNER OVERRIDE of the resource-bounded part, 2026-11-04, question
  tool, choice \"Keep built, state limit (Recommended)\": keep the helper that releases the lock, rechecks for
  new saves and exits, and record that the helper count is bounded only by how many commits arrive during those
  few milliseconds; the owner's reason: no save lost in any modeled case, no added commit time, no new lock or
  process. The limit, stated plainly: the number of live helpers is arrival-dependent and has no time bound (the
  owner's phrase 'those few milliseconds' is not a duration the code gives, and scheduling can stretch the
  windows): every client that needs the flusher path forks a helper whatever the lock's state (measured,
  descriptive and not a bound: from the launch_flusher call to the helper's acknowledgment, median 0.66 ms,
  maximum 14.2 ms over 20 launches in one run), the first to take the lock works, a helper that finds it taken
  ends but a delayed one lives until it runs, and the helpers in the release, recheck and exit step hold nothing
  and live until they finish; what is bounded is the number that hold the lock (one). Each launch also has an
  intermediate child, which exits at once after its second fork and that the launcher waits for at most the
  report bound, then kills and waits for at most the kill-reap bound; a child that is not reaped then fails the
  launch (IntermediateNotReaped), with no promise that it is reaped or of how long a residual child lives, and
  it holds nothing; this is a different process from the detached worker, which lives until its flush ends.
  LIVE-CUTOVER OBLIGATION: see the not-yet row of the cutover group. Protocols 2 to 5 of the comparison (the
  singleton retry owner, inline forcing while sealed, inline forcing whenever the token is held, the baton) are
  not built; the comparison (model_helper_bound_r20_tests.rs, handover rounds 18 and 19) stays as recorded
  evidence with its claims corrected (its crash exemption excuses every visible receipt until replacement, its
  maxima are lower bounds where a search stopped at a miss, its foreground count is a number and not a time, and
  it omits client death during inline forcing, worker death and partial forcing, launcher timeout and death,
  mixed-version helpers, enqueue rejection, non-atomic transfers and blocking time). The non-recursive part
  stands: the launch is a function in a new session and not a second start of the program. The token kept to
  exit with the running syncer as the only retry misses a receipt
  (a_token_held_to_exit_misses_a_receipt_when_no_owner_runs_to_retry_it,
  a_token_held_to_exit_still_misses_a_receipt_when_the_client_cannot_start_the_owner_during_the_shutdown). Also
  open: the transient unit route is not built; a refused fork is injected by a real process limit and a memory
  cap is not; a process with more than one thread is refused, shown on test copies of a process and not at real
  call sites, which do not exist yet; a launch is bounded by the report, the intermediate child's exit and the
  acknowledgment, plus the bounded reap of a killed child; a launch that fails records every receipt of the
  lane.

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
- Narrower, control: checkpoint and recovery between rebuild steps:
  `a_crash_between_a_checkpoint_and_a_rebuild_is_rebuilt_from_the_tail` in `model_support_r18_tests.rs`. Not
  shown: persistence of the retained inputs and a complete checkpoint protocol: the model checks derived-cache
  recovery assuming the current tip and tail survive every modeled crash unchanged, and a checkpoint stores one
  decision bit (retained-byte sufficiency is the NotYet row below).
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
- Covered, control: the whole request family against the reference (all two-entry combinations and four
  selected three-entry starts, tip fixed at zero, selections 1 or 2; red under revision 17):
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
- Covered, control: disposable Git experiments (actual FIFO, hook phases, diagnostics, backend, packed refs,
  linked worktrees, the side-ref transaction, creation, deletion and symbolic-ref updates, the client-lock and
  submission handoffs, the order of the ref and the answer, a kill at each persistence boundary), Git 2.56.0,
  files and reftable, a local file system:
  `a_fifo_transaction_prepares_holds_through_a_pause_and_commits_on_commit` in `model_git_experiments_tests.rs`.

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
- Narrower, control: worker error (every unforced receipt recorded, joiners included, each retried by the next
  owner): `a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each` in
  `model_receipt_r18_tests.rs`. Not shown: the error path is now four observable phases (list, record, release,
  list again; a_receipt_that_joins_between_the_error_listing_and_the_release_is_recorded_by_the_second_listing
  shows the joiner), with one forcing error per run and one erroring flusher; a failure of the recording itself,
  repeated errors and the real error file are not modeled here (the file is shown by the component tests); a
  record written after the last startup sweep while an owner runs is retried by that owner's syncer (coordinator
  pick of round 15; a_running_owner_retries_an_error_recorded_after_its_startup_sweep, red for the control whose
  syncer never retries).
- Covered, control: worker error on the actual lane files (every receipt whose name is not forced is recorded,
  the joiner too, also one that joins after the listing):
  `a_receipt_that_joins_after_the_error_listing_is_recorded` in `commit_flusher_window_tests.rs`.
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
  `commit_state_log_tests.rs`. Not shown: survival under a concurrent reclaimer on the component (it is modeled in
  `model_capture_reclaim_r23_tests.rs`); a checkpoint covers an earlier acknowledgment only when its document is shown to contain
  the state that acknowledgment promised: both are documents in the checked representation (LaneDocument) and
  the new one names every request, saved content, decision mark and ledger and order extent of the old or
  discharges it (r17_a_checked_document_reclaims_and_covers_and_anything_unchecked_is_refused_for_reclamation,
  r23_a_mark_is_retained_until_its_request_is_discharged_by_the_ledger); the marker, logical head and tail,
  request states and consumption map are not containment obligations, because no file lies behind them and the
  newer generation supersedes them
  (r23_a_changed_state_still_reclaims_and_covers_because_state_has_no_file_behind_it); the containment lines for
  marks and the order extent are defense in depth that the non-drop check already implies, and no test fails
  without them; an opaque
  document is never shown to contain anything, so an opaque checkpoint reclaims no segment and records no
  lineage (r17_a_compaction_of_a_later_opaque_record_does_not_reclaim_or_cover_an_earlier_acknowledgment,
  r16_a_checkpoint_that_changes_the_document_does_not_cover_an_earlier_acknowledgment,
  r15_an_unrelated_document_that_retains_the_same_items_does_not_cover_the_acknowledgment); one hop only, so an
  acknowledgment older than the previous checkpoint is not covered; the older test
  p9_completion_cannot_delete_a_dependency_it_validated_and_forced shows only the state-directory refusal.
- Narrower, control: reclamation during capture:
  `a_reclaimer_that_removes_the_segment_for_an_unrelated_candidate_loses_the_promise` in
  `model_capture_reclaim_r23_tests.rs`. Not shown: the model abstracts the log to one promised record, one candidate
  and the old segment's name, with one power loss; it shows the rule (the departure returns on a durable cover, the
  reclaimer removes the old segment only for a durable candidate whose lineage names the record:
  a_reclaimer_that_removes_the_segment_before_the_candidate_is_durable_loses_the_promise_in_a_power_loss,
  a_departure_that_returns_on_a_visible_cover_loses_the_promise_in_a_power_loss); the departure's own forcing and
  its durable-cover check are redundant guards (either alone keeps the green; removed together they are the
  visible-cover control); a component test of a real departure against a real reclaimer needs the departure at a
  real call site (plan step 2).
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
- Narrower, control: non-drop for later appends (a record cannot omit what its predecessor retained):
  `r14_a_record_cannot_omit_what_the_previous_record_retained` in `commit_state_log_tests.rs`. Not shown:
  complete closure: only a document in the checked representation (LaneDocument: the references requests, saved
  content, decision marks and the ledger and order extents, and the state fields with no file behind them) can
  be checkpointed with reclamation, its closure must equal the references the document names and hold everything
  the lane holds, marks and the order file included
  (r23_a_closure_lacking_a_decision_mark_the_lane_holds_is_refused,
  r23_a_closure_shorter_than_the_order_file_the_lane_holds_is_refused,
  r16_a_reclaiming_checkpoint_must_retain_what_the_lane_holds_whatever_its_document_declares,
  r17_a_checked_document_reclaims_and_covers_and_anything_unchecked_is_refused_for_reclamation); a document the
  component cannot read, one that depends on a file outside the representation such as a journal file, is opaque
  and its checkpoints never reclaim
  (r17_an_opaque_document_that_needs_a_file_outside_the_derived_places_never_reclaims_its_predecessor), and a
  checkpoint that compacts an opaque append reclaims nothing
  (r17_a_checkpoint_cannot_compact_an_opaque_append_and_reclaim_its_segment); the real state document is
  expressed in the representation (owner decision 21, 'Checked format';
  r23_a_full_real_state_document_round_trips_and_only_the_canonical_bytes_parse,
  r23_a_request_position_beyond_the_order_extent_is_not_a_document) and a reclaiming checkpoint can retire
  (begin_checkpoint_reclaiming_retiring); not shown: any caller, since no code writes the document at a real
  call site yet (plan step 2), the spool and the attempt directories of a request, which the log never deletes
  and the document does not retain, and the protocol's rule for when a transition may drop a consumption entry
  or change a phase, which is the models' business and not the log's; needs lines
  (r15_a_document_that_names_a_dependency_cannot_be_checkpointed_or_appended_without_it) remain a check for
  opaque records; the references of a checked document are validated against the retained set at append
  admission, begin and recovery
  (r18_a_checked_document_that_names_a_dependency_the_set_lacks_is_neither_admitted_nor_selected,
  r23_a_document_naming_an_order_extent_its_set_lacks_is_not_admitted,
  r23_a_mark_with_content_is_not_a_mark_and_a_named_mark_must_be_in_the_set).
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
- Narrower, control: mutable dependency (an existing ledger writer after capture):
  `r14_an_existing_ledger_writer_cannot_invalidate_a_root_that_captured_the_ledger` in
  `commit_state_log_tests.rs`. Not shown: saved content enters the store only through create_saved_content and
  an existing file is never adopted: capture of saved content is refused for any existing file and creation
  installs its own fresh inode over the name
  (r16_an_existing_read_only_inode_at_the_name_is_replaced_and_never_adopted,
  r15_a_raw_file_in_the_saved_directory_is_not_saved_content_and_the_component_made_one_cannot_be_reopened_for_writing);
  a read lease is an extra check where granted and never a refusal
  (r15_a_file_system_that_refuses_read_leases_still_creates_and_checkpoints_saved_content); the premise is that
  nobody with the lane's account chmods and rewrites sealed content; a privileged writer or a writable parent
  directory can still replace it, which the checksum name detects and does not prevent.
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
- Narrower, property: saved content survives until authorized disposal:
  `a_removal_before_the_line_is_durable_leaves_a_root_that_names_a_lost_item` in
  `model_disposal_r23_tests.rs`. Not shown: the model shows the order (the person's explicit disposal, then the
  disposed line durable, then the removal and the root's drop:
  retirement_never_removes_saved_content_and_the_control_that_does_is_red,
  a_root_that_drops_the_item_before_the_line_is_durable_is_red); no component removes saved content yet, so the
  order is a rule the disposal component must follow and the model is its specification; the non-drop check already
  refuses a checkpoint that drops saved content without a disposed line in its own closure; a disposal command and a
  component test of it are not built.
- Not yet, obligation: live cutover excludes old probe-capable holders or provides recovery responsibility
  (review 15). Needs: a cutover rule: an old client that probes the flusher lock and holds it without sweeping,
  overlapping the new implementation, recreates the schedule where a new client's helper loses to the probe and
  the probe owner dies before it forks; cutover must exclude old holders (a version fence on the lane) or give
  the receipts a recovery responsibility, and a test with an old-behaviour holder (the legacy probe seam is the
  stand-in).
- Narrower, control: push exclusion and frozen updates against actual Git:
  `a_negative_refspec_keeps_the_internal_namespace_off_the_remote_and_a_named_object_is_pushed_as_given` in
  `model_git_experiments_tests.rs`. Not shown: the marker the lane acknowledges and the stacked push path; the
  test pushes to a local bare remote with plain `git push`.

### M1 to M3, MG1 to MG3, MG6c

- Covered, control: concurrent staging: `a_later_staging_is_replaced_under_twelve_and_left_under_thirteen` in
  `model_capture_r13_tests.rs`.
- Covered, control: conditional installation keeps displaced versions:
  `overlapping_staged_and_worktree_versions_are_all_kept` in `model_merge_r14_tests.rs`.
- Narrower, control: no foreign lock removal, no live lock removed, and the lane reclaims a proven-stale lock
  of its own dead incarnation (owner decision of 2026-11-04; claim files on every file system):
  `a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_claim_chain_and_a_dead_claim_is_passed_over` in
  `model_m1_reclaim_r19_tests.rs`. Not shown: the claim chain is the current policy and this is its evidence;
  the flock hold is comparison evidence (a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_flock) and
  the mixed-mount control (the_mixed_mount_residue_is_reachable_and_is_the_stated_one) shows why mixed exclusion
  is unsuitable and is not an accepted residue; the model has two one-shot reclaimers and at most one death,
  scans for a slot and places the claim in one step, clears the claims in one step and has no retry, so the
  green is not unconditional eventual reclamation (a stale lock may stay after a death) and says little about
  slot reuse and cleanup interleavings; the slot protocol and its leftover handling are the unreviewed branch
  fix/git-policy-cli-index-lock-reclaim-without-flock at b071e48f74; a person removing the lock by hand between
  the check and the unlink is the labelled premise break; no component calls the mechanism yet.
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
  signing-failure outcomes, which the model keeps as one Failed symbol without outcome detail or commit
  identity; the stored notice carries the attempt that produced it, and its print, removal and delivery are
  bound to that tag (an_old_notice_printed_after_a_new_attempt_was_chosen_does_not_deliver_the_new_attempt, with
  a monitor that reads the stored tag and not the owner's bookkeeping); after a crash with surviving positive
  authority the model reconciles a notice that still says scheduled
  (a_settled_recovery_reconciles_a_notice_that_still_says_scheduled); the first version of this row overstated
  both, the reviewer's schedule is now a regression.
- Narrower, control: notice crash and reclamation:
  `recovery_data_is_kept_until_the_outcome_notice_is_delivered` in `model_fresh_attempt_r19_tests.rs`. Not
  shown: crash points of every notice kind; the model covers the fresh-attempt notice only, for one request with
  at most two power losses; reclamation needs the forced removal of the current attempt's own outcome notice,
  tracked by the stored tag, with print, remove and force-removal as separate steps
  (an_earlier_attempts_delivery_does_not_authorize_reclaiming_a_later_attempt,
  an_old_notice_printed_after_a_new_attempt_was_chosen_does_not_deliver_the_new_attempt); the straight-line test
  named here shows the plain case only.
- Narrower, property: decision 19, fresh attempt with a stored notice when no decisive positive authority
  survives: `decision_nineteen_is_green_over_every_schedule` in `model_fresh_attempt_r19_tests.rs`. Not shown:
  one request and at most two power losses; one Failed symbol for conflict, refusal and signing failure, with no
  outcome detail and no commit identity; the notice file is a visible and a durable content with a
  producing-attempt tag, not the real file; no component implements the recovery; the green is exhaustive within
  these bounds and was false twice before (a settled notice left scheduled, delivery of an earlier attempt, an
  old notice retagged at print), each now a regression with its own control.
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

## Round 15 state

Superseded in parts by "Round 16 state" (saved-content adoption, closure derivation, covering, launch handover); the
round 16 section is the current word where they differ.

Round 15 answers `commit-models-sol-9a.txt` (components and evidence) and `commit-models-sol-9b.txt` (models and map)
against the unchanged revision 19 acceptance target, and applies the owner decision of 2026-11-04 ("Reclaim it
(Recommended)") that replaces R-never. Nothing here is wired into the live wrapper. A red-first test is committed before
its fix; where the red is a single-defect control of a model and not a failing replay, the entry says so.

### Review 9a section 3, launch and flusher

- 3A, launch success ignored a first-child failure. Red (`commit_helper_launch_fault_tests.rs`):
  `a_failed_second_fork_is_a_failed_launch_and_no_helper_runs`,
  `a_helper_whose_stream_setup_fails_is_reported_to_the_launcher`,
  `a_helper_whose_descriptor_closing_fails_is_reported_to_the_launcher`, each returned success. Green: the launch waits
  for the helper's own ready byte on a pipe and names `FirstChildFailed`, `SetupFailed` or `LostBeforeReady`.
- 3B, the single-threaded precondition. Red: `a_launch_from_a_process_with_another_thread_is_refused_before_the_fork`
  (the reviewer's mutex held by another launcher thread; the launch returned success and the helper stayed blocked).
  Green: the launch reads `/proc/self/task` and refuses with `LaunchError::Threaded`. The harness runs each test on a
  second thread, so tests launch from `in_single_threaded_process` (a forked copy with one thread).
- 3C, one helper per client. Red: `clients_that_find_the_flusher_lock_held_start_no_helper` (four forks) and
  `many_clients_start_one_helper_and_it_holds_the_lock` (four forks). Green: `launch_flusher` takes `flusher.lock`
  first and hands the open file description to the single helper it forks. Options considered:
  - Option 1, the launcher takes the lock and the helper keeps it (chosen): exact bound of one helper per holder, no new
    lock; con: the launch needs a keep-descriptor parameter.
  - Option 2, a non-blocking probe before the fork: no handoff; con: the bound is the probe window, not one.
  - Option 3, an `O_EXCL` launch marker: con: stale on a crash and foreign removal is banned.
  - Ranking: option 1 > option 2, because only option 1 gives a bound; option 2 > option 3, because a marker leaves a
    stale file that nobody may remove.
- 3D, a receipt that joins after the error listing. Red: `a_receipt_that_joins_after_the_error_listing_is_recorded`
  (needs the fault point `flusher:after-error-snapshot`). Green: the error path lists, records, releases the lock and
  lists once more.
- 3E, a forced file whose directory entry was not forced. Red: the renamed
  `a_forcing_error_records_every_receipt_whose_name_is_not_forced_the_joiner_included` and
  `a_launched_flusher_records_a_forcing_error_and_releases_the_lock` (expectation `a`, `b`, `c`). Green: names enter
  `forced` only after the directory is forced.

### Review 9a section 2, components

- 2A, a document and an independent list. Red:
  `r15_a_document_that_names_a_dependency_cannot_be_checkpointed_or_appended_without_it`
  and `r15_recovery_does_not_select_a_document_whose_needs_its_set_does_not_cover`. Green: a document names its
  dependencies with `cli-git-needs <hex path>` lines (`needs_line`); the writer and recovery accept a record only when
  its retained set covers them. This is a design choice the design does not settle. Options:
  - Needs lines owned by the state log (implemented): the record is checkable by recovery; con: the lane's real state
    document must adopt the line grammar.
  - Non-reclaiming checkpoints (empty `reclaims`): no grammar; con: the log never shrinks and the reclamation rows lose
    their evidence.
  - A typed document parsed by the caller: con: the same independence as before.
  - Plainly: the needs lines bind the declaration into the document bytes; a writer that omits a needs line for
    something
    its document semantically depends on is still accepted, because the state log cannot read the meaning of the
    document. The reviewer asked for document-bound authority or a non-reclaiming checkpoint, and will test exactly
    this.
  - Ranking: needs lines > non-reclaiming, because reclamation stays available with a checkable obligation;
    non-reclaiming > typed parse, because it is safe without trusting a caller. The owner or the design reviewer may
    prefer the second; switching is one change of `reclaims`.
- 2B, content addressing detects and does not prevent. Red (first version, superseded by the coordinator pick below):
  `r15_saved_content_with_a_surviving_writable_descriptor_is_refused_and_sealed_content_cannot_be_reopened`. Coordinator
  pick under the owner's rule "Must work perfectly even when flock isn't available. Do better." (veto open to the
  owner):
  the protection may not depend on read leases, so immutability is enforced by the creator. Saved content enters
  the store
  only through `create_saved_content`, from bytes and never as an existing file: `O_TMPFILE` where the file system
  supports it, else a random private name in the lane's 0700 `saved-staging` directory; written, forced, made 0444
  before
  it has its final name, linked in without replace, the directory chain forced to the lane, the only descriptor closed.
  Capture and completion refuse an existing file with a write bit. The checksum name stays as detection. A read lease is
  an extra check where the kernel grants it (a writer that is open is refused) and never a refusal.
  Red first:
  `r15_a_raw_file_in_the_saved_directory_is_not_saved_content_and_the_component_made_one_cannot_be_reopened_for_writing`
  and `r15_a_file_system_that_refuses_read_leases_still_creates_and_checkpoints_saved_content`. Green coverage
  written with
  the code, not red first: `r15_saved_content_created_without_tmpfile_ends_sealed_and_leaves_no_staging_file` and
  `r15_creating_saved_content_twice_replaces_nothing_and_a_writable_occupant_is_refused`.
  Premise, stated next to the lane's other assumptions (the state log module header): the only way to write saved
  content
  afterwards is a deliberate chmod by the account that owns the lane. Remains: a privileged writer or a writable parent
  directory can still replace sealed content, which the checksum name detects and does not prevent.
- 2C, retirement and the authoritative ledger. Red:
  `r15_recovery_reaches_a_retirement_record_after_the_disposal_removed_the_content`
  (`Damaged`) and `r15_a_disposal_line_in_an_unrelated_prefix_file_does_not_discharge_saved_content`. Green: recovery
  selects the last record whose own set validates within the chain of records whose declared sets follow by non-drop or
  by a ledger proof; only the lane's `ledger` (`LANE_LEDGER`) discharges an item.
- 2D, forced paths, not forced prefixes. Red:
  `r15_a_departure_forces_the_longer_ledger_prefix_of_a_state_that_grew_while_it_forced`
  (fault point `departure:after-items-forced`). Green: a departure accounts items by path, length and checksum.
- 2E, inclusion, not succession. Red:
  `r15_an_unrelated_document_that_retains_the_same_items_does_not_cover_the_acknowledgment`
  and the flipped forged-hash assertion of
  `r14_an_acknowledgment_survives_completion_and_not_a_substitute_that_drops_its_support`.
  Green: a replacement authority records the chain hashes of the records it replaces (`lineage`), and coverage across
  segments needs the promised record among them. Remains: one hop, so an acknowledgment older than the previous
  checkpoint is not covered (fail closed).

### Review 9a section 4, test limitations

- The three tests that were not red first keep their names where they still describe the behaviour. The joiner test was
  renamed and its expectation corrected (3E); the launched-flusher test now launches from a single-threaded copy; the
  healthy-lane test is unchanged.
- `r14_every_save_is_attempted_after_a_stop_error_and_a_saved_content_error` now supplies an order file and asserts the
  whole failure inventory (a strengthening; it passed before, so it was not red first).
- `a_recheck_that_finds_work_and_the_lock_taken_hands_over` keeps its raw holder; the real second flusher is
  `a_flusher_launched_in_the_release_window_carries_the_receipt_the_first_hands_over` (not red first: it needs the
  lock-first launch to be deterministic).

### Review 9b

- Item 1, the receipt error path is four observable phases (list, record, release, list again). Red by control:
  `EighteenErrorNoRelist` and `EighteenErrorIgnoresJoiners` in
  `a_receipt_that_joins_between_the_error_listing_and_the_release_is_recorded_by_the_second_listing`; the old
  transition could not express the schedule.
- Found by the split, picked by the coordinator under the owner's rule "nothing should ever wait forever or
  indefinitely" (veto open): a receipt that joins a flusher's error path after the next owner's startup sweep is
  recorded,
  and waiting for yet another owner's start can wait without end. The running owner's syncer therefore retries recorded
  errors, and the final sweep enumerates them; this is the default rule, not an excused case.
  `a_running_owner_retries_an_error_recorded_after_its_startup_sweep` is green for the rule and red for the control
  `EighteenSyncerNeverRetries`. The first draft of this round carried a named excuse (`fresh_record`); it is gone.
- Item 2, the fresh-attempt model. A: `a_settled_recovery_reconciles_a_notice_that_still_says_scheduled` (red under
  `SettledKeepsNotice`). B: `an_earlier_attempts_delivery_does_not_authorize_reclaiming_a_later_attempt` (red under
  `DeliveryIsRequestWide`). Delivery is the forced removal of the attempt's own final notice; print, remove and
  force-removal are separate transitions.
- Item 3, map corrections: the worker-error, checkpoint-and-recovery and decision 19 rows are Narrower with honest
  remains; the notice rows' remains no longer say only "other texts"; the request-family row states the real family.
  The reference-interpreter row stays Covered as the reviewer accepted.

### Owner decision of 2026-11-04, R-never replaced

`model/model_m1_reclaim_r19.rs` models the lane reclaiming a proven-stale `index.lock` of its own dead incarnation:
holder proven gone and the judged file remembered, exactly one reclaimer (an exclusive `flock` on the stale file, or the
claim chain where `flock` is absent), a check that the name still refers to the judged file, and an unlink that is a
separate step. Checks kept: no foreign lock is removed, a live lock is never removed (each unless a person removed the
stale lock by hand first), two reclaimers never hold at once, and no stale lock of the dead incarnation is left unless a
reclaimer died holding. Red: never-remove (the superseded rule), no hold, no name check, an unproven holder, and the
mixed-mount residue. `model_m1_split_r13.rs` stays as the split compare and unlink control and is labelled
superseded for
this lock in its header. What waits for the review of `fix/git-policy-cli-index-lock-reclaim-without-flock` (read at
`b071e48f74`, not yet reviewed): the claim chain's slot protocol and its leftover handling; the map row is Narrower for
that reason. The design file was not edited.

### Supported scope: paragraph for the design, not yet placed

The design lives on `fix/git-policy-cli-commits-never-fail` in another worktree, so it was not edited from here; the
coordinator places this text (coordinator picks under the owner's rules, veto open):

> Supported scope for the acceptance target: Linux; the Git version the tests run on (pacman's current Git in the test
> image, 2.56.0 when round 15 ran; an older Git only warns); the files and reftable ref backends; local file systems.
> Packed refs, linked worktrees, the side-ref transaction, and the creation, deletion and symbolic-ref updates through
> the transaction are supported, and each is an obligation to demonstrate with a disposable backend experiment before
> the live protocol ships; until its test exists its row stays Narrower. Excluded from the supported scope: level 2 (off
> by the owner's decision), a launch from a process with more than one thread (refused by design), a memory or process
> cap (outside our control), and the manager-started transient unit launch route until it is built.

### Findings and limits for the reviewer (none adds a row)

- The launch refusal of a threaded process means the real call sites must be single-threaded at the launch; no call site
  exists yet, so this is shown on test copies of a process only.
- `r15_` in test names is the round number and is unrelated to design revision 15 (`model_submission_r15`).
- `a_fork_refused_by_the_process_limit_is_reported_as_refused` returns early when run as root, because `RLIMIT_NPROC`
  does not bind a process with `CAP_SYS_RESOURCE`; run it as the unprivileged user (the host does).
- The `cli-git-service` processes on the host are the shipped wrapper's own services started by commits, not helpers of
  these tests.
- The model-level reds for items 9b 1 and 2 are single-defect controls, not failing replays of the submitted model.
- The needs-line grammar binds a document's declared dependencies; a writer that omits a needs line for something its
  document depends on is still accepted (the non-reclaiming alternative is ranked above).
- Contradictions with the design: the design still says R-never in its choices list (the coordinator amends revision
  19); the design's "supported scope" had no text, now supplied above; the receipt retry by the running syncer is a
  coordinator pick the design text (retry at the next owner's start) does not state.

Gates at the tip: the full `native:test:model` run is 442 passed, 0 failed
(`/var/home/user/temp/agent/r15/full-model.log`); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r15/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has
64 covered, 17 narrower, 9 historical and 17 not yet.

## Round 16 state

Round 16 answers `commit-models-sol-10a.txt` (components and evidence; "does not meet the claimed items as
submitted") and
`commit-models-sol-10b.txt` (models and map; "no, as submitted") against the unchanged acceptance target. Nothing
here is
wired into the live wrapper. Where the red is a single-defect control of a model and not a failing replay of the
submitted
model, the entry says so.

### A correction to round 15

A round 15 edit (the red commit for the coordinator pick on saved content) cut the test
`r15_an_unrelated_document_that_retains_the_same_items_does_not_cover_the_acknowledgment` out of
`commit/commit_state_log_tests.rs`, while the green commit for 2E had been committed with it. The review inputs of
round 15
therefore lacked that test and the handover still named it. It is restored in round 16 (commit
`test(git-policy-cli): red first, review 10a section 2E ...`), next to the new 2E control. The map check only tests the
names in its `test` column, which is why it did not notice.

### Review 10a, launch and flusher

- 3A, the launcher's copy of the lock. Red:
  `a_client_that_joins_while_the_launcher_still_holds_its_copy_has_an_executor`
  (client X found the lock held by the launcher's copy after the helper had finished; no executor, no record) and
  `a_client_that_joined_a_launch_whose_setup_failed_is_recorded`. Green: `launch_detached_holding` gives the helper the
  descriptors and nothing else, the launcher closes its own copy as soon as the helper reported ready and only then
  sends
  the handover byte the helper waits for before it works, so the helper cannot release before the launcher's copy
  is gone;
  a failed launch drops the lock and records every receipt of the lane (joiners included) in `receipt-errors`.
- 3B, live helpers through the release, recheck and exit step. Not fixed; a design choice. The lock bounds the
  helpers that
  hold it; a helper that released it and is in its recheck or exit step holds nothing, and a launch in that step starts
  another, so processes stopped from outside can pile up. The false comments are corrected, the row is Narrower and
  says so.
  Options:
  - Accept and state it (done): the step is a listing and at most one relock attempt, it waits for nothing, and the
    number
    of live helpers is bounded by the launches in that step; con: not a bound under SIGSTOP of helpers.
  - A second lock held from start to exit (`flusher.exit`): a launch needs both free. Con: it breaks the argument that
    makes the recheck sound, because a client that created its receipt after the holder's last listing but before the
    holder's exit finds the second lock held and starts nothing, and nobody lists again; and it is a second lock against
    "Use as few locks as possible."
  - Keep the recheck under the lock and have every client that finds it held wait for a hand-back: con: a wait, against
    "nothing should ever wait forever".
  - Ranking: accept and state it > a second lock, because the second lock is unsound; a second lock > waiting clients,
    because it at least waits for nothing. I found no protocol of release-then-recheck that bounds the
    stopped-process case.
- 3C, a launch with descriptor 0 closed. Red: `a_launch_with_descriptor_zero_closed_leaves_the_helper_holding_the_lock`.
  Green: the lock and the pipes are moved above the standard streams when they are made, and the launch refuses a held
  descriptor that is 0, 1 or 2 (`LaunchError::LowDescriptor`).
- 3D, a ready byte over an unreaped intermediate child. Red:
  `a_ready_helper_does_not_make_the_launch_succeed_while_the_intermediate_child_is_unreaped`. Green: a first child
  that does
  not leave in time is killed and reaped before the launch returns.
- Section 4, the real handover. `a_second_executor_pinned_across_the_handover_decision_carries_the_receipt` pins
  the second
  executor with a FIFO receipt and replaces the nondeterministic test (not red first: it passed on the old code
  too, because
  the old code had no race for it to lose in this form).

### Review 10a, components

- 2A, complete closure. Red:
  `r16_a_reclaiming_checkpoint_must_retain_what_the_lane_holds_whatever_its_document_declares`.
  Green: a reclaiming checkpoint (the only record that removes its predecessor) derives its obligations from the
  lane: every
  request in `inbox/`, every saved content, and the ledger up to its length now must be in the closure or
  discharged by a
  ledger line inside it. Choice made: derive where the component can see (the lane's own directories), keep needs
  lines for
  dependencies it cannot see (an order file), and leave plain appends alone because they reclaim nothing. Options:
  - Derive from the lane (built): reclamation stays available and the closure cannot omit what exists; con: an
    over-approximation, and a ledger that grows between capture and begin makes the caller capture again.
  - Non-reclaiming checkpoints for every document: no derivation; con: the log never shrinks.
  - Declared needs lines only (round 15): con: a writer can omit one.
  - Ranking: derive > non-reclaiming, because the log stays bounded and the lane's own files are exact; non-reclaiming >
    declared lines, because it does not rely on a writer's completeness. Remains: a dependency outside inbox, saved
    and the
    ledger is still declared only.
- 2B, the adoption boundary. Red: `r16_an_existing_read_only_inode_at_the_name_is_replaced_and_never_adopted` (with
  leases
  unavailable, the reviewer's condition). Green: capture of saved content is refused for any existing file and
  entries come
  only from `create_saved_content`, which installs its own fresh inode over the name (rename), so an inode of unknown
  provenance is replaced and never adopted; a holder of the replaced inode writes into a file nothing refers to. The
  stated premise stays for what the component cannot enforce (a deliberate chmod by the lane's account).
- 2E, a checked covering relation. Red:
  `r16_a_checkpoint_that_changes_the_document_does_not_cover_an_earlier_acknowledgment`.
  Green: lineage is recorded only for a compaction of the head (the head's document at the head's generation); a
  checkpoint that changes the document covers no earlier acknowledgment, and the caller forces again. Remains: one hop.

### Review 10b

- The stored notice carries the attempt that produced it; its print, removal and forced removal are bound to that tag.
  Red by control: `an_old_notice_printed_after_a_new_attempt_was_chosen_does_not_deliver_the_new_attempt` (the
  reviewer's
  schedule; `PrintRetagsToCurrentAttempt` is red, `Nineteen` is green). The monitor is independent of the owner's
  delivery
  bookkeeping: `printed_log` is set from the tag stored in the notice file at the print, and the reclamation
  invariant needs
  it as well as `delivered`.
- Map: the three fresh-attempt rows, the launch rows, the closure, adoption and covering rows are corrected. The reclaim
  row points at the claim-chain test (the owner chose claim files on every file system), the flock test is comparison
  evidence, mixed mode is a control that shows why mixed exclusion is unsuitable and not an accepted residue, and
  the row
  does not claim unconditional eventual reclamation. `without_the_name_check_a_late_reclaimer_removes_the_landings_lock`
  now reaches the removal.

### Gates at the tip

The full `native:test:model` run is 451 passed, 0 failed (`/var/home/user/temp/agent/r16/full-model.log`);
`native:clippy` and `native:clippy:seams` are clean (`/var/home/user/temp/agent/r16/clippy.log` and
`clippy-seams.log`); the `source_guard` tests pass (39 of 39); the map has 64 covered, 17 narrower, 9 historical and
17 not yet.

## Round 17 state

Round 17 answers `commit-models-sol-11.txt` ("no, as submitted"). Nothing here is wired into the live wrapper.
Finding 7 is
modeled and not built, because the model finds schedules where the proposal misses a receipt.

### Findings 3 to 6, launch and descriptors

- 3 (window A, the handover wait expires). Red:
  `a_helper_whose_handover_wait_expires_runs_nothing_and_the_launch_accounts_for_every_receipt`. Green: the helper
  waits for the
  handover byte and an expired wait, an end of input or a read failure is not a handover; it closes its report end
  and ends
  without running its entry. The launcher waits for the helper's acknowledgment after it sends the byte; no
  acknowledgment is
  `LaunchError::HandoverNotAcknowledged`, and `launch_flusher` then records every receipt of the lane, joiners included.
- 4 (window B, the intermediate child holds a copy). Red:
  `no_copy_of_the_lock_but_the_executors_exists_when_the_handover_is_sent`. Green: the order is fixed: the helper
  reports ready,
  the launcher reaps the intermediate child (killing it when it does not leave), closes its own copy, and only then
  sends the
  handover byte.
- 5 (the unchecked kill and wait). Red: `a_refused_kill_of_the_intermediate_child_is_a_failed_launch`. Green:
  `kill_and_reap`
  reports a refused kill (not "no such process"), repeats an interrupted wait, accepts "no child" as reaped, and is
  bounded by
  `KILL_REAP_LIMIT`; a child that is not reaped is `LaunchError::IntermediateNotReaped`. The launch is bounded by
  three limits
  (report, the intermediate child, the acknowledgment) plus this one.
- 6 (descriptor ownership). Red: `a_launch_whose_descriptor_normalization_fails_leaks_no_descriptor` and
  `a_lock_acquisition_whose_descriptor_move_fails_leaks_no_descriptor` (both drive `EMFILE` with the standard
  descriptors closed
  and a small `RLIMIT_NOFILE`). Green: the pipes and the lock are `OwnedFd` from the moment they exist, and
  `move_above_streams` takes and returns an owner, so a failure closes what it holds.

### Findings 1 and 2, closure and covering

- Choice (veto open): a document is reclaiming only in a checked representation the component can read.
  `LaneDocument` names
  the requests, the saved content and the ledger extent a state relies on and nothing else; a dependency outside
  it, such as an
  order file, cannot be written in it. Opaque documents (`begin_checkpoint`, `checkpoint`,
  `begin_checkpoint_retiring`) never
  reclaim. `begin_checkpoint_reclaiming` and `checkpoint_reclaiming` take a `LaneDocument` that equals what the
  closure holds,
  the closure must hold everything the lane holds, and a segment is reclaimed only when every record of it is a
  checked document
  that the new one contains (each request and saved content kept or discharged, the ledger no longer). The lineage
  names the
  records of the reclaimed segments only, so it never stands in for preservation.
- 1. Red: `r17_an_opaque_document_that_needs_a_file_outside_the_derived_places_never_reclaims_its_predecessor` and
  `r17_a_checkpoint_cannot_compact_an_opaque_append_and_reclaim_its_segment`. Green: opaque checkpoints reclaim nothing.
- 2. Red: `r17_a_compaction_of_a_later_opaque_record_does_not_reclaim_or_cover_an_earlier_acknowledgment` (the
  reviewer's one
  checkpoint schedule). Green: the segment that holds the acknowledged record stays and `state_covers` is false.
  The positive
  case is `r17_a_checked_document_reclaims_and_covers_and_anything_unchecked_is_refused_for_reclamation`.
- Remains: the lane's real state document is not implemented, so whether it fits the representation is open; a
  state that does
  not fit stays on the non-reclaiming path and its log grows. Options: extend `LaneDocument` with the references
  the real state
  needs (ranked first, because the log stays bounded and the references stay derivable), or keep such states
  non-reclaiming.

### Finding 7, the token kept until exit: modeled, not built

The proposal (the admission token kept until process exit, no release and no recheck; a client that finds it held
leaves its
receipt to the continuing retry of the running owner's syncer) was added to `model_receipt_r18.rs` as
`EighteenTokenToExitWithSyncerRetry`, with the syncer retrying receipts that are still V as well as recorded
errors. It misses a
receipt. Schedule one (`a_token_held_to_exit_misses_a_receipt_when_no_owner_runs_to_retry_it`):

1. client 0 creates its receipt; a planned shutdown creates `lane/sealed`; client 0 starts a flusher (the token is
   taken);
2. the service's final sweep enumerates and forces, and the service exits;
3. flusher 0 lists the receipts and finds none unforced;
4. client 1 creates its receipt, reads the lane (no service), finds the token held and starts nothing;
5. flusher 0 exits with the token. Receipt 1 is V, no service runs, and nothing retries it until some later command
   starts an
   owner, which can be never.

Schedule two
(`a_token_held_to_exit_still_misses_a_receipt_when_the_client_cannot_start_the_owner_during_the_shutdown`), with
the retry owner started by the client itself when it finds no service (the `EighteenTokenToExitClientStartsOwner`
variant): the
same, except client 1 reads while the service is still in its shutdown (sealed, the lane held), so it cannot start
an owner;
the flusher exits, the service exits, and no owner starts.

Why the proposal cannot be repaired inside its terms: the retry owner must be a process that outlives the
contender. With a
service it exists, and the built protocol already uses it (the running syncer retries recorded errors and V
receipts on its
wakes). Without a service, a contender can start one only by starting a long-lived process per contender, which is the
resource bound again, or by waiting, which is forbidden. Release-then-recheck is the one step that makes a no-service
schedule sound, and it is exactly what lets a helper that released the lock stay alive and unaccounted. Checked
against "We
should never invoke ourselves recursively.": starting the retry owner through the wrapper's command line would be
recursive;
starting it as the service component is not, but it is the start of a long-lived process by a contender, and the
schedule
above shows it is not always possible.

Options, ranking (the owner decides; the narrowed row stays, with these schedules):

- Keep release, recheck and exit with the bound stated (built): sound in every modeled schedule; con: live helpers
  are bounded
  only by the launches that arrive in the release, recheck and exit step.
- Token to exit with the running syncer's retry: bounded helpers; con: misses a receipt when no owner runs (the two
  schedules).
- Token to exit and the contender waits or hands the receipt back: con: a wait.
- Ranking: the built protocol > token to exit, because a missed receipt breaks "never discard user data" and an
  unbounded
  step breaks only a resource bound that no stopped-process-free run reaches; token to exit > waiting, because it
  waits for
  nothing.

### Gates at the tip

The full `native:test:model` run is 462 passed, 0 failed (`/var/home/user/temp/agent/r17/full-model.log`; nextest
reports one
leaky test, the stalled intermediate child that its own alarm ends); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r17/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 64
covered, 17 narrower, 9 historical and 17 not yet.

## Round 18 state

Round 18 answers `commit-models-sol-12.txt` ("no, as submitted"): findings A to C are fixed with red controls
first, and the
live-helper bound is a comparison of five protocols in a new model for the owner to choose from. Nothing of the
five is built in
the components. The owner has not been asked yet.

### Findings A to C

- A (launcher death before the handover). Red:
  `a_launcher_that_dies_before_the_handover_leaves_no_receipt_without_an_executor`.
  Green: no process but the launcher carries the lock during a launch (the intermediate child and the helper close
  the held
  descriptors at once); after the handover, or at the end of the input when the launcher died, the helper takes the
  lock itself and
  works when it gets it. A launcher that is only slow still holds the lock, the helper finds it taken and ends, and
  the launcher
  fails with `HandoverNotAcknowledged` and records the lane's receipts.
- B (the failure tail and cleanup errors). Red:
  `a_client_that_arrives_after_a_failed_launch_finds_the_lock_free_and_has_an_executor`.
  Green: because no other process holds the lock, the launcher's own drop is the last release, and a launch that
  fails records the
  lane's receipts after that drop; a client that arrives later finds the lock free and starts its own flusher. The
  same holds when
  the intermediate child cannot be killed: it holds no copy.
- C (checked references on admission). Red:
  `r18_a_checked_document_that_names_a_dependency_the_set_lacks_is_neither_admitted_nor_selected`.
  Green: a document in the checked representation is validated against its retained set at append admission, at the
  start of a
  checkpoint and at recovery; a document that starts as a checked document and does not parse is not covered.

### The live-helper bound: five protocols on one model

`model/model_helper_bound_r20.rs` runs three clients against a service with a planned shutdown, a crash and a later
start, from two
start states (a running service, and no service at all with no seal). A miss is a state where only the environment
can move and a
receipt is still visible. Live processes are helpers, workers and the owner. The table is asserted in
`model_helper_bound_r20_tests.rs`.

- The built release, recheck and exit.
  - Misses: none in the model.
  - Live processes: as many as there are clients that launch in the release step (three with three clients; it
    grows with the
    clients).
  - Waits: none. Locks added: none. Recursion: none (an internal entry). Foreground cost of an ordinary commit: none.
- The reviewer's singleton retry owner.
  - Misses: none. Live processes: two (the owner and one worker). Waits: none in the foreground; the owner wakes on
    a bounded
    interval. Locks added: none (it holds the token for its life).
  - Real costs, not in the model: one extra process per lane that needs the flusher path, alive until a verified
    transfer or for
    ever; it starts through the internal entry (a function, not the wrapper command line, so no recursion), but the
    fallback
    launch leaves it in the client's session scope, which a logout ends, so a lasting owner needs the
    manager-started unit route,
    which is not built; across an upgrade the old owner runs old code until it retires. "Verified responsibility
    transfer",
    concretely: the owner retires only when a running service has registered its syncer for the lane (a persisted
    record with the
    boot identity) and the owner's last listing found nothing; without a service it does not retire. Persisted
    responsibility:
    the receipts in `lane/receipt/` are the registration and the owner is restarted by the next client that finds
    the token free.
  - Foreground cost: the same as today (create the receipt, try the token, fork when free).
- A client that forces its own receipt inline while the lane is sealed (the first of my suggestions).
  - Misses: yes. Schedule: no service runs and the lane is not sealed (the service never started), a flusher holds
    the token,
    client 2 creates its receipt after the flusher's last listing, finds the token held, is not in the sealed case
    and leaves it;
    the flusher exits. `forcing_inline_only_while_sealed_still_misses_when_no_service_runs`.
  - So the sealed condition is not enough. Live processes: one.
- A client that forces its own receipt inline whenever the token is held, the token kept to exit (my second suggestion).
  - Misses: none in the model. Live processes: one (plus launch intermediates). Locks added: none. Recursion: none.
  - Waits: only the client's own synchronisation of its receipt file and its directory, bounded by the disk and the
    same "Force
    at leaving" cost the wrapper already accepts for its own syncs. It is paid only by a client that finds a
    flusher at work,
    which happens only when no service runs or the lane is sealed; a commit with a running service rings it and
    pays nothing.
  - Measured foreground cost (one run of twenty on this host at load average 14, file sync plus directory sync of a
    small file):
    minimum 3.6 ms, median 11.4 ms, maximum 77.1 ms; one run is not a spread (QNB), so read it as the order of
    magnitude, far under
    the five seconds of "commits vs native git should not take 5s longer", unless the disk stalls, in which case
    every forced save
    of the wrapper stalls with it.
  - Why it holds: a receipt is forced by the process that created it or by a flusher that took the token after the
    receipt existed;
    no retry owner is needed and no exit race can lose it. Under "Use as few locks as possible" it adds none and
    removes the
    release, recheck and exit step.
- The baton (another protocol found while modeling).
  - The running service is the retry owner while it runs; when it exits it starts the singleton owner, and the
    owner retires when
    a running service has the lane and acknowledged the transfer.
  - Misses: none. Live processes: two (owner and worker), and the owner exists only while no service runs. Waits:
    the service's
    shutdown starts the owner (a bounded fork and handshake, not a foreground wait). Locks added: none.
  - Cost: the owner must outlive the service's unit, so it has to be started outside the service's control group
    (the unit route
    that is not built); a crash hands no baton, which the model excuses like every crash.
- The control: the token kept to exit with nobody to retry misses (round 17's two schedules); live processes one.

Owner rules, protocol by protocol (a plain reading, not a measurement unless marked):

- "Commits should never fail." and "never discard user data": all but the control and the sealed-only inline keep
  every receipt
  owned in the model; a forcing error is recorded per receipt in each.
- "nothing should ever wait forever or indefinitely": no foreground wait in any; the inline client waits only for
  its own
  synchronisation; the owner protocols add a background loop with a bounded wake interval.
- "Use as few locks as possible.": none adds a lock.
- "Force at leaving, else background.": the inline protocol is this rule applied to the receipt of a client that
  cannot hand it
  to a worker; the others are background.
- "We should never invoke ourselves recursively.": none starts the wrapper command line.
- "commits vs native git should not take 5s longer": only the inline protocol puts a synchronisation in a
  foreground path, and
  only for a client that finds a flusher at work.

Ranking, my reading and the owner decides: the inline-whenever-held protocol > the built protocol > the baton > the
singleton
owner. The inline protocol bounds live processes at one with no extra long-lived process and no unit route; the
built protocol
misses nothing but its live-helper count grows with the clients; the baton and the singleton owner bound the count
at two and
need the unit route that does not exist yet, and the singleton owner stays alive when a service already runs.

### Gates at the tip

The full `native:test:model` run is 471 passed, 0 failed (`/var/home/user/temp/agent/r18/full-model.log`; nextest
reports three
leaky tests, the stalled intermediate children that their alarms end); `native:clippy` and `native:clippy:seams`
are clean
(`/var/home/user/temp/agent/r18/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 64
covered, 17 narrower, 9 historical and 17 not yet.

### Round 18 corrections (review 13, the part built before the owner's answer)

- Built: the acquisition result is part of the helper's acknowledgment. Red:
  `a_helper_that_cannot_take_the_lock_is_a_failed_launch_with_every_receipt_recorded`. A helper whose `prepare`
  fails reports it
  (`LaunchError::HelperCouldNotPrepare`) instead of acknowledging, the launch fails and the launcher records every
  receipt of the
  lane; the helper also records them itself when it has no launcher to tell. The red run found a second defect on
  the way: a
  launcher and its helper recorded at once and `writeln!` interleaved their lines; `record_errors` now writes each
  line in one
  write.
- Corrected claims: a helper that receives no handover does not "run nothing"; it takes the lock itself and works
  when it gets
  it. The claim that no holder is left unaccounted is withdrawn for two schedules of the review, which stay open
  until the owner
  has chosen the protocol: the helper times out while the launcher is slow and the launcher then dies before it
  accounts, and
  the intermediate child that is delayed before its first close and not reaped in time still holds a copy when the
  launcher
  drops its own and records.
- Narrowed claims about the comparison: it is exploration and not a basis for a ranking. Its crash exemption
  excuses every
  visible receipt until replacement, including receipts created after the crash; its maxima are lower bounds in the
  searches that
  stopped at a miss; its foreground count is a number of synchronisations and not a time (the 20 samples are not a
  bound, and
  `fsync` has no deadline); and it omits client death during inline forcing, worker death and partial forcing,
  launcher timeout
  and death, mixed-version helpers, enqueue rejection, non-atomic service and owner transfers and blocking time.
  The ranking in
  the round 18 section is withdrawn as a recommendation; the table of misses and counts stands only as far as these
  omissions allow.
- Not started, by instruction, until the owner has chosen: the timeout then launcher death, the inherited copy
  before the
  intermediate child's first close, the client-death and launch-interleaving schedules of protocol 4, and its
  foreground bound.

## Round 19 state

### Owner decision of 2026-11-04 on the helper count

Question tool, choice "Keep built, state limit (Recommended)": keep the helper design (a helper releases the lock,
rechecks for
new saves, then exits) and record that the helper count is bounded only by how many commits arrive during those few
milliseconds, overriding the reviewer's limit. The owner's reasons: no save lost in any modeled case, no added
commit time, no new
lock or process. The question also said: "Whichever option you pick, the plain launch bugs the review found still
get fixed."

Recorded in the map: the launch row's "resource-bounded" part is overridden with this quote and date, and the row
states the
limit plainly. The helpers in the release, recheck and exit step, and the launched helpers that do not yet hold the
lock, are
bounded only by arrivals; what is bounded is the number that hold the lock, which is one. Protocols 2 to 5 of the
round 18
comparison are not built; the comparison stays as recorded evidence with its corrected claims (round 18 corrections).

For the design file's acceptance-target notes (the coordinator places it): "Owner decision, 2026-11-04, question
tool, 'Keep
built, state limit (Recommended)': the resource-bounded part of the launch obligation is overridden. The helper
that releases the
lock, rechecks for new saves and exits is kept; the number of helpers is bounded only by the commits that arrive
while the lock is
free or being released. The number that hold the lock is one."

### Review 13 section 1, the two receipt-responsibility findings

- A (a helper whose handover wait expires while the launcher is slow, the launcher then dying before it accounts). Red:
  `a_helper_that_times_out_while_the_launcher_is_slow_and_then_loses_it_leaves_every_receipt_with_an_executor`.
- B (the intermediate child delayed before its first close, not reaped in time, holding a lock copy when the
  launcher drops its own
  and records). Red:
  `a_client_that_arrives_after_a_launch_that_could_not_reap_its_intermediate_child_has_an_executor`, with the
  kill-refused seam now stalling the child before it closes anything.
- Fix for both, one design change and no second lock, no unbounded wait and no recursion: the launcher holds no
  lock for the
  launch. `launch_flusher` probes the lock (takes and releases it) and forks a helper only when it was free; the
  helper takes the lock
  itself in `prepare`, after its setup, and the result is part of its report. No lock exists before the helper
  holds it, so a copy
  cannot sit in a child, the handover byte and its two timeouts are gone, and a launcher that dies or is slow
  leaves nothing to
  account for. A helper that finds the lock taken by now ends: the holder took it after the client's receipt
  existed and lists after
  taking it, so it sweeps that receipt. The cost, within the owner's decision: every client that probes while the
  lock is free forks
  a helper; one wins the lock and the others end at once.
- Why a fix with a held lock was not possible: a child's inherited copy cannot vanish at the instant of the fork,
  so a launcher
  that holds the lock has to account for a child it may not reap within any bound, and a child killed before it
  runs cannot
  account; holding nothing removes the copy instead of accounting for it.

### Gates at the tip

The full `native:test:model` run is 473 passed, 0 failed (`/var/home/user/temp/agent/r19/full-model.log`; nextest
reports four
leaky tests, the stalled children that their alarms end); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r19/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 64
covered, 17 narrower, 9 historical and 17 not yet.

## Round 20 state

Round 20 answers `commit-models-sol-14.txt` ("no, as submitted"). The owner's helper-count decision is not reopened.

### The blocking finding: a probe is not an executor

In round 19 `launch_flusher` probed the flusher lock (took and released it) before it forked. A probe is a holder
that lists and
forces nothing, and other clients and helpers read a held lock as proof of a responsible executor. Smallest
schedule: D holds its
successful probe; E creates a receipt, sees the lock held, returns and starts nothing; D dies before it launches.
The same
confusion reaches a launched helper that gets `Ok(None)` against a probe and ends, and the release and recheck path
that reads a
failed reacquisition as a handover.

- Red (against the probe), all three:
  `a_client_that_sees_a_probe_hold_the_lock_still_has_an_executor_when_the_probe_owner_dies`,
  `a_helper_that_loses_the_lock_to_a_probe_and_a_client_that_sees_it_do_not_leave_receipts_to_the_probe_owner`,
  `a_recheck_that_loses_the_lock_to_a_probe_does_not_leave_the_receipt_to_the_probe_owner`. Each asserts that the
  surviving clients'
  receipts (dangling links, which any executor records) appear in `receipt-errors`, not only that the launch succeeded.
- Choice made: drop the probe; a client takes no lock and looks at none, and always forks a helper that takes the
  lock itself.
  The only code that acquires the lock is `flush_lane_with` and the helper's `prepare`, and both list after they
  acquire, so
  there is no holder that is not an executor and a failed acquisition (`Ok(None)`) always means a flusher that
  lists after it took
  the lock, and again after it releases it. The alternative, testing the lock without taking it (`F_OFD_GETLK`),
  needs the lock to
  be an OFD lock; it is an `flock` (`File::try_lock`), for which no test-only call exists, and moving to OFD locks
  changes the
  primitive for no gain here. The cost of the chosen one is a fork per client that needs the flusher path, and a
  helper that ends
  at once when another works.
- Measured foreground cost (QJ1): from the call of `launch_flusher` to the helper's acknowledgment, 20 launches in
  a single-threaded
  copy of the test process on this host (load average 4): minimum 0.43 ms, median 0.66 ms, maximum 14.2 ms. One
  run, not a spread.

### Corrected claims (review 14 section 3), here and in the map

- "One wins and the others end at once" is withdrawn as a guarantee: helpers can lose to a held lock without a
  worker winning, and a
  delayed helper lives until it runs.
- "No lock exists before the helper holds it" was false while the probe existed; it is true now in the sense that
  matters: the only
  process that holds the lock is one that lists after it took it.
- "Those few milliseconds" is the owner's phrase and not a duration the code gives. The count of live helpers is
  arrival-dependent,
  with no time bound; the count that hold the lock is one.
- The intermediate child (which exits at once after its second fork and is reaped, or killed and reaped, by the
  launcher within the
  report bound) is distinguished from the detached worker (which lives until its flush ends).
- Tests that named a removed handover timeout are renamed
  (`a_slow_launcher_that_then_dies_leaves_every_receipt_with_an_executor`,
  `a_launcher_that_dies_right_after_the_helper_is_ready_leaves_no_receipt_without_an_executor`,
  `a_second_executor_pinned_across_the_recheck_decision_carries_the_receipt`), and the launch tests that counted
  forks now say a
  helper per client (`clients_that_find_the_flusher_lock_held_fork_helpers_that_end_without_working`,
  `many_clients_fork_a_helper_each_and_exactly_one_holds_the_lock`).

### Design file

The owner's decision of 2026-11-04 on the helper count was placed in `doc/planning/cli-git-commits-never-fail.md` on
`fix/git-policy-cli-commits-never-fail`, commit `8c4ae77cd1`, after the paragraph of "Implementation obligations
the showing must
demonstrate (not new rules)", in the corrected wording.

### Gates at the tip

The full `native:test:model` run is 476 passed, 0 failed (`/var/home/user/temp/agent/r20/full-model.log`; nextest
reports four
leaky tests, the stalled children that their alarms end); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r20/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 64
covered, 17 narrower, 9 historical and 17 not yet.

## Round 21 state

Round 21 answers `commit-models-sol-15.txt` ("yes after named fixes" for the models and components' claimed
coverage; not approval of
the live protocol). Comments, tests and notes changed; the product behaviour did not.

### Corrections to the notes of rounds 19 and 20

- The ownership argument. "The holder took it after the client's receipt existed" was false: F can acquire and
  sweep empty, C then
  exposes R, C's helper loses to F, and F releases and rechecks. The real argument: a receipt visible before a
  failed acquisition is
  covered by the holder's next locked sweep or, when that sweep is past, by the holder's release and recheck, which
  lists it, takes the
  lock again or finds another flusher, and accounts for a forcing failure. The notes of rounds 19 and 20 above that
  say "took it
  after" or "lists after it took it" are superseded by this. The control is
  `a_receipt_exposed_after_the_last_empty_sweep_is_covered_by_the_holders_release_and_recheck` (F holds the lock at
  its last empty
  sweep through the new fault point `flusher:after-last-empty-sweep`; R does not exist yet; C's launch is
  acknowledged and its helper
  loses the lock, marked; F's recheck forces R; every step is asserted).
- The acquisition-site inventory, production: the reacquisition of the recheck in `flush_held` (it loops to a sweep),
  `flush_lane_with` (it calls `flush_held`), and the helper's `prepare` in `launch_flusher` (its entry calls
  `flush_held`). No other
  code takes the flusher lock. Not production holders: the lock a test takes without sweeping (for example in
  `clients_that_find_the_flusher_lock_held_fork_helpers_that_end_without_working`), and the `legacy` probe seam,
  which reproduces the
  removed probe for negative controls and exists only in the test-seams build.
- The round 20 probe controls did not establish their interleavings: the probe-owner callback reacted to an event
  the fixed code never
  emits, waits returned booleans nobody read, and child results were dropped, so a passing run could be an ordinary
  successful sweep.
  They are replaced. The negative controls run the removed probe through the `legacy` seam and assert every
  coordination step (D
  holds its probe, the helper loses to it, E's launch ends, D dies) and then that the receipts are never recorded
  (`the_removed_probe_loses_a_receipt_when_its_owner_dies_before_launching`,
  `the_removed_probe_loses_the_receipts_of_a_helper_that_lost_to_it_and_of_a_client_that_saw_it`,
  `the_removed_probe_makes_a_recheck_hand_a_receipt_over_to_a_holder_that_never_lists`); the fixed-implementation
  counterparts assert
  that the receipts are recorded and that no probe exists
  (`the_fixed_launch_gives_a_client_an_executor_whatever_another_client_does`,
  `the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock`).
- "All three red" against "the first two red": the round 20 red log (`/var/home/user/temp/agent/r20/probe-red.log`)
  shows all three
  failing before the fix, but those failures are not evidence of the named interleavings for the reasons above.
  What stands now is the
  reproducible negative controls above, which lose the receipts under the legacy seam in every one of the three
  scenarios.
- Reap wording. The launcher waits for the intermediate child at most the report bound, then kills it and waits at
  most the kill-reap
  bound; a child not reaped then fails the launch (`IntermediateNotReaped`). There is no promise that it is reaped
  or of how long a
  residual child lives. The earlier "reaped, or killed and reaped, within the report bound" is withdrawn.
- Timing sample. Its endpoints are the `launch_flusher` call and the helper's acknowledgment (the map's "fork to
  acknowledgment" is
  corrected). It is descriptive, one run of 20 launches, not a bound and not a whole-command measurement.

### Live-cutover obligation (not a same-version defect)

An old probe-capable client that overlaps the new implementation recreates the original schedule: old D holds its
probe, new E
exposes R, E's helper loses to D and ends, D dies before it forks. Cutover must exclude such holders (a version
fence on the lane)
or give the receipts a recovery responsibility. Recorded as a not-yet row of the cutover group of the map (one row
added; it is an
obligation of the existing lifecycle and helper integration gate and not a new criterion; the counts below include it).

### Gates at the tip

The full `native:test:model` run is 479 passed, 0 failed (`/var/home/user/temp/agent/r20/full-model.log`; nextest
reports four
leaky tests, the stalled children that their alarms end); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r20/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 64 covered,
17 narrower, 9 historical and 18 not yet.

## Round 22 state

Round 22 follows the review of round 21 ("yes for the claimed, explicitly limited coverage; not approval of the
live protocol").

### Two limits recorded in the map

- The negative probe controls observe the absence of a record for 1.5 s only; an executor acts within milliseconds,
  which is
  observed and not proven.
- `the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock` does not force a failed recheck acquisition
  (the hook
  runs once and whether the recheck wins or loses depends on scheduling), so it is not deterministic evidence of
  that branch;
  `a_second_executor_pinned_across_the_recheck_decision_carries_the_receipt` pins it.

### Disposable Git experiments for the supported scope

Each runs on a throwaway repository under the build's temporary directory, in the capped scope of the model run,
with the real Git
(`/usr/bin/git`, version 2.56.0 here), on the files and the reftable backends unless noted, on a local file system.
They are in
`model/model_git_experiments_tests.rs` (21 tests). Results:

- Packed refs (`a_fifo_transaction_updates_creates_and_deletes_packed_refs`, files backend): after
  `pack-refs --all` every branch is
  in `packed-refs`; one transaction updates a packed ref, deletes another and creates a third, and the deleted ref
  is gone from
  `packed-refs` too. The reftable backend has no `packed-refs` file.
- Creation and deletion (`creation_and_deletion_through_a_transaction_check_their_expected_values`): a creation
  with the zero old
  value, a deletion with the right old value succeed; a deletion with a wrong old value is refused at `prepare`
  with a diagnostic
  naming the ref, no `prepare: ok`, and the ref stays.
- Symbolic references
  (`symbolic_references_are_created_updated_and_deleted_through_a_transaction_and_head_is_followed`):
  `symref-create`, `symref-update` and `symref-delete` work in 2.56.0 (`symref-delete` needs
  `update-ref --no-deref`, otherwise Git
  answers "cannot operate with deref mode"); `update HEAD` follows the symbolic `HEAD` to the branch it names and
  `HEAD` stays
  symbolic.
- The side-ref transaction (`a_side_ref_is_created_hook_free_and_makes_its_commit_reachable_for_a_prune`): an
  unreferenced commit is
  a prune candidate (control), a ref under `refs/cli-git/` keeps it out of an immediate-expiry prune view, the
  transaction with an
  empty `core.hooksPath` runs no `reference-transaction` hook, and the same transaction without it runs
  `preparing`, `prepared`,
  `committed`.
- A foreign lock on the branch
  (`a_foreign_branch_lock_blocks_the_branch_and_not_the_side_ref_on_the_files_backend_but_blocks_both_on_reftable`):
  on the files backend the branch's own `main.lock` refuses the branch update at `prepare` with a diagnostic naming
  a lock and does
  not block a side ref; on the reftable backend the lock is the stack's `tables.list.lock`, one for every ref, so
  it blocks both.
  Finding: "save on a side ref" cannot rely on a different lock under reftable; the design already makes the spool
  the guarantee
  and the side ref best effort, so nothing changes, and the behaviour is now measured.
- A torn final instruction (`a_torn_commit_instruction_at_end_of_input_never_commits`): the writer sent a prepared
  transaction and
  `comm`, then every writer closed; Git exits with an error that names the torn command, prints no `commit: ok`,
  and the ref stays.
- Publication evidence precedence (`the_ref_moves_before_the_commit_answer_is_written`): with a hook that blocks at
  `committed`, the
  ref already names the new commit while `commit: ok` has not been written, and the answer appears after the hook
  returns. The ref
  is the evidence and the answer line a report of it.
- Linked worktrees (`a_linked_worktree_shares_branches_and_keeps_refs_worktree_private`): a transaction run from a
  linked worktree moves
  a shared branch (even one the main worktree has checked out); `update HEAD` there moves the linked worktree's own
  branch; a ref
  under `refs/worktree/` (the scratch ref `refs/worktree/cli-git-service/<seq>` of the decision record) is visible
  only in the
  worktree that created it.
- The client-lock handoff (`the_transaction_child_inherits_no_client_lock_descriptor`): the transaction child holds
  no descriptor of
  the client's lock file, and the lock is free once the client lets go although the child lives.
- The submission handoff (`a_second_writer_keeps_the_transaction_alive_when_the_first_closes`): a second writer
  opens the FIFO, the
  first closes, the transaction stays alive and the second commits; only the last writer's close is an end of input.
- Interruption at the persistence boundaries by a kill of `update-ref`
  (`killing_update_ref_after_prepare_leaves_the_old_value_and_a_stale_lock_and_after_commit_the_new_value`): after
  `prepare: ok` the ref
  is unchanged and Git leaves the lock it held (files: `refs/heads/main.lock`; reftable:
  `reftable/tables.list.lock`), which a later
  transaction reports as a lock error; after `commit: ok` the new value is there and no lock remains. Git does not
  clear the stale lock
  itself. The design's pick for a ref lock left by an uncatchable kill (the lane never removes it; the person
  clears it) is
  consistent with this; nothing was changed.
- The earlier tests of the same file (the FIFO bootstrap, abort, end of input, buffered commit, `ENXIO`, hook
  phases, a hook refusal,
  stale and existing-ref diagnostics, the push exclusion) are unchanged.

Not shown, and why the row is covered only for this scope: Git versions other than 2.56.0 (the owner's decision:
the tests run on
the current Git and an older one only warns), a power loss (a kill is not a power loss; durability is the forcing
rows' business),
and a file system other than a local one.

The row "disposable Git experiments" is now Covered; the push-exclusion row stays Narrower (the marker the lane
acknowledges and the
stacked push path).

### Ordered plan

Written in `doc/planning/cli-git-commits-never-fail.md` on `fix/git-policy-cli-commits-never-fail`, commit
`a1308f6d8c`, section
"Ordered plan to the live protocol (round 22)": seven steps (the real state document; lifecycle and continuing
forcing at the real
call sites; cutover with a fence; recovery and preservation; installation; whole-command latency and notices; scope
and enablement),
each with what it builds, which rows its evidence closes and what it depends on. Four owner questions, none
decided: the real state
document and its representation, the launch route (manager-started unit or the fallback only), the cutover fence,
and the notice
texts.

### Gates at the tip

The full `native:test:model` run is 490 passed, 0 failed (`/var/home/user/temp/agent/r22/full-model.log`; nextest
reports four leaky
tests, the stalled children that their alarms end); `native:clippy` and `native:clippy:seams` are clean
(`/var/home/user/temp/agent/r22/clippy.log` and `clippy-seams.log`); the `source_guard` tests pass (39 of 39); the
map has 65 covered,
16 narrower, 9 historical and 18 not yet.

## Round 23: owner answers to the plan questions (2026-11-04T13:50:08-05:00 and later)

Recorded in `doc/planning/cli-git-commits-never-fail.md` on `fix/git-policy-cli-commits-never-fail` (commit
`e2df97729c`, whose time is 2026-11-04T13:50:08-05:00, and the correction `1b7c3d82a7`, whose time is
2026-11-04T13:53:51-05:00): decision 21 (the real state document, "Checked format (Recommended)"), decision 22 (the
launch route, "Systemd unit + fallback (Recommended)") and decision 23 (the cutover fence, "New lock name
(Recommended)"), each with the owner's words and the meaning as asked.

The supported-scope paragraph quoted in the earlier section of this handover (the one that ends "and the manager-started
transient unit launch route until it is built") is superseded: the manager-started route is now in the supported scope
as the primary launch route, an obligation to build and show (plan, step 2), and the double fork is the fallback where
no manager is present. The statements in this handover that the manager-started route "is not built" stay true as
statements about the build.

Question 4 (the notice texts) waits for the owner; the six drafts, in options A and B, are in the plan under Step 6.

### Step 1 built: the real state document in the checked representation

Owner decision 21 ("Checked format (Recommended)"). New module `commit_lane_state.rs` (the state fields) and changes in
`commit_state_log.rs`; the red controls are `commit_lane_state_tests.rs`, committed before the implementation (the red
state was a compile failure, since the representation did not exist; the commit says so).

Classification of the design's state document (the module documentation of `LaneDocument` says it too):
- References, derived from the lane and checked against the closure: requests, saved content, decision marks (a new
  `EntryKind::Mark`: an empty file `decision/<identity>.withdraw` or `.withdraw-unresolved`), the ledger extent and the
  adoption order extent (`order`, a second designated append-only path).
- State with no file behind it (`LaneState`): the marker, the logical head, the logical tail, the state and resume
  point and order position of every live request, and the consumption map. They are canonical and consistent with the
  references (a state for an unadopted request, a position outside the order extent, a head that is not an object
  name and an unsorted entry are not a document; a document is read only if re-encoding gives the same bytes), and
  they are not containment obligations: a reclaiming checkpoint compares `references()` with its closure.
- Not retained, as a decision: the spool and the attempt directories. The log never deletes them.
- Found and fixed on the way: a reclaiming checkpoint could never retire anything, because the non-drop check accepts
  only what the writer names and the reclaiming entry points named nothing. `begin_checkpoint_reclaiming_retiring` names
  it.
- The journal file replaces the order file as the example of a file outside the representation (the earlier fixtures
  would have stayed green and changed meaning).

Controls (red before, green after): `r23_a_full_real_state_document_round_trips_and_only_the_canonical_bytes_parse`,
`r23_a_request_position_beyond_the_order_extent_is_not_a_document`,
`r23_a_closure_lacking_a_decision_mark_the_lane_holds_is_refused`,
`r23_a_mark_with_content_is_not_a_mark_and_a_named_mark_must_be_in_the_set`,
`r23_a_mark_is_retained_until_its_request_is_discharged_by_the_ledger`,
`r23_a_closure_shorter_than_the_order_file_the_lane_holds_is_refused`,
`r23_a_document_naming_an_order_extent_its_set_lacks_is_not_admitted`,
`r23_a_changed_state_still_reclaims_and_covers_because_state_has_no_file_behind_it`,
`r23_a_reclaiming_checkpoint_compares_references_not_state`.

Guard removals (`/var/home/user/temp/agent/r23/mutations2.log`, each guard removed alone and the file restored): the
lane scan of marks, the order extent in the covering check, the lane scan of the order file, the references-only
comparison, marks in the covering check, the canonical re-encoding and the state consistency check each turn at
least one of the new controls red. A first run showed that the two order controls did not discriminate (the non-drop
check refused for them); they were rebuilt with no predecessor. Not shown to fail when removed: the mark and order
lines in `segment_is_contained`; the non-drop check already implies them, so they are defense in depth and the map
says so.

Rows: no status changed (65 covered, 16 narrower, 9 historical, 18 not yet). The two Narrower rows were rewritten:
the sentence "the real lane state document is not implemented" is replaced by what is built and what is not shown (no
caller writes the document at a real call site yet, which is plan step 2; the spool and attempt directories; the
protocol's rule for when a transition may drop a consumption entry or change a phase, which belongs to the models).
The rows "non-drop for later appends" and "captured authority valid through completion or verified substitution" stay
Narrower for those reasons. The rows "reclamation during capture" and "saved content survives until authorized
disposal" were Not yet and are Narrower after the models below. "Damaged established roots fail closed" and the others
of step 1 are not started: they need an establishment record or callers this step does not add.

Gates: full `native:test:model` 499 passed, 0 failed (4 leaky, the same stalled children as before:
`/var/home/user/temp/agent/r23/full-model.log`); `native:clippy` and `native:clippy:seams` clean
(`clippy.log`, `clippy-seams.log` beside it); `source_guard` 39 of 39.

Next: the rest of step 1 (the models for reclamation during capture and for disposal), then step 2 (the call sites,
where the first caller of the document appears).

### Step 1 models: reclamation during capture and the disposal of saved content

New models `model_capture_reclaim_r23.rs` and `model_disposal_r23.rs` with `_tests.rs` files, registered in `mod.rs` and
listed in `model_labels_tests.rs` (now 41 current names). Both use `model_observe::World` for visible and durable facts,
one power loss, write-back as environment steps.

- Reclamation during capture: a departure captures the acknowledgment of a record in the current segment, forces the
  descriptor it captured and returns on a durable cover; the owner writes a candidate that descends from the record
  or one that changed the document, forces it with its directory entry, and removes the old segment. Invariant: a
  returned departure always has a log that a power loss would leave covering the record. Controls, each red for its
  defect only: removal for any durable candidate, removal before the candidate is durable, return on a visible cover.
- Disposal: the person disposes explicitly; the owner appends the `disposed` line, forces it, removes the content and
  may checkpoint a root that no longer names the item. Invariants: no removal without the person's authorization; no
  durable root that names an item whose content is gone with no durable line; no durable drop without the line.
  Controls: removal before the line is durable, removal by retirement, root drop before the line is durable.
- Guard removals (`/var/home/user/temp/agent/r23/mutations-models.log`): ignoring lineage, ignoring candidate
  durability, removal without a durable line and root drop on a visible line each turn the green test red. The
  mutation "departure returns on a visible cover under the checked rule" stays green because the departure's forcing
  precedes its return; that is an equivalent mutant, recorded in the row.
- Map: "reclamation during capture" and "saved content survives until authorized disposal" moved from Not yet to
  Narrower (65 covered, 18 narrower, 9 historical, 16 not yet).

## Round 24: the step 1 review fixed, and plan step 2 built (2026-11-04)

The cross-vendor review of the step 1 bundle (`/var/home/user/temp/agent/reviews/commit-models-step1-sol.txt`) said "correct
after named fixes; not correct as is". Dispositions, by finding:
- A, ledger discharge proof: fixed. `commit_ledger::complete_entries` reads only a prefix of complete valid lines and fails closed on
  any damaged line or an unterminated tail; `ledger_prefix_has` uses it. Red first
  (`r23_a_ledger_fragment_without_its_newline_or_after_damage_is_not_a_discharge`). Any genuine terminal outcome discharges a mark.
- B, order positions: fixed. A position must be a line boundary whose line begins `adopt <that request> `, checked at admission, at the
  checkpoint and at recovery where the order file is read (`r23_a_position_must_be_the_adopt_line_of_its_request`). The state module's
  own check stays a range check and its documentation says where the line check lives.
- C, closure construction: fixed. `from_closure` returns `None` for a closure that names a path twice
  (`r23_a_closure_with_two_captures_of_one_path_is_not_a_document`).
- D, state replacement: documented, not enforced. The module documentation of `commit_lane_state.rs` and both Narrower rows say that this
  layer does not check transitions (`Install`, `MergeInstall`, `AttachWatch`, the marker) and name where they are checked (the models of
  the index replacement and the install). A `Suppress` entry may now outlive the envelope of its request; an obligation-carrying kind
  must still name an adopted request (`r23_a_suppression_may_outlive_its_request_and_an_obligation_may_not`).
- Finding 5, capture-reclaim model: rebuilt with root selection (an unrelated higher-generation candidate is the root and does not cover
  an old capture), the departure's recapture, and the ENOENT open of an unlinked path. Consequence recorded in the map: with recapture the
  lineage restriction on removal is green in the model, so it is not what keeps a departure's promise.
- Finding 6, disposal: fixed on the component side, in the direction the review recommended. The procedure is the disposed line, a durable
  replacement root that no longer names the item (its closure holds the forced ledger prefix), then the unlink
  (`commit_disposal.rs`; `removing_the_content_before_the_replacement_root_leaves_recovery_with_no_root` shows the wrong order on the real
  recovery). The model was rebuilt to that procedure.
- Finding 7: the visible-cover control is named a combined weakening; two disposal guards are equivalent mutants (removal needs both the
  line and the root durable); overlapping representation tests are named in the map row.
- Finding 8: the two Narrower rows name what is not shown.

Plan step 2, built as callables that nothing in the shipped commit path calls. Reachability pick: the shipped `cli-git-service` does
not schedule the syncer pass; the functions exist and are tested, and `git commit` reaches nothing new (plan step 3 is the cutover). The
only change to a shipped binary is the `--flush-lane <lane>` mode of `cli-git-service`, which runs the flusher entry and exits and is
reached only when that flag is passed.
- `commit_lane_lifecycle.rs`: `create_receipt`, `client_after_accept` (receipt first, ring a running service, otherwise launch),
  `syncer_pass` (retries the recorded errors by renaming the record to `receipt-errors.retrying`, merging unmatched lines back on failure),
  `startup_sweep` (one read of the state log, the same pass, takes up a dead syncer's copy), `seal_for_shutdown` (final pass, then a
  reclaiming checkpoint of `capture_lane_closure`, carrying the newest checked document's state and naming dropped requests as retired),
  `departure_acknowledge` (retries ENOENT and a root that does not cover, at most the caller's attempts).
- `commit_flusher_unit.rs`: decision 22. The primary route is `systemd-run --user --collect --quiet --unit=<fresh name>` waited for until
  the start is confirmed (no `--no-block`), running `cli-git-service --flush-lane <lane>`; the double fork is the fallback when the manager
  refuses, is missing or does not confirm within the service start ceiling (measured, `wait_ceilings.rs`). A fresh unit name per launch,
  so a refusal never means a flusher already runs. No `CPUQuota` or `MemoryMax` on the unit: no measurement of a flusher exists to cite.
  Decision 22 note (recorded by the coordinator, so nobody re-asks it): the route is `systemd-run --user`, not a D-Bus `StartTransientUnit`
  call, and that is within the decision. The question offered "Systemd unit + fallback (Recommended)"; the owner chose the route, not the
  call; and the manual of `systemd-run` (systemd 262 on this host) says it may be used "to create and start a transient .service or
  .scope unit". The repository's bus client marshals only string arguments. The single-thread requirement binds the fork only.
- `commit_drain.rs` and the register row `DRAIN-REQUEST`: the caller's bounded poll and its "draining, upgrade not applied" answer. No
  `WAIT_DRAIN` constant is shipped because it must be measured and no measurement exists.
- Tests: `commit_lane_lifecycle_tests.rs` (17, including the real user manager and the departure racing the real seal through the fault
  point after the capture), `commit_disposal_tests.rs` (4), `commit_drain_tests.rs` (3), the register test.
- Guard removals: `mutations-lifecycle-2.log`, `mutations-disposal.log`, `mutations-models-2.log` beside the step 1 logs. A first run
  showed three unproven guards (the ENOENT retry, the merge-back of unmatched record lines, the unit start wait); the controls were
  strengthened and all then turned red.

Rows: "hook outlasting an upgrade caller, the caller returns within its bound", "reclamation during capture" and "saved content
survives until authorized disposal" are Narrower; 65 covered, 19 narrower, 9 historical, 15 not yet. Not shown, left for the next steps:
the spool and attempt data that recomputation needs (durable request dependencies), a recovery that lets a root discharge a missing item
(deliberately not built), I/O errors other than ENOENT at the departure's forcing, the person's disposal command, the shipped service
scheduling the syncer, a real ring to a running service, the `WAIT_DRAIN` constant, and the establishment record for "damaged established
roots fail closed".

Gates: `native:test:model` 543 passed, 0 failed (4 leaky, the same stalled children as before); `native:clippy` and
`native:clippy:seams` clean; `source_guard` 39 of 39 (logs in `/var/home/user/temp/agent/r23/`).

## Round 25: the leftovers of step 2 and the second review of step 2 (2026-11-04)

The reviews of the step 2 bundle (`/var/home/user/temp/agent/reviews/commit-models-step2-sol-a.txt` and `-sol-b.txt`) said "correct after
named fixes". Dispositions:

Decision 22 note: the route is `systemd-run --user`, within the decision (recorded beside decision 22 in the design). The earlier
"deviation" wording of round 24 is withdrawn.

Part A, P1:
- The departure forces what the acknowledged record retains and confirms coverage afterwards (`departure_acknowledge`, `force_closure`);
  a lineage is carried across checkpoints (`an_acknowledgment_survives_two_successive_seals`). Fixed.
- Disposal versus a seal: `capture_lane_closure` never captures saved content whose `disposed` line is in the ledger, so no seal can name a
  retired item again; `Retired` is bound to its lane and state directory, consumed by the unlink, and checked against the current root first.
  The model has the schedule (`SealRescansRetired` red, `RecheckOnly` red because a recheck alone is a check-then-act race). Fixed.
- Receipt directory ancestry: the flusher forces the lane directory with the receipt directory. Fixed.
- Manager start is not executor confirmation: `Type=exec` makes a missing executable a refusal and the fork fallback follows; a unit that
  dies after its execution leaves its receipts for the next sweep, the flusher records its own lock failure (`flusher_entry` records every
  receipt, as the fork helper does), and the limit is recorded by a test (`a_unit_that_executes_and_dies_is_reported_started_and_leaves_the_receipt_for_the_next_sweep`).
  The failure owner is the next startup or syncer pass; nothing in the shipped service schedules one yet (plan step 3).
Part A, P2:
- Relative lanes are made absolute before the receipt is created and the unit launched. Fixed.
- Retry-file rotation: the rotation is gone. The syncer reads the record, forces every receipt, and empties the record in place under its
  exclusive lock only when it is still what the pass read; recorders append under the shared lock and skip names already recorded. A
  pre-opened recorder and competing passes are tested. Fixed.
- `startup_sweep` serves the receipts first and reports a state error beside them. Fixed.
- Waits: `wait_for_drain` clamps its pause to the remaining time (the probe must return promptly: a contract, documented); `run_within` bounds
  its cleanup; the drain tests bound by a factor of the bound. Fixed, with the contract for probes not enforced.
- Unit names use eight random bytes (the fallback is the clock mixed with the process identity, a collision makes the manager refuse and the
  caller falls back). Fixed; the mutation that removes the random source survives because the clock alone is distinct in the test.
- `--flush-lane` validates its lane (absolute, a directory that is not a link, with a receipt directory); `sync_path` opens without
  following a link or blocking and refuses anything but a regular file or a directory, so a FIFO receipt is a recorded error and not a
  hang. A wrong caller can have `flusher.lock` and `receipt-errors` created in a directory that has a receipt directory, and regular files
  under its receipt directory forced; the module documentation says so. The flusher tests that used a FIFO as a blocking device now hold a
  helper with a fault point. Fixed.
Part B:
- The capture-reclaim model keeps the selected root apart from the surviving one, has retained support, the ENOENT and orphan-descriptor
  branches and a finite attempt budget; its conclusions are qualified in the map (recapture is progress before a return, the non-drop rule
  and the durable publication keep the support after it; one replacement only, the file and directory force interleavings are excluded).
- `RemovesOnRootWithoutLineProof` is labelled a combined weakening; the two disposal equivalent mutants are equivalent only individually.
- The map corrections were made as listed (only DRAIN-REQUEST had moved from Not yet to Narrower in round 24).

The leftovers the coordinator named:
1. `WAIT_DRAIN`: not measured, not shipped. The shipped service has no drain request, so there is no real drain to run. The nearest
   measurable thing, the time from SIGTERM to exit of an idle `cli-git-service`, was 0.28 ms median (min 0.22, max 1.0, 20 alternations,
   host load average 4.94 at the start) against 0.28 ms median for a `sleep` process killed the same way: no difference, and no information
   about a drain behind a person's hook. The bound of a caller waiting for a hook is a policy value for the owner, not a measurement
   (`/var/home/user/temp/agent/r24/stop-latency.ts`).
2. Forcing errors (EIO, ENOSPC, EROFS), through a new single-shot sync-error seam: a failed file sync or directory sync is returned with the
   disk's number, never retried (after an `fsync` error a second one can succeed without the data being durable), and a seal that cannot
   force its closure leaves the older root and can be repeated. Tests in `commit_io_error_tests.rs`; the guard removal that retries every
   error turns three of the four red.
3. Crash injection at the line, the root and the unlink, with real recovery: `resume_disposals` finishes an authorized disposal from its
   durable `disposed` line (the startup sweep calls it); the line is written once. The crash is a real death of a forked process; it does not
   lose dirty pages.
4. The establishment record: plan step 1 owned it (the row was in its closes list); it is built because recovery is the first caller. A
   completed checkpoint writes `state.established` durably before its predecessors are removed, and recovery never falls back past an
   established root (`r24_an_established_root_that_is_damaged_does_not_fall_back_to_an_older_segment`, a torn record fails closed, a candidate
   that never completed still falls back). Every checkpoint now costs one more file sync and one more directory sync, in the background
   (the sync-count test names them). The row is Covered.
5. Spool and attempt data: what a recomputation needs is the inbox envelope alone (one checksummed file with the request record and both
   sides of every change); the spool of revision 11 and the attempt directories are not inputs. Found and fixed: any ledger line used to
   discharge a request envelope, contrary to the design's retention rule; an outcome that promises saved inputs (`withdrawn`, `conflicted`,
   `unresolved`, `rewritten-away`) now keeps the envelope until a `disposed` line, and only `published`, `refused`, `already` and
   `superseded` discharge it. A mark is still discharged by any terminal outcome.

Rows: 66 covered, 19 narrower, 9 historical, 14 not yet. Moved: "damaged established roots fail closed" to Covered; "retained bytes suffice
for recomputation" to Narrower (the recomputing component does not exist).

Not done: the shipped service scheduling the syncer; a real ring; a repeated seal race after verification in the model (one replacement
only); the person's disposal command; crash injection inside the replacement root's own steps; held-version completion (step 4).

Gates: `native:test:model` 577 passed, 0 failed (4 leaky, the same stalled children); both clippy tasks clean; `source_guard` 39 of 39
(`/var/home/user/temp/agent/r24/`).

## Round 26: plan step 3, the cutover fence and the service side behind a switch (2026-11-04)

Built on the models branch, nothing reachable from `git commit`:
- The fence (owner decision 23): the new protocol's lock is `flusher.v2.lock`; `flusher.lock` is `LEGACY_FLUSHER_LOCK`, taken only by a test
  seam (`hold_legacy_lock`) and by the legacy-probe seam. The two window controls of review 14 that lost a receipt to an old probe are now
  fenced (an old probe cannot make a new helper lose, and cannot take a recheck's handover); the old clients' own loss (probe against probe)
  stays as a negative control outside this protocol. Old and new flushers overlap harmlessly.
- The service side, behind `--lane <dir>` (and `--lane-interval-millis`) of `cli-git-service`: the doorbell (first an abstract datagram socket named
  by the lane; replaced in round 28 by a socket file in the lane directory), the startup sweep, and a loop that runs the syncer's pass on a ring and (round 28: at most one interval after the start of the previous pass, not "at least once per") every `SYNC_INTERVAL` (the service
  start ceiling, one minute; a pick, veto open). The periodic pass is the failure owner of a unit that died after its execution. The switch
  is the flag: the shipped unit files do not pass it, so the shipped service touches no lane; a test starts the real binary without it and
  shows that. Tests on the real binary: startup sweep, periodic pass (readiness read from the startup sweep, not from a ring), ring, the
  client step's real ring (`ring_running_service`).
- The nonce control forces a clock collision (`inject_frozen_clock`); with the random source removed it goes red. The `is_absolute` removal
  was rerun after a relative-path control was added, and is red.
- Guard removals: `mutations-step3.log`, `mutations-step3b.log` (nonce, lane path, startup sweep, periodic pass, ring, lock name).

What the rest of the leftovers belong to:
- the person's disposal command: plan step 6 (the commands and notices; the authorization has one constructor and its caller is the command);
- crash injection inside the replacement root's own steps (the candidate written, forced, named, established, the predecessor removed):
  plan step 4 (recovery and preservation, "crash injection at every step");
- successive replacements of the segment in the capture model: plan step 4 as well (preservation); the component test of two seals exists;
- `WAIT_DRAIN`: when a step makes the service handle a drain request, measure it on that real path with a person's hook; if no workload bounds
  it, it is an owner question with options, not a constant.

Rows: the cutover obligation is Narrower now (66 covered, 20 narrower, 9 historical, 13 not yet).

## Round 27: the third round of review of step 2 (2026-11-04)

Correction first: the sentence of round 25 "no seal can name a retired item again" was too strong. A seal that began capturing before the
`disposed` line existed could publish its earlier closure over the replacement root. It is closed now by serialization, and the claim is
this: the state writer is held by a seal from its capture to its publication and by a disposal from its line to its root; the unlink
revalidates and unlinks under the writer; capture never includes content whose line exists. The model has the split capture and publication
(`NoSerialization` is red, `RecheckOnly` is red, `SealRescansRetired` is red).

Part A (code), by finding:
- Recovery could unlink behind an uncompleted replacement (a visible candidate that omits the item is not a durable cover): the unlink now
  forces the selected root and everything it retains first (`recovery_forces_the_selected_root_before_it_unlinks_behind_an_uncompleted_replacement`,
  with a crash point between the candidate and its completion).
- The check and the unlink race: both happen under the writer (`the_state_writer_serializes_the_unlink_the_retirement_and_the_seal`).
- Empty closures left the state directory's entry unforced: an acknowledgment and a completion force the lane directory too. The expected
  event lists of three older tests gained that sync.
- Establishment checked a generation only: the record binds segment, generation and the full hash (64 hexadecimal characters); recovery
  refuses a segment older than the established one, a different first record at the same number, and a same-generation fallback past a
  completed checkpoint (`an_established_checkpoint_at_the_same_generation_is_not_replaced_by_the_older_segment`,
  `the_establishment_record_binds_the_segment_and_the_hash`). A completed later checkpoint rewrites the record, so the record always names the
  newest completed one.
- The retry record: compared by bytes under the exclusive lock (`a_new_record_of_the_same_length_survives_a_pass_that_read_the_old_one`).
  Two recorders may both add a line for one name; that is harmless and stated.
- Several pending disposals: the replacement retires every omitted item (`several_pending_disposals_recover_together`).
- Duplicate closure paths: refused at admission, at a checkpoint and when a record is parsed.
- The departure retries ENOENT in `state_covers` and in the forcing of support a disposal removed.
- Flush mode: the error record is opened without following a link or blocking and must be a regular file (a link is not written through, a
  FIFO neither hangs the flusher nor the syncer), and its shared lock is waited for at most `RECORD_LOCK_WAIT` (two seconds, a pick, veto
  open). A timed-out command that is not reaped in its window is handed to a thread that waits for it, so no zombie is left.
- Startup: the damaged-state test now proves the good receipt was forced and the state error reported.
- Not changed, stated: a post-exec death of a unit leaves its receipts to the next pass, which exists in the real service behind `--lane`
  and not in any shipped unit; the double fork still dies with a session scope; the ring in the older lifecycle tests was a callback, and
  the real ring is tested on the real binary (round 26); the manager tests skip without a user manager.
Part B (models and map): lineage text corrected in the map (it is carried across checkpoints, and still needed for the substitution relation
of `state_covers`); `ConfirmsAgainstDurableRoot` is named as not invariant-red; `NoNonDrop` is isolated and its counterexample needs no removal;
the capture model's support never undergoes an authorized removal (stated; the disposal model covers that); "damaged established roots fail
closed" stays Covered now that the identity is bound and the boundary tests exist.

Plan step 3 so far is in round 26. The full `commit_flusher.rs` is in the bundle of this round.

Gates: `native:test:model` 605 passed, 0 failed (4 leaky, the same stalled children); both clippy tasks clean; `source_guard` 39 of 39.
Guard removals: `/var/home/user/temp/agent/r25/mutations-review3.log` (ten, all red; the retirement-takes-the-writer-first guard was shown
red by hand).

## Round 28: the three reviews of step 3 (2026-11-04)

The reviews: the component tests (`commit-models-step3-sol-a2.txt`), the code against whole files (`-sol-a1.txt`) and the models, map and
handover (`-sol-b.txt`), all in `/var/home/user/temp/agent/reviews/`. Data safety first, then the doorbell, then the rest.

Data safety:
- The disposal revalidation hole (a1, 1): the unlink compared the stale token with the document's references only. A newer record whose
  document omits the item and whose retained set holds it (the public append admits an over-approximating set) was made ineligible by the
  unlink. The revalidation now reads the newest record's whole retained set, and recovery treats such a record as still retaining the item.
  Red first (`a_newer_record_that_retains_the_item_only_in_its_closure_refuses_the_stale_unlink`).
- The leaky tests were not all harness. Evidence (PIDs of the stragglers, their parent, their wait channel, and `coredumpctl`): the four
  stragglers were (a) children stalled on purpose by launch seams (`pause`, alarm after twenty seconds) that kept the runner's pipes, (b) a
  helper sleeping four hundred milliseconds that kept them too, and (c) a process in `do_coredump`: a **product defect**. The helper's report
  end was closed by number (`libc::close`) and then dropped as an `OwnedFd`, a double close that aborts a debug build (`SIGABRT`, a core dump
  per helper that returned, `coredumpctl` listed several) and in any build closes whatever descriptor reused the number. Fixed by one owner
  (`drop(report_write)`); the seam children now point their standard streams at `/dev/null` and their alarm is seven seconds, longer than the
  launcher's bounds. The full suite has no leaky test any more (624 passed, none leaky). Control: `a_helper_that_returns_from_its_entry_ends_with_its_code_and_does_not_abort`
  (a subreaper reads the helper's status); red when the double close is put back.
The doorbell:
- A blocking datagram send could wait for ever on a full queue. The send is non-blocking and a full queue is read as "rings are waiting"
  (`a_ring_to_a_receiver_that_does_not_read_never_blocks`, red when the send is made blocking again: the child hangs).
- The doorbell was a name in the namespace shared by every account. It is now `service.ring` in the lane directory, reached through the
  directory's descriptor (a socket path is limited to about a hundred bytes), bound over a stale socket only, refused over a live one or a
  foreign file, and one service owns a lane by an exclusive lock on `service.lock`. A lane must belong to the account and be writable by no one
  else (`validate_lane`), and the error record must have one name (a hard link is refused).
- A successful send is an enqueue, not an acknowledgment, and the recovery claims need a surviving or restarted service: a service that has
  died serves nothing until the next client's ring finds nobody and launches a flusher, or the next start sweeps. The claim "a receipt waits
  no longer than a pass interval" holds for a live service only. The interval is the spacing of the starts of passes while they are shorter
  than it; a longer pass is followed at once by the next. A zero interval, a doorbell that cannot be bound and a lane that is not valid stop
  the service before it is ready (exit 2), and interrupted disposals are retried on every pass.
The rest of a1: the establishment record is read before the scan and the first record's identity travels in `Selected`, so a healthy concurrent
seal is not "damaged"; a killed command that is not reaped in its window waits in a pending list until the fork that may follow is done
(a reaper thread before it would have failed the single-thread requirement of the fallback); state and lane files are read as regular files
without blocking (a FIFO named like a log no longer blocks recovery), and the error record is read to its extent with a limit. Not done, stated:
the flusher lock spans an until-empty sweep loop, so sustained arrivals can hold it (it does useful forcing meanwhile); the ring in
`client_after_accept` is a callback for the older tests and the real ring is tested on the real binary.
Component tests (a2), each with its guard removed and shown red (`/var/home/user/temp/agent/r25/mutations-round4.log`, `mutations-cutover.log`):
forcing failures of the selected root and of its closure with no unlink and the selected segment forced by its name; deterministic overlap of a
seal and a retirement (the hook proves the window was reached) and of an unlink with another writer; FIFO, link and hard link at the error
record at the open and at the clearing; the lock wait against `RECORD_LOCK_WAIT` with a stated allowance; a batch recovery with real
crashes, a sentinel and incomplete replacements; the disk parsers refusing a duplicate path and a refused append leaving the root;
departure hooks asserted fired; reaping with a zombie check; the real-binary tests wait for the service's own lines (a pass is reported
whether or not it found an error); restart between a ring and its pass; two services on one lane; concurrent legacy and new recording and
concurrent old and new flushers over arriving receipts. One mutation survived and was stated as redundant (the service lock against a second
service); that note is WITHDRAWN in round 29: the lock is what keeps two services that meet at a stale doorbell from both starting.
Models: the disposal model resets the volatile capture, writer ownership and read on a power loss; `RecheckOnly` and `SealRescansRetired` are
labelled alternative weakenings; the lock-lifetime abstraction is stated.

Gates: `native:test:model` 625 passed, 0 failed, none leaky; both clippy tasks clean; `source_guard` (below).

## Round 29: the two reviews of the step 3 fix round (2026-11-04)

The reviews: the diffs of code, tests, models and handover (`commit-models-step3b-sol-a.txt`, "correct after named fixes") and the whole current
files (`commit-models-step3b-sol-b.txt`, "correct after named fixes"), both in `/var/home/user/temp/agent/reviews/`. Both say the disposal
revalidation and the blocking datagram operations are fixed. Ordered as asked: data safety, then receipts, then waits without a bound, then the rest.
Every defect below has a red test committed before its fix, or a stronger control with its guard removed
(`/var/home/user/temp/agent/r26/mutations-round5.log`, `mutations-round5-b.log`).

Data safety:
- The establishment record (review B, P1): `establish` opened a fixed `state.established.tmp` with create and truncate, following links and
  accepting hard links, so a link or a hard link at that name truncated a file outside the state log, and a FIFO there blocked the checkpoint. It now creates a
  fresh exclusive name (a random nonce, `create_new`, `O_NOFOLLOW`, 0600) and renames that inode over the record; a failed write removes the temporary.
  Red first (`a_checkpoint_never_opens_an_existing_name_for_its_establishment_record`: the valuable file was truncated).
- `saved` as a link (review B, P1): a disposal read the item through a link and `remove_file` removed the outside name. Red first (the outside
  file was removed). `saved`, `inbox` and `decision` must be real directories of the lane (`open_lane_subdirectory`, `O_NOFOLLOW|O_DIRECTORY`): the capture, the
  retirement (before any line is written), the recovery listing and the unlink refuse a link, and the unlink removes the name relative to the
  descriptor it opened (`unlinkat`). `open_lane` refuses a lane whose `state`, `saved`, `inbox` or `decision` is a link or a file.
- An opaque selected root (review B, P2): the seal published a newer checked root with an empty state. The seal now refuses a root it cannot read, and
  the opaque root stays the root. The older test that expected the seal to succeed over an opaque record is replaced
  (`a_departures_acknowledgment_of_an_opaque_record_is_kept_because_the_seal_refuses_it`). Owner-visible: a seal over an opaque root is now an error.
- A defect the new control found (not named by either review): recovery of a disposal whose item a newer record retains in its closure only
  (review A's closure-only recovery control) failed on every start, because the replacement root did not name that item among those it retires.
  The retired list now includes the items the previous root held by its retained set. Red first
  (`recovery_finishes_a_disposal_that_a_newer_record_retains_only_in_its_closure`).

Receipts:
- The doorbell (review A, section 1; review B, P1): `ring_running_service` followed a link and trusted any socket, so a link to another lane's live
  doorbell, or a socket others can open, made the client return `Rung` and launch nothing. The entry is now checked through the same
  directory descriptor that is used for the send (a socket, owned by the account, mode private), and `bind_ring` applies the same check; an untrusted doorbell is an
  error, which `client_after_accept` reads as "nobody answered" and launches the flusher. Red first.

Waits without a bound:
- Opens (review B, P1/P2): `has_disposed_line`, `check_saved_content_sealed`, `create_receipt`, the retained-item force and the segment opens now open without
  following a link or blocking and judge the type from the descriptor (`open_regular`). Red first: a FIFO named `ledger`, a read-only FIFO named like saved content and
  a FIFO named like the receipt each blocked until the harness limit.
- Reads: `read_regular` reads to the length the file had when it was opened (a file that grows during the read is read as it was). `O_NONBLOCK` is all
  the open gives: it is no deadline on a read of a regular file. No size cap is added (a cap would be a shipped limit with no workload measurement;
  open for the owner).
- A service pass (review A and B): the pass drained the lane until a sweep found nothing, so receipts that kept arriving held the disposals after
  it and every later pass back. The service's pass is now one round (the receipts present when it began), says `more` when receipts arrived meanwhile, and the next pass is
  due at once (`next_due`); the one-shot flusher still drains until empty, because a launcher that finds the lock held relies on the holder to carry what arrives. Red first
  (the pass did not end). Not a bound on the time from a receipt to its forcing.
- The reaper (review A and B): the moved reaper fixed that launch, not later ones: a pending child that never ends kept a waiter thread alive and the next fallback was
  refused as threaded. There is no thread now: `reap_pending` polls the list with `try_wait`, and `launch_routed` polls before and after the manager start. Red first.

The rest:
- The lock and the lane boundary (review A, section 1 and 4; review B, P1): the lane is opened and checked once (`open_lane`: absolute, a directory that is not a link, owned
  by the account, not group or other writable, a real `receipt/`) and every later step of the service goes through that descriptor (`/proc/self/fd`), so a rename of the lane or of a
  directory above it after the check changes nothing the service touches (`the_real_service_works_on_the_directory_it_checked_even_when_the_name_leads_elsewhere_afterwards`).
  `start_lane_service` does the check itself; the public function is tested on a writable lane, a linked `saved` and a non-lane. `service.lock` is created 0600 and an existing one must be
  a regular file owned by the account, private, with one name. The directories above the lane are not checked: a directory above it that other accounts can rename into can
  still change what the name leads to before the check; `/var/mnt/encrypted` on this machine is mode 0777 without the sticky bit, so a walk would refuse the owner's own
  build directories (open for the owner: accept the precondition, or name a trusted root).
- `service.lock` is not redundant (review A, section 4): two services that meet at a stale doorbell can both start without it (the first removes the second's live socket). The earlier
  note that it was redundant is withdrawn; `two_services_that_meet_at_a_stale_doorbell_are_not_both_started` pauses the first between its probe and its removal (fault point
  `service:before-stale-removal`) and is red without the lock. The lock is held for the life of the service, which is a stronger lifetime than the race needs; it was kept because the lane
  has one service and a lifetime lock needs no second mechanism.
- Control repairs (review A, section 2): the bind probe is shown non-blocking on a full queue; the hard link is tried at the clearing; the selection control asserts a successful selection; the old and new flusher control asserts the new flusher is never `Held`
  and forced receipts; the closure-only recovery control; the volatile `checked` flag of the disposal model is shown set before the loss; the reaper wiring is shown through `launch_routed`.
  Not repaired, stated: legacy and new recording at the same moment is a stress and not a proof (the overlap of the two writers is not forced; the map says so).
  The fd-relative unlink is shown by a swap of the saved directory for a link in the window between the open and the removal.
- Documentation: the `FlusherLock` doc described a client probe that no longer exists; the lane service module doc no longer says a receipt waits no longer than a pass interval.

Guard removals (thirty-six planted, in two shards on two worktrees: `mutations-round5.log` and `mutations-round5-b.log`, and `mutations-round5-unlink.log`): thirty-four were red
at once. Two survived the first control and are stated. (1) The unlink by path survived: the `open_lane_subdirectory` check ahead of it already refuses a link, so only a swap inside
the window shows the descriptor-relative removal; the control was strengthened and the mutation is then red. (2) The blocking open in `force_item` survives and is not reachable by a
FIFO: `select` validates every retained item with `read_regular` before any force, so the item is refused first; the change is a defensive layer for a swap after the selection, with no
control that reaches it. Layered guards were removed one at a time and each fell to its own control, except the listing of `saved/` in `resume_disposals` (its check overlaps the capture
and the unlink checks), which was not mutated.

Limits stated: the ledger, order file and spool reads outside the state component (`commit_ledger.rs`, `commit_spool.rs`, `commit_inbox.rs`) still use `fs::read` and plain opens; they
were not changed in this round and the reviews did not name them. The service pass can still wait on the file system as long as the file system takes. The flusher lock files other
than the service lock are opened by path under the pinned lane. Live-cutover obligations are unchanged.

Gates: `native:test:model` 644 passed, 0 failed, none leaky; both clippy tasks clean; `source_guard` 39 of 39.
