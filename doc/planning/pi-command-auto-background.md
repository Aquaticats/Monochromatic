# Pi command auto-background planning

Purpose:
the design tree for making pi's model `bash` commands auto-background
at a threshold and telling the model when that happens.
Grilling is in
progress.
The open frontier at the end is unanswered,
and nothing here is
accepted until the user confirms the shared understanding.

State recorded 2026-11-08T01:25:38-05:00.

Citation shorthands used below:
`CA/` is
`package/pi-plugin/bash-poke/node_modules/@earendil-works/pi-coding-agent/`,
and `AC/` is the `@earendil-works/pi-agent-core` 1.5.0 package that
`CA` depends on.

## Requirement as stated

The request,
quoted from the user 2026-11-08T00:59:43-05:00:

> Plan to make pi's commands auto background after 1s always,
> and give the
> model a steer saying "command backgrounded after 1s" whenever this happens.

## Settled decisions (2026-11-08T01:08:07-05:00)

Answers from the first grilling round,
question ids `command-surface`,
`trigger-semantics`,
`always-binding`,
and `steer-payload`:

- Command surface:
  pi's model `bash` tool calls.
  `!` stays bash-poke's
  immediate background handling and `!!` stays pi's own foreground bash.
- Trigger:
  run in the foreground up to the threshold.
  A process still alive at
  the threshold detaches and fires the notice.
  A process that exited earlier
  returns its output normally with no notice.
- Threshold:
  a settings key,
  default `1s`.
  The behavior is always on;
  the
  question variant that also offered an off switch was not chosen.
- Notice payload:
  the lead text `command backgrounded after 1s`,
  the command
  text,
  and a job handle.

## Mechanism evidence (2026-11-08T01:25:38-05:00)

- pi's model shell tool is `bash`,
  input `{ command, timeout? }` with no
  default timeout and no background mode or job ids
  (`CA/dist/core/tools/bash.js:28-31`).
- An extension that registers a tool named `bash` replaces the builtin in both
  the definition and execution registries
  (`CA/dist/core/agent-session.js:2888-2913`).
- A tool call produces a tool result whenever `execute` returns
  (`CA/dist/core/tools/bash.js:286-294`).
  Auto-backgrounding at the threshold
  therefore means the call returns at the threshold,
  so a tool result exists at
  that moment in every design.
- A steer is "delivered after the current assistant turn finishes executing its
  tool calls,
  before the next LLM call"
  (`CA/dist/core/agent-session.d.ts:570-573`,
  `AC/dist/types.d.ts:239-244`).
  A steer queued at the threshold can never reach the model earlier than the
  tool result of the call it describes.
- The default `steeringMode` is `one-at-a-time`
  (`CA/dist/core/settings-defaults.js:6`),
  and that mode drains one queued
  message per poll (`AC/dist/agent.js:66-82`,
  `AC/dist/agent.js:335-341`).
- pi ships no background job machinery at all.
  Job registry,
  spooling,
  and
  completion pokes exist only in `package/pi-plugin/bash-poke/`,
  scoped to `!`
  commands.
- Upstream pi refuses background bash for the model tool and recommends tmux or
  a userland replacement tool instead (upstream README through v0.74.0 and
  maintainer comment 2026-05-21 on upstream issue 4850;
  external,
  fetched by a
  fact probe rather than browser-verified).

## Retracted question (2026-11-08T01:25:38-05:00)

The first `notice-channel` question offered "a pi steer message,
delivered
asynchronously" against "the command call's return value".
It was invalid as
asked,
on two counts:

1.  The return value is not optional.
    The call returns at the threshold in every
    design,
    so its result text exists regardless of any steer.
2.  pi has no asynchronous steer delivery.
    A steer lands at the same context
    boundary as that tool result,
    never sooner,
    and under the default steering
    mode a second queued steer waits for a later turn.

The rebuilt question is `notice-channel` under the open frontier.

## Open frontier (2026-11-08T01:25:38-05:00)

Asked in the grilling round that opened 2026-11-08T01:25:38-05:00 and not yet
answered:

- `host-and-reuse`:
  which package hosts the behavior,
  and whether bash-poke's
  job machinery is reused.
- `notice-channel`:
  where the notice sentence lives,
  rebuilt as described under
  the retraction.
- `completion-delivery`:
  how the model receives a finished job's output.
- `cancel-surface`:
  how a backgrounded job is cancelled,
  and by whom.
- `timeout-parameter`:
  what the model-facing `timeout` parameter means once
  every call backgrounds at the threshold.
- `notice-wording`:
  how the lead text reads when the threshold is not `1s`.
- `agents-ext-tightening`:
  whether to apply the proposed `EXT` rule tightening
  that this round's miss motivates.

## Assumptions adopted without asking

Veto any of these and say so:

- A command that exits before the threshold returns pi's native `bash` result
  shape (`output`,
  `truncated`,
  `full_output_path`,
  `exit_code`,
  `wall_time_seconds`) unchanged.
- Jobs run concurrently and each threshold crossing fires exactly one notice.
- A job still running when pi exits is orphaned,
  matching bash-poke's stated
  v0.x limitation.
- Output spooling and middle-out truncation follow bash-poke's pattern.
- `powershell`,
  `read`,
  `edit`,
  and the other pi tools are out of scope.

## Rejections and limits

- Notice channel as first asked,
  rejected with reasoning under the retraction.
- Hard-coded threshold with no settings key,
  rejected in favor of a settings
  key.
- A settings key with an off switch,
  rejected;
  the behavior is always on.
- Notice payload of the command text only,
  or of the bare sentence alone,
  rejected in favor of sentence plus command plus handle.
- An `ODM` rule tightening that would forbid unverified option demonstrations
  measures 206 normalized characters and so breaks `RLM`'s 200 character cap;
  `EXT` is the home for that fix instead.

## Unknowns

- Whether pi's provider replay validators accept the exact tool result shape a
  replacement `bash` tool returns at the threshold.
- Whether two extensions registering `bash` collide;
  pi keeps the first
  registration per name (`CA/dist/core/extensions/runner.js:410-421`).
- The completion message delivery options,
  `triggerTurn` and `deliverAs`,
  for a
  model-initiated job,
  since bash-poke only proves them for `!` jobs.

## Commits

This record ships alone as
`docs(planning): pi command auto-background design tree`.
Later rounds append
to this file and commit the same way.

## Next action

Collect the frontier answers,
then keep grilling the branches those answers
unblock,
starting with job handle contents which depends on the cancel and
completion answers.
Before asking the user to confirm shared understanding,
a
cross-vendor reviewer re-reviews this design and its findings fold in first.
