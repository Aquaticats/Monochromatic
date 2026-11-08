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
  1. For hard,
     unsolved,
     or unfinished work the reviewer's `remaining_work` names a concrete next attempt that differs from the attempts already shown,
     and never tells the primary to stop,
     explain impossibility,
     or ask for a different task.
  2. The `ask_user_question` routing applies only to what only the user can supply,
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

### Unassisted attempt, running

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
  The final result is not judged yet.
- Acceptance criteria,
  fixed before the run.
  Satisfactory means all of these hold and the model's report matches reality:
  1. After `build`,
     `lint:types`,
     `test:unit` and `lint` of `//package/pi-plugin/goal` all exit 0 in the worktree,
     and the new test names appear in the unit test output.
  2. For a session that cannot reach the user,
     the reviewer is not told to ask the user or to use `ask_user_question`.
  3. For hard,
     unsolved,
     or unfinished work,
     the reviewer is required to give one concrete next attempt that differs from the earlier ones,
     and stopping,
     explaining impossibility,
     or asking for a different task is forbidden.
  4. For a session that can reach the user,
     routing to `ask_user_question` remains for what only the user can supply.
  5. The signal for "can reach the user" comes from the real session mode at the production call site.
  6. The new tests fail on the base without the model's source change.
  7. Scope:
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

## Next steps in order

1. When the wait notifies,
   read `out/summary_solo.json` and the model's report,
   then `git diff` in the solo worktree.
   Check by hand that no mutation is left in the source:
   `rg 'violations.length >= 0' src`,
   and read the prompt builder against its tests.
2. Run `build`,
   `lint:types`,
   `test:unit` and `lint` of `//package/pi-plugin/goal` in the solo worktree myself,
   each in its own capped `systemd-run --user`,
   and confirm the new test names.
   For criterion 6,
   set aside the non-test source change,
   rebuild,
   run the new tests,
   and expect failures,
   then restore.
3. Judge the solo attempt against the criteria and against the handheld one.
   Review the post-filter for false positives on legitimate guidance:
   a sentence that merely mentions "open problem" or "ask them" is replaced by the fixed instruction,
   and the reviewer's specifics are lost.
   The repository's rules also discourage magic lists.
4. Commit the solo attempt on its branch with explicit pathspecs,
   including the new files.
   Auto-push is on.
5. Measure both with the same Collatz rerun.
   Baseline:
   Collatz proof runs `f` (read only) and `h` (read and bash) in
   `~/temp/agent/pi-rpc-trial/out/goal-summary_f.json` and `goal-summary_h.json`,
   every denial cycle's `remaining_work` asked the user.
   New runs use the isolated home with `settings.json` `packages` listing the synthetic provider and the branch's
   `package/pi-plugin/goal` (rebuilt,
   so `dist/final/node/index.mjs` is current),
   the reviewer `synthetic/hf:openai/gpt-oss-120b`,
   once with no `--tools` allowlist and once with `--tools read,bash`.
   Success criteria,
   fixed before the run:
   zero `remaining_work` texts that ask the user,
   and more than one distinct approach across cycles.
   Run each arm twice for spread.
   Rerun `solve 3x + 1 = 0 and state the value of x` once,
   which must still be approved.
6. Update `doc/agent/pi-goal-extension-driven-through-rpc.md` with the results,
   the planning doc paragraph "Human decisions use `ask_user_question` inside the primary run" in
   `doc/planning/pi-goal-tool-free-completion.md`,
   and the matching line in `package/pi-plugin/goal/README.md`.
7. Decide which branch lands,
   or a combination.
   That is the owner's decision after the evidence is in.
   The user's live Pi keeps the old build until someone rebuilds the goal package in the main checkout,
   because `package/pi-plugin/goal/dist` is git-ignored (`.gitignore` line 53) and Pi loads `dist/final/node/index.mjs`.
   Say so in the final report.
   Do not rebuild in the main checkout before the change lands.

## Cleanup when finished

- `systemctl --user stop pi-goal-solo`,
  then confirm no `pi --mode rpc` process or `pi-goal-*` unit remains.
- Delete `~/temp/agent/pi-goal-agent-home/auth.json`,
  the copy of the credentials file.
- Remove both worktrees with `git worktree remove` after their branches are merged or dropped,
  then delete the branches.
- Delete `~/temp/agent/pi-solo-shim` when no run needs it.
- No stray files outside the worktrees:
  the first goal test left `/tmp/collatz.py`,
  so check `/tmp` and the throwaway fixture folder `~/temp/agent/dsh-rpc-trial/fixture` after every subagent run.
