# Pi 0.87.1 and 1.0.0 instruction snapshots can miss later prompt transformations

## Pi 1.2.0 complete workspace assessments exceeded single-request construction on 2026-10-15

After capture ceilings were removed,
the configured workspace still failed tool preparation with
`Clause relation requires one through 256 original instruction candidates`.
The emitting owner was
`package/pi-plugin/auto-mode/native/contract/collector/program-rule-batch-sdk-copy-v2/input/instruction-batches.ts`.
Process-local controls measured 615 instruction candidates,
617 total questions,
and 7420722 bytes in the internal selection request.
That byte count is not a model token count or live heap measurement.

The evaluator selected every original candidate into one complete logical assessment,
but the transport encoded that assessment as a single provider request.
The provider's [Decisions API](https://developers.openai.com/api/reference/resources/decisions/methods/create)
documents at most 200 questions per request.
The workspace was not invalid;
the construction and transport layers had conflated a complete assessment with one request.
Other owned checks also rejected aggregate claims,
wire bytes,
and complete retained answers at fixture-derived thresholds.

The user authorized three sequential bundles of at most three parallel calls on 2026-10-15.
Initially only the average USD 0.001 per judgment cost limit was removed.
The user subsequently raised the preparation-inclusive deadline to 15 seconds on the same date.
The unverified size-triggered manual fallback in `8fe7e33410` was withdrawn by `95bd6db768`.

The maintained sender now partitions original question names without changing shared context,
then merges only complete attributed responses.
The native judgment owns bundle and call counters across dispatches and eligible retries.
Sibling failures cancel outstanding transport work;
partial answers cannot authorize execution.
Saved dispatch capabilities cannot enter later dispatches.

The configured native CLI passed Allow execution and prohibition,
denial,
Stop,
dismissal,
abort,
generic prerequisite,
and bare-policy controls with the full copied repository instructions and configured skills.
Its requests contained 200, 200, 200, and 17 questions across two bundles.
These are network-isolated fixtures with authored model replies,
not live-provider latency measurements.

A subsequent real-service replay of those exact request bodies expired at five seconds.
The first bundle dispatched 2566713, 2690737, and 4538472 bytes;
no response headers arrived before cancellation,
and the remaining request was not sent.
Every request contains the same 2152170-byte shared state.
The largest retained source is a 592915-byte `message_end` receipt collection;
the corresponding selected-rule question repeats that collection.
This establishes a remaining live-use failure,
not proof that payload size alone caused the timeout.
The configured offline successes must not be described as full live functionality.

After the deadline increase,
the unchanged third request returned HTTP 400 with `context_length_exceeded`,
`type: invalid_request_error`, and `param: input`.
Its exact response message was
`Your input exceeds the context window of this model. Please adjust your input and try again.`
The response supplied no token count or effective serving limit.
The [Luna model page](https://developers.openai.com/api/docs/models/gpt-6-luna)
documents a 1050000-token context window;
request bytes are not a token measurement.

The user identified the more fundamental regression:
the legacy context selector sends a recent visible-message window,
not every captured source receipt on every decision.
`src/visible-context.ts` selects the larger of five latest visible messages and the latest-user span.
`src/context.ts` obtains Pi's compaction-aware projection;
`src/judge-messages.ts` supplies loaded project files separately.
The native `wire.mjs` instead serializes the complete governing-source collection,
and `wire-clause.mjs` repeats selected source text in question instructions.
The restored assessment scope must be verified before declaring the native repair complete.
Scratch-only byte-sharing experiments are not a replacement for that requirement.

`mise run //package/pi-plugin/auto-mode:test:assessment-bundles`
exercises the built sender with the actual built judgment owner.
The test proves three parallel starts before releasing the first bundle,
complete answer identity,
provider-boundary counts,
cancellation,
malformed-response rejection,
and ownership retirement.
Removing the three-bundle guard through a process-local import hook makes the fourth-bundle test fail
with `Expected original budget operation to reject`;
the unmodified built consumer passes.

This was an owned integration defect;
no upstream Pi issue or patch was filed.

## Pi 1.2.0 native prompt submission rejected by owned capture ceilings on 2026-10-15

### Symptom and root cause

The user restarted the native Pi host and submitted a prompt with bypass off.
The owned `RunPromptCustodyError` displayed
`Native run prompt derivation is incomplete or no longer current`.
The nested deciding error was
`Original callback option input exceeds the source byte bound`.
This was an auto-mode integration defect, not a Pi token limit or provider budget.

At pre-repair commit `5ccee1f7b7`,
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/handler-option-inputs.mjs:16`
applied a single-text fixture limit to a complete serialized callback-option bundle:

```javascript
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/handler-option-inputs.mjs:16 at 5ccee1f7b7
if(Buffer.byteLength(content)>65_536)throw new SourceCollectionError('Original callback option input exceeds the source byte bound');
```

The configured CLI fixture measured 83938 bytes at that boundary.
The tool-loadout source then measured 113905 bytes at another 65536-byte limit.
Process-local controls exposed further rejection of a 2850095-byte linked snapshot at one MiB,
then a follow-up run at 5282773 bytes against four MiB.
With user/project skills,
the shared receipt check failed with 13 resource receipts occupying 158177 bytes
and 110 session receipts occupying 4298044 bytes.
These are repeated serialized evidence bytes, not heap measurements.

The first functionality claim was unsupported:
its CLI smoke test used a minimal policy and only `codemode,read`.
That fixture could pass while the configured workspace and follow-up prompt failed.
Raising one threshold at a time moved the failure to another owned threshold.

### Repair and verification

The user explicitly accepted removal of arbitrary capture size/count ceilings on 2026-10-15.
The repaired capture paths retain complete text, metadata, source lists and clause indexes.
They retain ownership, freshness, cancellation and descriptor validation.
The generic diagnostic now includes the deciding nested error message.

The metadata copier replaces incidental depth-based cycle rejection with explicit ancestor tracking at
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody/messages.mjs:60`:

```javascript
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody/messages.mjs:60
if(active.has(value))throw new SourceCollectionError('Tool metadata contains a cycle');
active.add(value);
pending.push({leave:true,value});
```

The work stack removes an object from `active` after its descendants finish.
Shared metadata can therefore occur in separate branches;
message aliases retain their separate rejection rule.
Receipt counter validation now checks exact numeric representation at
`package/pi-plugin/auto-mode/src/input-receipt-budget.ts:161`:

```typescript
// package/pi-plugin/auto-mode/src/input-receipt-budget.ts:161
if ((!Number.isSafeInteger(resourceUsage.count + sessionUsage.count,))
  || (!Number.isSafeInteger(resourceUsage.bytes + sessionUsage.bytes,))) {
```

Run the built consumer controls from the repository root:

```bash
# Repository root
mise run //package/pi-plugin/auto-mode:build
mise run //package/pi-plugin/auto-mode:test:native:prompt-capacity
mise run //package/pi-plugin/auto-mode:test:native:resource-input
mise run //package/pi-plugin/auto-mode:test:native:event-receipts
mise run //package/pi-plugin/auto-mode:test:native:governing-bindings
```

The original prompt regression was committed red in `49b0dd1b08`.
The repaired built tests pass complete initial/follow-up prompts,
text and lists beyond the former ceilings,
a 10000-level data tree,
shared metadata,
event histories beyond 128 receipts,
and source indexes beyond 512 clauses.
Negative controls still reject cycles, message aliases, unsupported values,
proxy records, accessors, stale sources and duplicate source occurrences.
Type checking passes.

The installed CLI also completed initial and follow-up prompts with bypass off,
the copied repository instructions,
the actual configured packages,
default tools,
and user/project skills.
The private `capture-removal-repro` task under
`${HOME}/temp/agent/pi-tool-call-scope-20261014` uses a disposable home,
read-only package mounts,
network isolation and local provider responses.
It applies no capacity overrides.
This establishes offline prompt handling, not live-provider or physical-terminal behavior.

### Tradeoffs and rejected approaches

Complete capture can retain more memory than the former failing fixture profile.
Arbitrary serialized-byte ceilings did not establish a total-memory guarantee.
Provider/request constraints and executable-program admission remain separate.
Run owners retain original scopes through their session history;
session/resource pools retain original receipt publications and owned copies.
No source is silently truncated or dropped to satisfy a fixture threshold.

Repeated threshold increases were rejected because the next independent threshold failed.
The standard SDK with the legacy extension completed the same prompt control,
but was not installed as a rollback or treated as proof of tool-guard correctness.

### Upstream filing decision

No upstream issue or draft is appropriate.
This incident's deciding checks are owned repository code.
Upstream fault is disproved by the source trace;
fixability, supported upstream use, contribution policy and maintainer willingness
are not gates for this local repair.
The tested fix is the owned consumer patch, not an upstream prototype.
The existing upstream tracker discussion concerns instruction visibility,
not these repository-added capture ceilings.

## Symptom and relevance

An authorization consumer that copies project context during its `before_agent_start` handler
can miss an instruction added by a later handler.
Switching to `ctx.getSystemPrompt()` addresses that particular run-option difference in the tested methods,
but does not expose every request-local message or payload transformation.
A snapshot taken from one surface is not evidence that all governing instructions were collected.

This matters to auto-mode's Q21 requirement:
follow applicable governing instructions,
including legitimate `AGENTS.md` rules,
rather than treating missing human grants as missing authority.
Unknown instruction coverage is not an instruction-free case.
The [confirmed contract](../planning/pi-auto-mode-effect-contract.md) remains unqualified at the actual host.

The original phase used inspected source-method observations.
The separately recorded SDK phase exercised actual sessions and callbacks.
Neither is a reproduced TUI incident or a claim that Pi cannot support a suitable collector through another interface.
No production extension or upstream source was changed.
The fixture markers demonstrate visibility,
not authority:
an extension-added string or provider-payload field does not become a governing instruction merely by appearing there.
Source authority and any legitimate delegation need separate code-owned evidence.

## Source identity

The read-only Pi checkout is `~/temp/agent/pi-input-provenance-2026-09-26`,
at commit `f07218c4d4bbc12bef056a7058c3dd49dfe41abe` for Pi `0.87.1`.
Pi source paths in this document are relative to that checkout.
The probe copied installed compiled code and declarations for `@earendil-works/pi-coding-agent@0.87.1`,
along with pure rendering/replay helpers from its installed `pi-ai` dependency.

The private workspace is `~/temp/agent/auto-mode-consumer-contract.mDLkyNoP`.
`inputs-manifest.json` lists every copied input and hash.
`execution-manifest.json` binds the listed owned controllers,
probe sources,
fixed plan,
and copied inputs.
It is not a hermetic runtime or host-dependency closure.

Execution-manifest SHA-256:
`8ea2c0fd07492c58a3007a680a5aad175076d4fbc52a6d83c0d9dc9fff379c01`.

Image:
`sha256:d15c48492a7dd21a977612aefa661297de4d3253d14c375e7d0157f5bcb5617a`.

Preparation used host Node `v26.10.0`.
The isolated probe reported Node `v24.21.0`.
Do not silently describe this as actual-host Node `v26.10.0` verification.

## Root cause trace

### Base construction options and the current getter are different views

`packages/coding-agent/src/core/extensions/types.ts:352` declares `getSystemPrompt()` on ordinary context.
Line 361 places `getSystemPromptOptions()` on command context.
The corresponding implementation at `extensions/runner.ts:889` constructs that command-only addition:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:897
context.getSystemPromptOptions = () => {
  this.assertActive();
  return this.getSystemPromptOptionsFn();
};
```

The callbacks are bound separately at `agent-session.ts:3124`:

```typescript
// packages/coding-agent/src/core/agent-session.ts:3124
getSystemPrompt: () => this.systemPrompt,
getSystemPromptOptions: () => this._baseSystemPromptOptions,
```

The getter selects run options when present:

```typescript
// packages/coding-agent/src/core/agent-session.ts:1239
get systemPrompt(): string {
  return buildSystemPrompt(this._runSystemPromptOptions ?? this._baseSystemPromptOptions);
}
```

The actual method controls exposed no base-options method on ordinary context,
exposed it on command context,
and observed a later run contribution through the getter while the base-options callback remained unchanged.
The supplied inactive-context check rejected access;
real host lifecycle invalidation was not exercised.

### A handler observes its position in the composition chain

`extensions/runner.ts:1317` creates one mutable normalized option set.
Handlers receive that set in order,
and the combined result is returned after the chain:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1317, selected statements
const currentOptions = normalizeBuildSystemPromptOptions(systemPromptOptions);
const renderCurrentSystemPrompt = (): string => buildSystemPrompt(currentOptions);
```

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1323
ctx.getSystemPrompt = () => {
  this.assertActive();
  return renderCurrentSystemPrompt();
};
```

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1339 and 1363, selected statements
systemPromptOptions: currentOptions,
return { messages, systemPromptOptions: currentOptions };
```

The incumbent copies only the context-file contribution during its own callback,
in `package/pi-plugin/auto-mode/src/index.ts:258`,
`handleBeforeAgentStart()`:

```typescript
// package/pi-plugin/auto-mode/src/index.ts:258, selected statements
const { systemPromptOptions, } = event;
const { contextFiles, } = systemPromptOptions;
currentProjectContext = buildProjectContext(contextFiles,);
```

In the controlled handler chain,
the earlier owned copy retained 1 context file,
while the actual combined options retained 2 after a later addition.
The addition was present in the getter after installing those combined run options in the host stand-in.
That positive control establishes the observed difference;
it does not prove all real extension sources or their authority were captured.
The early observer was a synthetic fixture with the relevant copy behavior,
not execution of auto-mode's handler.
The omission depends on a later contributor existing.
Actual installed extension order was not measured,
so this does not establish that the deployed guard currently misses a particular instruction.

### Request-local transformations do not update the getter's option fields

`extensions/runner.ts:1221` runs `context_with_system` handlers on the full message list:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1225, selected statements
const event: ContextWithSystemEvent = { type: "context_with_system", messages: currentMessages };
const handlerResult = (await handler(event, ctx)) as ContextEventResult | undefined;
currentMessages = handlerResult?.messages ?? currentMessages;
```

The method returned a message containing the request-local marker,
while the session getter still rendered the separately held run options without that marker.
The supplied original message array remained unchanged.
The fixture did not execute the complete SDK request lifecycle.
The stand-in installed combined run options explicitly;
real `AgentSession.prompt()` ordering and next-turn refresh remain unverified.

### Forced text is another projection

`agent-session.ts:1433` wraps the preceding transformation and then applies forced text:

```typescript
// packages/coding-agent/src/core/agent-session.ts:1436, selected statements
const transformed = previousTransformContext ? await previousTransformContext(messages, signal) : messages;
const forced = this._runSystemPromptOptions?.forceSystemPrompt;
if (forced === undefined) return transformed;
```

```typescript
// packages/coding-agent/src/core/agent-session.ts:1440, selected statements
const head: SystemMessage = {
  role: "system",
  content: forced,
  ...(current?.toolsAdded ? { toolsAdded: current.toolsAdded } : {}),
  timestamp: current?.timestamp ?? Date.now(),
};
return [head, ...transformed.filter((message) => message.role !== "system")];
```

In the source-method control,
forced text replaced the preceding request-local marker,
retained the synthetic tool declaration,
and left the supplied transcript array unchanged.
This is not verification of persisted-session replay or tool-removal behavior.

### The provider-payload hook is a separate mutable view

`extensions/runner.ts:1253` accepts and returns a payload independently of the run-option fields:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1264
const handlerResult = await handler(event, ctx);
if (handlerResult !== undefined) {
  currentPayload = handlerResult;
}
```

The actual hook-chain method added a marker to a synthetic payload without changing the session getter.
The test did not use a real provider's payload schema,
encoder,
transport,
or hosted prompt processing.
Do not generalize its synthetic field name into a provider contract.

### A missing leading system message is reported but still returned by this runner method

At `extensions/runner.ts:1230`,
the runner checks whether the handler removed a previously present leading system message,
emits an extension error,
and ultimately returns `currentMessages` at line 1250.
The decisive source fragments are:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1230, beginning of the diagnostic branch
if (hadLeadingSystemMessage && currentMessages[0]?.role !== "system") {
  this.emitError({
```

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1250
return currentMessages;
```

Pi's `ExtensionRunner.emitContext()` reports:

```text
Handler removed the leading system message; the request has no prompt or initial tool declarations. Keep it at index 0 or replace a dropped prefix with getCurrentSystemMessage().
```

The fresh unforced fixture captured that exact diagnostic and received the changed list with no system message.
The diagnostic was retained in the result rather than filtered from stderr.
This does not establish what every downstream provider or a forced projection would do with that list.

## Verification

The frozen source-method phase ran once through `mise`:

```bash
# Private workspace: ~/temp/agent/auto-mode-consumer-contract.mDLkyNoP
mise --no-env --no-hooks run probe
```

This is the recorded invocation,
not an instruction to replay the completed phase.
The controller rejects existing output files.
A separately authorized reproduction needs a new disposable output namespace and its own preserved receipts.
Do not delete current results or bypass the replay guard.

`proc_fcd6` exited successfully.
`probe.verified.json` confirms all 8 scheduled observations and both source-integrity controls.
The copied-input hash guard rejected an equal-length change to an unexecuted opening comment.
A separate artifact with only that hash assertion omitted accepted the same change;
the extracted method-body hash remained identical.
The committed guard was not removed from its original file.

The runtime invocation used 2 GiB memory,
2 CPUs,
zero extra swap,
PID and file-descriptor limits,
a 60-second container lifetime limit,
a non-root user,
a read-only filesystem,
no network,
and no host mounts.
No external model call,
SDK startup,
real session-history load,
or represented action occurred.
Unexpected stderr was not accepted;
the retained successful run's stderr is empty.

### Clean observations

- Ordinary context returned the base prompt with the copied full policy.
- Command context exposed base options and called the supplied active-context check.
- Combined run options and the getter exposed the deliberate later handler contribution.
- Forced projection returned its specified text and retained the fixture tool declaration.
- Fixed case order,
  control receipts,
  baked file hashes,
  image identity,
  current-policy identity,
  and host/runtime execution-manifest binding passed their checks.

### Views that did not contain a later contribution

- An earlier owned context-file copy omitted the later context-file addition.
- Command base options omitted the run-only contribution.
- The session getter omitted the request-local system-message change.
- The session getter omitted the synthetic provider-payload change.

These are view differences,
not interchangeable failures of a single complete-instruction interface.
Actual host integration and human authority remain unverified.

## Actual SDK session observations

The subsequent private `contract/sdk/session-esm` phase ran the same view questions
through real Pi `0.87.1` `AgentSession` instances,
registered extension callbacks,
and an inert custom tool.
A first-party scripted provider supplied all responses;
no external model call or represented operation occurred.

`proc_da53` passed all 4 fixed cases in separate containers:
4 sessions,
8 scripted responses,
and 4 inert tool executions.
Each session emitted one session-start,
agent-start,
and agent-end event,
then completed disposal with empty stderr.
The runtime was Node `v26.10.0`,
with 2 GiB memory,
2 CPUs,
zero extra swap,
64 PIDs,
256 file descriptors,
a read-only root,
disposable tmpfs,
and no external network or host state mounts.

The corrected phase freeze is
`3dcb6e07fdd2a9e9dc813d5011b4693510ddabc7e08e1f1297529cdf2fea255b`.
The image is
`6527f73d748deee6ad0ea84aaea611ad70c8fead2a8c1d05c5b836393fa30c17`.
It used the complete 31,214-byte policy snapshot
`2f4377aa7b950d178999a88337bfd7ec1dba1723313d1650d712e06b2dc3a7cc`,
not the older source-method policy epoch.
The scripted provider views retained the full new policy in every case,
including forced projection.

### Baseline and resource owner

The actual `ResourceLoader` exposed the global fixture file and project policy file.
Its configured inline system prompt had no source path.
The early callback,
tool-time getter,
and both scripted-provider requests exposed the baseline and global markers with full policy text.
The owner inventory remained unchanged throughout the run.
This establishes representation and ownership access from the SDK constructor,
not legitimacy of arbitrary files or production-extension access to that owner.

### Later run contribution

The earlier callback and resource-owner inventory omitted the later fixture contribution.
The actual tool callback's getter and both provider requests included it.
The deliberately registered observer/contributor factory order was exercised by the real runner.
Installed production-extension ordering remains unmeasured.

### Request-local change

Both provider requests contained the marker introduced by `context_with_system`.
The tool-time getter did not.
The handler ran twice,
and the resource-owner inventory stayed unchanged.
This confirms that the measured getter cannot substitute for the request-stage observation.

### Forced projection

The request-local handler produced its marker twice,
but neither provider request retained it after forced projection.
The tool-time getter and provider requests contained the forced marker and complete policy instead.
They omitted the baseline/global markers,
while the resource owner still retained its original inventory.
A changed rendered view does not itself establish a change in instruction authority.

### Preserved failed setup and limits

The original `contract/sdk/session` attempt stopped before creating a session
because its owned helper used CommonJS resolution for Pi AI's import-only entry point.
The correction changed only that import selection in a new epoch;
reference SHA-256 remained
`9acfe7b38a559b9044dceeed88b24c4caf5886d46093d0087f2743de9fa83020`.
No result was relabeled and no completed constructor was replayed.
The [staging report](pi-sdk-staging.md#fixture-import-condition-correction)
retains the diagnostic and source trace.

These are real SDK lifecycle observations,
not real-human authentication,
TUI/RPC qualification,
actual provider encoding,
complete producer coverage,
governing-instruction semantics,
finalizer behavior,
or production timing qualification.
No production extension or cutoff was changed.

## Pi 1.0.0 terminal-stage controls

A later private phase,
`contract/collector/request-stage-controls/`,
ran stock installed Pi 1.0.0 with Node 26.10.0.
`proc_74bc` exited 0:
six new owned `AgentSession` completions,
six scripted local provider requests,
zero external model requests,
and zero represented tool effects.
This is not historical session-ledger reconciliation or a human-witnessed root.

The worker used a cleared disposable home,
no discovered extensions or context files,
no tools,
and disabled agent retries,
compaction,
and cache warming with effective-setting readback.
Its complete streams were private before SDK import,
worker stderr was empty,
and a 60-second process bound was enforced.
The 512 MiB old-space setting is not a total-memory bound.
Listed source and Node hashes were checked;
complete transitive runtime attestation was not claimed.

### Source path and callback order

Paths in this subsection are relative to installed `@earendil-works/pi-coding-agent@1.0.0`,
not the historical 0.87.1 checkout.
`dist/core/sdk.js:213` forwards the payload hook to the extension runner:

```javascript
// @earendil-works/pi-coding-agent/dist/core/sdk.js
const transformProviderPayload = async (payload) => {
    const runner = extensionRunnerRef.current;
    if (!runner?.hasHandlers("before_provider_request"))
        return payload;
    return runner.emitBeforeProviderRequest(payload);
};
```

The SDK installs that callback as `onPayload` at `dist/core/sdk.js:262`.
The provider must actually call it and use its returned payload.
`dist/core/extensions/runner.js:1060` runs the handler chain;
its replacement branch at line 1071 is:

```javascript
// @earendil-works/pi-coding-agent/dist/core/extensions/runner.js
if (handlerResult !== undefined) {
    currentPayload = handlerResult;
}
```

`dist/core/agent-session.js:1310` wraps the preceding context transform and then applies forced prompt text:

```javascript
// @earendil-works/pi-coding-agent/dist/core/agent-session.js
const transformed = previousTransformContext ? await previousTransformContext(messages, signal) : messages;
const forced = this._runSystemPromptOptions?.forceSystemPrompt;
if (forced === undefined)
    return transformed;
```

The owned traces asserted exact callback order,
not merely handler counts.
The combined case confirmed that forced projection replaced the request-local system head before payload rewriting.

### Observed boundaries

- Base and forced cases:
  the getter equaled the normalized provider prompt and final synthetic payload prompt.
- Request-local context case:
  the getter differed from both provider and final payload prompt.
- Payload case:
  the getter matched provider context but omitted the final native-payload additions.
- Combined case:
  forced text replaced the request-local head,
  then both payload handlers added their markers in registration order.
- Omitted-hook control:
  the getter and unchanged payload matched,
  yet the final-payload validator rejected the missing required transformations with `FinalPayloadCoverageError`.

Every persisted session retained the base marker and omitted the forced,
request-local,
and payload markers.
Own entries matched their persisted JSON values;
this was not raw-object identity or byte-for-byte serialization equality.
The existing collector's `assertObservedFresh()` passed on its observed surfaces
even when later payload content differed.
It correctly retained `run-producer-inventory`,
`final-request-transformations`,
and `provider-native-payload` as unqualified gaps.
Its freshness check was never a claim of terminal-request completeness.

A separate disposable guard-removal pair passed with the validator intact
and failed when the mismatch check was removed.
Unexpected setup,
SDK,
retention,
extension,
and cleanup errors were not accepted as that expected rejection.

The collector separately read the complete current mandatory policy,
with SHA-256 `15890c665cdb1c054f8c56c2f212cb75297292b812dc0a2e8065e21156483c39`.
The scripted SDK provider used owned marker prompts,
not a full-policy model assessment.
This phase qualifies source-stage observations for its synthetic native payload protocol,
not real provider encoding,
complete producer coverage,
current permission,
or governing-instruction authority.
The [built-in OpenAI serialization probe](#built-in-openai-serialization-boundary)
subsequently exercised a real provider adapter without external network access.

## Built-in OpenAI serialization boundary

The private `contract/collector/request-wire-controls/` phase completed as `proc_3c50`,
exit 0,
with seven new owned SDK completions and seven captured `openai-completions` requests.
It registered model metadata only,
not a custom stream implementation.
A wrapper around the SDK's public `agent.streamFunction` supplied an injected fetch implementation while delegating
through the original SDK and `ModelRuntime` pipeline.
Global fetch remained a failing tripwire.
No hosted model or external network was used.

Installed Pi AI 1.0.0 `dist/api/openai-completions.js:186` supplies `options.fetch` to its OpenAI client.
The request hook precedes actual client serialization:

```javascript
// @earendil-works/pi-ai/dist/api/openai-completions.js:186
const client = createClient(model, normalizedContext, apiKey, options?.headers, options?.fetch, cacheSessionId, compat);
let params = buildParams(model, normalizedContext, options, compat, cacheRetention, grammarToolInputProperties);
const nextParams = await options?.onPayload?.(params, model);
if (nextParams !== undefined) {
    params = nextParams;
}
```

The client sends those parameters through `chat.completions.create` at line 197.
The injected fetch captured the serialized JSON body,
not an invented payload field or a copy taken inside an earlier extension handler.
It returned owned OpenAI SSE frames,
which the built-in adapter decoded through normal SDK completion.
The central OpenAI 7.19.0 package,
ESM entry/client,
and streaming files were hashed alongside the listed Pi sources.
Full transitive attestation remains outside the claim.

The base,
forced,
request-local,
payload,
combined,
and omitted-hook observations matched the synthetic-payload phase.
An asynchronous first payload handler completed before the second handler ran.
A separate escaping case preserved quotes,
backslashes,
line breaks,
Unicode,
and literal SSE sentinel text through request serialization and response decoding.
The payload validator was reused only after checking byte identity and its existing intact/removal receipt.
Unchanged guard controls were not replayed.

Complete private worker streams and every outcome matched;
worker stderr was empty.
The worker retained its cleared environment and 60-second bound.
Native JSON-value persistence again retained only the base marker,
not forced,
request-local,
or payload additions.
Omitting `onPayload` forwarding produced a normally completed SDK request but failed the final-payload coverage check.
Matching a getter to an encoded prompt is therefore insufficient to prove that every required producer ran.

This establishes the observed serialization boundary for the tested text-only `openai-completions` configuration
and injected transport.
It does not establish hosted behavior,
all provider/transport families,
instruction authority,
current human permission,
or production-ready collection.
The [consumer-owned observation controls](#consumer-owned-observation-controls)
subsequently exercised immutable receipts,
SDK response/root association,
and missing,
foreign,
and stale rejection.

## Consumer-owned observation controls

The private `contract/collector/request-observation-owner/` module passed `proc_c523`,
exit 0:
25 intact cases and five independent guard-removal failures.
The changed copies failed on the exact consumer `Missing expected exception` assertion;
arbitrary exceptions did not count.
Original source stayed unchanged.
These controls reused the existing source collector and made no SDK sessions or external requests.

Its interface wraps a bound stream function,
returns an immutable receipt for an exact response object,
revalidates receipt ownership and observed freshness,
and invalidates old generations.
Receipt copies and response copies do not inherit ownership.
Reused response objects become ambiguous rather than silently changing request association.
The module retains the source collector's coverage gaps and never creates instruction authority or permission.

The unchanged owner then passed stock SDK binding in `proc_cf38`,
exit 0,
under `contract/collector/request-observation-sdk/`:
eight completed sessions,
16 serialized requests,
one inert tool execution,
and no external model requests.
The actual `beforeToolCall` context carried the decoded response object accepted by the owner.
Its receipt matched the exact serialized first request.

Observed stream instances were extensible;
`result` was inherited rather than an own property.
Decorating the method worked,
with iterator identity preserved.
The original SDK argument preparation,
asynchronous permission hook,
and result hook remained active.
The inert tool received `requested-native-hook` and returned the result-hook replacement.
A native hook denial prevented observation lookup and execution.

Missing binding,
a response copy,
a foreign observer,
a fixture epoch change,
an owned resource-loader reload,
and explicit invalidation prevented the inert tool from running.
Copy and foreign-observer controls also verified that the original response still worked with its owning observer.
Root/source/invalidation controls first verified the intact observation.
The loader reload changed the inventory visible at tool preparation;
it was not deferred until another request for this source surface.
The follow-up model request made the earlier receipt stale.

The verifier reconciled exact traces,
measured method/iterator properties,
serialized and persisted tool-result IDs and content,
private worker streams,
listed source hashes,
and unchanged mandatory policy.
Worker stderr was empty.

This is a single-tool consumer check,
not composed-group admission,
production lifecycle-event wiring,
or complete cancellation and handback parity.
The epoch reader was fixture-owned;
stop-turn receipt consumption was not qualified.
Complete producer coverage,
instruction authority,
and a new authenticated SDK-linked human original remain separate requirements.
No installed source or auto-mode production implementation changed.

## Complete prepared-group consumer

The private `contract/collector/request-observation-batch/` follow-up passed `proc_70b6`,
exit 0,
using the unchanged observer and the already tested private SDK/group-dispatch copies.
It completed five SDK sessions,
ten serialized requests,
and five inert member executions,
with no external model requests.
Installed SDK files remained unchanged.

Parallel and sequential groups each completed native preparation for both declared members,
then shared one assessment and one owned request receipt before either member executed.
Final prepared arguments,
member order,
source freshness,
and native result processing were checked at the consumer.
Missing ownership stopped both members before assessment.
An owned loader change during preparation also stopped both before assessment.
Neither failure fell back to independent per-tool approval.

The partial-execution control admitted a sequential group while its observation was fresh.
The first inert member waited until both members had passed release,
then changed the owned loader inventory.
The waiting second member failed precisely at `member-execute`.
The first result remained successful and persisted;
the second was an explicit error.
This is freshness rejection,
not rollback of completed work.

Assessment counts were `1, 1, 0, 0, 1` and execution counts were `2, 2, 0, 0, 1`
for parallel,
sequential,
missing ownership,
preparation-time change,
and post-first-execution change respectively.
Full streams,
per-member outcomes,
source hashes,
and JSON-value persistence reconciled with empty worker stderr.
Real lifecycle epoch wiring,
complete producer coverage,
instruction authority,
and current human permission remain open.

## Actual tree lifecycle observations

The stock SDK diagnostic `contract/collector/request-tree-lifecycle/` completed as `proc_1da7`,
exit 0:
six SDK sessions,
six injected requests,
no tools executed,
and no external model requests.
A stop-response receipt matched its serialized request and remained fresh after normal SDK completion.

The no-op tree target emitted no tree event and preserved the receipt.
A cancelled navigation emitted only `session_before_tree`,
kept the original leaf,
and preserved the receipt.
With the fixture epoch fixed at zero,
actual navigation away and back also left the observer reporting freshness.
The manager identity and observed prompt/resource/policy surfaces remained unchanged.
Those checks did not substitute for a lifecycle epoch.

Wiring `observer.invalidate()` to `session_tree` rejected the old receipt after the callback ran.
Restoring the original leaf did not revive it.
However,
the receipt was still usable inside that callback before invalidation,
after the native leaf had already changed.
This demonstrates post-event rejection,
not closure of the transition window.

Calling `SessionManager.branch()` directly changed and restored the actual leaf without emitting extension tree events.
The constant-epoch observer remained fresh.
An event-only implementation therefore cannot claim direct-manager mutation coverage from these tree-hook controls.
These are limits of the tested observer configuration,
not an installed SDK fix or a grant of authority to recorded messages.

Private streams and all mode records reconciled with empty worker stderr.
The selected source and mandatory policy hashes remained unchanged.
Runtime replacement,
including same-manager in-memory forks,
remains the next separate surface to exercise before choosing lifecycle wiring.

## Actual runtime replacement and fork observations

The private `contract/collector/request-runtime-lifecycle/` phase completed as `proc_7d74`,
exit 0,
using real service/runtime factories.
It constructed 14 SDK sessions;
eight completed an owned response and the replacement sessions were not prompted.
Eight injected requests ran,
with no tools or external model requests.
Every constructed session completed its delegated disposal exactly once.

Cancelled `newSession` kept the original runtime and fresh receipt.
Successful new-session,
switch,
and persisted-fork paths replaced both the `AgentSession` and manager.
An in-memory fork replaced the `AgentSession` but reused its manager.
Without explicit invalidation,
that old receipt was rejected by the stale extension-context check rather than by manager identity.
The SDK extension runner emitted its existing
`This extension ctx is stale after session replacement or reload` diagnostic.
Manager identity alone did not explain that rejection.

In the tested replacement paths,
the receipt remained usable during `session_shutdown` and immediately before the host's
`setBeforeSessionInvalidate` callback.
Explicit invalidation in that callback rejected it before native disposal.
The observed ordering was shutdown,
host invalidation callback,
disposal,
replacement factory,
new session start,
rebind,
and `withSession`.
This does not close an earlier transition window merely because eventual rejection works.

A deliberately failing replacement factory preserved its exact error object.
The outgoing session was already disposed;
no replacement SDK session was constructed,
and its old receipt stayed unusable.
Private streams,
constructed/completed counts,
callback ordering,
selected source hashes,
and policy hashes reconciled with empty worker stderr.

These results cover the named native flows,
not a production lifecycle lease.
The existing registry's fixed-session branch snapshots must not be confused with a lifecycle-only epoch that
survives normal message appends but expires on root changes.
Direct mutations and borrowed mutable entry references still need an explicit coverage contract.

## Borrowed manager values and interception limits

The private `contract/collector/manager-mutation-coverage/` phase passed `proc_152e`,
exit 0,
with 18 owned native managers,
no `AgentSession` construction,
and no model or network requests.
Persistent and in-memory fixtures exercised `getEntry`,
`getEntries`,
`getBranch`,
`getTree`,
`buildSessionProjection`,
and the message object supplied to `appendMessage`.

Every tested path shared the native message object.
Changing its controlled `content` field changed both the active graph and its projection.
Existing persistent JSONL bytes did not change.
Restoring the field restored the graph;
all mutation stayed inside disposable fixtures.
A fresh outer array or tree wrapper therefore did not provide immutable nested values.

The method-dispatch controls distinguished mechanisms that must not be conflated.
An in-place instance hook intercepted an ordinary `branch()` call through another reference to that same manager.
A cached pre-install method and `SessionManager.prototype.branch.call(...)` each moved and restored the leaf
without invoking the instance hook.
Every route then passed a property-call positive control,
proving the hook was installed and capable of observing a call.

This narrows the conclusion:
raw-manager exposure alone does not bypass an in-place hook,
but the measured cached/prototype routes and mutable message aliases defeat a complete-coverage claim.
Method wrapping,
post-mutation events,
and persisted-file comparison are not interchangeable ownership guarantees.
No native getter behavior or installed implementation was changed.

## Copy-boundary experiment staging failure

The first private `contract/lifecycle/manager-copy-boundary/` attempt,
`proc_c72a`,
failed in staging before any worker,
manager fixture,
or model request ran.
Node 26.10.0 `readFileSync` emitted `ENOENT` for the assumed installed
`@earendil-works/pi-coding-agent/LICENSE` path.
The installed 1.0.0 package directory did not contain that file.
The package manifest declared MIT and named the upstream repository,
but that did not prove a local license-file path.

The assumption was in the authored stager,
not a demonstrated SDK runtime defect.
Its original sources and generated module prefix remain intact.
A fresh `manager-copy-boundary-v2` phase validates the retained MIT text against the
[upstream v1.0.0 license](https://github.com/earendil-works/pi/blob/v1.0.0/LICENSE)
before writing generated artifacts.
The verified Git blob is `b0a8e9b81083294360c69b4ec45d3d39a2b28197`.
No copy-boundary behavior or SDK compatibility result is inferred from the failed staging attempt.

The fresh recovery phase subsequently passed `proc_0468`:
30 paired cases,
32 owned native managers,
no SDK sessions,
and no model requests.
The original baseline preserved every tested alias effect.
The copy-in/copy-out variant prevented tested getter,
append-argument,
and preloaded-input mutations from changing canonical graph or projection values.
Raw own-state access still changed both variants.
Persistent bytes and fixture restoration were checked separately.
This first phase is a data-copy result for the inspected profile,
not full encapsulation.

The actual SDK follow-up `contract/lifecycle/manager-copy-sdk/` passed `proc_5f26`:
four completed sessions,
eight injected requests,
two inert tool executions,
and no external model requests.
Original and changed managers both preserved selected final-message replacement,
asynchronous native tool hooks,
before-settle draft preview,
append-only context edits,
continuation payload content,
and persisted JSON values.
The final settle callback was measured in both modes.
Public message-end event identity stayed intact while stored-message identity became separate in the copy variant.
That distinguishes live event coordination from retained state ownership.

This does not qualify auto-retry/recovery omission,
arbitrary extension payloads,
raw own-state protection,
or complete mutation custody.
The installed SDK remained unchanged.

### Private fields isolate data but leave method routes open

`contract/lifecycle/manager-private-fields/` passed `proc_50e1`:
58 declared observations using four owned native managers,
no SDK sessions or model requests.
The experiment converted the pinned copy variant's 11 state fields to JavaScript private fields.
All state fields disappeared from ordinary own-property enumeration.
Public writes to ten tested field names no longer changed canonical read results,
unlike the copy-only baseline.

The persistent `flushed` flag had a separate positive control.
Setting the baseline's public flag to false caused exact `EEXIST` at its owned session path:
the memory entry had been appended,
but the old file bytes remained.
The private-field variant ignored the public shadow and persisted the new append.
The failed baseline fixture was not reused.

Private fields did not close method custody:

-   Cached and prototype `branch` calls still moved/restored the leaf without the selected instance hook.
-   An own getter override still spoofed ordinary `getSessionId()` calls.
    A captured prototype getter returned the actual ID.
-   The compiled public `_loadEntries` method still replaced/restored graph content in both variants.

The result catalogs unexercised methods rather than claiming whole-module compatibility.
The follow-up `manager-private-fields-sdk/` passed `proc_809e`:
two changed-variant SDK sessions,
four injected requests,
one inert execution,
and no external model requests.
It reused the pinned prior probe logic with only the variant schedule and aggregate counts changed.
Measured consumer outcomes matched the retained copy-boundary baseline without replaying unchanged sessions.

Data isolation and those selected consumers are now qualified within their stated profiles.
A native lifecycle occurrence and protected method surface are not implemented or qualified.
The proposed next structural work is recorded in
[the private SDK custody proposal](../planning/pi-auto-mode-sdk-state-custody.md),
now accepted for private prototyping only,
not deployment.

### Intrinsic native occurrences and protected methods

The accepted private SDK custody prototype lives in `contract/lifecycle/sdk-custody-native/`.
It adds opaque root occurrences,
protects the instance/prototype/constructor method slots,
and makes low-level writers private.
The installed declaration exposes `_persist` publicly;
removing that writer is a deliberate interface change for this controlled prototype,
not arbitrary SDK-caller compatibility.

The first run,
`proc_b8f3`,
completed its native workers but failed independent verification.
Node `assert/strict` emitted `AssertionError [ERR_ASSERTION]` because the mutable-instance worker's message
did not match the predeclared method-assignment witness.
Its retained stack points to `sdk-custody-native/check.mjs:35:12`,
the public `leafId` shadow assertion.
The original failure and source remain intact.

`proc_04d7` then exercised four fresh paired controls.
The retained frozen prototype rejected ordinary method assignment in both variants.
Only the unfrozen instance admitted an own getter through `Reflect.defineProperty` and returned its spoofed value.
The intact instance rejected that definition.
This corrects the authored test assumption;
it is not an upstream defect claim.

`proc_2023` reconciled the retained workers with the specific stack/source witness and that supplement,
without replaying the native matrix.
It admitted 24 intact native-manager cases and 12 independent guard-removal outcomes.
Cached/prototype navigation changed the native occurrence;
returning to the old leaf did not revive it.
The append family preserved its lifecycle occurrence.
A fault immediately before the branch write retained the original error and old leaf,
while the occurrence had already changed.
Separate omissions exercised each structural marker site.

No SDK agent session or external model request ran in this native-manager phase.
The stronger protected-method variant then passed selected actual SDK consumers in `proc_51fd`:
two completed sessions,
four injected requests,
one inert execution,
no external model calls.
Its measured message/context-edit outcomes matched retained baseline records.
This does not establish every SDK caller's compatibility.

Independent source review found separate registry-integration defects:
a callback could invalidate a root after its last freshness check,
and a replacement-reader failure could occur after transition finalization.
`proc_f13e` reproduced nine failing controls on the preserved draft.
The remedy validates after callbacks,
prepares fallible replacement observations before finalization,
and publishes commit/cancellation state without further callbacks.
Failed preparation keeps transition cleanup available;
an unreturned begin handle is cleaned up internally.

`proc_9258` passed 24 retained legacy cases,
17 native-reader protocol cases,
nine callback/publication cases,
and 11 independent guard-removal variants.
These use constructor-owned doubles,
not actual SDK roots.
The actual binding in `root-lifecycle-intrinsic-sdk/` then passed `proc_2e77`:
six completed SDK sessions,
six injected requests,
no tool execution,
and no external model calls.
No instance mutator hook was installed.

No-op and cancelled navigation resumed the original receipt.
Managed navigation,
a native pre-write failure,
and cached/prototype calls retired old receipts;
restoring the original leaf did not revive them.
The exact native failure survived with unchanged leaf/entry values.
Five refused stream attempts stopped during suspension before request generation advanced.

This qualifies the tested native-manager occurrence,
registry,
and request-observer integration.
It does not establish all `AgentSession` or runtime-state custody,
instruction authority,
current human permission,
or production integration.
Session/runtime replacement and other state writers remain separate custody work.

### Direct session disposal is not a manager occurrence change

The focused follow-up `session-disposal-intake/` passed its diagnostic checks in `proc_7f17`:
one owned SDK session,
one injected request,
one disposal,
and no tools or external model calls.
This is a measured gap,
not a safety pass.

The manager-backed lifecycle lease and full request receipt were both active in an owned abort hook
before the SDK invalidated its extension context.
After disposal,
the manager occurrence and entry JSON were unchanged.
Lifecycle-only assertion and a new lifecycle capture still succeeded.
The full request observer rejected,
with Pi SDK 1.0.0 `ExtensionRunner.assertActive` emitting a plain `Error` whose message begins
`This extension ctx is stale after session replacement or reload.`
The collector propagated that SDK error;
it was not a `SourceCollectionError` fingerprint mismatch.

The abort portion of `AgentSession.dispose()` is explicit in `dist/core/agent-session.js:977`:

```js
// Installed SDK: dist/core/agent-session.js, disposal entry excerpt.
dispose() {
    try {
        this.abortRetry();
        this.abortCompaction();
        this.abortBranchSummary();
        this.abortBash();
        this.agent.abort();
    }
    catch {
        // Dispose must succeed even if an abort hook throws.
    }
```

The immediately following statement at line 988 begins with this call,
using the retained stale-context diagnostic:

```js
// Installed SDK: dist/core/agent-session.js, next statement's prefix.
this._extensionRunner.invalidate(
```

The same class declares ordinary fields at lines 82 and 83:

```js
// Installed SDK: dist/core/agent-session.js, field declarations.
agent;
sessionManager;
```

`dist/core/agent-session-runtime.js:6` imports its own installed manager module:

```js
// Installed SDK: dist/core/agent-session-runtime.js.
import { SessionManager } from "./session-manager.js";
```

Providing a protected manager to an initial SDK session does not itself change that import
or redirect the runtime's replacement constructors.

The private extension now observes session liveness and its actual manager pairing,
not inferred liveness from unchanged manager state.
`root-lifecycle-session/` passed `proc_59d2`:
24 legacy,
17 native-reader,
nine callback,
and six pair-reader cases,
plus 12 guard-removal variants.

The staged SDK session has a private monotone disposed flag,
a protected actual-manager slot,
and protected disposal entry points.
`session-lifetime-sdk-controls-v2/` passed `proc_deae`:
six SDK sessions/requests,
four intact cases,
and two exact guard-failure controls,
with no tools or external model calls.
Direct,
cached,
and prototype disposal rejected old lifecycle/receipt use before abort hooks.
Original abort delegation,
a controlled swallowed abort error,
SDK context invalidation,
and cleanup remained observable.
Removing the flag or moving it after abort hooks failed the same pre-abort assertion.

The first preparation `proc_f507` stopped before its SDK worker:
an authored `join()` path still named `root-lifecycle-native` while comparing
`root-lifecycle-session` source hashes.
Its Node `AssertionError [ERR_ASSERTION]`,
partial preparation,
and original source remain preserved.
The fresh v2 phase corrected that directory literal;
it did not replay an earlier SDK execution.

Compatibility checks against the changed factory also passed:
`proc_7c32` completed two content-consumer sessions,
four injected requests,
and one inert execution;
`proc_791c` completed six navigation-consumer sessions and six injected requests without tools.
No external model calls ran.
Message/context-edit behavior matched retained baseline outcomes,
and navigation preserved the tested suspension/retirement behavior.

Runtime operation-entry suspension,
replacement factory imports,
terminal disposal,
callback reentrancy,
other mutable state,
and content freshness remain separate requirements.
A runtime-owned operation module is a proposed scope extension,
not an implemented or qualified part of these results.

### Runtime-owned operation and constructor profile

The accepted private runtime extension reuses the existing registry rather than adding another owner registry.
`completeRetirement` passed `proc_68df` with 12 cases and three guard-removal controls:
it finishes an already-retired transition without issuing a replacement root.
It is distinct from failed retirement and from cancellation.

The stock SDK callback intake `proc_ef9b` constructed eight sessions across three cases,
without model requests.
Awaited nested replacements completed from both rebind and `withSession` callbacks.
A callback failure after application left the new session live.
The private coordinator preserves that behavior rather than treating application as reversible.

`runtime-custody/` passed `proc_a3be`:
23 pure cases and six guard removals.
`runtime-owned-sdk/` then passed `proc_07ea`:
12 actual SDK cases,
21 constructed sessions,
no model requests,
and no tools.
The cases exercised new/switch/import,
constant-manager and persisted forks,
cancellation,
pre-effect import failure,
factory failure,
post-apply callback failure,
post-apply nesting,
and terminal disposal.

The old lease was suspended before before-switch/fork handlers and retired before shutdown/disposal.
Cached/prototype dispatch reached protected native operation entry.
All factory managers came from the declared constructor functions.
Each constructed session entered native disposal once in the schedule;
completion followed the native return,
not merely the session's early closed flag.

This is a controlled factory profile.
The publisher in the private generator `contract/lifecycle/runtime-sdk-copy/generate-runtime.mjs:40`
contains only synchronous private-field assignments from prepared data:

```js
// Private repository: contract/lifecycle/runtime-sdk-copy/generate-runtime.mjs
this.#operations.apply({ result, publish: (prepared) => {
    this.#_session = prepared.session;
    this.#_services = prepared.services;
    this.#_diagnostics = prepared.diagnostics;
    this.#_modelFallbackMessage = prepared.modelFallbackMessage;
} });
```

Owned non-proxy results remain a producer precondition;
data-descriptor checks alone do not establish it.
`proc_4f1c` separately demonstrated that an arbitrary throwing publisher can leave an admitted successor
without a coordinator cleanup route,
and that contradictory arbitrary body results are not enforced by the generic helper.
Those are excluded-contract findings,
not approved recovery behavior.

Independent review then identified an initial-result admission asymmetry in the private coordinator,
not an upstream SDK defect.
The fresh `runtime-initial-admission-v2/` snapshot validates initial records and checks the initial manager
before invoking its factory.
`proc_ebd6` passed ten admission cases,
three guard-removal controls,
and a new assertion that failed against the retained previous source.
It also passed six SDK cases with 11 constructed sessions and no model requests.
Native cleanup failure after the early disposed flag propagated its original error;
incomplete cleanup was retried while the root stayed retired.
Completed cleanup was not repeated.
Nested factory failures preserved the original error and left the applied-but-disposed root unavailable.
A nested post-apply callback failure preserved the latest live root.
The earlier 12-case schedule was retained,
not replayed.

The injected native cleanup error occurred at `dist/core/agent-session.js:989` in Pi 1.0.0,
after its abort catch and extension-context invalidation:

```js
// Pi 1.0.0: dist/core/agent-session.js:989
this._disconnectFromAgent();
this._eventListeners = [];
```

The fixture threw once from the disconnection delegate after observing the private disposed flag.
This tested a native disposal that had entered but not completed,
not an observed default SDK failure.
Both failed attempts were retained;
all 11 constructed sessions subsequently completed cleanup.

Factory session-start callbacks in the actual runtime fixtures only recorded events:
constructor-phase model requests or protected actions were not admitted.
Cleanup after idle external disposal or mutation was not exercised.
Other mutable SDK state,
all instruction producers,
current human permission,
and production integration remain outside the claim.

### Constructor-phase requests and publication conflict

`proc_587f` passed a fresh diagnostic against the unchanged private runtime:
five cases,
nine SDK sessions,
and nine locally injected requests.
An initial `session_start` request completed before runtime creation returned.
A new-manager replacement request also completed during `session_start`,
but runtime publication then emitted
`RootTransitionError: Replacement manager already has an active root`.
A factory error after such a request left the candidate session and receipt active
when the old runtime closed.
Explicit fixture cleanup retired them.
A same-manager fork refused capture during the pending transition,
then succeeded after application.

Pi 1.0.0 `dist/core/agent-session.js:2582` awaits startup before later resource discovery:

```js
// Pi 1.0.0: dist/core/agent-session.js, bindExtensions
this._applyExtensionBindings(this._extensionRunner);
await this._extensionRunner.emit(this._sessionStartEvent);
this._extensionRunner.reportUnhandledMcpServers();
await this.extendResourcesFromExtensions(this._sessionStartEvent.reason === "reload" ? "reload" : "startup");
```

The private registry `contract/lifecycle/runtime-root-lifecycle/lifecycle.mjs:72`
creates a root for an owned pair during capture:

```js
// Private registry: contract/lifecycle/runtime-root-lifecycle/lifecycle.mjs
if(!root)root=createRoot({session,manager,canonical});
```

Its replacement publication at line 123 rejects that independently active root:

```js
// Private registry: contract/lifecycle/runtime-root-lifecycle/lifecycle.mjs
if(existing&&existing!==operation.root&&!retired(existing))throw new RootTransitionError('Replacement manager already has an active root');
```

This is a private occurrence-ownership/publication gap,
not an upstream defect or proof that all startup requests must be denied.
The required continuation is factory-scoped ownership in the existing registry:
adopt only the exact fresh candidate on success and retire abandoned candidates.
No startup request,
constructor identity,
or proposed occurrence token establishes human permission or another judgment budget.

### Idle external disposal blocks runtime cleanup

`proc_2d14` measured four actual SDK cases with four injected requests and no external calls.
Ordinary runtime close completed.
After direct complete or partial session disposal,
first runtime close propagated `NativeSessionDisposedError`,
then another close rejected the already-retired lease.
After native branch-away/back,
runtime close rejected the retired lease without disposing the still-live session.
Explicit fixture cleanup completed the owned sessions.

The private operation owner in `contract/lifecycle/runtime-construction-operations/operations.mjs`
used active-root entry for terminal work as well as replacements:

```js
// Private repository: contract/lifecycle/runtime-construction-operations/operations.mjs
if(!unavailable)transition=lifecycle.begin(lease);
```

The remedy separates terminal-only invalidation from ordinary active-root admission.
Its token must not become a replacement or construction token.
`proc_8d27` verified that distinction with 16 cases and five guard removals;
the original `proc_e63c` verifier failure remains preserved.
Native cleanup completion also needs a separate observation:
the existing disposed flag is set before abort and later cleanup can still throw.
A completion reader must stay incomplete during nested disposal and after an outer failure.
`proc_9e54` passed six intact SDK cases and three omission controls using nine sessions,
with no model requests.
The completion-aware runtime then passed `proc_4ad5`:
four sessions/four injected requests,
successful close in every declared mode,
no repeated completed native disposal,
and a successful retry after partial disposal.

Shutdown then host invalidation callbacks retained their order.
After a prior direct disposal they ran late and observed stale extension context;
this is an observed compatibility detail,
not complete native event parity.
This is a private implementation issue,
not an upstream SDK defect or permission grant.

### Registration-fault fixture used an invalid branch argument

The constructor-request tail `proc_cc01` expected a native occurrence change,
but its diagnostic callback called `SessionManager.branch(null)`.
Pi 1.0.0 declares `branch(branchFromId: string)` in `dist/core/session-manager.d.ts:347`.
Its implementation rejects the absent entry before changing the leaf:

```js
// Pi 1.0.0: dist/core/session-manager.js:1161
branch(branchFromId) {
    if (!this.byId.has(branchFromId)) {
        throw new Error(`Entry ${branchFromId} not found`);
    }
    this.leafId = branchFromId;
}
```

The lifecycle diagnostic callback retained `Error: Entry null not found`,
so no occurrence change happened and the candidate request/publication completed.
The correct native operation is `resetLeaf()` at `dist/core/session-manager.js:1172`.
The fresh paired control `proc_06bb` verified both outcomes:
the invalid call retained its exact error,
while `resetLeaf()` changed the occurrence,
rejected registration,
and cleaned the candidate before a request.

`proc_0c7e` then reconciled the intended constructor profile from retained and corrected cases:
nine cases,
15 SDK sessions,
and 14 injected requests.
The source-stage cleanup and invalid-branch diagnostics remain separate.
Original `proc_aa49` and `proc_cc01` failures were not relabeled successful or replayed.

### Auxiliary producers bypass ordinary run hooks

`proc_941b` exercised five actual SDK producers with five sessions and five injected requests:
ordinary prompt,
`sendUserMessage`,
manual compaction,
branch summary,
and bug-report summary.
No external model or tool ran.

Prompt and `sendUserMessage` traversed `before_agent_start`,
`context_with_system`,
and `before_provider_request`.
The auxiliary summaries used the same session stream function,
but supplied no ordinary payload hook and built different instruction text.
Their ordinary rendered getter/resource-base observations remained unchanged.
The branch-summary receipt became stale after native navigation.

Pi 1.0.0 forwards `this.agent.streamFunction` from `dist/core/agent-session.js:2111`,
line 3240,
and line 3478 into the auxiliary summarizers.
`dist/core/compaction/compaction.js:485` invokes that function directly:

```js
// Pi 1.0.0: dist/core/compaction/compaction.js:485
const produce = async () => streamFn
    ? (await streamFn(model, context, requestOptions)).result()
    : completeSimple(model, context, requestOptions);
```

This invalidates a blanket requirement that every request traverse the ordinary run-hook chain.
A producer inventory must follow the actual native request builder,
and any main-agent action consumer must distinguish auxiliary responses from main-agent responses.
Neither producer identity nor an observed hook chain authenticates governing instructions or a human grant.
The existing collector coverage gaps remain open;
they are not closed by deleting gap labels or by matching a hardcoded prompt marker.

### Producer and wire evidence constrain the group consumer separately

`proc_e862` exercised code-owned request origin at actual native main-agent and auxiliary call sites.
`proc_1aaa` then connected that origin to the existing complete prepared-group host.
The private helper in `contract/collector/request-producer-group-consumer/request.mjs:3` composes the checks:

```js
// Private prototype: contract/collector/request-producer-group-consumer/request.mjs
export function readMainAgentRequest({producerOwner,observer,stream,response}) {
  producerOwner.assertMainAgentResponse({stream,response});
  return observer.observationFor(response);
}
```

The private observer's `contract/lifecycle/root-lifecycle-retirement-entry/observer.mjs:52` calls `validate(scope)`.
That function invokes `assertScope` at line 41,
which calls `lifecycle.assertActive` and `collector.assertObservedFresh` at lines 37 and 38:

```js
// Private prototype: root-lifecycle-retirement-entry/observer.mjs:52
function observationFor(response) {
  const scope = object(response) ? responses.get(response) : undefined;
  if (!scope) throw new RequestObservationOwnershipError('Response is not associated with this observer');
  validate(scope);
  return scope.receipt;
}
```

Thus `observationFor` is not merely an identity lookup.
The helper does not grant permission or decide policy.

`proc_1aaa` qualified the producer/wire gate at the actual complete prepared-group consumer:
six SDK sessions,
13 injected requests,
and five inert executions.
Parallel and sequential groups each assessed once before both members executed.
Missing wire evidence,
a copied response,
and a genuine same-session auxiliary-summary response each blocked both members before assessment.
When the source changed after the first serial result,
the second member was blocked and the completed first result remained.
This still uses a fixed inert assessment,
not a qualified policy or semantic verdict.

`proc_bed1` reconciled the separate pure consumer controls without replaying workers:
three cases and two guard removals.
The original `proc_6333` summary incorrectly reported six removals from a copied literal count;
its source and result remain preserved.
The next boundary is one enclosing judgment's budget,
code-owned evidence/decision handling,
and dependency finalization.
Start its clock before provenance checks and preparation,
not after either gate.

The separate native-host omission control `proc_3b7a` used one session/two injected requests.
Removing the host checkpoint callback reached assessment where the fixture expected none,
producing exact `AssertionError [ERR_ASSERTION]: 1 !== 0`.
A separate assessment receipt assertion still prevented execution;
this proves omission detection,
not an unsafe execution.
No intact SDK worker was replayed.

## Pi 1.0.0 instruction originals versus source labels

The private instruction-origin intake distinguishes native file selection,
callback replacement,
and rendering.
It does not assign instruction authority or change production auto-mode.
The source checkout for this subsection is
`~/temp/agent/pi-sdk-1.0.0-source.qxeIPlda`,
commit `a13d35a742c6ef8462812a28fbe1d8c8b7431c32`.
Paths in its source excerpts are relative to that checkout.
The worker imported installed SDK 1.0.0 modules directly under Node 26.10.0.

### Source selection and replacement

The native loader selects the first supported context filename in a directory.
The selection order is explicit at
`packages/coding-agent/src/core/resource-loader.ts:185`:

```typescript
// packages/coding-agent/src/core/resource-loader.ts:185
const candidates = ["AGENTS.override.md", "AGENTS.md", "AGENTS.MD", "CLAUDE.md", "CLAUDE.MD"];
```

That selection is a loader fact,
not proof of a file's governing authority.
The fixture confirmed that a selected `AGENTS.override.md`
does not delete or change the underlying `AGENTS.md`.
The guard's required complete current policy remains a separate input.

The selected file records can then be replaced by a callback.
At `packages/coding-agent/src/core/resource-loader.ts:642`:

```typescript
// packages/coding-agent/src/core/resource-loader.ts:642
const resolvedAgentsFiles = this.agentsFilesOverride ? this.agentsFilesOverride(agentsFiles) : agentsFiles;
this.agentsFiles = resolvedAgentsFiles.agentsFiles;
```

The fixture callback returned a record with an actual original file's path
but different content.
The getter exposed the replacement;
the original file bytes stayed unchanged.
Pairing getter text with a path label therefore does not establish that those bytes came from that file.

The configured prompt path is also retained independently of replacement text.
At `packages/coding-agent/src/core/resource-loader.ts:646`:

```typescript
// packages/coding-agent/src/core/resource-loader.ts:646
const baseSystemPrompt = resolvePromptInput(systemPromptSource, "system prompt");
this.systemPrompt = this.systemPromptOverride ? this.systemPromptOverride(baseSystemPrompt) : baseSystemPrompt;
this.systemPromptSourcePath =
    systemPromptSource && existsSync(systemPromptSource) ? resolvePath(systemPromptSource) : undefined;
```

The probe observed the unchanged original path alongside callback-replaced prompt text.
Append text and source paths likewise follow different operations:

```typescript
// packages/coding-agent/src/core/resource-loader.ts:659
this.appendSystemPrompt = this.appendSystemPromptOverride
    ? this.appendSystemPromptOverride(baseAppend)
    : baseAppend;
this.appendSystemPromptSourcePaths = appendSources
    .filter((source) => existsSync(source))
    .map((source) => resolvePath(source));
```

The fixture returned reordered and added append text:
three text entries with one file-path entry.
The existing collector correctly leaves their text-to-path relation unestablished;
array position cannot supply the missing relation.

### Equal sections are not necessarily equal whole prompts

The first worker,
`proc_02e4`,
failed at its final comparison with Node's
`AssertionError [ERR_ASSERTION]: Expected values to be strictly equal`
at `instruction-origin-intake/probe.mjs:56:10`.
Its whole-prompt equality expectation was wrong.
The generated project section preceded `cwd`,
while the new custom section followed it.

The deciding builder statements are at
`packages/coding-agent/src/core/system-prompt.ts:164`
and line 170:

```typescript
// packages/coding-agent/src/core/system-prompt.ts:164
if (contextFiles.length > 0) promptSections.project_context = renderProjectContext(contextFiles);
```

```typescript
// packages/coding-agent/src/core/system-prompt.ts:170
promptSections.cwd = cwd.replace(/\\/g, "/");
for (const [name, content] of Object.entries(customSections)) {
    if (content) promptSections[name] = content;
}
```

The fresh builder-only recovery,
`proc_aa7a`,
verified equal project-section bytes but different unforced whole-prompt bytes.
It also verified whole-prompt equality through an explicit forced-text input,
without context-file originals.
That is a different representation path,
not a relabeled success for the failed unforced comparison.
The forced branch is explicit at
`packages/coding-agent/src/core/system-prompt.ts:190`:

```typescript
// packages/coding-agent/src/core/system-prompt.ts:190
if (input.forceSystemPrompt !== undefined) return { content: input.forceSystemPrompt };
```

Neither the section text nor forced rendered text authenticates its original source.

### Verification scope and next consumer work

Both namespaces used `mise --no-env --no-hooks run check`
from their respective directories under
`~/temp/agent/auto-mode-consumer-contract.mDLkyNoP/contract/collector/`.
The controllers refuse existing output namespaces.
Keep the consumed artifacts;
do not rerun them in place.

The original failed worker reached four completed loader assertion groups
before the final comparison failed.
The recovery checked original source identities,
the exact failure location,
and retained streams to reconcile that straight-line prefix.
Those are reconstructed completed-prefix observations,
not separately persisted successful case receipts.
The original worker remains failed.

The recovery created no additional resource loader and no `AgentSession`.
It exercised only the corrected native builder comparison.
Both phases made no model request or external fetch.
The original failed stream remains retained;
the recovery stream checks passed.
Cleared home,
private descriptors,
a 512 MiB old-space setting,
and a 60-second worker bound are fixture controls,
not OS isolation or total-memory guarantees.

Positive controls include exact native original file records,
unchanged source files,
and visibly different whole prompts when section order changes.
Rejected shortcuts are path-label authentication,
append-array index pairing,
and the initial whole-prompt equality expectation.
No production workaround or complete source-authority collector is qualified.
The next implementation must retain native originals before transformations,
link subsequent representations to their actual producers,
and keep source authority and legitimate delegation distinct from those observations.
The existing upstream filing decision remains unchanged:
these supported transformation behaviors expose a consumer requirement,
not a newly established upstream defect.

### Owned capture, pending cleanup, and SDK error wrapping

A private constructor owner now captures the native callback inputs before transformation,
not just the final getter results.
It fixes the selected reader and callback identities,
retains ordinary bounded input copies,
and publishes only after the inspected native reload returns with its required phases.
Direct callback calls do not create authority;
a forged call during a reload makes the later genuine phase fail instead of publishing that record.
`proc_5602` passed 17 pure controls and `proc_6b95` passed four actual native-loader cases.

The raw-output comparison is separate from byte equality.
The private `contract/collector/resource-input-custody-v2/owner.mjs:66`
checks both:

```javascript
// Private prototype: resource-input-custody-v2/owner.mjs:66, selected condition
outputs.raw.some((value,index)=>value!==references[index]) ||
JSON.stringify(outputs.copies)!==JSON.stringify(scope.phases.map(value=>value.after))
```

`proc_0116` removed only the identity term.
An identical copied array then failed the intended guard-sensitivity assertion:
`AssertionError [ERR_ASSERTION]: Missing expected exception (ResourceInputCustodyError).`
This was a new native-loader case,
not a replay of the intact matrix.

The first private owner delayed its guard failure while draining an unsupported Promise result.
Its `contract/collector/resource-input-custody/owner.mjs:86` awaited the result before handing back the failure:

```javascript
// Private prototype: resource-input-custody/owner.mjs:86
scope.unbound=(await Promise.all(scope.pending)).flat();
```

The corrected owner separates native reload completion from pending unsupported work.
`proc_ef3d` compared both sources with manually controlled settlement:
the original had recorded the failure but had not returned it,
while the correction returned the failure with cleanup still pending.
Both retained the later rejection,
including `undefined`,
and all test work was settled and drained.
New reloads and current-use capture stay refused until that cleanup finishes.
This is not a hard-stall guarantee or support for arbitrary Promise subclasses,
modified Promise behavior,
hostile thenables,
or a callback waiting on its own drain.

The actual SDK integration initially failed a fixture assertion in `proc_9b98`.
The fixture expected the collector's leaf error directly.
The existing private observer wraps source failures at
`contract/lifecycle/root-lifecycle-retirement-entry/observer.mjs:39`:

```javascript
// Private prototype: root-lifecycle-retirement-entry/observer.mjs:39, final throw
throw new RequestObservationStaleError('Observed request is no longer current',{cause:error});
```

The first completed baseline was preserved.
The fresh changed-source case in `proc_edba` asserted both
`RequestObservationStaleError: Observed request is no longer current`
and its cause,
`SourceCollectionError: Native resource reload replaced this snapshot`.
A same-byte native reload blocked the second serial root while retaining the first outcome.
No source-owner implementation change was needed for this fixture correction.

The intended SDK pair consists of the retained baseline and fresh changed-source case:
two sessions,
four injected requests,
two canned assessments,
and definition counts `[2, 1]`.
The original failed suffix adds a separate session,
two requests,
and one canned assessment to the execution ledger.
It remains failed,
not silently replaced by the recovery.

The source checks,
complete streams,
persisted outcomes,
and disposal checks passed for the accepted cases.
The tasks were `mise --no-env --no-hooks run check`
in the named private phase directories.
No external model ran.
The new owner still reports source authority,
raw-file identity,
and delegation as unestablished.
A successful native reload is not a permission grant.
The workaround is private input capture with conservative stale-result refusal;
unsupported publishers and subsequent prompt transformations remain outside its qualified scope.
There is no new upstream defect or filing artifact.

## Owned native base prompt and mutable working copies

### Measured helper gaps

The first private base helper checked only its canonical options identity,
then returned its retained view at
`contract/collector/base-prompt-custody/base-owner.mjs:40`:

```javascript
// Private base-prompt-custody/base-owner.mjs:40
return current.view;
```

It did not revalidate the resource occurrence.
`proc_14bd` first proved a valid original read,
changed the actual owned resource occurrence through reload,
and confirmed that the expected refusal was missing.
The fresh helper rechecks that original relation at current use:

```javascript
// Private base-prompt-custody-v2/base-owner.mjs:47
const frame=readBoundResourceFrame({binding:scope.binding,loader:scope.loader});
```

```javascript
// Private base-prompt-custody-v2/base-owner.mjs:48
if(frame!==scope.view.resourceFrame||current!==scope||active)throw new SourceCollectionError('Original resource occurrence or native base changed before current read');
```

A separate resource diagnostic,
`proc_8da2`,
replaced the ordinary output field with an accessor returning the same array.
The original reader invoked that accessor and accepted the view.
The correction checks the output's data-property descriptor before any native reader can invoke it:

```javascript
// Private base-prompt-custody-v2/resource-owner.mjs:74
if(!field||!Object.hasOwn(field,'value'))throw new SourceCollectionError('Native resource output field '+name+' is not an own data property');
```

The corrected pure controls,
`proc_3378`,
passed 17 cases.
Separate single-guard controls,
`proc_d6a3`,
removed only the original-resource comparison or only the output descriptor rejection.
Both produced
`AssertionError [ERR_ASSERTION]: Missing expected exception (BaseCompositionError).`
The accessor mutant invoked the getter once;
the intact case rejected it without invocation.
No SDK matrix was replayed for these pure sensitivity controls.

### Native copies and lifetime

The private native session now owns base options in a private field
and builds them through the recorded native read/normalization operation.
The inspected SDK normalizer explicitly constructs both optional prompt keys,
even when their values are undefined:

```javascript
// SDK 1.0.2: dist/core/system-prompt.js:11
customPrompt: input.customPrompt,
forceSystemPrompt: input.forceSystemPrompt,
```

The schema copy rebuilds nested skill-source metadata before freezing.
It does not freeze the loader's borrowed source object.
The private SDK supplies a new copy to each command-context read
and to the native run-hook input:

```javascript
// Private base-prompt-sdk-copy/stage-private/session.mjs:2826
getSystemPromptOptions: () => copyPromptOptions(this.#baseSystemPromptOptions),
```

```javascript
// Private base-prompt-sdk-copy/stage-private/session.mjs:1635
const result = await this._extensionRunner.emitBeforeAgentStart(expandedText, currentImages, copyPromptOptions(this.#baseSystemPromptOptions));
```

`proc_516c` verified this through actual SDK 1.0.2 sessions:
two sessions,
one injected request,
and no semantic assessment.
A command changed its returned copy without changing canonical base data.
A `before_agent_start` handler changed its mutable working copy,
and that change reached the actual request.
The canonical base stayed unchanged.
Same-value resource reload made the base stale until an actual native rebuild.
Missing opaque binding refused protected collection,
and disposal refused current reads while retaining inspection.

This is private copy and current-use behavior.
It does not authenticate strings merely because they match,
make a copied working object canonical,
or establish governing-source authority.

### Preserved fixture failure and original-judgment consumer

The first native fixture,
`proc_c33b`,
installed a direct stream wrapper rather than registering its consumer.
It created one session,
made no wire request,
completed disposal,
and persisted
`RequestProducerError: Native producer stream is not bound to this issuer`.
The deciding check was already present at
`contract/collector/request-producer-judgment-consumers/owner.mjs:77`:

```javascript
// Private request-producer-judgment-consumers/owner.mjs:77
if(!consumer)throw new RequestProducerError('Native producer stream is not bound to this issuer');
```

The corrected fixture used the existing `producerOwner.bindConsumer`.
It also separated the JSONL header from the entries comparison:
the private native manager's `getEntries()` excludes the header:

```javascript
// Private base-prompt-sdk-copy/stage-private/manager.mjs:1115
return structuredClone(this.#fileEntries.filter((e) => e.type !== "session"));
```

That comparison had not been reached in the original failure,
so it is not a second observed failure.
No base-helper or SDK behavior change was needed for this fixture correction.

`proc_02ab` exercised the base-linked collector in the actual original judgment:
two sessions,
four injected requests,
two canned assessments,
and root-definition counts `[2, 1]`.
The same-value native base rebuild produced
`RequestObservationStaleError: Observed request is no longer current`,
caused by
`SourceCollectionError: Native base prompt was rebuilt after this source snapshot`.
The first outcome remained recorded and the second definition did not start.
The original assessment budget stayed closed.

These native phases used `mise --no-env --no-hooks run check`
in their named private directories.
Source identities,
complete streams,
persisted results,
and disposal were checked.
No external model ran.
The original failed namespace remains unchanged.

### Limits and upstream disposition

The private remedy owns base data and refuses stale current use,
but does not claim complete run-option or final-request source coverage.
Legitimate handler mutations remain supported through working copies;
later handler ancestry and projection need their own records.
The finite profile is not a new universal policy restriction.
Unlinked native options remain observations only;
protected readers cannot release them as qualified source evidence.

The defects were in the owned helpers and fixture.
No upstream defect or contribution is established,
and no upstream issue or patch is proposed.
The existing upstream-filing decision remains unchanged.

## Pi 1.0.2 owned run publication and first-turn fixture failure

### Symptoms and deciding source

The private v2 run helper had separate measured defects.
`proc_73ca` changed the canonical prompt by mutating a retained handler object
after emitter return but before publication.
`proc_9709` observed one invocation of a thrown object's `toJSON`
during publication-size accounting.
Neither was an upstream defect.

The v3 transfer boundary is
`contract/collector/run-prompt-custody-v3/owner.mjs:120`:

```javascript
// Private collector/run-prompt-custody-v3/owner.mjs:120
scope.delivery=copyPromptOptions(working);
```

Native reconciliation receives this separate mutable copy.
Final publication copies again.
At `owner.mjs:130`,
the published failure projection selects inert fields instead of raw thrown objects:

```javascript
// Private collector/run-prompt-custody-v3/owner.mjs:130, projection expression
Object.freeze({occurrence:failure.identity,ordinal,at:failure.at})
```

Raw thrown values remain in retained inspection.
The tradeoff is deliberate:
serialized source observations identify failure occurrences,
while full error objects require diagnostic inspection.
Before/after observations do not prove that a handler solely authored every mutation
during an asynchronous interval.

The first SDK fixture,
`proc_602f`,
failed its own assertion:

```text
# Private collector/run-prompt-native-controls/probe.mjs:77
assert(retained.history.some(value=>value.kind==='next-turn-refresh'&&value.state==='published'))
```

It had produced one text response,
not a tool-bearing turn followed by another response.
The installed Pi agent-core 1.0.2 `dist/agent-loop.js:92-93`
guards next-turn preparation:

```javascript
// Installed @earendil-works/pi-agent-core 1.0.2, dist/agent-loop.js:92-93
if (lastCompletedTurn) {
    const nextTurnSnapshot = await config.prepareNextTurn?.(lastCompletedTurn);
```

The call sits inside the continuing-turn loop at `dist/agent-loop.js:90`.
Expecting it after a single terminal text response was the wrong fixture assumption.
The failed worker retained one session,
one injected request,
and completed disposal;
it did not persist a completed-case receipt.

### Verification and corrected consumer controls

`proc_1b81` passed 21 pure v2 controls.
`proc_e503` passed the v3 changed-boundary controls.
`proc_3237` independently removed only the reconciliation copy
or only the inert failure projection.
The first omission produced an `ERR_ASSERTION`:
actual `LATE_BORROWED_MUTATION`,
expected `undefined`.
The second produced `ERR_ASSERTION` with actual `1`,
expected `0`.
The intact controls passed.
These were pure helpers,
not additional SDK executions.

The clean native suffix,
`proc_0a79`,
used one SDK 1.0.2 session and one injected request.
Its ordered handlers exercised the event and context renderers,
forced text,
a same-tool base rebuild,
a custom text message,
and a partial mutation followed by `throw undefined`.
The error was reported once.
A retained alias mutation after the native normalization await did not reach the wire.
Current source capture,
public run/runner shadows,
native reset,
persisted entries,
disposal,
and retained inspection were checked.
This single-response case correctly observed no next-turn refresh.

`proc_d157` supplied actual tool-bearing turns and genuine subsequent responses:
two sessions,
four injected requests,
two canned assessments,
and root-definition counts `[2, 1]`.
Both genuine second responses observed `next-turn-refresh` source records.
The negative case also injected the real native refresh callback during the first inert root
to test source staleness;
that injection does not authenticate a scheduled second turn.
It changed only the run occurrence,
not its bytes or base identity.
The original judgment rejected the second root with:

```text
# Private collector/run-prompt-judgment-controls/result.json
RequestObservationStaleError: Observed request is no longer current
  cause: SourceCollectionError: Native run prompt was replaced after this source snapshot
```

The first outcome and original closed inference budget survived.
The decisive collector comparison is
`contract/collector/run-prompt-sdk-copy-v2/resource-owner.mjs:185`:

```javascript
// Private collector/run-prompt-sdk-copy-v2/resource-owner.mjs:185, comparison expression
readSessionRun(session)!==saved.run
```

Run native fixtures with `mise --no-env --no-hooks run check`
in their named fresh private directories;
the consumed namespaces must not be rerun.
The guard fixture uses the same task name.
Receipts verify full streams,
listed source identities,
persisted results,
and cleanup.
No external semantic model ran.

### Limits and upstream disposition

Do not repair this fixture by forcing next-turn preparation on the initial response.
That would change the behavior being tested.
Keep the failed worker separate from the corrected suffix
and use a genuine subsequent turn for refresh coverage.

The private graph preserves native mutable handler behavior while isolating its published values.
It does not establish source authority,
instruction applicability,
permission,
or final policy.
Images,
arbitrary publishers,
pending-hook session/runner replacement,
hard stalls,
and production retention limits remain unqualified.
History is retained for the session lifetime;
per-operation bounds are not a total-memory guarantee.

There is no new upstream defect or contribution to file.
The existing upstream-filing decision remains unchanged:
the faulty assumption and helper defects belong to the private consumer.

## Verified workarounds and limits

The source-method control showed that reading the later getter sees the tested run-option contribution
missed by the earlier owned copy.
It also showed that this is not a complete workaround for request-local or payload-only changes.
No production collector workaround is qualified by this phase.

The consumer design must retain mandatory current policy separately,
track admitted instruction sources and composition stages,
and distinguish incomplete coverage from established absence.
Those are contract requirements,
not a claim that a single existing getter already implements them.
Remaining integration work may use composition or a different host interface;
no whole-toolset impossibility is established.

## What does not work

- Treating an early context-file snapshot as all governing instructions.
- Treating command base options as the complete current run state.
- Treating `getSystemPrompt()` as proof that every request-local or payload rewrite was observed.
- Treating a role,
  filename,
  or prompt string as an authenticated source or instruction-coverage witness.
- Treating the missing-leading-system diagnostic as a veto by this runner method.
- Declaring Node `v26.10.0`,
  real TUI/RPC,
  persisted replay,
  human-authentication,
  or provider-encoding qualification from this Node `v24.21.0` method probe.

## Upstream filing artifact

Nothing is filed or drafted.
The completed probe establishes consumer-relevant composition behavior,
not a new upstream defect.

The bounded tracker searches were `getSystemPrompt` in issues
and `context_with_system` in pull requests,
each with a limit of 20 results.
Bodies and comments were read for the relevant matches:

- [Issue 3539](https://github.com/earendil-works/pi/issues/3539)
  already discusses context-file versus system-prompt visibility and later payload rewrites.
  Its earlier in-handler getter mismatch was fixed;
  the inspected `emitBeforeAgentStart()` binds a live chain renderer at `runner.ts:1323`.
  Do not re-prototype that historical mismatch against the current source.
- [PR 9846](https://github.com/earendil-works/pi/pull/9846)
  added the current separation between `context` and `context_with_system`.
  Its restoration behavior and full-system stage are present in the inspected source.
- [Issue 9932](https://github.com/earendil-works/pi/issues/9932)
  concerns tool-removal drift with forced text.
  This probe did not remove a tool and does not reproduce or fix that reported incident.
- [Issue 9432](https://github.com/earendil-works/pi/issues/9432)
  proposes stable session-start contributions with source metadata.
  It is a proposal,
  not an installed capability assumed by this contract.

### Upstream filing decision

1.  Upstream fault:
    not established.
    The tested transformations are documented extension capabilities.
2.  Fixability:
    no impossibility claim and no upstream fix target selected.
    A consumer collection interface still needs investigation and actual-host testing.
3.  Supported use:
    extensions and prompt transformations are supported;
    a complete authorization-source witness is not established by these methods.
4.  Contribution policy:
    the pinned `CONTRIBUTING.md` was read.
    It requires understood contributions,
    human-voice issue reporting or clearly labeled AI follow-up,
    and contributor approval gates.
    No human-authored filing or contribution is proposed here.
5.  Maintainer willingness:
    no refusal of this consumer use case is inferred.
    Existing discussion already explains the relevant visibility limits;
    no additive defect report has been established.
6.  Fix prototype:
    none.
    This is a source-method observation harness,
    not an upstream patch or a qualified production workaround.

`.out-of-scope/pi-gpt55-long-context.md` was checked;
its model-window topic does not cover these instruction-view observations.
The inspected `.out-of-scope/codex-harness.md` excludes the Codex harness,
not Pi;
`.out-of-scope/terminal-title-fork-parity-tests.md` excludes a particular cross-harness title test,
not this source-view probe.
No other exclusion filename indicated this topic.
Any later filing requires its own applicable exclusion,
contribution,
duplicate,
and tested-fix checks.
