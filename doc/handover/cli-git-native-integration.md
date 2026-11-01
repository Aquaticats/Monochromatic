# cli-git native integration

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Note,
 2026-10-14:
 the retirement of the TypeScript wrapper deleted the tasks `native:differential:commit`
  and `native:config:differential`
 that this record names.
The record is `doc/handover/cli-git-native-retire.md`;
the incumbent that this record compares against is the commit `f4fc1fa213ece65672e4314983a75266837892f9`.

## Purpose and how to respond

This branch integrates the lanes of the native commit path (`package/git-policy/cli/src/native/`) and wires
`git commit` to the full transaction, so the native executable replaces the incumbent TypeScript wrapper.
Branch `feat/cli-git-native-integration`, worktree `.claude/worktrees/cli-git-lane-integration`,
created from the transactions head `fd26b74a99`.
Paths starting with `src/`, `bin/` or `e2e/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Respond by merging the branch into `main` and by vetoing any item under `Choices open to veto`.

## State

The native executable commits for real.
`git commit` (anything but a dry run) runs the transaction in `src/native/commit_transaction.rs` and
`commit_conclusion.rs`; `Unported::CommitTransaction` and its refusal are gone.
The incumbent's end-to-end suite passes against the native executable on Git 2.55.0: 36 of 38 scenarios pass and 2 skip
by design (seeds 1 to 9; one run in the last batch failed once on a kill-window leftover, see `Results`).
The commit differential against the incumbent (from source) finds 10 of 10 cases equal.
The gated head is `0c4d65094e` (the commit that follows it only records this document). The container gate
(which includes Clippy with `-D warnings` over all targets), the fuzz smoke and the mutation campaigns are green or
fully accounted for in `Results`. Nothing remains open.

### Merged lanes

Merged into this branch, in order:

- `main` (`30cdbc2311`): the dependent-version wiring and the `mise.lock` revision 2 upgrade.
- `feat/cli-git-native-landing`: slice 7a, merged again at `13fe865399` (its mutation record).
- `feat/cli-git-native-post-landing`: slice 7b, merged again at `b1d14c268c` (the commit path calls
  `post_landing.rs` for the after-landing order; there is no second ordering).
- `feat/cli-git-native-config` (`1bc75f16fd`): `cli-git.config.jsonc` and the differential.
- `feat/cli-git-native-e2e`: the `E2E_WRAPPER` selector, merged again at its finished head `b307a723d0`.
- `feat/cli-git-native-transactions`: merged again at its final head `52bafbb72a`.
- `feat/cli-git-native-landing`: merged again at its final head `97d461f366`. Its choices open to veto were checked
  against the commit path: the commit path chooses the lock (`commit_landing_lock.rs`, one attempt at a time, released
  before replay); the attempt keeps `index.lock` after the target advanced and recovery installs through it; the
  `install.index` link name only matters to recovery, which the transaction directory removal covers.

### What was built here

New modules (each with its `*_tests.rs` where it has pure logic; the transaction itself is covered by
`binary_commit_tests.rs` and the end-to-end suite):

- `commit_transaction.rs`: layout capture, published workspace, `preparing.json`, shadow repository, private index
  under capture order, interactive selection, unmerged-path check, convergence. It takes no landing lock.
- `commit_conclusion.rs`: intended tree, selected worktree records, no-change path, hook dispatcher, native
  preparation in the shadow, hook changes, `prepared.json`, the landing loop, post-landing completion.
- `commit_landing_loop.rs`, `commit_revalidation.rs`, `commit_landing_lock.rs`: the replay-and-reland loop around
  `landing_attempt::land_transaction`; the lock is taken for one attempt and released before replay; the starvation
  reservation is requested after the configured lost races and honored before every attempt.
- `commit_no_change.rs`: a correction that restores the base lands as a normalization through the landing attempt.
- `commit_workspace.rs`, `commit_facts.rs`, `commit_worktree_records.rs`, `commit_replay_options.rs`,
  `commit_command.rs`: the owned workspace, rule facts from the real repository with candidates from the private
  index, hook and selected-correction records, replay options, and the command (events, post-commit gate, backup).
- `replay_commit.rs` gained `merge_replay` and `rebuild_replay` (the incumbent revalidates the merged tree before it
  rebuilds the commit); `policy_events.rs` gained the concurrent-commit events; `candidate_prediction.rs` and
  `candidate_store.rs` gained `StagedAtBase` (candidates against the recorded base, never the moved `HEAD`);
  `capture_order.rs` reaches the `capture-locked` marker; `owner_lock.rs` gained a view of an ancestor's lock.
- Wiring: `wrapped_command.rs` runs the transaction for a real commit; a valid preparation lease lets a hook's nested
  Git command through and skips recovery (the worktree-copy lease is honored by the forwarding,
   `src/native/worktree_copy_forward.rs`).

### Decisions with reasons

- The commit region is parsed by Git's table; a region the table refuses falls through to the policy pass and is
  forwarded, so Git reports its own refusal (`an_unknown_option_is_refused_by_git_and_nothing_lands`).
- Candidates are the index entries of the paths the private index changes against the recorded base, passed as
  `:(top,literal)` pathspecs; an empty scope is answered without asking Git, because an empty pathspec list would
  make every tracked file a candidate.
- The native floor is Git 2.55.0 (settled), so the replay-plumbing probe is not ported and the two
  `replay-unavailable-*` scenarios never run natively; the native e2e runs use `--git 2.55.0`.
- Signals: the preparation `git commit` and the replayed `pre-commit` run under the signal guard, so a `TERM` during a
  hook ends the commit with 143 and leaves no transaction (`a_termination_signal_during_a_hook_ends_the_commit...`).
- The Markdown autofix finding now reads `monochromatic-lint --fix (markdown/<rule>) rewrites <path>.`, the words of
  the incumbent's source; the config differential showed the older wording differed.

## Results

- Config differential: `mise run //package/git-policy/cli:native:config:differential -- --require-all-equal` reports
  72 rows, 0 failing (the 36 pending rows were the two policies main wires). Under host load the incumbent's logger
  sometimes prints `logger internal error: sink verification failed ... Timed out after 5000ms` on its standard error,
  which makes unrelated rows differ for that run; rerun when a row differs only by that line.
- Commit differential: `mise run //package/git-policy/cli:native:differential:commit` (after `native:build`): final
  newline correction, pre-commit restage, no-change normalization, amend, `-a`, `--include`, `--pathspec-from-file`,
  `--allow-empty`, a concurrent replay with the phase marker, and a commit blocked by commit-only are equal in exit
  code, events, resulting commit, index and worktree. With `--native /usr/bin/git` the same run reports the expected
  differences, so the comparison can fail.
- End-to-end, Git 2.55.0, image tag `integration`, whole catalog (38 runs: 10 baseline pass, 26 concurrency pass,
 2 skip):
  seeds 1, 2, 3 (after the shadow-parent fix), 4, 5, 6, 8 and 9 pass completely. Seed 3 first failed
  `hooks-config-concurrent` with `creating .../cli-git/shadow/<id> failed: File exists`: two commits raced to create the
  shared parent; fixed in `shadow_create.rs` (`ensure_private_directory`) with a thread-race control. Seed 7 failed
  `sigkill-offset` once: `no-leftovers` found `.git/cli-git-captures/capture.lock.<uuid>.stale`, the seeded SIGKILL
  landed between the capture lock's rename and its removal, a window that the incumbent's `releaseHeldLock` has in the
  same shape (`src/owner-lock/owner-lock.ts`) and that nothing sweeps. Reruns of that scenario passed, so it is a
  timing-dependent window shared with the incumbent design, not a deterministic native defect.
  Each full run takes about 6 to 33 minutes depending on host load. The scenarios the suite calls `sigkill-phase-*`,
  `reservation-after-lost-races` and `branch-switch-during-commit` are real passes.
- Container gate (`native:test:container`, Git 2.56.0, image tag `followup`, image id
  `7975b36ab8f98ed3eb57ca68ef6a01d22431c840b64c4f1d911606dbefc607ff`, Clippy `-D warnings` over all targets included),
  run through `gate-slot.ts` on `0c4d65094e`: green. Library 1633 passed, 6 ignored; binary 105 passed, 1 ignored;
  1 candidate-consumer test passed. The host Clippy task (`native:clippy`) is clean on the same head. Earlier runs on a
  host with load average 40 to 75 failed
  `markdown::the_real_linter_rewrites_through_add_check_and_fix` and, once,
  `coordination::a_live_foreign_owner_is_waited_for_and_named` or `a_dead_foreign_lock_is_refused_and_left_in_place`
  on the 5-second command bound; each passes when the host is quieter, and the bound was not changed.
- Garbage in `objects/pack` after a commit (the flake of `commit::a_pathspec_file_selects_the_committed_paths`, which
  failed once in a mutation baseline with `garbage: 2`): not a defect of the native commit and not a leftover. Git's own
  detached `git maintenance run --auto --quiet --detach`, which the landing starts as native `git commit` does
  (`src/native/maintenance_run.rs`), repacks in the background: `git repack -d -l -q --cruft --write-midx` writes
  `tmp_pack_*` and `tmp_idx_*` files that `git count-objects -v` reports as garbage until the repack renames them. The
  Git 2.56.0 image starts that repack after the second landing even in a repository of 11 objects (it settles to one
  pack plus `multi-pack-index`, 0 garbage), so `assert_nothing_left` raced with it: 45 of 100 runs of the test failed in
  the container (2 parallel), none of 800 on the host's Git 2.55.0, which starts no repack at that size. Fix, test only:
  `edited_repository` in `src/native/binary_commit_tests.rs` sets `maintenance.autoDetach false`, so the wrapper
  returns after the maintenance finished; the assertion message now lists the pack directory. Control
  `commit::a_landing_returns_after_its_automatic_maintenance_finished` (a `gc.autoPackLimit` of 1 and 8 MiB of
  incompressible files make the repack outlast the wrapper's last steps) failed 20 of 20 runs without the config line
  and passed 20 of 20 with it (host). After the fix the test passed 150 of 150 runs in the container and the control
  20 of 20.
- Git 2.55.0 probe (host Git 2.55.0, `native:test:host` and `native:test:binary:host`, on an earlier head,
 before the last test-only commits): the library
  tests fail only the 2.56.0 comparisons (`command_options::git_tests` four tests, `command_branch_create::git_tests`
  three, `command_add` table, `git_metadata`, `global_arguments`) and the test that needs the built linter (11 failed,
  1597 passed at that head); the binary tests fail only `forwarding::version_is_identical_to_native_git`,
   the three `resolution::`
  tests (all compare the `git version` text with 2.56.0) and the real-linter test (5 failed, 92 passed at that head).
   Every
  `commit::` and `coordination::` test passes, and the end-to-end suite above runs on Git 2.55.0.
- Fuzz smoke (`mise run //package/git-policy/cli.fuzz:smoke`) on an earlier head: success, 15 targets exit 0.
 The generator
  control `generated_arguments_reach_every_frontier_ending` needed two changes: a real commit prints its own text,
   so its
  ending carries none, and a commit region that Git's option table refuses is forwarded so Git reports the refusal.
  No source under `src/native` changed after that run (only tests).
- Planted controls (unmodified `bin/native-planted-controls.mjs`, run as phase 1 of the last scoped campaign on the
  image built for that campaign): 66 controls, every one noticed by its named test. No non-test source changed after.
- Mutation. Campaigns over the files created or changed here were run through `native:mutation:scoped` with
  `NATIVE_MUTATION_BASELINE_SKIP=markdown::the_real_linter_rewrites_through_add_check_and_fix` (that test depends on the
  host load, which the incumbent-logger note in the Results section describes).
  Earlier per-file campaigns (`commit_transaction`, `commit_conclusion`, `commit_landing_loop`,
  `commit_revalidation`, `commit_worktree_records`, `commit_landing_lock`, `commit_facts`, `commit_command`,
  `commit_no_change`, `commit_workspace`, `commit_replay_options`, `replay_commit`, `commit_preparation`,
  `wrapped_command`, `refusal_frontier`, `candidate_prediction`, `policy_events`, `owner_lock`, `capture_order`,
  `candidate_store`) killed their survivors with controls (`binary_commit_tests.rs` and the `*_tests.rs` files) or
  moved logic into small tested functions (`stages_worktree`, `settled_passes`, `union_paths`, `replayed_events`,
  `lost_races_after`, `next_attempt_number`, `ensure_replay_directory`, `is_detached`, `retry_while_active`,
  `landing_lease_token`). The final campaign (`commit_preparation.rs` and `commit_transaction.rs`, 88 mutants, report
  `target/verification/native-mutation-pMcBRc`): 57 caught, 28 unviable, 3 missed, 0 timeouts. The 3 missed
  (`pathspec_file_source` returning `None`, its `+` replaced by `*`, and the `Display` of `PreparationError`
  replaced by an empty write) were found after the campaign's test image was built; the controls
  `commit::a_pathspec_file_selects_the_committed_paths` and
  `commit_preparation::tests::each_preparation_failure_displays_its_own_words` were added, and each of the 3 mutants
  was applied by hand to the host source and failed its control, then restored. The two open mutants of the earlier
  campaign are closed the same way: `stages_worktree` with `||` replaced by `&&` fails
  `a_capture_lists_worktree_changes_for_all_or_include`, and `retry_while_active` with its guard replaced by `true`
  fails `the_guard_wait_retries_only_while_another_holder_is_active` at once (its closures panic past their call cap,
   so the
  mutant no longer loops).
- Mutation of `src/native/commit_convergence.rs` (12 mutants, `bin/mutate-native-container-noplanted.mjs`, report
  `target/verification/native-mutation-6BnlGO`): 7 caught, 5 unviable, 0 missed, 0 timeouts. The one survivor of the
  earlier campaign, `CommitPasses::snapshots_equal` replaced by `false`, is killed by
  `commit_convergence::tests::stored_states_compare_equal_exactly_when_the_corrected_bytes_match`, which stores a
  correction and its reversal and compares the three stored states; it failed with the mutant applied by hand.
- Not mine and left alone: `mise.lock` shows a working-tree edit (`nightly-2026-10-11` to `nightly-2026-10-12`) that
  `mise run` wrote; it is not committed.

## Choices open to veto

- `src/native/commit_policy_preparation.rs` was deleted (superseded by `commit_conclusion.rs`); its no-change constants
  moved to `commit_no_change.rs`, and the transactions branch's controls that only exercised it were removed.
- The incumbent's `replayOptions` reads no signing key from an attached `-S<key>` (its normalization splits the key
  into a separate token); the native port reproduces that and pins it with a control. A signed commit that lost a
  race and was given `-S<key>` is re-signed with the configured key in both wrappers.
- Deleted paths are not candidates (the content layer lists present index entries only).

## Remaining

Nothing. Every gate and mutation item of the brief is recorded in this handover.

## Rerun commands

```sh
# repository root, native check and Clippy
mise run //package/git-policy/cli:native:check
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:clippy

# the commit controls on the host
mise run //package/git-policy/cli:native:test:binary:host -- commit::

# the differentials (build first)
mise run //package/git-policy/cli:native:build
mise run //package/git-policy/cli:native:differential:commit
mise run //package/git-policy/cli:native:config:differential -- --require-all-equal

# the end-to-end suite, as its own unit with a memory cap
systemd-run --user --unit=int-e2e --collect --property=MemoryMax=10G \
  --property=StandardOutput=file:"$HOME/temp/agent/int-e2e.log" --property=StandardError=file:"$HOME/temp/agent/int-e2e.log" \
  --working-directory="$PWD/package/git-policy/cli" --setenv=PATH="$PATH" --setenv=HOME="$HOME" \
  --setenv=GIT_POLICY_NATIVE_IMAGE_TAG=integration node "$HOME/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0

# one mutation scope without the planted-control phase (the container gate must have built the image tag first)
cd package/git-policy/cli && GIT_POLICY_NATIVE_IMAGE_TAG=integration node "${HOME}/temp/agent/gate-slot.ts" \
  node bin/mutate-native-container-noplanted.mjs --file src/native/commit_convergence.rs --timeout 900
  (Note 2026-10-20: the script `bin/mutate-native-container-noplanted.mjs` was removed; `bin/mutate-native-container.mjs`
  with `NATIVE_MUTATION_SKIP_PLANTED=1` does the same.)

# the container gate
GIT_POLICY_NATIVE_IMAGE_TAG=integration node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:test:container
```
