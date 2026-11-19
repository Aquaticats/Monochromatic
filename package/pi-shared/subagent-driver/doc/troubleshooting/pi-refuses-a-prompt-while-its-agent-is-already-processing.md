# Pi 1.5.2 refuses a prompt that meets a running agent unless the client names a streaming behaviour

## Symptom

An RPC client sends `{"type":"prompt","message":"..."}` while Pi's agent is already in a turn,
and Pi answers with a failed response:

```text
{"type":"response","id":"driver-4","command":"prompt","success":false,
 "error":"Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message."}
```

A run this package drove on 2026-11-19 hit it with its wake message,
33 ms after Pi started a turn by itself to show the agent a background job's result
(`pi-runs/...-1/feed.jsonl`, lines quoted in `kept-while-work-runs.ts`):

```text
{"t":398896,"kind":"turn_start","n":42}
{"t":398929,"kind":"wake",...,"success":false,"error":"Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message.",...}
{"t":398931,"kind":"assistant","stop":"aborted","text":"","thinkingChars":0,"toolCalls":0,...}
```

The turn Pi had started was cut off, and the run ended `prompt-rejected`.
The refusal was read as a refusal of the message, which it never was:
the agent was awake in a turn of its own.

Pi documents the behaviour, not as a defect:
"**During streaming**: If the agent is already streaming, you must specify `streamingBehavior` to queue the message"
(`docs/rpc-commands.md` in the installed Pi).

## Root cause

The installed package is `@earendil-works/pi-coding-agent` 1.5.2
(`node_modules/@earendil-works/pi-coding-agent` under this package, the version the workspace's lockfile pins).
`mise exec -- pi --version` prints `1.5.2`, and its own `dist` is what runs.

RPC mode passes the client's field straight through
(`dist/modes/rpc/rpc-mode.js:314-331`):

```js
case "prompt": {
    let preflightSucceeded = false;
    activeCommands++;
    void session
        .prompt(command.message, {
            images: command.images,
            streamingBehavior: command.streamingBehavior,
            source: "rpc",
            preflightResult: (disposition) => {
                preflightSucceeded = true;
                output(success(id, "prompt", { disposition }));
            },
```

`session.prompt()` hands preflight to `_preparePrompt`,
which refuses only when the agent is streaming and no behaviour was named
(`dist/core/agent-session.js:1748-1760`):

```js
if (this.isStreaming) {
    if (!options?.streamingBehavior) {
        throw new Error("Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message.");
    }
    if (options.streamingBehavior === "followUp") {
        await this._queueFollowUp(expandedText, currentImages);
    }
    else {
        await this._queueSteer(expandedText, currentImages);
    }
    preflightResult?.("queued");
    return undefined;
}
```

`isStreaming` is the agent's run state, not the model's stream
(`dist/core/agent-session.js:1127-1133`):

```js
get isStreaming() {
    return this._isAgentRunActive;
}
get isIdle() {
    return !this._isAgentRunActive && !this.isCompacting;
}
```

`_isAgentRunActive` is set by `_startRun` (`dist/core/agent-session.js:1488`)
and cleared by `_emitAgentSettled` (`dist/core/agent-session.js:740`),
so any turn makes it true, including one Pi starts by itself rather than from a client prompt.

The agent loop decides when each queued message is read
(`@earendil-works/pi-agent-core` 1.5.2, `dist/agent-loop.js`,
the same package the installed Pi depends on):

- a steering message is drained after a turn ends and before the next request
  (`pendingMessages = (await config.getSteeringMessages?.()) || [];` near `agent-loop.js:191`),
  so it joins the running turn;
- a follow-up is drained only when the agent would otherwise stop
  (`const followUpMessages = (await config.getFollowUpMessages?.()) || [];` near `agent-loop.js:196`),
  so it continues the same run as another turn.

Two limits of that loop matter to a caller:

- a turn that ends with `stopReason` `"error"` or `"aborted"` returns at
  `agent_end` before either queue is drained, so a message queued into such a turn is never read;
- the `steer` and `follow_up` RPC commands queue without starting anything
  (`dist/core/agent-session.js:1836-1851`), so a `steer` sent to a resting agent is never read until something
  else starts a run.

The consumer's fault was reading `success:false` as a refusal of the message.
`refusedByIt` in `kept-while-work-runs.ts` treated every failed response alike,
and `watchUntilEnded` ended the run `prompt-rejected`.

## Verification

Version: Pi 1.5.2, the version this package's lockfile pins.

The runnable harness is the package's own stand-in Pi, in its scenario
`wakes-itself-when-the-work-ends` (`src/stand-in-pi-own-turn-plays-fixture.ts`):

```sh
mise run //package/pi-shared/subagent-driver:test:unit src/drive-pi-subagent.unit.test.ts
```

It plays a first prompt that leaves a real `sleep` reported through the `process` tool,
then starts a turn of its own as the work ends and answers the driver's wake the way Pi does.

Failing catalog, the wake sent with no streaming behaviour (run 2026-11-19, feed excerpt):

```text
{"kind":"turn_start","n":2}
{"kind":"wake","success":false,"error":"Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message."}
{"kind":"exit","code":0}
```

The summary said `"endReason":"prompt-rejected"` and its last reply was the turn that had ended before,
so the turn Pi started was lost.

Working catalog, the wake sent with `"streamingBehavior":"steer"`
(the same scenario after the fix):

```text
{"kind":"turn_start","n":2}
{"kind":"assistant","stop":"stop","text":"the build ended while I was reading it, and the task goes on",...}
{"kind":"settled","n":2}
{"kind":"wake","success":true,"disposition":"queued",...}
```

The summary said `"endReason":"idle"` and its last reply was that turn's.

A second, source-level catalog is the unit test
"keeps a run whose wake met a turn the agent had already started"
in `src/watch-pi-run.unit.test.ts`:
Pi answers the busy error anyway, and the run must still end `idle` after the turn settles.
The control beside it,
"ends a run whose wake message Pi refused as one Pi refused a prompt, not as one that finished",
shows a refusal still ends the run `prompt-rejected`.

Not run: a `followUp` wake against the stand-in. Its delivery point is read from
`agent-loop.js` above, not measured here.

## Verified workarounds

Both are the package's own, in `kept-while-work-runs.ts`.

- Send the wake with `streamingBehavior: "steer"`.
  Pi queues it into the running turn and `preflightResult("queued")` answers `success:true`,
  so the driver no longer sends a turn-killing refusal.
  Tradeoff: the message joins the running turn before its next request rather than after it,
  and a turn that ends in an error or an abort drops it.
- Read Pi's "Agent is already processing" answer as the agent being awake, not as a refusal:
  keep the run up for the turn that is running and let its settle be the run's end.
  Tradeoff: the message is then not delivered at all,
  which is right only when that turn is the agent's own reaction to the same event,
  which is what the answer means.

## What does not work

- Sending the same prompt again after the busy answer: it meets the same running turn and is refused again.
- Sending the RPC `steer` command instead of a prompt while the agent may be resting:
  it queues and nothing drains it, so a resting agent never reads it
  (`dist/core/agent-session.js:1836-1851` reads the queue only from a running loop).
- Ending the run on `success:false`: that is what cut off the turn on 2026-11-19.

## Where else the driver can meet this answer

The wake is not the only prompt this package sends while an agent may be at work.
Each was read on 2026-11-19, in the code named.

- Inbox delivery (`deliverMessage` in `watch-pi-run.ts`) sends `steer` while its view of the feed says Pi runs,
  and `prompt` otherwise.
  A turn Pi starts after that read, which is the same race, makes the prompt meet the busy answer;
  the driver keeps the message aside as `.failed-<name>`, skips it in later scans and never retries it.
  The message stays readable on disk and the run goes on, but it is never delivered.
  Sending it as a prompt with `streamingBehavior: "steer"` unconditionally would close this,
  because Pi ignores the field while its agent rests.
  Left unchanged by the fix for the wake.
- The model-error retry (`answerWithAnotherAttempt` in `answer-a-model-error.ts`) sends only when its view of the
  feed says Pi rests (`answerModelError` returns `still-running` while `record.isRunning()`).
  A turn Pi starts after that read makes the retry prompt meet the busy answer,
  which that code reads as a refusal and ends the run `prompt-rejected`.
  Queueing the retry behind the running turn would be wrong:
  the retry waits precisely because that turn may recover the error itself.
  A correct fix reads the busy answer as Pi having been at work, drops the attempt it was about to count,
  and looks again after the turn settles.
  Left unchanged here, because it needs its own attempt bookkeeping and test.
- The first prompt (`sendFirstPrompt` in `take-the-first-prompt.ts`) is what starts the run's first turn,
  so no turn exists for it to meet; a Pi with no active run answers it as it always did.
  This package's own extension starts no turn.

## Upstream filing decision

Nothing is filed, and nothing is worth filing.

1. Is it really upstream's fault?
   No. Pi documents the requirement and its answer names the fix.
   The defect was this package's reading of the answer.
2. Can upstream fix it? Not applicable.
3. Are they supporting this use case? Yes: `docs/rpc-commands.md` covers streaming prompts,
   `steer`, `follow_up` and their dispositions.
4. Would the repo welcome our contribution? Not assessed: no filing is planned.
5. Will they likely fix it? Not applicable: the behaviour is intended.
6. Have we prototyped a fix? Not needed.

The `.out-of-scope/` check found `pi-gpt55-long-context.md` as the only Pi entry,
which covers model context metadata and not RPC prompt behaviour,
so no exemption applies and none is needed.
