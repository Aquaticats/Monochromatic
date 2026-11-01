# Auto-mode on Pi 1.2.0 blocks authorized tool groups

## Process metadata and queued custom inputs on 2026-10-17

### Symptoms and separate incidents

Session `2026-10-17T04-53-14-520Z_01a14834-fc97-70de-9615-03cc0636384b`
reports two approval failures.
The mixed `process.start` and `bash` group containing `chatcmpl-tool-b60c3965f752386f`
receives `approve: 0.48`, `deny: 0.34`, `unknown: 0.18`.
The model answered,
but its score does not reach the retained `0.60` cutoff.
The final Bash pair containing `chatcmpl-tool-a7a4468548c5838a` instead throws
`AssessmentTransportError: Assessment HTTP request or response failed; no provider answer was admitted`.
The historical exception was discarded;
its underlying cause remains unknown.
Three unchanged saved-request replays of that final pair succeed at `approve: 0.88`.

The accompanying `notify-send` SIGTERM warning concerns approval notification delivery.
It does not establish either permission-review failure's cause.

Full configured replays expose additional boundaries:

- Visible process results and completion messages contain raw shell commands in metadata.
- Queuing a custom message during assessment produces
  `SourceCollectionError: Native run prompt was replaced after this source snapshot`.
- Queuing custom data during a primary model request produces
  `SourceCollectionError: Only original unregistered manager or native event input occurrences may extend this source snapshot`.

The two freshness messages come from different checks.
Neither proves that instruction text changed.

### Source trace

`package/pi-plugin/auto-mode/src/review-operation.ts:46`
originally projected only Bash calls.
The maintained projection now includes process starts while retaining other process actions:

```ts
// package/pi-plugin/auto-mode/src/review-operation.ts
if (name !== 'bash' && !(name === 'process' && input.action === 'start')) return input;
```

Replacing only the process command copies in the saved mixed-group request with parser structure
changes the result to `approve: 0.69`, `deny: 0.20`, `unknown: 0.11`
in three identical live trials.
Execution still uses the original input.

That call projection does not cover returned process metadata.
`package/pi-plugin/auto-mode/src/visible-context.ts:207`
and `:232` now select metadata projection by the original tool name or exact custom-message type.
`package/pi-plugin/auto-mode/src/review-process-result.ts:47`
projects known command fields while preserving output text and unrelated data:

```ts
// package/pi-plugin/auto-mode/src/review-process-result.ts
const projected=typeof details.command==='string'?projectReviewInput({name:'bash',input:details}):details;
```

The same module handles `process`, `processes[]`, and `result.info` records.
An arbitrary data property named `command` does not select this path.

The custom-message freshness chain begins in
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/prompt-input-source.ts:75`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/prompt-input-source.ts
to:'        captureNativeCustomMessageInput({manager:this.#sessionManager,message:appMessage});\n        if (options?.deliverAs === "nextTurn") {'
```

The injected capture precedes the SDK's delivery choice.
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-user-inputs.mjs:185`
retains its original construction:

```js
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-user-inputs.mjs
retainConstructedInput({
  carrier:message,
  value,
  metadata,
  inputField:'customMessage',
  inputOrigin:'native-custom-construction'
});
```

`loaded-pool.mjs:79` appends that origin to the constructed-input array.
Before the correction,
`run-prompt-custody-v3/lifecycle.mjs` responded to any array change with `retainContextInputs([])`.
`run-prompt-custody-v3/context.mjs:59` then replaced the run-composition object:

```js
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/context.mjs
const view=Object.freeze({
  ...scope.view,
  retainedMessageOrigins
});
check(scope);scope.view=view;scope.constructedInputs=constructedInputs;
```

`collector/rule-relevance-sdk-copy/input/resource-bindings.mjs:149`
rejects that changed identity:

```js
// package/pi-plugin/auto-mode/native/contract/collector/rule-relevance-sdk-copy/input/resource-bindings.mjs
if(readSessionRun(session)!==saved.run)throw new SourceCollectionError('Native run prompt was replaced after this source snapshot');
```

The lifecycle now retains pending custom data without replacing the already observed run.
Actual context admission still retains the original publication and serialized-request contribution proof.
User instructions do not take that custom-data path.

A second rejection occurs after queuing custom data during the primary request.
`integration/policy-source/manager-source-growth.mjs:77`
allowed appended manager and event inputs but omitted native custom constructions.
The focused `notification-primary` test fails against `native-release-GSPIDj`.
Diagnostic instrumentation identifies `native-custom-construction`,
role `custom`,
permission `false`,
matching original source/publication occurrences,
and no registration or run replacement.
The correction shares the custom-data classifier with lifecycle reads;
all existing ownership,
prefix-order,
permission,
and occurrence checks remain.
The 18-mode native consumer catalog passes against `native-release-J4fNNL`.

Transport recovery is separate.
`package/pi-plugin/auto-mode/src/assessment-transport.ts:174`
retains only a sanitized category for actual non-aborted fetch TypeErrors:

```ts
// package/pi-plugin/auto-mode/src/assessment-transport.ts
if (!options.signal?.aborted && error instanceof TypeError)
  throw new AssessmentRequestFailure('Assessment HTTP invocation failed with TypeError; no provider answer was admitted');
```

`src/assessment-failure.ts:22` classifies complete failed bundles for the existing bounded retry owner.
It does not manufacture a model answer or add another retry budget.
Credential,
cancellation,
response,
HTTP-status,
protocol,
mixed-bundle,
forged-name,
and cyclic-aggregate failures remain ineligible.
This recovery does not identify the discarded historical transport exception.

### Verification and current limit

Run the maintained consumer tasks from the repository:

```sh
# package/pi-plugin/auto-mode
mise run //package/pi-plugin/auto-mode:test:native:phases
mise run //package/pi-plugin/auto-mode:test:visible-context
mise run //package/pi-plugin/auto-mode:test:assessment-transport
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback
```

The pre-correction native lifecycle catalog passes parallel and sequential execution,
argument preparation,
awaited updates,
callback withholding,
denial,
cancellation,
and custom messages queued during assessment.
Queued user instructions and changed policy files withhold every member.
The new catalog adds custom messages,
user steering,
and policy changes during primary requests.
Every mode passes,
including context,
steering,
and followup delivery of custom data during primary requests.
User steering and policy changes during those requests execute no proposed member.
The package type check passes.
The denial mutant removes only the dispatch guard from a copied built module;
three actual executions make the maintained denial test fail,
while the production artifact hash remains unchanged.

Payload fixture `pi-hyper-rpc-allow-jZsxRg` contains nine raw structured command fields.
The same audit finds none across the 17 requests in `pi-hyper-rpc-allow-QaT10o`.
The latter still contains an intermediate tool failure,
so its payload result does not establish interaction completeness.
Configured fixture `pi-hyper-rpc-allow-GRsLAm` confirms the intermediate `process.output` succeeds,
then reaches the separate source-growth rejection before its next Bash pair.
The complete configured replay must pass before renewing MiMo handoff readiness.
The separate mocked configured replay `pi-hyper-rpc-allow-tN2zEc` exits with status `137`;
no termination cause is established,
and it is not counted as passing evidence.

### Corrections, tradeoffs, and rejected approaches

The correction preserves the `0.60` cutoff,
Luna selection,
source attribution,
complete-group decisions,
and the existing attempt budget.
Parsed shell structure replaces executable syntax only at the review boundary.
Literal command-looking data and complete visible output remain available.
Pending custom messages remain captured and must reach later actual requests.
Changing the input-origin classification does not allow new user instructions to reuse an old decision.

Lowering the cutoff would bypass the observed uncertainty rather than repair the missing projection.
Projecting proposed calls alone misses returned metadata.
Equating a new run object with changed instruction text was an incorrect diagnosis.
Suppressing all freshness errors would lose the user-stop and policy-change boundary.
Replaying only the reported final pair misses the preceding process and notification behavior.

### Upstream filing decision

The deciding failures are in this repository's projection and source tracking.
No external dependency defect is established for these incidents.
The upstream-filing check therefore fails its external-ownership prerequisite;
the remaining upstream contribution checks do not apply.
No upstream issue or external message is authorized or needed.

## Pi 1.2.0 review expiry and unprojected Bash inputs on 2026-10-16

### Symptom and retained evidence

Session `2026-10-16T03-32-01-642Z_01a142c4-45e9-75fa-b579-ea10d43323ae`
displayed `Auto-mode could not assess this operation: Original policy review and evidence closure failed`
for `chatcmpl-tool-a23317ad323e080b` and `chatcmpl-tool-9328d924e6057489`.
Review `01906150-53fc-4313-94ad-0c7ae40d26df` retains
`JudgmentDeadlineError: Judgment preparation and assessment deadline expired`.
Its first request begins `9772` milliseconds after review start and returns `1050` milliseconds later.
The final request has no retained answer before the deadline failure.
This evidence establishes elapsed-time rejection; it does not establish a low-confidence final model answer.

### Root cause and correction

At the parent of commit `77001d41e6`,
`package/pi-plugin/auto-mode/native/contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.ts:126`
rejects elapsed preparation and assessment time:

```ts
// package/pi-plugin/auto-mode/native/contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.ts
if(current>=deadlineAt)throw new JudgmentDeadlineError('Judgment preparation and assessment deadline expired');
```

The user clarified that fifteen seconds is an implementation guideline.
The corrected budget retains monotonic clock validation and cancellation,
and returns the remaining advisory interval even when negative.
The current `candidate.ts:129` contains:

```ts
// package/pi-plugin/auto-mode/native/contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.ts
return targetAt-current;
```

The HTTP sender carries the original `signal` directly into `fetch`;
there is no elapsed-target abort timer.
Startup relevance likewise uses an untimed signal instead of its former `45_000` millisecond timer.
Transport errors, actual cancellation, stale inputs, and provider-call accounting remain separate failure paths.

The user also required no model thinking and locally parsed Bash structure.
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/wire-material.mjs:43`
projects tool arguments before model requests:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/wire-material.mjs
return block.type==='toolCall'?{...block,arguments:projectReviewInput({name:block.name,input:block.arguments})}:block;
```

The same file's `projectPreparedOperation` covers prepared inputs.
Historical tool calls and direct Bash entries use the same maintained projection.
The incumbent `unbash` parser supplies quoting, nested execution, control flow, and redirection roles;
the original command stays local for execution and ownership checks.
Human confirmation uses a separate thinking-free summary so parsing failure does not disable manual review.
Literal argument data and quoted here-document bodies remain data even when they resemble shell commands.

### Verification and remaining boundary

Installed candidate `native-release-b8o8BL` contains the correction.
The focused consumer commands are:

```sh
# package/pi-plugin/auto-mode
mise run //package/pi-plugin/auto-mode:test:native:shell-projection
mise run //package/pi-plugin/auto-mode:test:native:startup-relevance
mise run //package/pi-plugin/auto-mode:test:native:judgment-budget
mise run //package/pi-plugin/auto-mode:test:assessment-transport
mise run //package/pi-plugin/auto-mode:test:decision-wire
mise run //package/pi-plugin/auto-mode:test:native:dependency-facade
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback
mise run //package/pi-plugin/auto-mode:test:visible-context
```

Passing catalogs include late responses, delayed credential preparation,
pipelines, logical operators, quoted literal substitutions, active substitutions,
process substitutions, arithmetic, parameter defaults, and quoted or active here-documents.
Rejected catalogs include incomplete Bash, cancelled requests, stale inputs,
unavailable credentials, malformed responses, and established prohibitions.
The thinking-exclusion and late-answer tests failed against the original installed behavior before correction.

The first complete-session replay stopped after eleven tool results with native exit status `137`.
No native fatal diagnostic was retained, and the output ceiling was not reached.
It is incomplete evidence, with no established termination cause.
The first bounded replay executes all original calls but exposes a harness defect:
`rg` is absent, and an all-success assertion contradicts the saved transcript's failed search.
The corrected final replay `pi-hyper-rpc-allow-Caf02F` mounts the actual executable,
retains the original conversation and configured resources,
and completes all 21 calls across 12 groups without confirmation in a disposable two-CPU, 2 GiB container.
Both reported tool-call identifiers complete successfully.
The original failed search retains exit status `1` rather than a missing-executable status `127`.
All 25 Luna requests succeed; saved-payload inspection covers proposed operations,
prepared members, per-question copies, and selected historical messages.
Those surfaces contain no assistant thinking, provider signatures, or raw Bash command fields.
Installed prohibition fixture `pi-hyper-rpc-prohibited-nNA56F` blocks both reads without confirmation.
The context, constructor, preparation, and live/retained dependency regressions also pass,
as do the shared analyzer tests, type checks, focused lint, and the 22-case fallback matrix.

### Workaround tradeoffs and rejected approaches

No larger hard timeout implements an advisory timing requirement.
Ignoring parsing failures would lose permission-relevant operations.
Instead, incomplete structure reaches original human confirmation when available,
and headless execution remains withheld.
Parsing alone does not prove runtime expansion values, reachability, or command effects.

### Upstream filing decision

Nothing to file upstream: the enforced timer and payload projection are repository-owned integration behavior.
The ownership gate fails for Pi and the parser; no upstream change is needed to implement this correction.
The other filing gates, supported use, contribution policy, maintainer interest, and upstream prototype,
do not become filing obligations for this consumer defect.

## Symptom

On 2026-10-15 the user reported `Root lease has been retired` and `Connection error.`.
The session beginning at 22:17:29 UTC records a switch from Hyper to
`llmgateway-devpass/mimo-v2.6-pro`,
two connection failures with recovery context edits,
then a switch to `opencode-go/mimo-v2.6-pro` and a retired-lease failure.
The user confirms Esc, edit/retry, `/tree`, and `/reload` between attempts.
The transcript does not record the full UI action order or the original nested connection-error cause.

The failures reproduce independently in the installed native host without tool calls or external network access.
Provider switching fails before the fixture transport receives the next request.
Successful tree navigation causes the next prompt to fail before transport.
These are separate mechanisms with a shared architectural coupling,
not evidence that one error caused the other.

## Root cause

The endpoint and root-lease excerpts describe the reproduction baseline at commit `c408b94879`.
The replacement described in "Implemented correction" removed those admission paths on 2026-10-15.

### Provider switching retains the startup endpoint

`package/pi-plugin/auto-mode/native/contract/integration/native-host/bootstrap.ts:173`
reads the model during session binding.
At line 187 it computes the endpoint passed into the observer:

```typescript
// package/pi-plugin/auto-mode/native/contract/integration/native-host/bootstrap.ts
const baseUrl = root.session
  .model
  ?.baseUrl
  ?? 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1';
const endpoint = new URL(`${endpointBase}/chat/completions`).href;
```

`package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs:60`
compares every later request with that fixed endpoint before invoking terminal transport at line 79:

```javascript
// package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs
if ((url !== endpoint) || (init?.method !== 'POST')
  || ((typeof init.body) !== 'string') || !init.body.isWellFormed())
  throw new RequestObservationProfileError('Only the declared serialized JSON POST shape is observed');
```

A process-local diagnostic at that catch records this exact profile error after switching providers.
The OpenAI SDK then reports `Connection error.`.
Installed `openai@7.19.0/client.mjs:837` wraps transport rejection in `APIConnectionError`;
`openai@7.19.0/core/error.mjs:74` supplies the default message:

```javascript
// Installed openai@7.19.0/client.mjs
throw new Errors.APIConnectionError({
  message: getConnectionErrorMessage(response),
  cause: response,
});
// Installed openai@7.19.0/core/error.mjs
super(undefined, undefined, message || 'Connection error.', undefined);
```

The reproduction has one initial transport call and no second call.
It establishes a local cause for this error string;
it does not reconstruct the unretained nested exception from the user's original request.

### Tree navigation invalidates the root used by ordinary prompts

`package/pi-plugin/auto-mode/native/contract/lifecycle/sdk-custody-native/generate.ts:197`
adds root-occurrence replacement to native branch movement:

```typescript
// package/pi-plugin/auto-mode/native/contract/lifecycle/sdk-custody-native/generate.ts
to: '        if (this.#leafId === branchFromId) return;\n        this.#changeRootOccurrence();\n        this.#leafId = branchFromId;\n    }'
```

The generated native `AgentSession.navigateTree` calls that manager branch operation,
refreshes finalized context,
and returns successfully.
The next stream invocation captures a root lease before the provider is called,
at `package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs:17`:

```javascript
// package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs
const rootLease = lifecycle.capture(boundRoot);
```

`package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/lifecycle.mjs:93`
retires the root when its native occurrence changed.
At line 146 the active-root check throws:

```javascript
// package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/lifecycle.mjs
if (current !== root.nativeOccurrence) {
  retireObserved(root);
  emit('lifecycle.native-occurrence.changed');
}
if (retired(root)) throw new RootLeaseRetiredError('Root lease has been retired');
```

The initial root is retained instead of replaced during capture at lines 187 to 193.
Old operation evidence must become unusable after navigation,
but requiring that old root for the next ordinary prompt gives retirement the wrong scope.

## Verification

The reproduction artifact reports Pi version `1.2.0`,
source commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`,
and compiler version `7.0.2` in its `source-manifest.json`.
The consumer regression is committed as `c408b94879`.

```bash
# Run from the repository root.
mise run //package/pi-plugin/auto-mode:test:native:session-actions
```

At the reproduction commit, the first case fails with `Follow-up after switch: Connection error.`.
The individual isolated fixtures are also runnable in
`${HOME}/temp/agent/pi-reported-errors-20261015/`:

```bash
# Run from the named scratch directory.
mise run reproduce switch
mise run reproduce tree
mise run reproduce retry
mise run reproduce reload
mise run reproduce abort
mise run inspect-cause
```

Failing catalog:

- Provider switch: initial prompt succeeds; follow-up reports `Connection error.`; one transport call total.
- Tree navigation: initial prompt and navigation succeed; follow-up reports `Root lease has been retired`;
  one transport call total.

Passing catalog:

- Retry control: authored transport failure on the first prompt; second prompt completes; two transport calls.
- Reload control: both prompts complete; two transport calls.
- Abort control: abort after the first completed prompt; second prompt completes; two transport calls.

The abort control does not exercise cancelling an in-flight response.
The retry control submits a second prompt; it does not exercise automatic retry backoff.
All fixtures use disposable homes, the built native host, and authored SSE responses.
No provider outage is necessary to reproduce either failure.

## Verified workarounds

No command-level workaround for the affected real session has been verified.
Reload succeeds on an otherwise valid session;
that does not establish recovery after a root has already retired.
Do not recommend reload as a demonstrated remedy for this incident.

## Implemented correction

The replacement moves judgment creation to complete-group dispatch.
Native branch identity scopes each operation instead of permanently retiring subsequent chat.
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/operation-currentness.mjs:58`
compares the captured occurrence and retains staleness only on that operation:

```javascript
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/operation-currentness.mjs
if(readCurrent()!==state.occurrence)throw new NativeOperationStaleError('Session branch changed during this tool decision');
```

Request capture failure is retained while the original stream continues.
`package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs:79`
records capture failure before selecting the caller's payload callback:

```javascript
// package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observation-stream.mjs
}catch(error){fail(error);}
const payload=scope.failed?{
  association:undefined,
  onPayload:options?.onPayload
}:prepareObservedPayload({
```

The expanded tree test exposed another primary gate:
capturing an extension event containing `AbortSignal` rejected the next prompt with `RunPromptCustodyError`.
Prompt and event capture now retain that failure for automatic assessment without owning native callback effects.
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/passive-capture.mjs:13`
keeps capture failure separate from the native fallback:

```javascript
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/passive-capture.mjs
function observe({capture,fallback}){
  if(!state.failure){
    try{return capture();}
    catch(error){fail(error);}
  }
  return fallback?.();
}
```

Unavailable automatic evidence opens manual approval for the complete prepared group.
Allow executes that group; Stop, cancellation, and unavailable UI execute nothing.
This is the user's corrected fallback, adopted on 2026-10-15.
The tradeoff is that an unsupported event can leave its source owner unavailable for automatic decisions;
subsequent affected tools still require manual review.
No test establishes universal automatic-evidence recovery after arbitrary event data.

The original approval regression found another important boundary:
retired authorization after the first dialog must not open a second evidence-free dialog.
Commit `e9864891ee` retains an established policy decision through that failure.
It preserves prohibitions and independent authorization requirements.

### Replacement verification

The disposable package under `${HOME}/temp/agent/pi-host-isolation-20261015/`
retains the build and consumer logs.
`complete-matrix.log` passes native session actions, passive callback and transport checks,
host lifecycle, runtime-hook installation, policy outcomes, and manual fallback on `native-release-JWvYLT`.
`approval-matrix.log` on `native-release-LNNqh3` passes original approval,
native-context reload, fresh confirmation, and uncertainty controls.
Its out-of-profile bypass expectation required updating for the accepted manual fallback.
The subsequent `approval-final.log` passes that Stop control, the complete approval matrix,
and selected-context verification with six assessments and eighteen native child executions.

The installed replacement is `native-release-Yhc7sX`.
`installed-configured.log` verifies the original installed command with all 17 configured packages:
Allow executes one read, while prohibition and an unmet independent prerequisite execute none.
`live-final.log` passes the same policy outcomes using live Luna against the disposable candidate.
`installed-live.log` then verifies live Luna through the installed command:
516 questions in three successful HTTP requests, one dialog, and one native read.
Primary responses and dialog selection remain authored fixtures.
No human keyboard approval or universal semantic accuracy is claimed.

`installed-manual-group.log` passes parent identity, native retirement, cancellation,
host replacement, one-shot execution, and returned-token rejection.
A process-local omission of the parent-identity check makes that committed regression fail with
`changed-parent must reject its stale Allow response`; the production check remains present.

The session-action cases include switching providers, tree navigation, explicit retry, reload,
completed and in-flight abort, combined actions, cancelled and no-op navigation, summary navigation,
fork, resume, agent retry, and provider retry.
The native callback tests exercise capture failure before and after callbacks,
exactly-once execution, unchanged return values, and original thrown-value identity.
The manual matrix independently injects observation and assessment failure and checks
Allow, Stop, cancellation, and headless behavior.

The configured-host harness initially mounted the scratch alias without its canonical generated-import path.
After mounting both paths, it also needed to preload the candidate's HTTP dispatcher before installing fixture fetch.
Preloading the installed release's dispatcher allowed candidate initialization to replace that fixture transport.
These are harness corrections, not additional provider incidents.

The `NEymiB` replay on 2026-10-18 verifies this namespace requirement against a private candidate
mounted at the repository's `dist` path.
Preloading the physical scratch URL while the CLI imports the repository mount URL creates distinct
dispatcher module instances.
The `ONKvxX` control records the capture hook as detached during extension registration and session start.
Resolving the actual CLI namespace inside the child preserves the hook at registration,
session start and agent start.
The corrected replay captures four Luna requests;
the required/excluded-data audit passes with its positive controls.
The earlier replay's empty capture does not establish payload correctness.

One verification attempt ended with Mise's `killed by SIGABRT` without a retained native fatal diagnostic.
A later commit hook reported `cannot fork()` and `Resource temporarily unavailable`.
The shared worker task limit was 256; a later sample measured 245 tasks.
Those samples do not prove the original abort's cause.
Verification now reduces fixture worker pools and runs the configured checks separately.
The user subsequently requested 2048 tasks.
The maintained hook and both live worker groups now use that limit while memory and CPU budgets remain unchanged.
The [worker-limits record](codex-shell-worker-limits.md) documents the coordinated update.

## What does not work

- Attributing the connection message to the remote provider without checking its nested cause.
- Treating a passing retry control as proof that tree navigation preserves prompt usability.
- Treating old-lease rejection as sufficient verification: the next prompt must still work.
- Declaring the decision-core replacement complete without ordinary provider-selection and navigation consumers.

The user identified an architectural problem after these reproductions.
The user accepted the
[host-isolation proposal](../planning/pi-auto-mode-host-isolation.md).
Its replacement passes the disposable session reproductions and the installed configured-host checks.

## Upstream filing decision

No upstream issue or patch is proposed.

1.  Upstream fault: no; the failing endpoint check and root lease are maintained auto-mode code.
2.  Upstream fixability: no upstream fix is needed for these demonstrated mechanisms.
3.  Supported use: the tests invoke the installed session's provider-selection and tree-navigation methods.
4.  Contribution policy: not investigated because an upstream contribution is not proposed.
5.  Maintainer willingness: not investigated for the same reason.
6.  Compatible upstream prototype: inapplicable; the correction belongs to the maintained integration.

## Pi SDK 1.2.0 first-group review failure with retained MCP catalogs

### Symptom

The MiMo session created at 2026-10-15 23:29:17 UTC displayed
`Auto-mode could not assess this operation: Original policy review and evidence closure failed`
for its first group of two direct Bash calls.
The user selected Stop and both calls returned `Operation aborted`.
The original session did not preserve its nested exception.

A disposable configured-host replay reproduces the same aggregate with 40 cached Linear tool descriptions.
The nested exception is `DependencyDeadlineError: Original dependency assessment deadline expired`.
This reproduces the observed boundary;
it does not establish the missing original nested exception as a historical fact.

### Reproduced causes and corrections

The stack enters `bindRunMessageSources` during complete policy input capture.
At commit `fc8b4fb6d0`,
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs:104`
called the full parent check while constructing every binding:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs at fc8b4fb6d0
assertCurrent(view);return Object.freeze({
```

The wrapper's `assertCurrent` delegates to `governing.assertCurrent(governing.view)`.
The maintained regression measured 257 full parent checks for 256 retained inputs,
compared with 2 checks for a single input.
Commit `5a8760d15f` performs local identity checks during construction and brackets the collection
with parent validation.
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs:116`:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs
governing.assertCurrent(governing.view);
return bindings;
```

The 40-tool replay then reaches ordinary uncertain-operation confirmation without the internal exception.
Foreign views and retired parents remain rejected.
A process-local omission of the final parent check makes the changed-during-construction control fail
with `Missing expected exception (GoverningPolicyBindingError)`.

The 140-tool replay exposed a second source-binding problem after removing the schema ceiling:
each input searched the entire retained-origin array.
The traced fixture contained 83,404 inputs and 83,399 retained origins.
One instrumented search took 113190.85 ms;
this is a diagnostic observation, not a repeated-run benchmark.
Commit `3827af2da9` builds a source-identity index once,
retaining the first origin for duplicate source identities.
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs:28`:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs
const retainedBySource=new Map();
for(const origin of run?.retainedMessageOrigins??snapshot.nativeConstructedInputs??[]){
 const {source}=origin;
 if(!retainedBySource.has(source))retainedBySource.set(source,origin);
}
```

The complete cached-catalog replay then passes without the internal exception.
The regression counts origin-source reads to detect a return to repeated linear searches.
The corrected release is installed as `native-release-tmMxY3`.

### Separate request-preparation ceiling

The full cache contains 66 BrowserOS and 74 Linear tools.
Before the schema correction,
request preparation stops before the provider call with
`Original request tool schemas exceed the 10000-property profile`.
At commit `5a8760d15f`,
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/tools.mjs:60` imposed this ceiling:

```js
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/tools.mjs at 5a8760d15f
if(schemaCapture.properties>10_000)throw new ExecutionContextOwnershipError('Original request tool schemas exceed the 10000-property profile');
```

Commit `ac68881314` removes that property count limit.
The schema walk retains its visited set,
ordinary-record check,
data-descriptor check,
and descriptor freshness checks.
A built-consumer test captures 12,000 properties and rejects a changed last property,
an accessor without invoking it,
and an unexpected prototype.

### Verification and rejected substitutes

The bounded host fixtures retain configured extensions,
home-discovered skills,
the real repository's read-only context,
and replayed MCP metadata.
They use disposable home directories and authored primary responses.
Offline cases use fake credentials and scripted Stop.
Separate live Luna checks bridge assessment requests to the real decision service,
retaining offline MCP metadata and scripted confirmation.
They do not execute external MCP operations or claim physical human input.

- Failing baseline: `mimo-mcp-forty-before.log` and `run-message-freshness-before.log`.
- Passing candidate controls: `mimo-candidate-repro.log`, `mimo-regressions.log`, and `mimo-schema-after.log`.
- Intermediate full-catalog failure: `mimo-full-diagnostic.log`, fixture `pi-hyper-rpc-stop-GRWOl5`.
- Passing full-catalog candidate: `mimo-full-index-after.log`, fixture `pi-hyper-rpc-stop-fsbmtV`.
- Final guard omission: `mimo-omission-final.log`.
- Installed regressions: `direct-installed-regressions.log` and `direct-installed-additional.log`.
- Installed live Luna confirmation: `direct-installed-allow.log`, fixture `pi-hyper-rpc-allow-LuIIFZ`.
- Installed live Luna prohibition: `direct-installed-prohibit.log`, fixture `pi-hyper-rpc-prohibited-QwVjXt`.
- Logs live under `${HOME}/temp/agent/pi-host-isolation-20261015/`;
  the named fixture lives directly under `${HOME}/temp/agent/`.

Maintained regressions run through
`mise run //package/pi-plugin/auto-mode:test:native:run-message-freshness`
and `mise run //package/pi-plugin/auto-mode:test:native:tool-schema-capacity`
after building the candidate artifact.
The configured replay tasks are `mimo-first`, `mimo-candidate`, and `mimo-diagnostic`
in `${HOME}/temp/agent/pi-tool-call-scope-20261014/`.

Disabling MCP omitted the failing surface and is not a verified repair.
The earlier 140-tool probe with a 512 MiB heap exhausted that heap before review;
that is a separate outcome.
The 20-tool immediate-start probe observed source changes during confirmation;
waiting for catalog initialization changes that experiment's inputs.
Increasing the diagnostic fixture's heap to 1536 MiB and duration to 180 seconds exposed the remaining deadline cause;
neither change fixes it or changes the production fifteen-second review budget.

The corrected runtime preserves the production fifteen-second review budget.
All demonstrated mechanisms are maintained integration code,
so the existing upstream filing decision applies and no external filing is proposed.

### Direct Bash automatic review

The user also required direct Bash calls to receive automatic review.
Commit `427ae71387` extends operation matching to native `bash` calls with string commands;
`d4467e1fc3` exports that matcher through the generated native adapter.
`package/pi-plugin/auto-mode/src/semantic-admission-scope.ts:74` checks the prepared operation:

```ts
// package/pi-plugin/auto-mode/src/semantic-admission-scope.ts
if ((configured === 'native-inspected-programs-v2') && (effect.prepared?.name === 'bash'))
  return typeof effect.prepared.input.command === 'string';
```

Native ownership and currentness checks still authenticate the operation.
The matcher adds no program-inspection facts or permission.
`test:native:direct-bash` verifies permission, prohibition, unknown answers, and mixed groups.
Each assessment contains both complete proposed operations.

The live candidate and installed prohibition checks returned HTTP 200 and blocked both commands without a dialog.
The live installed permission fixture returned HTTP 200 for 94 questions,
but unrelated clauses remained uncertain.
Scoped scripted Allow then executed both commands successfully.
That establishes live automatic review and confirmation behavior;
it does not establish automatic approval of that complete live context.
