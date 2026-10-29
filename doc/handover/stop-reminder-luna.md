# Stop-reminder hook: wording checks and the Luna situation check

## Purpose

State of the Claude Code `Stop` hook `ccsr` (package `claude-code-plugin/stop-reminder`) on 2026-10-29,
for the next session that works on it.
The hook refuses a stop in two ways: wording detectors on the final message, and, with forced continuation on,
a model reading of the situation the message describes (Luna, `openai/gpt-6-luna-decisions`).
The goal as the owner states it: the owner should never have to type "Continue." or "You didn't check, so check."
Evidence and measurements are in `doc/troubleshooting/claude-code-opus-5-premature-turn-end.md`.
This file holds the state, the decisions, and the next actions.

## Where things are

- Source: `package/claude-code-plugin/source/src/handler/stop-reminder/`.
  `index.ts` holds the decision and the handler.
  The wording checks are in `uncertainty.ts`, `uncertainty-phrases.ts`, `uncertainty-strip.ts`,
  `uncertainty-citations.ts` and `parked-decision.ts`.
  The Luna path is `luna-situation.ts`, `situation-action.ts`, `turn-history.ts` and `verdict-log.ts`.
  State releases are `continuation-depth.ts`, `continuation-progress.ts`, `continuation-tasks.ts` and
  `queued-notification.ts`.
  Most files have a `.unit.test.ts` beside them.
- Built bundle: `package/claude-code-plugin/stop-reminder/bundle/node/index.mjs`, committed.
  `node_modules/.bin/ccsr` runs it and `.claude/settings.local.json` registers `ccsr` as the `Stop` hook,
  so a rebuild changes the live hook for every session in this checkout.
  Rebuild with `mise run //package/claude-code-plugin/stop-reminder:build`.
- Gates for the source package:
  `mise run //package/claude-code-plugin/source:test:unit`, `:lint:oxlint` and `:lint:types`.
- Verdict log: `~/.local/state/monochromatic/stop-reminder/verdicts.jsonl`.
  One record per stop that reached the Luna path: six scores, the action, the last 600 characters of the message,
  and, since 2026-10-25, `stopHookActive`.
  The 600-character tail means any analysis of a long message must recover the full text from the session transcript
  under `~/.claude/projects/`.
  The first studies of 2026-10-25 skipped this and were corrected.
- Scratch scripts, not committed, in `~/temp/agent/`:
  `labeljoin.ts` joins verdicts to the next human message, `fullmsg.ts` recovers full text,
  `exp5.ts` reruns Luna comparisons through the shipped code, `chains.ts` and `chainflags.ts` cover stops after a
  block, and `miss2.ts` and `miss3.ts` cover the 2026-10-29 stop.

## What the hook does now

- First stop of a turn (`stop_hook_active` false): an uncertainty phrase (hedges and admitted gaps), an uncited
  categorical dismissal, a trailing question, and a choice left with the user in prose each add a reminder.
  Any reminder blocks.
- Inside a blocked chain (`stop_hook_active` true): those wording checks are off, except the parked-decision check,
  which runs once per phrase per turn.
  It finds its own earlier feedback record in the transcript since the last human turn.
- With `MONOCHROMATIC_STOP_AUTO_CONTINUE` and Luna on: Luna labels the final message with one of six situations and a
  label at 0.60 or higher acts.
  On a repeated stop Luna also receives the assistant text of the turn's earlier stops.
- Released without asking Luna: plan mode, a running subagent, and a finished task's notification still queued.
- Released by state alone: the depth limit, and a forced continuation that issued no tool call.

## Owner decisions on record

- Earlier in this work: "No audit, keep logging".
  Luna may also be used to detect uncertainty, but the phrase list is the fix for the music-player case.
  A Luna uncertainty label is not built; it needs its own calibration under #1772.
- 2026-10-25: admitted-gap wording extended with measure, try, look, reproduce, rerun, `I haven't shown`, `test gap`
  and `not viewed`.
  The bare adjectives `untested`, `unverified`, `unexplained` and `unmeasured` stay out.
- 2026-10-25: the parked-decision phrases are `say so if`, `say if you`, `tell me if you`, `if you'd rather` and
  `if you would rather`.
  `veto`, `your call`, `up to you` and a bare `if you want it` stay out.
  The check runs inside a chain.
- 2026-10-25: Luna's block rule stays the top label at 0.60.
- 2026-10-25: Luna receives the earlier stops on repeated stops only, and every verdict record logs
  `stopHookActive`.

## Commits

- `d6bfd615ef`, `20fbf6c825`, `322d119f64`, `1090d9fd53`: release a stop while a finished task's notification is queued.
- `0718734a62`: admitted-gap phrases and the quote stripper's contraction fix.
- `410c4eeb39`: more admitted-gap wording.
- `b2c225e2e9`: the parked-decision check, also inside a chain.
- `96b0d41d55`: earlier stops to Luna on repeated stops, and `stopHookActive` in the log.
- `1223a6246e`: final-message matching by its end, and the corrected Luna measurements.
- `14be22175a` and `0aacd31e09`: the measurement write-ups in the troubleshooting document.

## Misses and false blocks, in order

1. 2026-10-21, false block: a task had just finished, `background_tasks` lists only running tasks, and the agent was
   right to wait.
   Fixed by the queued-notification release.
2. 2026-10-22, music-player report: admitted gaps were not in the phrase list.
   Fixed with phrases, and with a fix to the quote stripper that hid text between two contractions.
3. 2026-10-24 23:11, Slint report (session `55e550ef`): "I didn't try", "I didn't measure", "Test gap".
   Luna released it at `work_closed` 0.48.
   Fixed with phrases, on first stops.
4. 2026-10-25 10:31, Slint stop (same session), inside a chain: "Say so if you want it".
   Fixed by the parked-decision check running in a chain.
5. 2026-10-25, this repository: a report ending "Veto this if you want turn history tried anyway".
   The owner chose the narrow phrase set, which has no `veto`, so this exact wording is still not caught.
6. 2026-10-29 03:10, session `a45b87f9`: the open case below.

## Open: the 2026-10-29 miss

The stop was inside a chain: the stop before it had been blocked on "I haven't checked whether the rule guards against
these", and the agent answered by checking.
Its final message said "I haven't run a fixture" and ended "Do you want me to?".
The wording and trailing-question checks were off because `stop_hook_active` was true.
Luna with the earlier stops scored `waits_on_user` 0.54 and `leaves_open_work` 0.37 and was released at the 0.60
cutoff; on the final message alone it scored `waits_on_user` 0.99, which would have blocked.
The turn history the owner chose on 2026-10-25 produced that split.

Three separate decisions are put to the owner, each with measured numbers in the troubleshooting section
"A chain stop that history turned into a miss, 2026-10-29":

- Run the wording checks inside a chain once per trigger, as the parked-decision check already does.
  On the logged chain stops this flags 14 stops nothing else blocked, including the 23:11 and 10:31 misses,
  one false hit ("as it should be"), and no trailing questions.
- Keep or drop the earlier stops as Luna context on repeated stops.
- Block when the always-blocking labels together reach 0.8 and no single label reaches 0.60.
  In the logs this happened 3 times in 832 released stops, and this stop (0.95) would be a fourth.

Answers are not recorded here yet.

## Not yet known

- Agreement between Luna's label and a human label (#1772).
- When Claude Code clears `stop_hook_active` inside a turn.
  Six of the 20 released chain stops that the wording checks would flag were followed by a hook block anyway,
  so the wording check ran at them.
- Precision of the parked-decision and admitted-gap phrases against a human reading; only snippets were read.
- Whether history on repeated stops changes live outcomes; `stopHookActive` makes that measurable from now on.
- Whether another checkout's `ccsr` runs a stale bundle.
  The shim runs the bundle at a path relative to its own directory, so each checkout runs its own copy.
  This was verified for this checkout only.
- The queued-notification release: 11 of 13 flagged records with no nearby delivery were not read, the `enqueue`
  record's presence on disk when the hook starts is unverified, and the queue record format is internal to
  Claude Code 2.1.305.
- The block on "Oxlint is still running" has no verdict-log or transcript record; its cause is unconfirmed.
- The standing-instruction and direct-question variants of the nudge study were not rerun on full messages.
- The 14 unblocked chain stops: 11 are unclassified.

## Next actions

1. Record the owner's answers to the three decisions, then implement the chosen ones with tests.
   The chain gate is in `stopRemindersDecision` in `index.ts`, the history switch is the `earlierStops` argument in the
   handler, and a split-label rule belongs in `situation-action.ts`.
2. Rebuild the bundle, verify on the built hook with a fixture transcript, and update the README and this file.
3. Open follow-ups: #1772 (calibrate the 0.60 cutoff, the running-task release and the `permissionMode` and
   `sessionCrons` skips from the verdict log), #1773 (replace the mise credential fallback), and #1786 (the installed
   `pi` command lacks `cross-spawn`).
