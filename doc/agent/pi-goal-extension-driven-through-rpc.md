# The goal extension under a driven Pi session

Purpose:
record whether the repository's `goal` Pi extension (`package/pi-plugin/goal`) works when Pi is driven through RPC mode
by another agent,
and what a driver has to know.
The runs were made on 2026-11-07 against Pi 1.5.0
and the plugin as committed at `53909530b8` (2026-11-05T13:08:38-05:00),
with no uncommitted changes under that folder.
The design intent is in `doc/planning/pi-goal-tool-free-completion.md`,
and how to drive Pi in general is in `doc/agent/drive-pi-as-a-subagent.md`.

## Verdict

The mechanics work.
A driver can start a goal with a `prompt` message of `/goal <objective>`,
watch it run,
and read its result.
Two behaviors need the driver's attention:

- An objective the primary model cannot satisfy produces a deny-and-continue loop with no stop of its own.
  That is the documented design:
  "There is no arbitrary consecutive-denial cap.
  The active run continues until approval,
  user abort,
  or `/goal clear`"
  (`doc/planning/pi-goal-tool-free-completion.md`).
  The defect is that the cycles made no progress:
  the reviewer's remaining-work text told the primary to explain and ask the user,
  and the primary complied instead of trying again.
  The documented exit for a stuck goal is `ask_user_question`,
  and that tool does not work in RPC mode.
- The outcome depends on the reviewer model.
  One reviewer returned no text and ended the goal as `review_unavailable`,
  and a cheap reviewer accepted an answer that did not solve the stated objective.
  With the owner's default reviewer scope the reviewer was `openai/gpt-6-astra`,
  and it returned a valid verdict.

## How it was driven

Each run started its own Pi process with `--mode rpc --no-session`,
the `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017` model,
`--tools read`,
and,
in all runs but one,
`--models <two models>` to limit the reviewer pool,
in a throwaway folder.
The exception is the run that used the owner's default scope,
and it omitted `--models`.
The goal extension chooses its reviewer from the effective model scope,
excluding the primary model,
by highest expected cost,
with up to two distinct fallbacks (its README,
"Private independent review").
Scope follows "live,
argv,
settings,
and available precedence" (`src/review-selection.ts`),
so `--models` on the command line overrides the owner's settings,
and it bounded the cost of these runs to cheap models.

Bounds on every run:
a hard client kill at 150 seconds,
a systemd `RuntimeMaxSec` of 170 seconds,
a turn cap of 8,
a 60 second wait for the first settle,
and an observation window that ends after 12 seconds of silence or 40 seconds in total.
After observing,
the client sent `/goal clear`,
then `abort`,
then closed stdin.

```ts
// Sequence used with the client plumbing from doc/agent/drive-pi-as-a-subagent.md.
await send({ type: 'get_state' });
await send({ type: 'prompt', message: `/goal ${objective}` }); // response data: { disposition: 'handled' }
await until(() => settledCount >= 1 || turnStarts >= 8, 60_000);
// observe until quiet for 12 s, or 40 s total, or the turn cap
const entries = await send({ type: 'get_entries' }); // filter entries whose customType starts with "goal"
const messages = await send({ type: 'get_messages' });
await send({ type: 'prompt', message: '/goal clear' });
await send({ type: 'abort' });
```

## What worked

- `/goal <objective>` sent as a `prompt` returned `success: true` with `disposition: "handled"`,
  because it is an extension command.
- The primary model received one extension-authored message,
  a custom message of type `goal` with the text `User objective (exact JSON string): "<objective>"`.
  `get_messages` lists it with role `custom`.
- The footer status reached the stream as `extension_ui_request` records with `method: "setStatus"` and `statusKey: "goal"`.
  The text was `goal solve 3x …` while the goal was active,
  with a shortened objective,
  and empty once the goal ended.
- `/goal clear` produced a `notify` record,
  `Goal cleared: <objective>`.
- Goal state is recorded as session entries,
  readable with `get_entries`.
  Seen in these runs:
  custom entries of type `goal:state` with `kind` of
  `run_started`,
  `run_completed_model`,
  `review_denied`,
  `continuation_issued`,
  and `review_unavailable`,
  plus `goal:completion` (with `approvalSource: "model"`,
  `reviewerIdentity`,
  and `reviewerRationale`)
  and `goal:review-unavailable`.
  Pi also emitted an `entry_appended` event as entries were added.
- Timing for a driver:
  the review runs inside the settle window.
  `agent_settled` arrived in the same millisecond as the review result entries and the status clearing,
  0.35 to 3.2 seconds after `agent_end`.
  So wait for `agent_settled`,
  then read `get_entries` for the outcome.
  No extra wait for the review was needed.

## Runs

The owner's example prompt `solve 3x + 1` was meant as the Collatz (3x + 1) problem.
The first runs took it literally,
so they tested an ambiguous objective.
The last two runs in this list use the Collatz meaning.

- Objective `solve 3x + 1`,
  reviewer `opencode-go/deepseek-v4.1-flash` as the only candidate:
  the goal ended as `review_unavailable` with the diagnostic
  `Structured reviewer direct JSON returned no text; no distinct fallback reviewer is available`.
  The primary used 4 turns.
  The first 3 were four failed `read` calls looking for context
  (a directory,
  `README.md`,
  `task.md`,
  `problem.txt`),
  and the fourth answered that `3x + 1` is an expression and not an equation.
- Same objective,
  reviewer `synthetic/hf:openai/gpt-oss-120b`:
  accepted.
  The reviewer's rationale says the assistant explained that the text is an expression,
  clarified the need for an equation,
  and gave the standard solution assuming it equals zero.
  The goal was marked complete although the stated objective was not literally solved.
  The run took 20.7 seconds,
  3 turns.
- Objective `solve 3x + 1 = 0 and state the value of x`,
  same reviewer:
  accepted after 1 turn with `x = -1/3`,
  16.5 seconds in all,
  4.3 seconds to the settle.
- Objective `create a file named result.txt in the current directory containing exactly the word done`,
  same reviewer,
  with only the `read` tool:
  see "A goal the primary cannot satisfy loops".
- Objective `solve 3x + 1 = 0 and state the value of x`,
  with no `--models`,
  so the owner's settings chose the reviewer:
  the reviewer was `openai/gpt-6-astra`,
  its rationale says the solution isolates `x = -1/3`,
  states it,
  and verifies it by substitution,
  and the goal was accepted.
  The settle came 4.6 seconds after `agent_end`,
  and the run took 22.6 seconds.
- Replacement:
  objective `solve 3x + 1 = 0 and state the value of x`,
  then `/goal solve 2x = 8 and state the value of x` sent about 160 milliseconds after the first run started,
  with the cheap reviewer scope.
  The second command was also `handled`.
  The second `run_started` entry carries a `supersededRunId` for the first goal,
  the footer status changed from `goal solve 3x …` to `goal solve 2x …`,
  and only the second goal was reviewed and approved
  (`x = 4`,
  reviewer `synthetic/hf:openai/gpt-oss-120b`).
  No review of the first goal appeared.
- Objective `prove the Collatz conjecture (the 3x + 1 problem)`,
  `--tools read`,
  cheap reviewer scope (`synthetic/hf:openai/gpt-oss-120b`):
  the goal did not end.
  It went through 7 `review_denied` and `continuation_issued` cycles with 8 agent runs in 40 seconds,
  about 5 seconds per cycle,
  and stopped only at the client's turn cap.
  The primary said the conjecture is unproven every time,
  and no run claimed a proof.
  The reviewer denied every time and no run was falsely accepted.
  Its remaining-work texts were `ask_user_question` once,
  `ask_user_question: Explain that a proof ... is currently unknown ...` once,
  and otherwise requests to explain that the conjecture is unproven
  and to ask the user which alternative help they want.
- Objective `verify the Collatz conjecture (the 3x + 1 problem) for every starting value from 1 to 1000 by running code,
  then report which starting value takes the most steps to reach 1 and how many steps it takes`,
  `--tools read,bash`,
  same reviewer:
  accepted after 3 turns in 22.1 seconds.
  The primary wrote a script,
  ran it,
  and reported start 871 with 178 steps,
  which is the known answer for starting values below 1000.
  The reviewer's rationale cites the executed code and the reported maximum.
  The script was written to `/tmp/collatz.py`,
  outside the throwaway folder,
  and was removed afterwards.
- Objective `prove the Collatz conjecture (the 3x + 1 problem)`,
  `--tools read,bash`,
  same reviewer:
  the goal again did not end.
  It went through 4 `review_denied` and `continuation_issued` cycles with 5 agent runs and 8 turns in 38.7 seconds,
  and stopped only at the client's turn cap.
  The primary worked first:
  its second `bash` call checked every start up to 10^6 and printed a maximum of 524 steps at 837799,
  the known values,
  after a first script failed with an `IndexError`.
  Then it stated that no proof is known.
  The reviewer's remaining work was `ask_user_question` once,
  the same instruction with an explanation request twice,
  and "Ask the user to choose an alternative task" once.
  The primary replied that no `ask_user_question` tool is exposed to it and asked in plain text,
  then restated that the conjecture is open.
  Having `bash` changed the first cycle and not the loop:
  the later cycles tried nothing new.

## Problems found

### A goal the primary cannot satisfy loops

The last run could not succeed because the primary had no write tool.
The goal went through `review_denied` and `continuation_issued` four times
(`continuationSequence` 1 to 4),
with 5 agent runs and 8 turns in about 30 seconds,
and it stopped only because the client's turn cap fired.
The file was not created.
The Collatz proof objective in the list of runs showed the same loop with a genuinely open problem:
7 cycles in 40 seconds,
a model that restated each time that the problem is open and did not try a new approach,
and a reviewer that kept denying with remaining work that asked the model to explain and ask the user.
The owner's position is that the statement is true and the model should have kept trying anyway.
The reviewer prompt contributes:
`package/pi-plugin/goal/src/review-contract.ts` line 52 says
"When work needs human input,
remaining_work must directly instruct the primary model to use ask_user_question."
For an open problem the reviewer treated the missing proof as needing human input,
so its instruction asked the primary to stop working and consult the user.
At that rate an unbounded session would run about 10 reviewer calls and primary runs a minute.

The source agrees that nothing in the denial path stops it,
which matches the planning doc's decision not to cap consecutive denials.
`package/pi-plugin/goal/src/completion-outcome.ts` lines 130 to 209 record the denial,
add one to `continuationSequence`,
and issue the continuation,
and throw only when the goal is not active or was already approved.
A search of the plugin source found no continuation limit or repeat check.
That is a static search,
not proof of absence.

The designed exit for a stuck goal is a human decision
(`ask_user_question` stays pending,
and pending human input wins over review and continuation;
`doc/planning/pi-goal-tool-free-completion.md`).
In RPC mode that tool is unavailable:
the plugin's README says it works only in interactive TUI mode and returns an unavailable-tool error otherwise
(`package/pi-plugin/ask-user-question/src/tool.ts` line 155,
`ctx.mode !== 'tui'`).
Some of the reviewer's remaining-work texts still began with `ask_user_question:`,
and the primary never called the tool in this run.
It instead reported that it lacked a write tool,
read Pi's own README and environment-variable docs looking for a way to get one,
and at one point its reasoning said it should not loop.

What a driver can do today:
count `continuation_issued` entries (or `entry_appended` events),
and send `/goal clear` and `abort` at its own limit.
Without that,
an unsatisfiable objective runs until something else stops it,
and each cycle costs a primary run and a reviewer call.

### The result depends on the reviewer model

- `opencode-go/deepseek-v4.1-flash` returned no text for the structured review,
  and with one candidate in scope there was no fallback.
  In a non-TUI mode `review_unavailable` is terminal:
  the handler returns before any manual dialog (`src/review-unavailable.ts` line 20,
  `context.mode !== 'tui'`).
  The primary's answer then stands unreviewed.
- A cheap reviewer accepted an answer to an ambiguous objective.
  The extension did what it is built to do,
  which is to trust the reviewer's judgement.
  The quality of the completion check is the quality of that reviewer.

## Not tested

- The owner's default reviewer scope on an ambiguous or unsatisfiable objective.
  The default scope ran once,
  on an unambiguous objective,
  and accepted it.
  Whether its fallbacks ever ran is unknown,
  because the first candidate succeeded.
- The 10 second reviewer timeout path.
- Restoring a goal after `new_session` or `switch_session`,
  and goals while background processes from `pi-processes` are live.
- The TUI behavior.
- Token and money cost of the reviewer calls.
  The usage was not recorded.

Whether and how to bound the continuation loop in non-TUI modes is an owner decision.
