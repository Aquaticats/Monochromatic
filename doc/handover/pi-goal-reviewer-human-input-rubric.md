# Handover: goal reviewer rubric, human input and hard work

State as of 2026-11-07T23:21:49-05:00.
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
  Change 3,
  leave the cap out,
  is a no-op.
- The primary stating that the conjecture is unproven is true,
  and it should have kept trying anyway.
- The two changes to make,
  in the owner's words "your 3 named changes are great.
  Do them":
  1. The reviewer rubric separates "only the user can supply this" from "hard or unsolved".
     For hard work `remaining_work` names one concrete next attempt that differs from the attempts already shown,
     and never tells the primary to stop,
     explain impossibility,
     or ask for a different task.
  2. The rubric omits the `ask_user_question` routing when the session cannot ask the user
     (`context.mode !== 'tui'`,
     the same condition as `src/review-unavailable.ts`).
- The work is done by Pi's DeepSeek V4.1 Flash as subagents driven over RPC,
  not by Claude Code subagents.
  The owner wants the sessions watched to learn Pi and model quirks.
  The lessons go in `doc/agent/drive-pi-as-a-coding-subagent.md`.

## Where the work is

- Worktree:
  `.claude/worktrees/pi-goal-reviewer-rubric`,
  branch `fix/pi-goal-reviewer-human-input-rubric`,
  created from `main` at `7c3cd01c49`.
  `node_modules` and `dist` were copied in by the git-policy worktree add.
- Commit `d3dab4839d` on that branch:
  `review-contract.ts` imports `TSchema` from `typebox`.
  Before it,
  `mise run //package/pi-plugin/goal:lint:types` failed with TS2305 in the main checkout and in the worktree,
  because pi-ai 1.5.0 no longer exports `TSchema`.
  After it,
  the type check exits 0.
  Unit tests passed before and after,
  0 failures.
- Task B,
  the rubric change and its tests,
  was handed to the subagent at about 82 s into its session
  and was still running when this was written.
  Brief:
  `~/temp/agent/pi-rpc-trial/prompts/task-b.md`,
  copied below in outline.
  Its exact rubric sentences are the owner-visible part of the change:
  - `humanInputAvailable` true:
    "Only when the remaining work needs something that only the user can supply (a decision,
    a missing requirement,
    a credential),
    remaining_work must directly instruct the primary model to use ask_user_question."
  - `humanInputAvailable` false:
    "The user cannot be asked in this session.
    remaining_work must never tell the primary model to ask the user,
    to use ask_user_question,
    or to wait for input;
    when a requirement is ambiguous,
    direct the primary model to choose the most reasonable reading and carry on."
  - Both:
    "When the work is hard,
    unsolved,
    or unfinished and the primary model can still make progress,
    remaining_work must name one concrete next attempt that differs from every attempt already shown in the evidence.
    It must not tell the primary model to stop,
    to explain why the work cannot be done,
    or to ask the user for a different task."
  - `GOAL_REVIEW_SYSTEM_PROMPT` becomes `buildGoalReviewSystemPrompt({ humanInputAvailable })`,
    `buildBudgetedGoalReviewPrompt` takes `humanInputAvailable`,
    and `promptedReviewers` in `src/review-selection.ts` passes `context.mode === 'tui'`.
- Subagent unit:
  `pi-goal-impl` (user systemd unit,
  `RuntimeMaxSec=2500`),
  driver `~/temp/agent/pi-rpc-trial/pi-rpc-agent.ts`,
  feed `~/temp/agent/pi-rpc-trial/out/feed_impl.jsonl`,
  steers by dropping a file into `~/temp/agent/pi-rpc-trial/out/inbox_impl/`,
  stop by creating `out/stop_impl`.
  The driver is reproduced in `doc/agent/drive-pi-as-a-coding-subagent.md`,
  because `~/temp/agent` does not survive a reboot.
- Throwaway Pi home:
  `~/temp/agent/pi-goal-agent-home` (`settings.json` plus a copy of `auth.json`).

## Next steps in order

1. When the subagent settles,
   read its report,
   then `git diff` in the worktree.
   Review it against the brief line by line,
   including the test assertions.
2. Run in the worktree,
   each in its own capped `systemd-run --user`:
   `mise run //package/pi-plugin/goal:lint:types`,
   `:test:unit`,
   `:lint`.
   Confirm the new test names appear in the output.
   The tests are named "built-artifact tests":
   if the new names do not show,
   run `mise run //package/pi-plugin/goal:build` first and repeat.
   Feed failures back to the subagent through the inbox.
   That loop is where model quirks show,
   so record them.
3. Commit the verified change on the branch with explicit pathspecs.
   Auto-push is on.
4. Rerun the measurement with the modified build.
   Baseline:
   Collatz proof runs `f` (read only) and `h` (read and bash) in
   `~/temp/agent/pi-rpc-trial/out/goal-summary_f.json` and `goal-summary_h.json`,
   every denial cycle's `remaining_work` asked the user.
   New runs use the isolated home with `settings.json` `packages` listing the synthetic provider and the worktree's
   `package/pi-plugin/goal` (rebuilt,
   so `dist/final/node/index.mjs` is current),
   the same reviewer `synthetic/hf:openai/gpt-oss-120b`,
   no `--tools` allowlist in one arm and `--tools read,bash` in the other,
   because the allowlist hides `ask_user_question` from the primary regardless of mode.
   Success criteria,
   fixed before the run:
   zero `remaining_work` texts that ask the user,
   and more than one distinct approach across cycles.
   Run it twice for spread.
   Rerun `solve 3x + 1 = 0 and state the value of x` once,
   which must still be approved.
5. Update `doc/agent/pi-goal-extension-driven-through-rpc.md` with the results and the confound below,
   the planning doc paragraph "Human decisions use `ask_user_question` inside the primary run" in
   `doc/planning/pi-goal-tool-free-completion.md`,
   and the matching line in `package/pi-plugin/goal/README.md`.
6. Merge or rebase the branch into `main`.
   The user's live Pi keeps the old build until someone rebuilds the goal package in the main checkout,
   because `package/pi-plugin/goal/dist` is git-ignored (`.gitignore` line 53) and Pi loads `dist/final/node/index.mjs`.
   Say so in the final report.
   Do not rebuild in the main checkout before the change lands.

## Confound to close in the goal doc

The goal doc says the primary replied that no `ask_user_question` tool is exposed to it,
and attributes that to RPC mode.
That reply came from runs started with `--tools read` or `--tools read,bash`.
`pi --help` says `--tools` is an allowlist of tool names to enable,
so the allowlist alone hides the tool.
Two separate reasons therefore exist,
and only the second is from the source:

- The allowlist in the measured runs hid the tool.
- `package/pi-plugin/ask-user-question/src/tool.ts` line 155 returns an unavailable error when `ctx.mode !== 'tui'`.

Without an allowlist in RPC mode the tool is probably registered and fails on call.
That was not run.

## Cleanup when finished

- `systemctl --user stop pi-goal-impl`,
  then confirm no `pi --mode rpc` process or `pi-goal-*` unit remains.
- Delete `~/temp/agent/pi-goal-agent-home/auth.json`,
  the copy of the credentials file.
- Remove the worktree with `git worktree remove` after the branch is merged,
  then delete the branch.
- No stray files outside the worktree:
  the first goal test left `/tmp/collatz.py`,
  so check `/tmp` and the throwaway fixture folder after every subagent run.
