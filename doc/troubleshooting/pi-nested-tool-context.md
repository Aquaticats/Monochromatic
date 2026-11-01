# Pi 1.0.0 nested tool calls need separate parent-execution admission

## Symptom

The private auto-mode prototype admits a complete model-issued tool group through its changed agent loop.
An extension tool calling `ctx.executeTool()` does not pass its children through that outer group interface.
In the measured profile,
a saved parent context also executed an inert child after the parent result and final assistant response.
The child reused the original parent's `/1` identifier and did not update its persisted nested-call record.

These are findings about the private consumer's coverage and parent-lifetime requirements,
not evidence of an upstream security promise or a production auto-mode bypass.
No installed SDK or production guard was changed.
The diagnostic used fixed inert parent definitions,
not Codemode source execution or semantic assessment of arbitrary programs.

A separate fixture configuration failure preceded the diagnostic:
agent-core emitted `Tool owned_leaf not found` for both attempted children.
The fixture had excluded that leaf through the SDK `tools` allow-list.
This was not a nested guard rejection or a missing execution context.

## Source identity

The installed artifacts are `@earendil-works/pi-coding-agent@1.0.0`
and `@earendil-works/pi-agent-core@1.0.0`.
Paths beginning with `dist/core/` refer to that installed coding-agent package;
agent-core paths are named explicitly.
The private manifests retain hashes of the inspected installed files and the derived SDK graph.
The upstream `v1.0.0` tag resolves to `a13d35a742c6ef8462812a28fbe1d8c8b7431c32`.
The read-only checkout `~/temp/agent/pi-sdk-1.0.0-source.qxeIPlda` is pinned to that commit.
Its `packages/coding-agent/src/core/nested-tool-calls.ts:163-259`
and `packages/coding-agent/src/core/extensions/wrapper.ts:17-21`
confirm the existing scope map and context-factory path.
The earlier `v0.87.1` source checkout is not substituted for these artifacts.

Private workspace:
`~/temp/agent/auto-mode-consumer-contract.mDLkyNoP`.

- Failed fixture:
  `contract/collector/nested-judgment-intake/`.
- Corrected diagnostic:
  `contract/collector/nested-judgment-intake-v2/`.
- Protected outer-group SDK graph:
  `contract/collector/judgment-sdk-freshness/stage-private/manifest.json`.

## Root cause trace

### The session runs nested calls outside the changed outer loop

`dist/core/agent-session.js:375` creates `NestedToolCallRunner` on demand.
Its host calls native `runToolCall` directly and selects the latest assistant message:

```javascript
// dist/core/agent-session.js:379-397, selected statements
const assistantMessage = this._findLastAssistantMessage();
return runToolCall(toolCall, {
  tools: this._getCallableTools(),
  assistantMessage,
  context: { messages: this.agent.state.messages, tools: this.agent.state.tools },
  beforeToolCall: (context) => this._beforeToolCall(context, parentId),
  afterToolCall: (context) => this._afterToolCall(context, parentId),
  signal,
  onUpdate,
});
```

That path retains ordinary nested hooks.
It does not call the private `beginToolJudgment`,
combined prepared-group assessment,
or captured outer-group freshness callback.
The corrected diagnostic observed those group callbacks for the parent only.

### A tool context captures a caller identifier and checks runner liveness

`dist/core/extensions/runner.js:690` creates the tool context.
The nested method checks the runner and forwards the captured identifier:

```javascript
// dist/core/extensions/runner.js:700-714, selected statements
executeTool: {
  value: async (name, args, options = {}) => {
    runner.assertActive();
    return runner.executeToolFn(toolCallId, name, args, {
      ...options,
      signal: options.signal ?? signal,
    });
  },
},
```

The omitted branch reports an unavailable nested-call function;
it does not add a parent-execution lifetime check.
Session/runner liveness therefore is not proof that the original parent invocation remains active.
A child-supplied signal also needs separate handling when a consumer must retain the original cancellation scope.
The `proc_4202` diagnostic did not test signal substitution.
The separate `proc_8bea` contrast measured that path after the context-lifetime prerequisite.

### Recorder lifetime and caller labels do not preserve execution occurrence identity

`dist/core/nested-tool-calls.js:99` creates a scope when the caller identifier has no current entry:

```javascript
// dist/core/nested-tool-calls.js:99-112, selected statements
let scope = this.scopes.get(callerId);
if (!scope) {
  scope = { recorder: new NestedCallRecorder(), nextId: 1, holdsQueue: false };
  this.scopes.set(callerId, scope);
}
const toolCall = {
  type: "toolCall",
  id: `${callerId}/${scope.nextId++}`,
  name,
  arguments: args ?? {},
};
```

`takeRecord` removes the parent scope.
`dist/core/agent-session.js:698-709` attaches that snapshot to the parent tool result and clears scopes at agent end:

```javascript
// dist/core/agent-session.js:698-709, selected statements
const summary = this._nestedToolCalls.takeRecord(message.toolCallId);
if (summary?.calls) message.nestedCalls = summary.calls;
// In the agent_end branch:
this._nestedToolCalls.clear();
```

The late-context case then created another scope through the same saved method.
The repeated child identifier described a different occurrence;
it did not authenticate continuation of the original parent or revive its judgment.

### The failed fixture confused allowed tools with active declarations

`dist/core/sdk.js:145-148` uses `options.tools` as both an allow-list input and the initial active selection.
`dist/core/agent-session.js:2780-2785` filters extension definitions before building the registry:

```javascript
// dist/core/sdk.js:145-148, selected statements
const allowedToolNames = options.tools ?? (options.noTools === "all" ? [] : undefined);
```

```javascript
// dist/core/agent-session.js:2780-2785, selected statements
const allCustomTools = [/* registered and SDK custom definitions */]
  .filter((tool) => this._isAllowedTool(tool.definition.name));
```

Agent-core `dist/agent-loop.js:486` reports the missing lookup as:

```javascript
// pi-agent-core dist/agent-loop.js:486
createErrorToolResult(`Tool ${toolCall.name} not found`);
```

The corrected fixture allowed both owned tools,
then called `session.setActiveToolsByName(['owned_probe'])`.
Before prompting,
it verified that only the parent was active,
both tools were callable,
and the leaf retained `codemode` exposure.

## Verification

`proc_8724` failed with `AssertionError [ERR_ASSERTION]: 0 !== 2`.
Its persisted parent result contains two `Tool owned_leaf not found` records.
No leaf executed,
and the later planned cases did not run.
That namespace and its consumed source remain unchanged.

`proc_4202` passed the corrected diagnostic using Node `26.10.0`:
three SDK sessions,
six injected wire requests,
three outer inert executions,
seven nested inert executions,
and three canned semantic attempts.
There were no external model requests.
The verifier checked complete streams,
source hashes,
parent persistence,
ordinary nested hooks/events,
and native disposal completion.

Working catalog:

- Registered and allow-listed leaf execution through the actual `ctx.executeTool()` method.
- Parallel children and the native sequential queue.
- Native nested hooks,
  parent identifiers,
  and complete persisted records for awaited children.
- A parent-only model declaration with both parent and leaf callable.

Coverage-gap catalog:

- Child execution did not reenter complete-group preparation,
  judgment construction,
  or the outer freshness callback.
- The cached parent context executed another leaf after the parent completed.
- That call reused the `/1` child identifier.
- The persisted parent record remained unchanged after the late call.

The recorded invocation was:

```bash
# Private one-shot fixture; use a fresh source namespace for another execution.
cd ~/temp/agent/auto-mode-consumer-contract.mDLkyNoP/contract/collector/nested-judgment-intake-v2
mise --no-env --no-hooks run check
```

The fixture deliberately refuses to overwrite its existing evidence directory.
Do not rerun it in the consumed namespace or treat raw result presence as terminal process success.
Heap limits,
cleared fixture directories,
and injected transport are fixture controls,
not OS isolation,
atomicity,
or a hard handback guarantee.

## Original cancellation and a substituted child signal

`proc_8bea` used two actual private SDK sessions and an active inert parent definition.
The parent aborted the agent's original signal,
then awaited a child call.
The default inherited signal produced the native `Operation aborted` result and no leaf execution.
Supplying another live signal executed one inert leaf instead.
Both cases kept the parent invocation active during the call;
this is separate from the saved-context-after-return incident.

Agent-core `dist/agent.js:218-219` implements the abort used by the fixture:

```javascript
// pi-agent-core dist/agent.js:218-219
abort() {
  this.activeRun?.abortController.abort();
}
```

The nested context still selected `options.signal ?? signal`
in `dist/core/extensions/runner.js:713`.
Agent-core's `prepareToolCall` checks that supplied signal after the before-call hook:

```javascript
// pi-agent-core dist/agent-loop.js:499-505, selected statements
if (signal?.aborted) {
  return {
    kind: "immediate",
    result: createErrorToolResult("Operation aborted"),
    isError: true,
  };
}
```

The diagnostic kept two injected wire requests,
two canned semantic attempts,
complete native records,
and successful disposal.
No external inference or real tool effects occurred.
See private `contract/collector/nested-cancellation-intake/result.json`.

A fresh consumer wrapper now forwards the actual parent signal when no different child signal is requested,
or composes both with native `AbortSignal.any`.
Additional child cancellation remains effective rather than being discarded.
`proc_4be3` passed ten runtime/helper controls,
including a positive native-signal control;
`proc_4dbb` removed each cancellation dependency separately and observed the intended assertion failure.
Those helper controls were followed by actual SDK composition in `proc_2cf6`.
Four SDK sessions and four injected requests produced child execution counts `[0, 0, 1, 0]`:
original cancellation blocked inherited and substituted child signals,
live signals allowed execution,
and additional child cancellation still blocked it.
The verifier reconciled complete native streams,
source identities,
parent records,
and disposal.
No deadline,
new model budget,
or permission was introduced.
The private consumer change is:

```javascript
// contract/lifecycle/nested-context-cancellation/wrapper.mjs, selected statements
const requested = options === undefined ? undefined : options.signal;
const effectiveSignal = requested === undefined || requested === signal
  ? signal
  : AbortSignal.any([signal, requested]);
const effectiveOptions = { ...options, signal: effectiveSignal };
```

The full wrapper retains the parent-invocation lifetime check and native context descriptors.
The protected cancellation claim requires an actual parent signal;
legacy unbound calls remain outside that claim.
This is still not an original-judgment carrier or complete nested group admission.

## Verified workaround and remaining implementation

Allow-listing both owned names and activating only the parent repaired the fixture setup.
Its tradeoff is explicit responsibility for maintaining allowed and declared tool sets separately.
This configuration does not establish a parent-execution lifetime or complete nested semantic group.

### Private invocation-lifetime prerequisite

The derived registered-tool wrapper now scopes new nested invocation to the parent's actual execute call.
`proc_cb12` passed seven interface controls using the native helper;
`proc_01d5` verified both retirement and invocation-check omissions.
`proc_fc01` exercised the changed private SDK in three sessions:
live parallel/sequential children remained functional,
while the saved context rejected `NestedContextClosedError` before another child started.
The fixture retained six nested executions instead of the diagnostic's seven.

The consumer-side source is
`contract/lifecycle/nested-context-lifetime/wrapper.mjs`
in the private workspace.
`contract/collector/nested-context-sdk-copy/stage.mjs`
binds it to the native helper and redirects the protected session's existing registered-wrapper imports.
The installed packages and upstream checkout remain unchanged.

The core change is the scoped method and the parent-return retirement:

```javascript
// contract/lifecycle/nested-context-lifetime/wrapper.mjs, selected statements
const descriptors = Object.getOwnPropertyDescriptors(context);
const invoke = descriptors.executeTool.value;
const guarded = Object.defineProperties(Object.create(Object.getPrototypeOf(context)), {
  ...descriptors,
  executeTool: {
    ...descriptors.executeTool,
    value: async (name, args, options) => {
      if (!isActive()) throw new NestedContextClosedError(toolCallId);
      return await invoke(name, args, options);
    },
  },
});
```

```javascript
// contract/lifecycle/nested-context-lifetime/wrapper.mjs, actual execution lifetime
let active = true;
const guarded = guardContext({ context, toolCallId, isActive: () => active });
try {
  return await definition.execute(toolCallId, params, signal, onUpdate, guarded);
} finally {
  active = false;
}
```

Tradeoffs and limits:

- Native context getters stay lazy,
  and method property descriptors are preserved.
- A child invocation already initiated while the parent was active may still settle afterward.
  This is not a claim that a queued child can start without another release check.
- The helper retains exceptional parent exit,
  including `throw undefined`,
  while retiring later invocation.
- Supplied native contexts are a trusted profile,
  not authenticated arbitrary caller objects.
- Original judgment identity,
  original cancellation,
  full nested group admission,
  and complete native event/cleanup parity are separate unfinished work.
- `proc_fc01` imported the actual derived SDK and checked full streams,
  persistence,
  and native disposal;
  it was not an installed-host guard test.

### Exact original occurrence context at native root entry

The later private carrier ties the native context to the existing producer response,
prepared member,
and one-use execution occurrence.
A fixed session reader resolves the original response through that session's current owned stream,
then validates the opaque execution token.
A context copied from its public fields is not an issued context.
The existing registered-wrapper fifth context slot carries that exact object;
no ambient current-parent state or new session/root registry was added.

`proc_681e` passed seven actual SDK profiles after interface and guard controls.
Missing selected context factories rejected the group before native preparation.
Copied contexts entered no definition.
Ordinary tool errors still allowed the next serial member,
while source staleness stopped it.
The issuer occurrence was returned before native result hooks;
local context return alone is not that proof.
The separate `proc_6251` native omission removed issuer retirement after definition return.
Its expected assertion compared active issuer states with required returned states.
Original origins also remained readable during result hooks.
Root effect counts were unchanged;
this is an issuer-lifetime witness,
not a claim of additional effects.

The focused stop-latch omission `proc_8319` removed only the stop-remaining check.
The first member guard-failed;
the otherwise intact second member then entered.
The expected Node assertion was `AssertionError [ERR_ASSERTION]: 2 !== 1`.
This added one session,
two injected requests,
one canned semantic attempt,
and no child executions.

`proc_b861` passed five changed-graph deadline/freshness regressions:
five sessions,
ten injected requests,
two canned semantic attempts,
and definition-entry counts `[0, 0, 0, 2, 1]`.
Missing beginning and combined hooks,
plus expired preparation,
entered no definition.
Execution after evidence close was not capped by the decision deadline;
target change after the first effect stopped the next member.

`proc_6894` passed the mixed-failure profile in one session with two injected requests and one canned attempt.
The first member raised an ordinary tool error;
the second completed;
the third encountered a nested-support guard failure;
the final intact member remained unentered.
Native error flags were `[true, false, true, true]`,
and issuer states were `[returned, returned, returned, unentered]`.
The completed second result remained in native persisted outcomes.
No child executed.

The root-entry graph tested by `proc_681e` has no complete nested-group handler.
The carrier therefore rejects that missing support before starting a child,
rather than delegating to the old independent path.
This is a verified closed default,
not functional qualification of arbitrary nested programs.

Private sources:
`contract/lifecycle/execution-context-carrier/`,
`contract/lifecycle/judgment-execution-occurrences/`,
and `contract/collector/execution-context-sdk-copy/stage.mjs`.

### Standalone declared-group prerequisite

A separate bounded Codemode declaration prototype now has native-worker evidence.
`proc_fda3` passed four positive source cases,
31 rejected source cases,
three catalog rejections,
and a compiled-source size rejection.
It retains the original parent source,
reparses generated callsite aliases,
records canonical argument JSON,
and reconstructs the original source exactly.
These structural checks do not estimate effects.

`proc_6838` passed nine run-local collection cases.
Each callback belongs to a private declared-member record;
two callbacks for the first member do not complete a missing second member.
Equal alias names from separate runs do not combine their membership.
Original cancellation,
native-call cancellation,
parent return,
and staleness retain their separate rejection paths.

`proc_68c2` passed seven actual Codemode 1.0.0 sandbox executions:
original/compiled pairs for ASCII and escaped Unicode parents,
plus duplicate-member,
swapped-argument,
and cancelled incomplete-set controls.
Original and compiled forms retained the tested argument JSON,
call ordering,
and ordered return values under controlled reverse settlement.
`proc_5d76` then omitted only actual-argument versus declared-callsite comparison.
Its expected native assertion was `AssertionError [ERR_ASSERTION]: 1 !== 0`:
the changed collector reached a group consumer that the intact guard prevented.
This added one sandbox execution,
not an SDK session or model request.

The prototype has not yet been connected to SDK child preparation,
the original judgment's child occurrences,
or the native nested recorder and queue.
It is not a workaround for the current closed SDK nested path.
Private evidence is in `contract/lifecycle/codemode-declared-group/`;
the concrete integration work is recorded in
`contract/lifecycle/runtime-continuation/native-declared-group-integration-next.json`.

### Collector lifetime and drain qualification

The unsupported-child recorder/event check `proc_4d8b` passed one SDK session with two injected requests.
An installed native-runner positive control produced child events and a recorder entry.
The protected SDK case produced no child event,
recorder start,
or persisted nested record.
This pins the refusal boundary,
not admitted-child parity.

Collector lifecycle diagnostics `proc_398f` then reproduced post-settlement cancellation relabeling,
a native abort during the final parent-reader callback,
and a separately thrown consumer failure lost behind an existing cancellation.
The fresh `codemode-group-lifecycle` collector passed 17 controls in `proc_2404`,
three actual native Codemode lifetime cases in `proc_3c7d`,
and four exact guard omissions in `proc_3102`.
Transport delivery and eventual completion are now separate:
the native fail-fast case retained a delayed successful completion after its transport call was cancelled.
Collector states do not establish actual SDK execution start.

The subsequent `proc_62e1` contrasts preserved two original failure witnesses and passed four fresh candidate cases.
A synchronous group-consumer callback could capture `drain()` before `work = dispatch()` assigned its result.
The original callback control failed with `AssertionError [ERR_ASSERTION]: true !== false`;
the candidate publishes its drain ticket before entering callbacks.
The original malformed completion array also caused Node to report
`Error: Owned unbound completion failure`.
The candidate observes and drains short,
sparse,
and extra completion arrays,
retaining their outcomes as unbound diagnostics rather than admitted member completions.
The initial review's no-window conclusion covered ordinary callers after invocation return,
not the synchronous callback exercised by this contrast.

The current collector source is
`contract/lifecycle/codemode-group-drain/collect.mjs`.
Consumed predecessors remain preserved.
These checks assume ordinary finite arrays from the fixed native consumer;
they do not qualify hostile publishers or callbacks that await their own drain to produce its pending completions.
No new judgment,
model attempt,
permission,
or runtime execution deadline was introduced.

### Connected literal native group profile

The first connected literal SDK profile passed in `proc_5927`:
two sessions,
four injected requests,
and one original canned assessment per session.
Both native child preparations completed before either child definition entered.
The valid program entered both children;
a native hook changing final child inputs entered neither.
Original child names,
text results,
update events,
recorder entries,
and persisted messages were checked.
No child judgment or model attempt was created.

`proc_994f` removed only the post-preparation input binding.
Its expected native assertion was `AssertionError [ERR_ASSERTION]: 2 !== 0`:
both inert children entered with transformed inputs where the intact group rejected them.
Returned child/root issuer states,
native outcomes,
complete streams,
and disposal were retained.

`proc_79f0` passed three additional SDK cases:
a copied definition with the same name,
schema,
and execute function rejected;
advancing the owned clock after evidence close still allowed both children;
changing a disposable target after the first serial child retained its result and blocked the second.
These cases used three sessions,
six injected requests,
three canned assessments,
and child-entry counts `[0, 2, 1]`.

The graph is the private `contract/collector/native-program-sdk-copy-v3/` derivation.
Its 21 artifacts bind parser dependencies explicitly,
so SDK workers keep their cleared `HOME`.
The failed original parser intake and a separate generator-escaping error remain preserved;
see [SDK staging](pi-sdk-staging.md).
The installed SDK and production auto-mode were not changed.

`proc_15e7` added native `Promise.allSettled` child-failure controls.
An ordinary child error allowed the next serial child to complete;
a required-group guard failure left the next child unentered.
The two sessions used four injected requests and two original canned assessments,
with child-entry counts `[2, 1]`.
This avoids confusing ordinary errors with `Promise.all` fail-fast transport cancellation.

The first queue fixture `proc_ff76` remains failed:
it treated the current `queueTail` as the active first lease.
Fresh `proc_bb45` observed actual queue publications and their predecessor identity instead.
Two parallel program parents contended for one sequential-child queue under one judgment.
It checked four child definitions,
separate recorder summaries,
opaque owner-key consumption before agent end,
four exact update payloads,
and synthetic per-parent usage totals.
This is not a claim about provider billing or arbitrary recursive queue ownership.

`proc_dd3b` cancelled the original agent while the first child definition was running.
The child observed its composed signal abort;
the second definition remained unentered.
Both already-prepared native attempts retained their start/end records.
One SDK session,
one injected request,
and one canned assessment completed with returned parent/child issuer state and clean stderr.
A recorder start is not a definition entry,
and retained diagnostics are not fresh cancelled evidence.

`proc_f6a8` forwarded the non-program root regressions onto the changed wrapper.
Two sessions and four injected requests retained definition-entry counts `[2, 1]`
for execution after evidence close and target change after the first effect.

This qualification covers model-issued native Codemode parents with the declared literal group shapes.
Full image/store parity,
arbitrary recursive queue ownership,
body-return/queued-release combinations beyond the named cases,
and general dynamic groups remain open.
The result-derived native profile then passed `proc_0963`:
three SDK sessions,
six injected requests,
one original canned assessment per session,
and child-definition counts `[3, 1, 1]`.
A producer generated its string only after evidence collection closed;
two later sinks received that exact owned value after complete native preparation.
Advancing the owned clock did not create an execution deadline.
An ordinary producer error left the successor group unreached,
with no successor handles or attempts.
Changing final successor inputs blocked both sink definitions.
The full parent source and one original judgment remained in use.

A separate boundary gap was measured in `proc_0b5a`.
Refusing a converted non-string value became an ordinary VM script error,
so an otherwise unstarted serial root tool still ran.
The fresh guard category and original-program latch passed `proc_408c`:
normal results,
binding refusal,
and ordinary producer failure produced tail-entry counts `[1, 0, 1]`.
Ordinary conversion errors remain distinct from code-owned result guards.

`proc_57e1` omitted only the final guard relay during program drainage.
The expected assertion `AssertionError [ERR_ASSERTION]: 1 !== 0`
showed that retaining a guard marker alone does not stop the sibling root.
`proc_b8ff` refused a copied native producer outcome even though its actual conversion succeeded:
the successor stayed unreached and the sibling root stayed unentered.
`proc_abc3` omitted only the final receipt-derived input comparison.
Its expected assertion `AssertionError [ERR_ASSERTION]: 3 !== 1`
retained the successful producer and two transformed inert sink entries.

Current private sources are
`contract/lifecycle/result-binding-guard/`
and `contract/collector/result-program-sdk-copy-guarded/`.
This is a finite string-result producer/successor profile,
not universal program analysis,
qualified semantic effect estimates,
governing-instruction authority,
human permission,
or production adoption.
Current-use checks remain separate from retained result and cleanup records.

`proc_4ede` then cancelled the original run at a native producer-result update,
after the owned string receipt existed and before the successor activated.
One SDK session made one injected request and one canned assessment.
The completed producer retained its `ok` outcome and unchanged converted value;
no sink started,
the successor stayed unreached,
and the parent issuer returned after drainage.
Receipt readiness is a recorded observation of the owned program,
not authority derived from an event label.
The independent guard review and source-reconciled findings are retained in
`contract/lifecycle/runtime-continuation/result-guard-review-resolution.json`.
Definition ownership is factory-closure ownership,
not session or human authority;
the fixed session reader and registered receiver separately bind each execution.


### Result guards across the native VM error boundary

The category gap belongs to the private integration,
not an upstream defect.
Installed Codemode 1.0.0 `dist/runtime/host.js:183`
serializes a host exception as error text:

```javascript
// Installed pi-codemode dist/runtime/host.js:183
reply = { type: "result", id, ok: false, payload: errorMessage(error) };
```

The VM creates an ordinary error from that text at
`dist/runtime/prelude-source.js:330`:

```javascript
// Installed pi-codemode dist/runtime/prelude-source.js:330
entry.reject(new ErrorCtor(payload));
```

Installed SDK 1.0.0 `dist/extensions/codemode/execute.js:373`
then represents a failed script as a tool result:

```javascript
// Installed pi-coding-agent dist/extensions/codemode/execute.js:373
...(result.ok ? {} : { isError: true }),
```

A private guard class cannot survive those conversions by its class identity alone.
The correction retains the owned guard occurrence in the original program record,
drains native work,
and relays that category at the root execution boundary.
It does not relabel an ordinary producer error or undo a successful native producer.

The intact and omission receipts are
`contract/collector/result-binding-guard-controls/result.json`
and `contract/collector/result-binding-relay-omission/result.json`
in the private prototype.
The consumed command was `mise --no-env --no-hooks run check`
from each corresponding directory.
The old diagnostic and all consumed namespaces remain unchanged.
No upstream issue or comment is proposed for this owned integration failure.

### General nested admission remains open

The connected literal profile is not a verified general nested-admission workaround.
The implementation must extend existing execution/scope ownership,
bind the exact original judgment and parent program,
and reject stale or foreign execution occurrences without manufacturing another budget.
Sibling transport arrival,
queue drains,
and cached caller labels must not stand in for complete semantic membership.
Preserve native hooks,
updates,
recorder/usage output,
and completed effects.

## What does not work

- Treating `codemode` exposure as an override of the SDK allow-list.
- Treating root or runner liveness as evidence that a particular parent execution is still active.
- Treating a child identifier as an unforgeable execution occurrence.
- Treating a successful outer-group guard test as coverage of the separate nested execution path.
- Treating the fixed-parent diagnostic as qualification of arbitrary Codemode programs.

## Upstream filing decision

1.  Upstream fault is not established.
    The excluded leaf was a fixture configuration error;
    the nested judgment requirement belongs to the private consumer.
2.  A consumer-side implementation remains under development.
    No claim of architectural impossibility is made.
3.  Native nested calls are supported;
    no cited upstream contract promises this private whole-program judgment or parent-lifetime policy.
4.  Contribution-policy review is not used to justify any filing here.
5.  No maintainer rejection or intent is inferred from the observed behavior.
6.  A complete nested-admission fix has not been qualified.
    The verified allow-list correction is not that fix.

The inspected `.out-of-scope/pi-gpt55-long-context.md` concerns context-window metadata,
not this case.
No upstream issue or comment is drafted or sent:
the fixture error supplies no upstream bug,
and a complete consumer-side fix is not yet available.
No claim that the upstream tracker lacks a related issue is made.
