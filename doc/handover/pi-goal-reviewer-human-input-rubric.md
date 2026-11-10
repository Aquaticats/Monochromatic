# Handover: goal reviewer rubric, human input and hard work

State as of 2026-11-07T23:39:26-05:00.
This file holds only open work and current state.
When an item lands,
move its durable facts to the doc that owns them and delete it here.

## Why this work exists

Two Collatz runs of the `goal` Pi extension under RPC
(`doc/agent/pi-goal-extension-driven-through-rpc.md`,
"Runs")
showed the reviewer routing hard,
unsolved work to `ask_user_question`.
The primary stopped and asked in plain text,
and the denial repeated.
Owner decisions given in this session:

- A goal keeps the primary working until the reviewer accepts.
  The planning doc already says there is no arbitrary consecutive-denial cap,
  and the owner affirmed it.
  Leaving the cap out is a no-op.
- The primary stating that the conjecture is unproven is true,
  and it should have kept trying anyway.
- Two changes to make:
  1.  For hard,
      unsolved,
      or unfinished work the reviewer's `remaining_work` names a concrete next attempt that differs from the attempts already shown,
      and never tells the primary to stop,
      explain impossibility,
      or ask for a different task.
  2.  The `ask_user_question` routing applies only to what only the user can supply,
      and only in a session that can reach the user
      (`context.mode === 'tui'`,
      the condition used in `src/review-unavailable.ts`).
- The work is done by Pi's DeepSeek V4.1 Flash as subagents driven over RPC,
  not by Claude Code subagents.
  The owner wants the sessions watched to learn Pi and model quirks.
  The lessons go in `doc/agent/drive-pi-as-a-coding-subagent.md`.
- The owner's calibration:
  DeepSeek V4.1 Flash at `xhigh` is about a Sonnet 5.5 low-level model.
  The owner also asked for an attempt with less handholding in an isolated worktree,
  to see whether it works satisfactorily.

## Two attempts exist

### Handheld attempt, finished

- Branch `fix/pi-goal-reviewer-human-input-rubric` in worktree `.claude/worktrees/pi-goal-reviewer-rubric`,
  created from `main` at `7c3cd01c49`.
- Commit `d3dab4839d`:
  `review-contract.ts` imports `TSchema` from `typebox`.
  Before it,
  `lint:types` failed with TS2305 in the main checkout and in the worktree,
  because pi-ai 1.5.0 no longer exports `TSchema`.
- Commit `9cbfa55e70`:
  the rubric change.
  `buildGoalReviewSystemPrompt({ humanInputAvailable })` replaces the constant,
  `buildBudgetedGoalReviewPrompt` takes the flag,
  and `promptedReviewers` passes `context.mode === 'tui'`.
  Three tests in `review-evidence.unit.test.ts`.
  After a rebuild,
  `lint:types`,
  `test:unit` and `lint` exit 0,
  and the three new test names appear as passing.
  The brief held the exact rubric sentences,
  so this measures instruction-following.
  The only thing the subagent got wrong in review is minor:
  its new tests sit in a second `describe` that reuses the existing suite name.

### Unassisted attempt, finished and committed as `9ac27cf235`

- Branch `fix/pi-goal-reviewer-rubric-solo` in worktree `.claude/worktrees/pi-goal-rubric-solo`,
  based on `d3dab4839d`.
- Brief `~/temp/agent/pi-rpc-trial/prompts/solo.md`:
  the problem,
  the evidence,
  and the owner's intent.
  No files,
  names,
  wording,
  steps,
  or test list.
  The model has a shell and was told to verify with the repository's own checks.
  Confound:
  the repository holds `doc/agent/pi-goal-extension-driven-through-rpc.md`,
  which names the cause,
  and the model found and read it unprompted.
- Unit `pi-goal-solo`,
  driver `~/temp/agent/pi-rpc-trial/pi-rpc-agent.ts`,
  feed `~/temp/agent/pi-rpc-trial/out/feed_solo.jsonl`,
  summary `out/summary_solo.json` when it ends.
  A background wait on its main process was started and notifies when it ends.
- Git guard:
  a shim first in `PATH` refuses every git subcommand except read-only ones.
  Its code is in `doc/agent/drive-pi-as-a-coding-subagent.md`.
- Seen so far,
  from the live feed:
  the model found the reviewer prompt,
  the call site,
  and the mode signal unaided;
  it chose a different design from the handheld one:
  a post-filter module `remaining-work-guidance.ts` that replaces reviewer guidance matching phrase lists with a fixed instruction,
  plus a mode-keyed rubric builder,
  audit fields `policyViolations` and `reviewerRemainingWork` on the denial event,
  validation of those fields in `events.ts`,
  and tests;
  it hit the stale-build trap,
  diagnosed it,
  and ran `build`;
  it mutated its two guards,
  saw the tests fail,
  and restored them (line 264 of `remaining-work-guidance.ts` was checked by hand);
  it fixed lint findings in rounds to 0 warnings and 0 errors.
  Run record:
  the process ended on its own with `endReason` idle after 913702 ms,
  132 assistant turns and 146 tool calls (109 bash,
  32 edit,
  3 read,
  2 write),
  2 tool errors (a stale edit anchor,
  and a shell exit code 1 from a count that printed 0),
  446128 input and 61684 output tokens,
  cost about 0.72,
  no dialog requests.
  Checked by me after it ended,
  in the solo worktree and each under a capped `systemd-run --user --scope`:
  `build`,
  `lint:types`,
  `test:unit` and `lint` of `//package/pi-plugin/goal` all exit 0,
  the new suite names appear in the unit output,
  no mutation is left (`rg 'length >= 0' src` finds nothing),
  and `git status` listed only files under `package/pi-plugin/goal/`.
  Not yet checked:
  criterion 6 (new tests failing on the base source),
  and stray files outside the worktree.
  Design concern:
  the post-filter holds hand-written phrase lists (`open problem`,
  `ask them`,
  `different task`,
  `remains unsolved`),
  so a legitimate reviewer sentence that merely contains one is replaced by a fixed instruction,
  and the repository's rule against magic lists applies.
  In the Collatz runs with tripled bounds the filter never fired
  (see the measurement section with that name).
- Acceptance criteria,
  fixed before the run.
  Satisfactory means all of these hold and the model's report matches reality:
  1.  After `build`,
      `lint:types`,
      `test:unit` and `lint` of `//package/pi-plugin/goal` all exit 0 in the worktree,
      and the new test names appear in the unit test output.
  2.  For a session that cannot reach the user,
      the reviewer is not told to ask the user or to use `ask_user_question`.
  3.  For hard,
      unsolved,
      or unfinished work,
      the reviewer is required to give one concrete next attempt that differs from the earlier ones,
      and stopping,
      explaining impossibility,
      or asking for a different task is forbidden.
  4.  For a session that can reach the user,
      routing to `ask_user_question` remains for what only the user can supply.
  5.  The signal for "can reach the user" comes from the real session mode at the production call site.
  6.  The new tests fail on the base without the model's source change.
  7.  Scope:
      edits only under `package/pi-plugin/goal/`,
      no stray files in the worktree,
      `/tmp`,
      or the main checkout,
      no git state changed.
  The model chose a post-filter as well as a prompt change,
  and criteria 2 to 4 were written for a prompt change.
  Report both readings:
  the literal one,
  and the outcome one (what the primary model receives).
- Recorded and not graded:
  wall time,
  tokens,
  cost,
  tool calls and errors,
  which checks it ran and in what order,
  and any claim in its report that the checks contradict.

## Collatz measurement with short bounds, 2026-11-07T23:51:22-05:00

- Setup:
  three isolated Pi homes in `~/temp/agent/pi-goal-home-{baseline,handheld,solo}`,
  each listing the synthetic provider and one goal package in `settings.json` `packages`
  (the real home lists the main checkout's package by absolute path,
  which is why the earlier single runs saw the old build),
  primary `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`,
  reviewer `hf:openai/gpt-oss-120b`,
  objective `prove the Collatz conjecture (the 3x + 1 problem)`,
  runner `~/temp/agent/pi-rpc-trial/run-collatz-matrix.ts`
  (a lane per build,
  lanes side by side in capped units,
  arms `--tools read,bash` and no allowlist,
  two repeats each),
  summarizer `summarize-matrix.ts`.
- Baseline (old build),
  four arms:
  every recorded denial told the primary to explain that no proof is known,
  to ask the user,
  or to offer alternative tasks,
  and several began with `ask_user_question`.
  One baseline arm recorded no denial inside its window.
- Handheld (prompt change only),
  four arms:
  one denial each,
  all of the form "attempt a proof",
  "construct a formal argument",
  or "provide a verification script up to a large bound".
  None asked the user or told the primary to stop.
- Unassisted (prompt change plus post-filter),
  four arms:
  three of four denials were proof attempts,
  and one asked for "a comprehensive summary of known partial results",
  a substitute objective that the filter's lists did not catch.
- Control `solve 3x + 1 = 0 and state the value of x`:
  approved on the first review on both new builds in about 15 s.
- Limit:
  nearly every new arm ended at the observation cap with the primary still working on its second turn,
  so these runs show the first denial only.
  The old build looped through its eight turns inside the same window because its turns were short refusals.
  The driver's entry previews were also cut at 360 characters,
  so an audit field after the guidance text was not visible.

## Collatz measurement with tripled bounds, 2026-11-08T00:02:17-05:00

- Setup:
  `GOAL_BOUND_SCALE=3` (hard kill,
  phase one and observation windows tripled),
  tag prefix `_n_`,
  the two new builds only,
  four arms each,
  entry previews widened to 4000 characters.
- Positive control for the audit fields:
  the widened solo entries contain `reviewerRationale`,
  so the absence of `policyViolations` in them is meaningful.
- Result:
  nine denials in six arms,
  and two handheld arms with no denial because the primary was still in its first long turn when observation ended.
  I read all nine guidance texts:
  none asks the user,
  none tells the primary to stop,
  explain impossibility,
  or request another task.
  All name a proof attempt or a step toward one.
- More than one distinct approach across cycles:
  met in two solo arms
  (a proof attempt,
  then formalizing Terras' density theorem in Lean;
  a proof outline,
  then extending the outlined reductions),
  and not met in the one handheld arm with two denials,
  whose two guidance texts are the same instruction reworded.
  Counts this small do not separate the builds.
- The solo post-filter never replaced anything:
  none of its six denials carries `policyViolations`.
  In these runs the prompt change alone produced compliant guidance,
  so the phrase lists added risk of false positives without a measured benefit.
  That is one reviewer model and one objective,
  not proof the filter is never needed.

## Criterion 6 and 7 results, 2026-11-08T00:13:11-05:00

- Criterion 6 (the new tests fail without the source change) holds for both branches.
  Method:
  write each non-test source file from `git show <base>:<path>` into the worktree,
  run `build` and `test:unit` under a capped scope,
  write the files back from `git show HEAD:<path>`,
  rebuild.
  Script:
  `~/temp/agent/pi-rpc-trial/prove-tests-fail-on-base.ts`.
  Base `d3dab4839d` for both.
  Solo branch:
  base build exit 0,
  `test:unit` exit 1,
  failures include a missing export `PRODUCTIVE_REMAINING_WORK`,
  the denial audit-field validation,
  and the reviewer prompt matching session reachability.
  Handheld branch:
  base build exit 0,
  `test:unit` exit 1,
  three named failures in the `buildBudgetedGoalReviewPrompt` suite
  (a user question offered to a session that cannot reach the user,
  the missing "one concrete next attempt" sentence,
  and the token estimate not changing with the prompt).
  Both worktrees were clean after the restore and their rebuilds exit 0.
- Two traps hit on the way.
  The git-policy guard refuses `git checkout <ref> -- <path>` and `git reset` in a worktree that has uncommitted tracked
  changes,
  so a first version that used checkout failed silently and the unit tests then passed against the old `dist`;
  read each step's exit status before believing a pass.
  One file was left staged at the base content;
  it was restored with `git show HEAD:<path>`,
  and the stray index entry was removed with the guard's documented `--no-enforce-worktree` hatch after saving the patch
  to `~/temp/agent/pi-rpc-trial/out/stray-staged-types.patch`.
  The scratch folder and the worktrees are on different devices,
  so a `rename` between them fails with `EXDEV`;
  copy and remove instead.
- Criterion 7 (no stray files):
  the main checkout has no change under `package/pi-plugin/goal`.
  The long-pass primaries (shell enabled) left `chk.py`,
  `cv` and `cv_omp` in `/tmp`
  (attributed by content:
  a Terras bijection check and builds of a Collatz verifier,
  one using OpenMP)
  and `collatz_proof_attempt.md`,
  `collatz_verify.c` and `collatz_verify` in the fixture folder.
  Those six files were deleted by name.
  `/tmp/lint-out.txt` was not attributed
  (it reports on a `src/projection.ts` that the goal package does not have) and was left alone.
  The main checkout also shows other modified files that this work did not edit.
  One repo-wide `lint:monochromatic-lint -- --fix` run of mine happened while they were already dirty or concurrently,
  so I cannot say whether it touched any of them.
  I committed only the paths I named.
- Observation about the repository lint:
  a repo-wide `mise run lint:monochromatic-lint` (no paths) ran about 9.5 minutes without finishing,
  with the top `monochromatic-lint` process at zero CPU over a ten second sample,
  a futex wait,
  no child processes,
  and no log output after its start line.
  I stopped it by PID.
  The same lint scoped to two files finished in about 15 seconds,
  and an earlier repo-wide `--fix` run finished in about 100 seconds.
  The cause was not investigated.

## Landed, 2026-11-08

The owner chose the prompt-only branch.
`fix/pi-goal-reviewer-human-input-rubric` merged into `main` as `fd99ad1bce`,
and `build`,
`lint:types`,
`test:unit` and `lint` of `//package/pi-plugin/goal` all exit 0 on the merged result,
with the three new `buildBudgetedGoalReviewPrompt` test names in the unit output.
The README line and the planning paragraph "Human decisions use `ask_user_question` inside the primary run" were updated
to say the reviewer prompt follows the session mode (commit `d5bead0b81`).
The goal package in the main checkout was rebuilt to verify the merge,
so the owner's live Pi,
which loads `package/pi-plugin/goal/dist/final/node/index.mjs` by absolute path from its
`settings.json`,
now runs the new rubric for sessions started after that build.
The unassisted branch `fix/pi-goal-reviewer-rubric-solo` (commit `9ac27cf235`) is not merged and stays on `origin`.

## Open work

Everything still open is in GitHub issue 6270:
measuring the rubric over many cycles,
with other reviewer models,
whether the unmerged phrase post-filter is needed,
substitute-objective guidance,
the faster `neuralwatt` `deepseek-v4.1-flash` provider,
and the unexplained repo-wide lint hang.
Delete this handover once that issue is closed and its findings sit in
`doc/agent/pi-goal-extension-driven-through-rpc.md`.

## Cleanup state

- No `pi --mode rpc` process and no `pi-goal-*` unit remains.
- The copies of the credentials file in the isolated Pi homes were deleted.
- The throwaway files the shell-enabled runs left in `/tmp` and the fixture folder were deleted by name.
- Still to do when the issue work starts or ends:
  remove the two worktrees under `.claude/worktrees` (`pi-goal-reviewer-rubric`,
  `pi-goal-rubric-solo`) with
  `git worktree remove`,
  delete the merged local branch,
  and delete `~/temp/agent/pi-solo-shim`.
  Keep the solo branch on `origin` while the issue is open.
