# Planted controls and durable survivors: resume note

Branch `fix/git-policy-cli-planted-control-and-durable-survivors`,
worktree `.claude/worktrees/mutation-controls-0528`.

## Done

- Two stale planted controls repaired in `bin/native-planted-controls-entry.ts` and `bin/native-planted-controls-landing.ts`
  (commit subject `fix(git-policy-cli): the planted controls follow the guards that moved ...`).
  - Self-exclusion by identity and content:
    `from` is now the whole multi-line `if same_file(...) || is_wrapper_launcher(...)
    || identical_content(...) {` condition in `src/native/invocation/real_git_candidate.rs`,
    `to` keeps only the launcher test.
  - The worktree copy lock control pointed at `src/native/worktree_copy_lock.rs`,
    deleted by `ee30d244f6`.
    It is replaced by "worktree copy recovery leaves a live owner's journal" in
    `src/native/worktree_copying/worktree_copy_recovery.rs` (`if !journal_owner_is_dead(&journal) {` becomes
    `if false && ...`),
    named test `pending_check_leaves_a_live_journal_and_recovers_a_dead_one`.
- All 48 controls match exactly once and name an existing test (`node bin/native-planted-controls.unit.test.ts` passes;
  on main it crashes with ENOENT on the deleted file).
- Two tests for the `index_is_hardened` survivors in `src/native/landing/service_durable_tests.rs`
  (commit subject `test(git-policy-cli): control the index_is_hardened answer in both directions`).
  Red against `true`:
  both new tests;
  red against `false`:
  `a_landing_begun_with_the_index_named_reports_it_hardened_and_the_default_returns_when_it_ends`.
  Green on the real function.
  `index_is_hardened` has no production caller (only its definition and a handover mention).
- Gate passed:
  4720/4720,
  exit status 0,
  Clippy finished.
  Log `/var/home/user/temp/agent/gate/gpc-gate-planted-control-and-durable-survivors-20261104T070548-0500.log`,
  image tag `gpc-gate-planted-control-and-durable-survivors-20261104T070548-0500` (the unit has ended).
- `lint:types` and `lint:oxlint:paths` on the two edited `bin/` files are clean.

## Next

- Show the two edited controls noticed by a run,
  not only by the static check:
  run
  `mise run //package/git-policy/cli:native:mutation:planted` with `GIT_POLICY_NATIVE_IMAGE_TAG` set to the image tag above,
  as its own capped systemd unit.
  It was started once as unit `mutctl-planted-0528` and stopped at the owner's request after
  its first control (log `/var/home/user/temp/agent/mutation-controls-0528/planted.log`,
  no result recorded).
  No unit is running.
  The two controls to watch are "self-exclusion by identity and content" and "worktree copy recovery leaves a live owner's journal".
  If either is not noticed,
  fix its `to` text,
  not the test.
- Report to the coordinator:
  tip,
  controls fixed,
  the check of every other control,
  red and green test names,
  gate count.
