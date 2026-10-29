# stop-reminders

Claude Code `Stop` hook that detects uncertain language in Claude's responses
and reminds it to investigate rather than guess,
and that can optionally refuse a stop whenever pushing could plausibly help.

## What it does

Scans Claude's final response for hedging phrases like "probably",
 "maybe",
 "I think",
"I believe",
 and similar markers.
 When found,
 blocks the stop and sends a reminder
to search for evidence,
 read code,
 or check documentation before responding.

The hook strips code blocks,
 inline code,
 blockquotes,
 and quoted strings before scanning
to reduce false positives from code,
 quoted material,
 or verbatim mentions.

**Once per trigger inside a chain**:
 the wording detectors (hedging,
 dismissal,
 trailing question,
 choice left in prose) run on every stop.
On a first stop every trigger raises its reminder.
When `stop_hook_active` is true the hook has already blocked this turn,
and a trigger raises its reminder only if the transcript holds no reminder with the same opening sentence since the
last human turn,
so a block can be answered and the stop then goes through.
The first hit of each detector decides: a second, different hedge in the same message is not looked at once the first
one has been raised.
Before 2026-10-29 these detectors were switched off in a chain,
and a stop that asked "Do you want me to?" after one block went through.

## Forced continuation

Off by default;
 set `MONOCHROMATIC_STOP_AUTO_CONTINUE` to `on`,
 `1`,
 `true`,
 or `yes` to enable it.
The response-quality detectors run either way.

When enabled,
 separately from the response-quality detectors,
 the hook decides whether the stop is premature.
Luna reads the final message and names the situation it describes (see "Luna situation check"),
 and state decides on its own when the depth limit is reached or the last forced continuation issued no tool call.
When Luna is unavailable, the hook falls back to the state-only behavior described in "Releases".
It was built for one observed failure:
 in this repository's transcripts,
 `claude-opus-5` sessions often ended a turn by announcing their next action
instead of performing it,
 leaving the user to type `Continue.`
The same corpus shows a much lower rate on `claude-fable-5` and `claude-opus-4-8`,
 the comparison is confounded,
 and the repository owner reports that `claude-opus-5-5` shows it much less but not never,
 so it is opt-in and, when on, decided by the Luna check below.
See [`claude-code-opus-5-premature-turn-end.md`](../../../doc/troubleshooting/claude-code-opus-5-premature-turn-end.md).
Measured across this repository's transcripts,
 blocking a stop puts the agent back to work most of the time,
 with a median of nine tool calls per forced continuation.
The rate depends on what did the blocking,
 and the conservative figure is the right one to plan against:
 text-detector blocks issued a tool call 88% of the time
and a state-changing call 84% of the time,
 falling to 82% and 82% on Opus 5,
 where 27% still needed a human nudge afterwards.
Goal-condition blocks scored 100% on every metric,
 which is why pooling the two overstates the case.

The state-only fallback reads none of the response text.
That was the point of its design:
 every pattern-conditioned rule can be satisfied by changing the text,
 and a rule keyed on the phrase `Next:` is satisfied by deleting that sentence,
 which turns an informative stop into a silent one.
The Luna check reads the text, so it can be satisfied by writing a message that reads as closed;
 a background wait is therefore blocked only on state the agent cannot rewrite (no running task).

A trailing question takes precedence,
 because instructing Claude to resume work
and to route its question through `AskUserQuestion`
in the same reason would be contradictory.

## Luna situation check

Since 2026-10-18, with forced continuation on, a model reads the final message and picks one of six situations.
The model is `openai/gpt-6-luna-decisions`, reached through the Gateway Decisions route.

- **announces_own_next_step**:
   blocks with the resume-now reason.
- **leaves_open_work**:
   blocks with a reason to do the listed items now,
   when the message names open items, failures, or follow-ups that the agent could act on itself and stops.
   Added 2026-10-19 for the case the announce label missed.
- **waits_on_user**:
   blocks and routes the question through `AskUserQuestion`.
- **waits_on_background_work**:
   blocks only when the Stop event reports no running background task, since then nothing can resume the turn;
   with a task running the stop is released.
- **pauses_for_handoff** and **work_closed**:
   release.

A stop with a running subagent skips the call and is released, because the subagent reports back on its own;
a running shell command, monitor, or workflow does not skip it.
A stop in plan mode also skips the call, because the agent cannot perform the work it announces.
A stop with a finished task's completion notification still queued skips the call and is released too:
the Stop event lists only running tasks, so a task that ended a moment earlier is absent from it,
yet Claude Code delivers the queued notification as a new turn right after the stop.
The hook finds the notification in the transcript's queue records, and only when no tool result follows it.

A label is acted on at probability 0.60 or higher (interim; revisit from the verdict log).
A verdict whose top label is under 0.60 still blocks when `announces_own_next_step`, `leaves_open_work` and
`waits_on_user` together reach 0.90: Luna is then sure the stop is premature and only unsure which kind.
The reason is that of the largest of the three, and the verdict log's `detail` says "blocking labels together".
The message is sent in full, the end kept when it exceeds 16000 characters, with no thinking blocks.
On a repeated stop (the hook already blocked this turn, so `stop_hook_active` is true) with a background task
running, the request also carries the assistant text of the turn's earlier stops, each cut to its last 1500
characters and all of them to the last 8000, as context for the same question.
Any other stop sends the final message alone.
Measured on repeated stops, history removed all three verified repeated false blocks, each of which had a task
running, and lowered the block score of repeated misses, four of eight of which had none; see the troubleshooting
document below.
Each verdict record in the log carries `stopHookActive`, so repeated stops can be told apart from first stops
(records before 2026-10-25 lack it).
One request, 5 second deadline, no retry.
Any failure falls back to the state-only behavior and adds a `Stop says:` notice.

Environment:

- `STOP_REMINDER_LLMGATEWAY_API_KEY`:
   read from the hook environment first, else from one `mise env --json` in the session directory.
   The mise read is a stopgap until a mise-independent credential mechanism exists.
- `MONOCHROMATIC_STOP_LUNA`:
   `off`, `0`, `false`, or `no` skips the model.
- `MONOCHROMATIC_STOP_VERDICT_LOG`:
   overrides the verdict log path, `~/.local/state/monochromatic/stop-reminder/verdicts.jsonl` by default.

Measurements, limits, and the retraction of the blanket background-task release are in
[`claude-code-opus-5-premature-turn-end.md`](../../../doc/troubleshooting/claude-code-opus-5-premature-turn-end.md).

## Releases

The hook lets a stop through in four situations,
 three of them read from state rather than from the response text.
With the Luna check answering, only the depth limit and the no-tool-call release apply before it;
 the background-task and finished-tasks releases below belong to the state-only fallback.

- **A background task is running.
** The session waits on something another turn cannot advance;
 `background_tasks` on the `Stop` event carries this.
- **The previous forced continuation issued no tool call.
** Pushing an agent that already did nothing buys prose,
 which the measured rescue rates show rarely recovers.
- **Every tracked task is finished.
** Replayed from `TaskCreate` results and `TaskUpdate` calls in the transcript.
 A session with no task list does not count as finished,
 since most sessions never create one
and releasing there would disable the mechanism everywhere.
- **The depth limit is reached**,
 default 25,
 overridden with `MONOCHROMATIC_STOP_AUTO_CONTINUE_MAX`.

**Blocked on a decision?
** The block reason tells Claude to use `AskUserQuestion` rather than stop.
That tool waits for the user's answer,
 which is what a blocked agent actually needs,
 whereas a stopped turn only ends the work and waits to be restarted by hand.

**Termination is shared with Claude Code,
 which is not the same as delegated to it.
** The CLI caps consecutive blocks through `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`,
 but only reliably when the agent is idle.
Measured against 2.1.224,
 sessions producing no tool calls were overridden after nine blocks,
 while a session running one shell command per continuation
reached thirty-one and was never overridden.
The platform cap catches an idle loop,
 not a busy one,
 so this hook bounds the busy case itself.
Counting reads the transcript rather than a sidecar,
so there is no state to corrupt,
no cleanup to miss,
and resuming a session cannot lose the count.

`stop_hook_active` is false on the first dispatch and true on every one after,
 never clearing,
 so honoring it caps forced continuation at one.

**Opt-in switch**:
 any value of `MONOCHROMATIC_STOP_AUTO_CONTINUE` other than `on`,
 `1`,
 `true`,
 or `yes` (compared case-insensitively after trimming),
 including leaving it unset,
 keeps forced continuation off.

## Detected patterns

- **Modal hedges**:
   probably,
   maybe,
   perhaps,
   possibly,
   likely,
   presumably
- **Epistemic hedges**:
   I think,
   I believe,
   I assume,
   I suspect,
   I imagine,
   I guess,
   I suppose
- **Conditional hedges**:
   might be,
   could be,
   should be
- **Uncertainty markers**:
   not sure,
   not certain,
   hard to say,
   difficult to tell
- **Approximation markers**:
   if I recall,
   if I remember,
   from what I recall,
   as far as I know
- **Admitted gaps**:
   I didn't check,
   I haven't verified,
   I haven't tested,
   I haven't read,
   I haven't confirmed,
   I haven't run,
   I haven't measured,
   I haven't tried,
   I haven't looked,
   I haven't reproduced,
   I haven't rerun (or re-run),
   I haven't shown,
   not verified,
   not checked,
   not tested,
   not viewed,
   test gap
   (each "I haven't" entry also in the "did not", "didn't" and "have not" forms)
- **Choice left in prose**:
   say so if,
   say if you,
   tell me if you,
   if you'd rather,
   if you would rather
   (the reason tells the agent to use `AskUserQuestion`)

## Installation

Add the package as a workspace dependency in the root `package.json`:

```json
{
  "devDependencies": {
    "@monochromatic-dev/claude-code-plugin-stop-reminder": "workspace:*"
  }
}
```

Then run `pnpm install` to link the binary.

## Setup

Add to `.claude/settings.json` or `.claude/settings.local.json`:

```jsonc
{
  "hooks": {
    "Stop": [{ "type": "command", "command": "ccsr" }],
  },
}
```

## Binary

`ccsr`:
 **C**laude **C**ode **S**top **R**eminders
