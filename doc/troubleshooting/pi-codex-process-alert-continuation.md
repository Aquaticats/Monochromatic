# Pi 0.80.6: Codex process alerts can emit an unexpected turn-continuation report

## Symptom

Pi displayed this text as part of an assistant response,
 immediately after a background-process completion
or log-watch notification:

```text
第四user code report: Unexpected turn continuation: expected assistant or tool message.
Received: user
```

The incident record is `~/.pi/agent/sessions/--var-home-user-Monochromatic--/`
`2026-07-09T22-31-45-304Z_019f4902-2598-7eb1-822e-239618ee2503.jsonl`.

The diagnostic was observed in assistant messages produced by
`openai-codex/gpt-5.6-sol` through `openai-codex-responses`.
It was not entered by the human user.

The observed triggering notification shapes were:

- Process lifecycle messages,
   for example `Process 'name' completed successfully`.
- Process log-watch messages,
   for example `Watch matched for 'name'`.

At investigation time,
 a JSONL scan found 14 diagnostic assistant messages and 14 process lifecycle updates
whose direct parent was one of those diagnostic messages.
 The same active branch contained 655 other
process-update messages without the diagnostic.
 Process alerts are therefore necessary in the observed
incidents,
 but are not by themselves sufficient to reproduce it.

## Root cause

This is a Pi and Codex-provider continuation interaction,
 exposed by the installed
`@aliou/pi-processes` extension.
 It is not application code and is not ordinary model prose.
The Codex backend may be the component that emits the text,
 but the available evidence does not establish
whether Pi sends an invalid continuation or the backend rejects a valid one.

`@aliou/pi-processes` decides whether an ended process should trigger an agent turn in
`src/hooks/process-end.ts:24-30`,
 then sends a custom update with that decision at `:58-66`:

```ts
// @aliou/pi-processes@0.9.4, src/hooks/process-end.ts:24-30,58-66
const triggerAgentTurn =
  (info.status === "killed" && info.alertOnKill) ||
  (info.status === "exited" && info.success && info.alertOnSuccess) ||
  (info.status === "exited" && !info.success && info.alertOnFailure);

safeSendMessage(pi, {
  customType: MESSAGE_TYPE_PROCESS_UPDATE,
  content: message,
  display: true,
  details,
}, { triggerTurn: triggerAgentTurn });
```

Pi receives that custom extension message in
`package/coding-agent/src/core/agent-session.ts:1388-1410`.
During streaming it queues the message as steering;
 otherwise a `triggerTurn` update starts another agent
prompt:

```ts
// pi v0.80.6, package/coding-agent/src/core/agent-session.ts:1403-1410
} else if (this.isStreaming) {
  if (options?.deliverAs === "followUp") {
    this.agent.followUp(appMessage);
  } else {
    this.agent.steer(appMessage);
  }
} else if (options?.triggerTurn) {
  await this._runAgentPrompt(appMessage);
}
```

Pi converts every custom message into an LLM `user` message in
`package/coding-agent/src/core/messages.ts:162-167`:

```ts
// pi v0.80.6, package/coding-agent/src/core/messages.ts:162-167
case "custom": {
  const content = typeof m.content === "string" ? [{ type: "text" as const, text: m.content }] : m.content;
  return {
    role: "user",
    content,
    timestamp: m.timestamp,
  };
}
```

The active global Pi settings select `transport: "auto"`.
 The Codex provider attempts WebSocket transport
when the setting is not `"sse"` (`package/ai/src/api/openai-codex-responses.ts:272-278`) and treats
`"auto"` as cached WebSocket context (`:1394-1398`).
 For a matching continuation it sends only the delta
plus `previous_response_id` (`:1343-1352`):

```ts
// pi v0.80.6, package/ai/src/api/openai-codex-responses.ts:1394-1398,1343-1352
const useCachedContext = options?.transport === "websocket-cached" || options?.transport === "auto";
const requestBody = useCachedContext && entry ? buildCachedWebSocketRequestBody(entry, fullBody) : fullBody;

const delta = getCachedWebSocketInputDelta(body, continuation);
if (!delta || !continuation.lastResponseId) {
  entry.continuation = undefined;
  return body;
}
return {
  ...body,
  previous_response_id: continuation.lastResponseId,
  input: delta,
};
```

This explains the report's `Received: user`:
 Pi converts the process update to that role before it reaches
the provider.
 The literal diagnostic is absent from the installed Pi sources,
 the installed process extension,
and this workspace's Pi plugins.
 It is persisted as assistant content with provider metadata,
 which makes a
provider or backend protocol diagnostic the strongest explanation for its origin.

## Verification

The installed packages and sources were checked against Pi `v0.80.6`:

- Installed `@earendil-works/pi-coding-agent`:
   `0.80.6`.
- Source tag:
   `v0.80.6`,
   commit `2b3fda9921b5590f285165287bd442a25817f17b`.
- Installed `@aliou/pi-processes`:
   `0.9.4`.
- Source commit:
   `eb523640a02aa90e2c1f665aba62efa53ed88be1`.

Use this read-only JSONL scan to confirm the incident shape without sending a model request:

```js
// Run from the repository root with `node --input-type=module -e '<script>'`.
import { readFileSync } from "node:fs";

const file = "/var/home/user/.pi/agent/sessions/--var-home-user-Monochromatic--/"
  + "2026-07-09T22-31-45-304Z_019f4902-2598-7eb1-822e-239618ee2503.jsonl";
const events = readFileSync(file, "utf8").trim().split("\n").map(JSON.parse);
const diagnostics = events.filter((event) => event.type === "message"
  && event.message?.role === "assistant"
  && event.message.content?.some((part) => part.type === "text"
    && part.text.includes("Unexpected turn continuation")));
const diagnosticIds = new Set(diagnostics.map(({ id }) => id));
const alerts = events.filter((event) => event.type === "custom_message"
  && event.customType === "ad-process:update"
  && diagnosticIds.has(event.parentId));

console.log({ diagnostics: diagnostics.length, linkedLifecycleAlerts: alerts.length });
```

Known-working cases:

- The same session contains process-update messages that are not attached to a continuation diagnostic.
- Pi's mocked regression test
  `package/ai/test/openai-codex-stream.test.ts:1554-1696` accepts a normal cached-WebSocket continuation
  whose response delta begins with one `user` message.

Known-failing cases:

- The saved session has lifecycle and log-watch process-update notifications associated with the quoted
  diagnostic.
- The diagnostic reports that the continuation state expected an assistant or tool message but received the
  user message created from the custom process update.

Two live isolated probes injected one custom process-like update while a Codex response was streaming.
Neither produced provider output:
 the first was stopped after 30 seconds and the second after 47 seconds.
They therefore neither confirm nor reject the hypothesized continuation failure.
 The Pi source test could not
be executed in a clean Node container:
 `npm ci` reported that it could not find a usable lockfile,
 then
`npm install --ignore-scripts` failed with `ERR_INVALID_ARG_TYPE` after the repository bootstrap hook.
Those environment failures do not test the provider behavior.

## Verified workarounds

No end-to-end workaround has yet been verified against the Codex backend.

The following source-verified mitigations reduce the triggering surface and require a fresh provider test
before being called fixes:

- Start routine successful background commands with `alertOnSuccess: false`.
  `pi-processes` still displays the lifecycle update,
   but the end hook does not request a new agent turn.
  Tradeoff:
   the agent does not automatically react when that process succeeds.
- Set `transport` to `"sse"` in `~/.pi/agent/settings.json` to bypass the cached WebSocket continuation
  path.
   Tradeoff:
   this changes latency,
   retry,
   and reliability behavior.
   Pi issue
  [#4945](https://github.com/earendil-works/pi/issues/4945) records environments where SSE has a separate
  first-event reliability problem.

Setting `steeringMode` or `followUpMode` to `"one-at-a-time"` can reduce queued-message batching,
 but is
not a verified remedy.
 The saved failures include individual lifecycle alerts.

## What does not work

- Treating the text as a user-authored prompt.
   The session records it as assistant content from the
  `openai-codex` provider.
- Blaming the configured default model.
   The affected messages use `gpt-5.6-sol`,
   while the global default is
  `gpt-5.6-terra`.
- Assuming every process alert fails.
   The measured session has many process updates with no diagnostic.
- Updating within the installed release.
   The affected version is Pi `0.80.6`,
   the current checked source tag.
  No post-tag fix in the checked path changes the cached-continuation behavior.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/codex-harness.md` does not apply:
 it excludes Codex harness plugins,
 while this incident
uses the Pi harness and an OpenAI Codex provider.
 `.out-of-scope/pi-gpt55-long-context.md` also does not
apply because this is not a request to change model context metadata.

The Pi tracker was searched for the exact diagnostic and for custom-message continuations.
 No matching issue
was found.
 Existing Pi PR [#5162](https://github.com/earendil-works/pi/pull/5162) addressed a different
cached-WebSocket continuation miss that produced duplicate item identifiers.

The filing gate is not met:

1.  **Is it really upstream's fault?**
     No conclusion yet.
     The evidence narrows the fault to the Pi and Codex
    protocol boundary,
     but does not show the raw WebSocket request and response needed to attribute fault.
2.  **Can upstream fix it?**
     Probably.
     Pi can bypass cached continuation for extension-originated messages,
    and the backend can accept a valid continuation.
     The correct boundary is not established.
3.  **Are they supporting this use case?**
     Yes.
     Pi documents extensions and custom `sendMessage` turns,
     and
    ships the `openai-codex` provider.
4.  **Would the repository welcome this contribution?**
     No. Pi's `CONTRIBUTING.md` requires issue text in a
    human voice and states that automated or agent-generated reports can cause a permanent block.
5.  **Will they likely fix it?**
     Unknown.
     No direct duplicate was found,
     and the related continuation fix
    shows the maintainers have addressed this provider area.
6.  **Have we prototyped a minimal compatible fix?**
     No. The live probe did not reach a response,
     and a
    source-only change would not establish the Codex protocol contract.

No issue or comment should be filed from this investigation.
 A future human filing needs a sanitized raw
`response.create` payload and raw server event stream that reproduces the role mismatch.
