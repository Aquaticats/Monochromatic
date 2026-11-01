# cli-git native replay core

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Lane `replay-core` of the native commit path (slice 8 of `doc/handover/cli-git-native-transactions.md`).
Branch `feat/cli-git-native-replay-core`, worktree `.claude/worktrees/cli-git-lane-replay-core`, based on `0faec78d77`.
Paths starting with `src/` are relative to `package/git-policy/cli/`.

## What was built

The lane ports the replay half of the incumbent commit path into `src/native/`,
without wiring it into the commit refusal (`Unported::CommitTransaction` stays).
The public entry point for the integrator is `replay_commit::replay_commit`,
which returns either the replay data of a clean merge or the data of a conflict.

Pure modules:

- `replay_patch` parses `git diff-tree -p` hunks over index-named trees, byte-exact.
- `replay_reverse_apply` is the strict reverse check, a port of Git's `apply.c` matching with its defaults.
- `replay_containment` is the one-sided extension of subsumption over zero-context hunks.
- `replay_object` parses raw commit objects, rewrites unsigned ones byte-exactly, lists dropped headers,
  and derives the identity variables of a signed rebuild.
- `replay_capture_order` parses the first-parent change listing and decides capture order per path.
- `replay_reservation` holds the grant order, the request bytes, the earned-count rule,
  and the reservation view; the landing lock is taken through a caller closure.
- `replay_revalidation` has the replayed index path, the shadow `HEAD` move, and the `pre-commit` re-run predicate.

Git-starting modules:

- `replay_shadow` runs one shadow Git command, with the caller's optional private index and input.
- `replay_shared_paths` lists paths both sides changed, with their three entries, in two `diff-tree` runs.
- `replay_subsumption_text` decides text candidates from blobs and three index-named trees
  (three `diff-tree -p` runs, three `mktree` runs, one blob batch).
- `replay_subsumption` decides identity and text subsumption after capture order,
  and writes the synthetic merge base (`read-tree`, `update-index`, `write-tree`, `hash-object`).
- `replay_capture_history` lists the first-parent history (two processes) and reads `captured.json`
  and the landed-capture records.
- `replay_merge` runs `merge-tree --write-tree --merge-base`,
   finds the first landed commit that touched a conflicting path,
  and writes the replayed commit: a raw rewrite when unsigned, `commit-tree -S` when signed.
- `replay_commit` composes the steps into one replay and returns `Replayed` or `Conflict` data.

## Commits

- `5f09b5e7e3` parse replay subsumption patches
- `1d2c0284df` port the strict reverse check of subsumption
- `7eb7ca62e4` port the one-sided extension of subsumption
- `b86cdb274d` port raw commit object replay rewriting
- `6922d836ad` duplicate of `b86cdb274d`, made empty by the retry described under Facts for the integrator
- `ed67604e8d` port capture order decisions of replay
- `8d984649d3` port the landing reservation of replay
- `2d8aac2af6` port shadow runner and shared-path listing of replay
- `ecb19f94ae` port the text half of replay subsumption
- `f7467f0bf8` port subsumption decisions and the synthetic merge base
- `7ae9a5b643` port capture-order history and shared-path decisions
- `a833b5d547` port the merge tree and the replayed commit rebuild
- `b8c0cb93a9` port the pure revalidation steps of replay
- `2f1a55a9d2` compose the replay result for the integrator
- `f5d5ce3713` close the replay lint findings and register replay planted controls
- `141ee73902` cover the replay acceptance cases for conflicts, modes, deletions, gitlinks and binaries
- `93e7531f68` list replay's shared paths once for subsumption
- `0e009022a2` name the replay tests that notice the planted guard removals

## Acceptance

### Unit cases of SPEC lines 3716 to 3727

Each case of the SPEC list maps to a test:

- Non-overlapping hunks in one file replay cleanly:
  `replay_merge::tests::non_overlapping_hunks_merge_into_one_tree`,
  `replay_commit::tests::non_overlapping_hunks_replay_onto_the_target`.
- Prepared bytes holding the landed edit plus an adjacent own edit land as they are:
  `replay_commit::tests::an_adjacent_own_edit_replays_as_captured`,
  `replay_subsumption_text::tests::an_adjacent_own_edit_is_subsumed_by_the_one_sided_extension`.
- A far-apart edit is not subsumed and merges:
  `replay_containment::tests::a_far_apart_prepared_change_does_not_contain_the_landed_change`,
  and the far-apart path of `replay_subsumption::tests::the_scenario_settles_each_path_by_its_rule`.
- An adjacent edit without the landed one conflicts:
   `replay_commit::tests::an_adjacent_edit_without_the_landed_edit_conflicts`.
- Identical additions and additions keeping every landed line are subsumed, a rewrite is not:
  `replay_subsumption_text::tests::an_addition_on_both_sides_is_subsumed_against_the_empty_blob`,
  `replay_containment::tests::additions_against_an_empty_base_keep_every_landed_line`.
- Delete against modify conflicts either way round; a deletion on both sides is subsumed:
  `replay_commit::tests::delete_against_modify_conflicts_either_way_round`,
  `replay_commit::tests::a_deletion_on_both_sides_is_subsumed`.
- Binary blobs are subsumed only when identical:
  `replay_subsumption::tests::identical_gitlinks_and_binary_blobs_are_subsumed_only_when_identical`
  (identical subsumed, different not),
  `replay_subsumption_text::tests::a_binary_blob_is_never_subsumed_by_the_text_check`.
- A landed mode change merges three-way even when the content is subsumed:
  `replay_commit::tests::a_landed_mode_change_merges_three_way_even_when_the_content_is_subsumed`.
- An identical gitlink is subsumed and a different one merges without being read as a blob:
  `replay_subsumption::tests::identical_gitlinks_and_binary_blobs_are_subsumed_only_when_identical`.
- The strict reverse check agrees with `git apply --reverse --check` on seeded cases:
  `replay_reverse_apply::tests::strict_reverse_agrees_with_git_apply_on_seeded_cases`
  (120 seeded cases, both outcomes asserted to occur).
- An adjacent edit captured on top of another commit's edit replays as captured:
  `replay_commit::tests::an_adjacent_own_edit_replays_as_captured`, at library level (see Choices open to veto).

### Binary-level replay

The four replay binary-level controls run `replay_commit` against disposable real repositories with real Git.
The wrapper's commit path stays refused until integration, so these are not wrapper-binary runs.

- Non-overlapping hunks replay with `commit-replayed` data (`Replayed`: `onto`, `prepared_oid`, `from_base`, `oid`).
- Overlapping hunks give `concurrent-commit/replay-conflict` data
   (`ReplayConflict`: `winning_oid`, `paths`, `prepared_oid`),
  and the prepared commit cherry-picks onto the base:
  `replay_commit::tests::overlapping_hunks_conflict_and_the_prepared_commit_cherry_picks`.
- The reservation is granted after the configured lost races, with the request naming the count:
  `replay_commit::tests::the_reservation_is_granted_after_the_configured_lost_races`.

### Revalidation

Only the pure parts are implemented: the replayed index (`read-tree` into `replay-<r>/commit.index`),
the shadow `HEAD` move (`update-ref`, with `--no-deref` when detached), and the `pre-commit` re-run predicate.
Policy re-runs and the `pre-commit` hook run are the integrator's.

### Fuzz

No fuzz target was added. The TypeScript has no sidecar for the replay parsers:
`rg -n "fast-check|fc\.|property" src/policy-engine/commit-replay*.unit.test.ts src/policy-engine/commit-capture-order*.unit.test.ts`
matched nothing, and `cli.fuzz/fuzz_targets` has no replay target.
The seeded differential test is the parser's randomized check.

## Gates

### Tests

- Host, Git 2.55.0 (`/usr/bin/git`): `native:test:host -- replay_` passes, 103 tests.
- Container, Git 2.56.0, image `localhost/git-policy-native-test:replay`:
  the library passes 959 tests with 2 ignored, the other test binaries pass.
  The container gate exits 1 only at its Clippy step, on four findings outside this lane (see Clippy).
- Image identifier: the gate run prints `localhost/git-policy-native-test:replay`;
  the final identifier is recorded under the Final gate heading.

### Clippy

`native:clippy` and the container Clippy step report four findings, none in a file this lane touches:

- `entry_tests.rs:114`: `forward_after` is never used.
- `index_lock_wait_tests.rs:190` and `index_lock_wait_tests.rs:201`: missing `return` statement.
- `index_writer_coordination_tests.rs:192`: a `match` that can be `?`.

All four predate this lane: the files last changed at `0faec78d77`, the head this lane branched from.
Every file this lane added or changed passes Clippy with the gate's flags.
The lane did not edit those files, because they belong to other lanes.

### Mutation

Partial, and stopped as trailing work. The scoped campaign over the 14 modules
(`mutation:scoped`, cargo-mutants 27.1.0, `--timeout 90`) reached its mutant phase once, with 417 mutants:
299 caught, 16 missed, 33 unviable, 69 timeouts, and no rerun of the timeouts.
The 16 missed mutants are the ones the tests in `ebfd97cc83` target; no campaign has re-measured them.
The later campaigns stopped before a mutant total: one was killed by a drain signal, one exited 4 at the baseline,
and one was stopped by the coordinator during planted controls.

### In flight

The branch was migrated to the data-only `cli-git.config.jsonc` and merged with `main`; the container gate on that
merge (tag `replayG`) passed with 1812 unit, 137 binary and 1 consumer test and Clippy clean.
The mutation campaign ran 2026-10-15 from 10:06 to 11:38 EDT as the systemd user unit `cli-git-replay-campaign`
(driver `~/temp/agent/replay-campaign-20261015.ts`, logs `~/temp/agent/replay-mutation-P1.log` to `-P5.log`,
`--timeout 300`): 429 mutants, no timeouts, 15 missed (1 in `replay_patch`, 12 in `replay_reverse_apply`, 2 in
`replay_containment`); lanes P4 and P5 had none.
Dispositions:

- `replay_patch.rs:249` (the cursor after a trailing incomplete-line marker): killed by the cursor assertion in
  `incomplete_lines_lose_their_newline`.
- `replay_containment.rs:122` and `:157` (`&&` to `||`): killed by
  `own_lines_between_landed_lines_need_both_ends_anchored` and
  `a_prepared_change_shorter_than_the_landed_range_does_not_cover_it`.
- `replay_reverse_apply.rs:129` (`-` to `+`): equivalent. With the end anchor only the position
  `image.len() - preimage.len()` passes `matches_at`, and the widening search reaches every position, so the starting
  position cannot change the result. The `/` mutant divides by zero for an empty preimage and is killed by
  `a_removal_only_hunk_reverses_by_restoring_its_lines_at_the_end`.
- `:135` (three mutants of the range guard): the guard duplicated the `usize` conversion and `matches_at`'s end check,
  so it was removed instead of tested.
- `:150`, `:180`, `:190`, `:100`: killed by the new reverse-apply controls (search backwards, restored lines,
  search start, middle line without a newline).

The controls and the guard removal are committed. Verification on the gate head (tag `replayI`: 1812 unit, 137 binary,
1 consumer test): `replay_reverse_apply.rs` `find_position`, `reverse_hunk` and `matches_at` 54 mutants (2 missed before
the long-image control was added), `replay_containment.rs` and `replay_patch.rs` 52 mutants with none missed, and a
`find_position` rerun of 17 mutants with the single equivalent survivor at line 129 described in the Mutation section.
Open: landing and the crate release 0.1.5 together with the deletion of the dead policy patch modules already on `main`.
The container gate image bakes in the source snapshot, so a new test needs a new gate before a mutation run sees it.
A long fuzz campaign (900 s per target, 15 targets) runs as the unit `cli-git-fuzz-campaign`
(log `~/temp/agent/fuzz-campaign-20261015.log`); read it for crashes before the release.
The fuzz sidecar had fallen behind the subject (stale lockfile version, byte paths in the journal helper, a refusal
control for linked-worktree creation that the executable replaced with the copy lifecycle); all three are fixed on
`main`. Every crate version bump must also run `mise run //package/git-policy/cli.fuzz:lock:members`, or the fuzz
build stops at `cargo vendor --locked`.

### Remaining after drain

1. Read `~/temp/agent/replay-core-container-4.log` and `~/temp/agent/replay-core-mutation-2.log`. Confirm that
   every planted control is noticed and that the container tests pass.
2. If the campaign finished, record its caught, missed, unviable, and timeout totals here.
   Justify each survivor with a code-based proof. Rerun any timed-out file alone and record the result.
3. Record the image identifier under Final gate.
4. If the campaign did not run, rerun it from the worktree with
   `GIT_POLICY_NATIVE_IMAGE_TAG=replay node <gate-slot path> mise run //package/git-policy/cli:native:test:container`
   and then
   `GIT_POLICY_NATIVE_IMAGE_TAG=replay node <gate-slot path> mise run //package/git-policy/cli:native:mutation:scoped -- <the --file list>`.
   The `--file` list is the 14 modules named in the `Mutation scope:` note.

Mutation scope: `replay_patch`, `replay_reverse_apply`, `replay_containment`, `replay_object`,
`replay_capture_order`, `replay_reservation`, `replay_shared_paths`, `replay_subsumption_text`, `replay_subsumption`,
`replay_capture_history`, `replay_merge`, `replay_shadow`, `replay_revalidation`, `replay_commit`.

## Trailing gates

The user decision of 2026-10-11 moved lane mutation campaigns behind the cutover.
This section records what ran, and the commands to resume.

Ran:

- Container gate at `ebfd97cc83`, the last code change (`GIT_POLICY_NATIVE_IMAGE_TAG=replay`):
  library tests 973 passed, 2 ignored; the other binaries passed. Clippy failed only on the four pre-existing findings.
  Image `localhost/git-policy-native-test:replay`,
  identifier `c39103e37cf37772b99d9ccef9b7a71fe949ddca4eff455b33fb7343b67384e6`.
- Planted controls in the campaign run at `ebfd97cc83`: 15 of 15 noticed.
- Mutants, the third campaign (`native-mutation-y5KnYD`, before `ebfd97cc83`):
  417 found, 299 caught, 16 missed, 33 unviable, 69 timeouts.
  The timeouts were not rerun, so the campaign exited 3. The timeouts are 27 in `replay_patch.rs`,
  27 in `replay_reverse_apply.rs`, 8 in `replay_reservation.rs`, 2 in `replay_containment.rs`,
  2 in `replay_merge.rs`, and one each in `replay_capture_order.rs`, `replay_revalidation.rs`,
  and `replay_subsumption.rs`.
- Survivors killed by tests:
  - `replay_patch.rs` 73:9, the `Display` impl of `ReplayPatchFormatError`:
    `replay_patch::tests::the_format_error_displays_its_message` (`fd054ff503`).
  - `replay_patch.rs` 293:23, the `<` to `<=` length guard in `find_bytes`:
    `replay_patch::tests::a_needle_equal_to_the_haystack_is_found` (`fd054ff503`).
  - The other 14 missed mutants of the third campaign are in `replay_reverse_apply.rs`, `replay_containment.rs`,
    `replay_object.rs`, `replay_capture_order.rs`, `replay_reservation.rs`, and `replay_subsumption.rs`.
    Each has a targeting test in `ebfd97cc83`: `an_incomplete_preimage_line_does_not_match_a_longer_image_line`,
    `a_hunk_is_found_by_walking_forward_past_the_expected_line`,
    `a_suffix_match_needs_the_prepared_hunk_to_end_at_the_landed_end`,
    `a_header_without_a_value_keeps_its_whole_name`, `the_object_error_displays_its_message`,
    `an_equal_sequence_is_not_later_than_the_landed_capture`, `the_capture_order_error_displays_its_message`,
    `releasing_without_a_request_file_is_quiet`, `a_free_reservation_is_not_granted_to_a_younger_requester`,
    `a_non_regular_triple_takes_the_landed_mode`, and `the_exact_record_width_follows_the_present_entries`.
    The `+` to `*` mutant at `replay_reverse_apply.rs` 147:44 and the `&&` to `||` at `replay_containment.rs` 130:45
    are targeted by the forward-walk and suffix tests above; neither is measured as killed yet.
- Fourth and fifth campaigns: the fourth stopped with exit 4 at its baseline on
  `index_lock_holders::tests::a_real_child_holding_the_lock_is_found_on_linux`, a test outside this lane
  that depends on real child-process timing under host load. The fifth was stopped during planted controls.

Not run, and the exact commands to resume:

- The timed-out files, one at a time. The mutation task has no timeout option; its `--timeout` is fixed at 90 s
  in `bin/mutate-native-container.mjs`. Changing it to 300 s would edit a shared gate, which this lane does not do.
  Command:
  `GIT_POLICY_NATIVE_IMAGE_TAG=replay node <gate-slot.ts> mise run //package/git-policy/cli:native:mutation:scoped -- --file src/native/<file>.rs`
  for each file named in the `Mutation scope:` note.
- The full scoped campaign over the 14 modules:
  `GIT_POLICY_NATIVE_IMAGE_TAG=replay node <gate-slot.ts> mise run //package/git-policy/cli:native:mutation:scoped -- <the --file list in ~/temp/agent/replay-core-mutation-files.txt>`.
  Run it after the container gate, with the baseline failure above in mind.

The `git apply --reverse --check` differential stays in the lane's gates. The interop matrix and fuzz campaigns
remain deferred as before, and the non-UTF-8 path field gate stays recorded under Choices open to veto.

## Deviations from the plan

- `subsume_landed_changes` takes the shared-path list from its caller.
  The first version listed the paths itself, which made four `diff-tree` listings per replay where the SPEC allows two.
- The reservation request bytes follow the TypeScript object,
   not the empty marker SPEC line 2091 names (see Choices open to veto).
- The plan's Git-starting tests use real repositories through `test_support`, not the wrapper binary.

## Choices open to veto

- Non-UTF-8 output from `git merge-tree` or `git rev-list` fails the replay (strict decoding, as the incumbent's
  `TextDecoder` with `fatal: true`). The byte-preserving alternative keeps paths as bytes end to end and needs the
  landing lane's path type to change. Rejected for now because the incumbent refuses such output too.
- `reservation-request` contains the TypeScript object
   `{"schemaVersion":2,"state":"reservation-request","lostRaces":n}`.
  SPEC line 2091 calls it an empty marker. Both wrappers read the same file, so the TypeScript bytes are kept.
  The SPEC wording is stale and should be corrected to the object.
- Hunk ranges accept ASCII digits only. The TypeScript used `Number(text)`, which accepts an empty range start as zero.
  Git never prints such a range, so the stricter parse only changes behaviour on foreign patches.
- A signed rebuild takes the caller's global options as UTF-8 strings. Non-UTF-8 global options would be lossy here.
- Test files are not mutation targets; the mutation scope is the 14 implementation modules this lane added.
- Replay reports `from_base` as `None` for an unborn branch. The TypeScript event for that case was not checked.
- Revalidation's policy and hook re-run stay with the integrator, as the brief asked.
  `replay_commit` returns the replayed tree before revalidation, so the integrator must run it before landing.

## Facts for the integrator

- Commit `6922d836ad` is an empty duplicate of `b86cdb274d`.
  The first `git commit` call for that module timed out in the tool, but the commit had landed; the retry replayed it
  through the installed wrapper, which made the second commit empty.
  History was not amended, so the integrator sees two commits with the same subject.
- `capture_records`, `transaction_registry`, `owner_lock`, and `blob_batch` were used read-only.
  No function in those modules was changed.
- `acquire_reserved_landing_lock` takes a closure for the landing lock. The integrator passes the landing lane's
   acquisition.
- `replay_commit` expects the shadow git directory, the object directory with the alternates,
   and the transaction directory
  that already holds `captured.json`. Replays of a moved target use the same transaction directory
  with a new replay number.
- The planted controls registered in `bin/native-planted-controls.mjs` are additive:
  `replay_reverse_apply`, `replay_patch`, `replay_capture_order`, `replay_reservation`, `replay_object`,
  and `replay_subsumption`.

## Files outside the lane

- `src/native/lib.rs`: module registrations only, one block per module after `capture_records`.
- `bin/native-planted-controls.mjs`: six additive entries at the end of the controls list.

No other lane's module was edited.

## Final gate

Recorded after the final container and mutation runs; see the Mutation results section for the campaign outcome.
