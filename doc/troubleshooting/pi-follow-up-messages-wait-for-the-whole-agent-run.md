# Pi 1.6.0 holds a `followUp` custom message until the whole agent run ends, so a bash-poke result sent mid-run never shows

Status:
incomplete and unapproved.
Diagnosed by a Claude Code session on 2026-11-25T18:05:00-05:00
from Pi's installed source, bash-poke's own log, its spool files, and the live session file.
bash-poke now steers its poke into a busy agent (commit `6795da19a6`);
the busy-agent probe in "Busy-agent probe" shows the change.
The probe is now the package test `package/pi-plugin/bash-poke/src/busy-agent-delivery.unit.test.ts`
(commit `c4540bf992`),
in the default unit run;
with the poke switched back to `followUp`,
its busy case failed (`expected 15 to be below 14`) and passed again once restored.
Not yet done:
hosted-source permalinks for the Pi citations,
and a drive of the interactive Escape key itself
(the queue-clearing call it makes was exercised;
see "Escape discards the queued pokes").

## Symptom

In an interactive Pi session whose agent is working,
the user types `! tweakcc --help`.
The transcript shows a pending block that never changes:

```text
$ tweakcc --help
(running in background; output will be sent when it finishes)
```

No `bash-poke` card follows,
and the model never mentions the output.
It looks as if the command never started.

It did start.
bash-poke's log for that Pi process
(`node_modules/.monochromatic/2026-11-25/2026-11-25T21-58-56.664Z.0791b085-68ac-441a-b942-a388bd59da49.log.jsonl`)
records both jobs:

```text
[pi-bash-poke] [startJob] started job bffd7428-fdf7-41b5-a456-193c836b3222 in /home/user/Monochromatic: tweakcc --help
[pi-bash-poke] [startJob] job bffd7428-fdf7-41b5-a456-193c836b3222 closed: {"code":127}
[pi-bash-poke] [deliverPoke] delivered poke for: tweakcc --help
[pi-bash-poke] [startJob] started job 72beada1-e68b-4ce0-89a4-d14b1abb5eb3 in /home/user/Monochromatic: tweakcc --help
[pi-bash-poke] [startJob] job 72beada1-e68b-4ce0-89a4-d14b1abb5eb3 closed: {"code":0}
[pi-bash-poke] [deliverPoke] delivered poke for: tweakcc --help
```

The first job ran at 2026-11-25T17:03:53-05:00 and exited within a tenth of a second
(`tweakcc: command not found`, before `tweakcc` was installed).
The second ran at 2026-11-25T17:04:51-05:00 and exited within a second with the usage text.
Both outputs are in
`node_modules/.monochromatic/bash-poke/pi-bash-poke/bffd7428-fdf7-41b5-a456-193c836b3222.log`
and `node_modules/.monochromatic/bash-poke/pi-bash-poke/72beada1-e68b-4ce0-89a4-d14b1abb5eb3.log`.
"delivered" in that log only means the message was handed to `pi.sendMessage`.

The session file
(`~/.pi/agent/sessions/--home-user-Monochromatic--/2026-11-25T21-58-55-223Z_01a214b8-09f6-7768-830a-0431427f809d.jsonl`)
held no `bashExecution` entry and no `bash-poke` custom message at 2026-11-25T18:04:56-05:00.
Every assistant message since the user's prompt at 2026-11-25T17:01:02-05:00 ends with `stopReason: "toolUse"`:
the agent has been in one unbroken run the whole time,
with the user's later messages steered into it.

## Root cause

### bash-poke sent the poke as a follow-up

Before commit `6795da19a6`,
`package/pi-plugin/bash-poke/src/poke-delivery.ts` sent:

```ts
// package/pi-plugin/bash-poke/src/poke-delivery.ts
pi.sendMessage(
  { customType: POKE_CUSTOM_TYPE, content, display: true, details, },
  { triggerTurn: true, deliverAs: 'followUp', },
);
```

The choice was inherited from the stopgap described in issue #498
("queues behind current work otherwise")
and recorded in `doc/handover/issue-498-bash-poke-promotion.md` ("Message delivery").
Steer was not weighed there.

### Pi puts a streaming follow-up in the agent's follow-up queue

`@earendil-works/pi-coding-agent@1.6.0`, `dist/core/agent-session.js:1965-1975`:

```js
// @earendil-works/pi-coding-agent@1.6.0 dist/core/agent-session.js
if (options?.deliverAs === "nextTurn") {
    this._pendingNextTurnMessages.push(appMessage);
}
else if (this.isStreaming && options?.triggerTurn !== false) {
    if (options?.deliverAs === "followUp") {
        this.agent.followUp(appMessage);
    }
    else {
        this.agent.steer(appMessage);
    }
}
```

When the agent is idle,
`deliverAs` is ignored and `triggerTurn` starts a run at once (`dist/core/agent-session.js:1976-1982`),
so the `! sleep 7200` case bash-poke was built for is unaffected.

### The agent loop drains follow-ups only when the run would stop

`@earendil-works/pi-agent-core@1.6.0`, `dist/agent-loop.js:191` and `:196-203`:

```js
// @earendil-works/pi-agent-core@1.6.0 dist/agent-loop.js
pendingMessages = (await config.getSteeringMessages?.()) || [];
// ...
// Agent would stop here. Check for follow-up messages.
const followUpMessages = (await config.getFollowUpMessages?.()) || [];
if (followUpMessages.length > 0) {
    // Set as pending so inner loop processes them
    explicitContinuation = false;
    pendingMessages = followUpMessages;
    continue;
}
```

Steering messages are read after every tool batch (line 191).
Follow-ups are read only once the model ends a response without tool calls.
A run that keeps calling tools,
or that the user keeps steering,
holds every follow-up for its whole length.

### The placeholder and the record are also held

Pi renders an extension's returned bash result once
and parks it in the pending area while the agent streams
(`dist/modes/interactive/interactive-mode.js:6060-6068`).
`recordBashResult` defers the `bashExecution` entry until the run ends
(`dist/core/agent-session.js:3305-3309`):

```js
// @earendil-works/pi-coding-agent@1.6.0 dist/core/agent-session.js
// If agent is streaming, defer adding to avoid breaking tool_use/tool_result ordering
if (this.isStreaming) {
    // Queue for later - will be flushed on agent_end
    this._pendingBashMessages.push(bashMessage);
}
```

bash-poke's progress row is removed when the job exits
(`package/pi-plugin/bash-poke/src/register.ts`, `reportCompletion`),
so a command that finishes in under a second shows its row for under a second
and nothing on screen marks it as finished.

### Escape discards the queued pokes

Exercised through RPC on 2026-11-25,
not through the interactive key.
Escape while streaming calls `restoreQueuedMessagesToEditor({ abort: true })`
(`dist/modes/interactive/interactive-mode.js:2567-2569`),
which calls `session.clearQueue()` (`:4063-4064`):

```js
// @earendil-works/pi-coding-agent@1.6.0 dist/core/agent-session.js
clearQueue() {
    const steering = [...this._steeringMessages];
    const followUp = [...this._followUpMessages];
    this._steeringMessages = [];
    this._followUpMessages = [];
    this.agent.clearAllQueues();
```

`_steeringMessages` and `_followUpMessages` hold only typed text,
which goes back into the editor.
Extension custom messages live only in the agent's queues,
which `clearAllQueues()` empties with nothing restored.

RPC's `clear_queue` command calls the same `session.clearQueue()` (`dist/modes/rpc/rpc-mode.js:358`).
The harness `run-clear-queue-arm.ts`,
kept beside the busy-agent probe,
queues a poke during a 1.5 s tool call,
sends `clear_queue` then `abort` as Escape does,
then sends a second prompt.
In both the follow-up control and the steer build,
`clear_queue` answered `{ "steering": [], "followUp": [] }`
and no `bash-poke` message appeared in either run.
The busy-agent probe is the positive control:
without the clear,
the poke always arrived.

## Verification

Versions:
`@earendil-works/pi-coding-agent` 1.6.0 and `@earendil-works/pi-agent-core` 1.6.0,
read from the pnpm store paths the running `pi` resolves through
`package/agent-harness-shared/advisor/node_modules/.bin/pi`;
bash-poke at commit `049d4f1320`.

Evidence collection,
run from the repository root:

```sh
# repository root
rg --fixed-strings 'pi-bash-poke' node_modules/.monochromatic/2026-11-25/2026-11-25T21-58-56.664Z.0791b085-68ac-441a-b942-a388bd59da49.log.jsonl
ls -la --time-style=full-iso node_modules/.monochromatic/bash-poke/pi-bash-poke/
rg --count 'bashExecution|"bash-poke"' ~/.pi/agent/sessions/--home-user-Monochromatic--/2026-11-25T21-58-55-223Z_01a214b8-09f6-7768-830a-0431427f809d.jsonl
```

### Delivery that reached the model mid-run

- `@aliou/pi-processes` 0.13.8 sends attention `turn` notifications with `{ triggerTurn: true, deliverAs: "steer" }`
  (`extensions/processes/notification-sender.ts`, `attentionToSendOptions`).
  The same session holds 24 `ad-process:notification` entries:
  18 with attention `turn` (steered)
  and 6 with attention `context` (sent with `triggerTurn: false`).
  Every one sits between a `toolResult` and the next assistant `toolUse`,
  the first at 2026-11-25T17:19:57-05:00.
  This is the positive control:
  a steer lands within the run on this Pi version.

### Delivery that did not

- Both bash-poke follow-ups,
  sent at 2026-11-25T17:03:53-05:00 and 2026-11-25T17:04:52-05:00,
  were still undelivered at 2026-11-25T18:04:56-05:00 while the run continued.

### Busy-agent probe

A disposable,
credential-free Pi 1.6.0 host in RPC mode loads the built bash-poke artifact
and a guarded fixture extension.
The fixture registers a `fauxProvider` that scripts six `wait_step` tool calls of 400 ms each,
then `RUN_COMPLETE`, then spares.
After the first `tool_execution_start`,
the runner sends `{ "type": "bash", "command": "printf poked" }`.
The control arm is the same bundle with only `` deliverAs:`steer` `` rewritten to `` deliverAs:`followUp` ``.
Harness files,
kept outside the repository with the run outputs:
`~/temp/agent/bash-poke-steer-delivery-check-2026-11-25/keep/harness/run-busy-agent-arm.ts`
and `busy-agent-fixture.ts` beside it.

Results on 2026-11-25:

- Follow-up control:
  the poke's `message_end` came after `RUN_COMPLETE`,
  with seven assistant messages before it,
  and the run then took one more turn (`AFTER_POKE`).
- Steer, first build:
  the poke landed right after the first tool result,
  with one assistant message before it and `RUN_COMPLETE` after it.
- Steer, final build after the lint fixes in commit `181cb07a36`:
  the same order as the first steer run.
- Steer, final build, with `steeringMode` and `followUpMode` set to `all` in the host's settings
  (the values the user's own Pi uses):
  the same order.
- Steer, final build, idle host (the `bash` command sent with no prompt running):
  the poke started a run at once and was its first message after the system prompt.

## Verified workarounds

Each item names its tradeoff.
Only the bash-poke change was exercised,
by the probe in "Busy-agent probe":

- Read the spool file named in bash-poke's log.
  The output is complete there at exit;
  the model does not see it until the run ends.
- Use `!!` for output only the human needs.
  Pi's own foreground path runs it at once and streams into the pending area while the agent works
  (`dist/modes/interactive/interactive-mode.js:6080-6095`, source reading);
  the output is excluded from model context.
- Applied in commit `6795da19a6` by the owner's choice on 2026-11-25:
  bash-poke's busy-agent delivery is `deliverAs: "steer"`,
  as `@aliou/pi-processes` does.
  The poke lands after the current tool batch;
  it interrupts the model's plan the way a typed message does,
  and a tool call that itself waits (a long `sleep`) still delays it.
  The owner chose no extra on-screen signal at job exit.
  A Pi process started before the change keeps the old code until `/reload` or a restart.

## What does not work

- Pressing Escape to make the poke arrive:
  per "Escape discards the queued pokes" it ends the run and drops the queued pokes,
  as the `clear_queue` probe showed.
- Watching the progress row for completion:
  the row disappears when the job exits.

## Upstream filing decision

`.out-of-scope/` has no Pi queue entry (`pi-gpt55-long-context.md` is unrelated).
Nothing is filed.

1.  Is it upstream's fault?
    Follow-up timing:
    no.
    Waiting for the end of the run is what `followUp` means,
    and bash-poke chose it.
    Escape dropping extension messages:
    not established,
    since it was read and not reproduced,
    and it may be intended.
2.  Can upstream fix it?
    Yes for the Escape path:
    `clearQueue` could return custom messages to the extension or keep them queued.
3.  Are they supporting this use case?
    Not checked.
4.  Would the repository welcome the contribution?
    Not checked.
5.  Will they likely fix it?
    Not checked.
6.  Prototype?
    None.

Constraint 1 fails for the timing,
so no draft is kept for it.
The Escape path needs a reproduction before constraints 3 to 6 are worth walking.
