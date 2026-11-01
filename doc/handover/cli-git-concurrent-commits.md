# cli-git concurrent commits handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Updated 2026-09-26.

## Landed

The feature branch `feat/cli-git-concurrent-commits` fast-forwarded `main` to `cf2bf70d2` on 2026-09-26
and was pushed;
main's wrapper was rebuilt from it.
Final verification on that head:
build,
types,
oxlint,
`test:unit` (3 consecutive green runs before the last merge,
145 passing suites after it),
`test:built:trust`,
container end-to-end seeds 1 to 3
(114 runs each,
0 failures,
documented skips only,
Git 2.55.0,
2.40.0,
and 2.39.5),
lifecycle latency budgets,
and the concurrent-commit benchmark.
`landing.reserveAfterLostRaces` defaults to 1 after repeated sweeps (veto open).

GitHub's ruleset reported one bypassed violation on the push:
commit `8ed5daa61` is unsigned and authored by the test fixture identity `T <t@example.invalid>`;
all 465 other commits in the range are signed by the owner.
Cause:
the slice 2 subagent exported a smoke-test `HOME` and `GIT_CONFIG_GLOBAL` (identity `T`,
 no signing) in one shell
and then committed in the real worktree from that shell;
no test or fixture wrote to a real worktree.
A correction comment is on the commit,
and issue #575 now covers this environment-override case.
Owner decision 2026-09-26:
leave the commit in `main`'s history with its correction comment;
do not rewrite or force-push `main`.

Cleanup done:
issue #571 closed;
the merged worktrees and the local and remote branches
`feat/cli-git-concurrent-commits*`,
`fix/cli-git-concurrent-commits-per-commit-cost-grows-with-history`,
`fix/cli-git-manual-push-scans-already-published-history`,
and `fix/cli-git-worktree-copy-stalls-and-blocks-linked-worktrees` are removed.
The first commit through main's rebuilt wrapper (`e06d3d98c`) was signed,
auto-pushed,
and left no transaction state.

## Goal

cli-git lets several `git commit` invocations against the same worktree and branch run at the same time,
and every one lands as its own sequential commit,
instead of all but the first failing on `index.lock` `EEXIST` (issue #560 was a real collision).

## Status

- Design grilling finished;
  the owner confirmed shared understanding on 2026-09-25.
- Canonical design record:
  [`doc/planning/cli-git-concurrent-commits.md`](../planning/cli-git-concurrent-commits.md).
  Every decision,
  the evidence behind it,
  rejected options,
  and the owner's recommendation calibration live there.
- Owner instruction after confirmation:
  do the changes in a new worktree,
  then merge to `main` after verification.
- Worktree:
  `/var/home/user/worktrees/cli-git-concurrent-commits`,
  branch `feat/cli-git-concurrent-commits`,
  created from `4044f7d40`.
  `mise run prepare:pnpm:install` completed there.
  cli-git skips ignored-state copying for worktrees created from the main worktree,
  so dependencies had to be installed separately.
  The worktree's `cli-git.config.ts` was byte-identical to the trusted main copy
  and was enrolled with `git cli-git trust --yes`.
  The `forbidden-strings` scanner binary had to be built there
  (`mise run //package/cli/forbidden-strings:build`)
  before commits passed the `security/forbidden-strings` policy.
- Decision record committed on the branch as `d322d083e`
  (`doc/decision/cli-git-concurrent-commits.md`).
- Implementation issue:
   #571.
- Implementation plan committed on the branch:
  `package/git-policy/cli/doc/concurrent-commits-implementation-plan.md`.
- The owner went to sleep on 2026-09-25 and asked for autonomous work,
  building both options when two look equally good.
  Agent-made decisions are recorded,
  veto open,
  in the decision record section "Implementation-time decisions".
- Running in parallel on the branch
  (each commits with explicit pathspecs and retries on `index.lock`):
  the `SPEC.md` and `README.md` rewrite,
  slice 1 (per-transaction journals and live-owner-skipping recovery),
  and the container end-to-end suite
  (expected to fail its concurrency scenarios until later slices land,
  which is its positive control).
- Slice 1 landed on the branch
  (`9f53f7857` through `ce4de39e8`):
  per-transaction directories under `<git-dir>/cli-git-transactions/<uuid>`,
  `owner.json`,
  live-owner-skipping recovery,
  and reflog nonce search.
  Unit tests and `test:built:trust` were green at its last commit.
  Known limit until slice 3:
  a dead unlanded transaction fails closed if another transaction later moved `HEAD`.
- Container end-to-end suite committed on the branch
  (`2be50139e` through `d2e6b0512`;
  results in `package/git-policy/cli/e2e/README.md`):
  `mise run //package/git-policy/cli:test:e2e:concurrent [--seed N] [--scenario a,b] [--git 2.40.0,2.55.0]`.
  Git 2.40.0 is the declared minimum
  (`merge-tree --merge-base`).
  On the slice 1 build all 19 baselines and the checker positive control pass,
  and every concurrency scenario fails as expected
  (`index.lock` `EEXIST`,
  `git add` exit 128,
  `gc --prune=now` deleting a held blob,
  hook-phase `SIGKILL` leaving locks).
  Follow-ups for the suite:
  kill markers for landing,
  ref-update,
  and index-install phases once implemented;
  binary and rename scenarios;
  a less timing-dependent shared-file start window;
  per-exit remote checks.
  Observed separately:
  the `linked-worktree-only` policy rejects lint-staged's `git stash` in a main worktree.
- Slice 2 landed on the branch
  (through `ab62a2cd2`):
  shadow preparation for every commit,
  the hook dispatcher shim,
  serial landing that reports `concurrent-commit/head-moved` instead of replaying,
  and recovery after `SIGKILL` at 11 phases.
  Build,
  types,
  oxlint,
  unit,
  and `test:built:trust` were green.
- Running in parallel,
  each in its own worktree:
    - slice 3 in the feature worktree:
      replay,
      revalidation,
      moving private-index objects into the shadow store,
      native semantics for hooks that restage files,
      e2e kill-phase markers,
      and syncing `SPEC.md` with slices 2 and 3;
    - slice 6 in `/var/home/user/worktrees/cli-git-concurrent-commits-index-lock-classification`
      (branch `feat/cli-git-concurrent-commits-index-lock-classification`):
      `core.lockfilePid` injection,
      foreign `index.lock` classification and waits,
      index-writer coordination,
      and the narrowed worktree-copy settlement lock;
    - slice 8 finished in `/var/home/user/worktrees/cli-git-concurrent-commits-single-flight-auto-push`
      (branch `feat/cli-git-concurrent-commits-single-flight-auto-push`,
      through `b6860a9e8`):
      single-flight auto-push,
      green on build,
      types,
      oxlint,
      unit,
      and `test:built:trust`.
  Both merge into `feat/cli-git-concurrent-commits` after they are green.
  Slices 4 (read sets and `inputs`) and 5 (reservation) follow slice 3.
- The slice worktrees and branches were first created as `ccc-locks`/`ccc-push`;
  the owner called the names non-descriptive,
  and they were renamed on 2026-09-26.
  The lock branch is published under its new name
  (slice 6 finished at `6143d837d`)
  and the old remote `feat/ccc-locks` is deleted.
  The auto-push branch is published under its new name
  and the old remote `feat/ccc-push` is deleted.
- Claude Code crashed twice on 2026-09-26 while running a wrapped manual `git push` of the renamed branch.
  An isolated rerun as a systemd user service showed the cause:
  the wrapper's `manual-push` scan listed the whole history for a ref the remote lacks
  and spawned one `git diff-tree` per commit concurrently
  (3256 tasks,
  1.3 GB peak,
  then `spawn /usr/bin/git EAGAIN` from `final-newline`).
  Code:
  `pushedCommits`
and `createManualPushCandidates` in `package/git-policy/cli/src/policy-engine/manual-push-candidates.ts`
(moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/manual_push_range.rs`).
  The owner ruled it wrong by design:
  the scan must never walk every commit.
  Fixed on branch `fix/cli-git-manual-push-scans-already-published-history`
  and merged into `main`
  (pushed as `f1f8d6640`;
  main's wrapper rebuilt):
  the scan excludes everything reachable from the destination remote's tracking refs,
  scans only the tip tree when nothing on the remote is known,
  and uses a fixed number of processes.
  Measured on the real repository:
  the new-branch push that failed at 3256 tasks and 1.3 GB now takes 5.7 s and 73 MB.
  `doc/troubleshooting/cli-git-tag-push-eagain.md` now records the root cause.
- While pushing,
  the released cli-git on `main` failed closed with `content-unavailable`
  because another session's commit transaction was live in the main worktree;
  slice 1's live-owner skip resolves that once the feature branch merges.
- Slices 3 and 6 were interrupted by the crashes and resumed from their transcripts.
- Slice 3 finished on the feature branch
  (through `0dbebe896`):
  replay,
  revalidation,
  shadow-store objects,
  hook-staged trees kept,
  phase markers,
  and the `SIGKILL` hang fix
  (the container's PID 1 never reaps orphans,
  so zombie lock owners looked alive;
  Linux states `Z` and `X` now count as exited).
  Container seed 1:
  4 of 66 runs failed.
- Owner decisions 2026-09-26,
  not yet in the decision record because a merge is in progress in the feature worktree:
    - Replay per overlapping path:
      if the captured bytes already contain the landed change
      (its diff applies in reverse to the captured version),
      land the captured bytes;
      otherwise three-way merge,
      and a conflict fails.
      This amends the Q4 answer
      after the trace scenario showed pure three-way merging conflicting on adjacent edits in a shared worktree.
    - Amending a published commit while others are in flight:
      the e2e checker exempts commits landed on the amended history from `remote-contains`
      and requires the non-fast-forward push failure to be surfaced with exit `0`.
- Slices 6 and 8 merged into the feature branch
  (`61d5622c8`,
  `d52991dc0`;
  integration fixes through `cb2ef918b`,
  including process-group teardown for fixtures).
  All checks green;
  container seeds 1 and 2 failed only the two scenarios the owner decisions address.
  Both decisions are now in the decision record (`ef003b7cf`).
- In progress:
    - feature worktree:
      subsumption replay,
      slice 5 reservation,
      and the amended-history checker expectation;
    - `/var/home/user/worktrees/cli-git-concurrent-commits-policy-read-sets`
      (branch `feat/cli-git-concurrent-commits-policy-read-sets`):
      slice 4 read sets and `inputs`.
  Slice 4 finished at `96d96533d`,
  all checks green.
  Its agent made 9 commits with `/usr/bin/git` after a `require-root` finding,
  skipping commit-time policies and auto-push;
  they were pushed through the wrapper afterwards and its manual-push policies passed.
  The owner chose a deterministic hook over more rule wording:
  issue #575.
  Subsumption replay,
  slice 5 reservation,
  and the amended-history checker finished through `5a863e43b`:
  container seed 1 passed 71 of 72 runs,
  seed 2 passed 70 of 72;
  only `concurrent-trace-replay` still conflicted,
  where a later capture rewrote lines an earlier in-flight commit had just added.
  Owner decision 2026-09-26:
  for a path both commits captured from the same worktree,
  the later capture wins
  (decision record,
  "Capture order").
  Capture order landed with the slice 4 merge
  (`f44b7e8d4` through `255de52b4`):
  container seeds 1 to 3 pass every scenario on both Git versions except documented skips.
  Open before merging to `main`:
    - `test:unit` flakes on the trust registry's recursive lock,
      which gives up after about 1 s even while its owner is alive;
      an agent in the feature worktree is applying the owner-liveness wait,
      recording the index-commit capture and checker decisions,
      and re-measuring the latency baseline.
    - A `git worktree add` from a linked worktree stalled after copying about 2 GB of ignored files,
      blocking every wrapped command in linked worktrees until an agent deleted the copy journal by hand.
      Investigation and fix in `/var/home/user/worktrees/cli-git-worktree-copy-stalls-and-blocks-linked-worktrees`
      (branch `fix/cli-git-worktree-copy-stalls-and-blocks-linked-worktrees`).
  Update 2026-09-26 evening:
    - Trust registry lock,
      test home isolation,
      and the untrust-overlap reader fix landed;
      worktree-copy fix merged (`6f613c26e`);
      Git 2.39.5 degrades by capability probe;
      feature head `da8ba33cf` passed unit (3 runs),
      `test:built:trust`,
      container seeds 1 to 3 (90 pass,
      24 documented skips,
      0 fail per seed),
      and latency budgets.
    - Per-commit cost growth was unconsolidated packs from `index-pack --keep` landings,
      because plumbing skips native `git commit`'s auto-maintenance;
      fixed on `fix/cli-git-concurrent-commits-per-commit-cost-grows-with-history`
      (344 ms per commit at 2000 packs,
      from 1062 ms).
    - Running:
      final integration
      (merge the maintenance fix,
      re-run the reservation sweep with the rule "tails within the band:
       lower median wins",
      benchmarks,
      full verification,
      documentation consistency),
      and removal of about 1351 stale trust records of deleted `/tmp` repositories
      from the owner's real registry after a backup (owner approved).
    - Issues opened this session:
      #572 (under-scoped verification),
      #573 (logger sink timeout noise),
      #574 (short-form names),
      #575 (absolute-path git bypass hook),
      #576 (logger logs under `node_modules`).
  After final integration:
  merge the feature branch into `main`,
  rebuild main's wrapper,
  and close #571.
- `SPEC.md` has no placeholders left;
  the shadow snapshots every real ref at invocation so hooks resolve tags and other branches.
- Private `HEAD` shape chosen from a disposable-repository prototype:
  a shadow repository with alternates to the real object store
  (decision record commit `0d19ff973` on the branch).
  Culled shapes and their evidence are in the decision record.
  A docs agent is replacing the `SPEC.md` placeholders to match.
  Slice 2 starts after slice 1 lands,
  because both change the transaction workspace files.

## Evidence produced this session

- Private `HEAD` prototype,
  hook environment probe,
  and `index.lock` holder research:
  summarized in the planning doc sections of the same names.
- Git hook quirks,
  with source traces,
  upstream filing decisions,
  and a prototype Git patch:
  [`doc/troubleshooting/git-hook-disable-switches.md`](../troubleshooting/git-hook-disable-switches.md)
  and
  [`doc/troubleshooting/git-private-admin-dir-hook-environment.md`](../troubleshooting/git-private-admin-dir-hook-environment.md).
- Planning doc commits:
  `4b0335b1b` through `c56b7e6c5` on `main`;
  troubleshooting docs `4044f7d40`.

## Next actions

1.  Review the `SPEC.md`,
    slice 1,
    and end-to-end suite results as they arrive.
2.  Choose the private `HEAD` shape from the prototype,
    then implement slice 2.
3.  Implement the remaining slices in the plan's order:
    per-transaction journals and recovery,
    private `HEAD` preparation with the hook dispatcher shim,
    landing critical section with replay,
    policy read-set tracking and `inputs`,
    starvation reservation,
    foreign `index.lock` classification and index-writer waits,
    single-flight auto-push,
    concurrent fixtures and benchmarks,
    and the container end-to-end suite replaying realistic workloads against fresh dummy repositories
    (see the decision record section "Container end-to-end verification";
    inherent to this design,
    omitted until the owner pointed it out,
    agent-behavior gap tracked in #572).
4.  Verify at the user boundary with the packed shadow-bin fixtures,
    then merge `feat/cli-git-concurrent-commits` into `main`.

## Open items for the owner

- Upstream Git report for the hook disable switches:
  all six filing constraints hold,
  but Git rejects mail that looks AI generated,
  so the owner must rewrite it,
  rerun the reproduction,
  and sign off before sending.
  Nothing was filed.

## Owner working instructions

- Work autonomously;
  when two choices look equally good and promising,
  build both and let the build show which is better.
- Use as many worktrees as useful:
  one per competing prototype,
  branched from `feat/cli-git-concurrent-commits`.
- Cull competing prototypes early once one clearly crowds out the others;
  record the culled option and the evidence in the decision record.

## Cautions

- Other sessions commit to `main` concurrently
  (AGENTS.md batches,
  issue 570);
  touch only this task's files.
- The shim executables must follow SCR:
  no shell scripts.
