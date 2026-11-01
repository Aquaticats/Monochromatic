# LLM Gateway Luna Decisions returns complete answers without complete native admission

## Permission review receives unrelated agent guidance on 2026-10-16

### Symptom and evidence

Session `2026-10-16T02-17-34-349Z` reached manual approval while inspecting task headings with `grep`
and reading the handover with `sed`, after preceding inspection calls succeeded.
The installed whole-operation request included the full root policy and Pi's hidden tool-use guidance.
Three identical live requests returned `deny: 0.57`, `approve: 0.41`, and `unknown: 0.02`.
The `0.60` action cutoff correctly withheld automatic approval.
The defect was the selected review scope and inputs, not a transport timeout.
The concurrent sessionStorage warning is explicitly outside this incident's investigation.

### Source trace and correction

The accepted scope is execution permission and required consent.
Tool-use and implementation conventions remain coding-agent guidance.
The shared question scope is defined in
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/permission-review-scope.ts`.
Both automatic review phases consume it.

`package/pi-plugin/auto-mode/native/contract/integration/policy-source/decision-context.mjs:31`
selects readable project policy sources and visible conversation before clause indexing:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/decision-context.mjs
const sources=Object.freeze([
  snapshot.hostGoverningPolicy,
  ...(snapshot.readablePolicySources??[]),
  ...(conversation?[conversation]:[])
]);
```

Hidden prompts, tool metadata, and skill descriptions remain local provenance inputs.
They do not become review questions or shared model context.
Additional policy inputs must bind to actual readable files through the incumbent observer.
Complete root-policy copies in transcript strings are replaced with a selected-excerpt reference.

The existing relevance cache was disconnected from production initialization and admitted no exclusions.
Commits `d9e9771bd8`, `c35670685a`, and `22532b415e` connect initialization, selection, and wire projection.
`package/pi-plugin/auto-mode/native/contract/integration/native-host/startup-relevance.mjs:25`
constructs requests from individual changed rules and fixed remit text:

```js
// package/pi-plugin/auto-mode/native/contract/integration/native-host/startup-relevance.mjs
const body=JSON.stringify({model:ruleAssessmentModel,state:{remit:request.definition.remit},questions});
```

No complete policy field accompanies those questions.
Unchanged classifications persist across owners; changed rule text or classifier profile requires fresh estimates.
The user's subsequent instruction to cull uncertain rules is implemented in
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/decision-material.mjs:11`:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/decision-material.mjs
const excluded=Object.freeze((relevance?.rules??[]).filter(function notAdmitted(rule){return rule.probability<0.6;}));
```

Selected excerpts retain original source spans and freshness checks.
Relevance selection does not grant execution permission.
Startup classification failure preserves chat; tool calls require scoped confirmation or remain withheld without UI.

### Verification and limits

Installed release `native-release-8uGKtw` passes built startup and readable-context regressions.
The relevance boundary includes `0.60` and excludes lower estimates.
Cold, warm, and changed-rule fixtures request four, zero, and one classifications respectively.
A disposable build fails the regression with culling removed and passes when restored.
The startup failure matrix covers chat, Allow, Stop, and headless withholding.

```sh
# Repository root; built consumer tests with disposable fixtures
mise run //package/pi-plugin/auto-mode:test:native:startup-relevance
mise run //package/pi-plugin/auto-mode:test:native:decision-context
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback
```

The full reported conversation replay executes nine original calls across six groups without confirmation.
The live explicit prohibited-read control blocks both calls without confirmation.
Receipts and the resource comparison are indexed in
[the current handover](../handover/pi-auto-mode-current-frontier.md).
These observations establish the exercised interactions, not universal classifier accuracy.
Removing uncertain rules can omit a relevant restriction when its relevance estimate is low;
that tradeoff follows the user's explicit selection policy.

### Rejected approaches

- Raising or lowering the action cutoff does not repair unrelated review inputs.
- Filtering only shared context still exposes omitted instructions in per-clause questions.
- An initial-command replay does not establish behavior after the reported preceding turns.
- Keeping uncertain rules contradicts the subsequent explicit selection instruction.

### Upstream filing decision

Nothing to add upstream: source selection, permission scope, and disconnected cache integration are application defects.
The upstream-fault gate fails.
The Decisions route supports the requests exercised here; upstream fixability, contribution policy,
maintainer intent, and prototype gates do not justify a provider report for these local corrections.
No external issue or message is prepared.

## Repository inspection blocked by unresolved prerequisites on 2026-10-16

### Symptom and retained evidence

Installed auto-mode release `native-release-tmMxY3` blocked a handover-directory listing
and `pwd` plus `git rev-parse --show-toplevel` with:

> An independent authorization prerequisite remains unsatisfied or unresolved;
> session confirmation cannot satisfy it

The original session beginning `2026-10-16T00-30-17-591Z` retains both tool calls and their rejected results.
Neither proposed operation required another authorization under the reported task.
The original local logs omitted per-clause distributions,
so the exact clause causing that incident is unknown.
An unchanged installed replay subsequently reached a dialog;
its `PX2` prerequisite score was `0.39`, below admission.
That replay does not establish that `PX2` caused this incident.
The separate 2026-10-15 `proof.txt` incident has its own retained `0.60` evidence.

### Source trace and repair

Before commit `ad6d59cb92`,
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/review-outcome.ts:84`
converted unresolved authorization requirements into the quoted terminal explanation:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/review-outcome.ts, before ad6d59cb92
if (authorization.state === 'unresolved') {
```

The agent's initial interpretation, routing those estimates directly to a dialog, was rejected.
The user required the decision model to reconsider the complete operation first.
Current `package/pi-plugin/auto-mode/native/contract/integration/action-policy/decision-core.mjs:108`
now selects that path before native confirmation:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/decision-core.mjs:108
const operationDecision=policy!==undefined&&(automaticOutcome.state!=='resolved'||automaticOutcome.verdict==='ask')
  ?await decideCompleteOperation()
  :undefined;
```

The new question retains complete selected context and every original prepared tool operation.
It shares the existing model, categorical cutoff, deadline, and transport allowance.
A recoverable clause-assessment failure also reaches this model question.
An established governing prohibition remains terminal.
Unresolved or failed final review reaches scoped confirmation when available.
Allow authorizes the exact operation without rewriting prerequisite estimates as satisfied external facts.

The source-change control exposed another failure:
changing `AGENTS.md` during final model review initially entered the independent manual fallback and executed.
Commit `8854618f2a` prevents replacing that stale review with a new dialog.
The subsequent deadline control distinguishes source freshness from an exhausted assessment budget,
so a model timeout can still reach last-resort manual review for an unchanged operation.

### Reconstructable diagnostics

The user identified missing decision evidence as a logging defect.
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/compiler.mjs:138`
records each selected clause, distribution, serving model, priority, and admission result:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/compiler.mjs:138
recordDecision({judgment,event:'clause-admission',details:{id,binding:projection,model:record.model,matchingModel,answer:record.answer,admitted,priority}});
```

`package/pi-plugin/auto-mode/native/contract/integration/action-policy/decision-diagnostics.ts:28`
retains one local identifier for the original judgment:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/decision-diagnostics.ts:28
const reviewId = reviewIds.get(judgment) ?? randomUUID();
```

Assessment records include complete selected model input and parsed replies under `assessmentId` and `operationId`.
Question evidence joins those records to `reviewId` through the same operation digest.
Decision records retain routing, model outcomes, scoped confirmation, and nested failures.
Invalid reply records retain the malformed reply.
Transport credentials and the archived primary-provider request are excluded from assessment records.
These local logs contain command and policy contents and are not public issue attachments.

### Verification and limitations

The original routing regression failed because only one assessment ran.
The candidate then passed an authored false-prerequisite response followed by final-model approval.
The live candidate replay uses the user's exact commands,
the actual repository working directory and policy,
17 configured packages,
27 home-discovered skills,
and all 140 cached MCP tool definitions.
It made four successful Luna calls,
executed both commands,
and opened no dialog.
Receipt: `${HOME}/temp/agent/pi-host-isolation-20261015/repo-decision-candidate.log`.

The built consumer regression covers automatic approve and deny,
unknown and lower-scoring answers,
malformed replies,
initial and final transport failures,
Allow and Stop,
dismissal,
missing UI,
source changes,
cancellation,
original-deadline expiry,
and explicit fresh confirmation.
It reconstructs the authored false prerequisite from the actual tagged logger's records.

```sh
# Repository root; authored provider replies and disposable native sessions
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback
```

The earlier synthetic working-directory policy did not establish behavior under the real repository policy.
The older original logs cannot be repaired retroactively.
Authored cases establish routing and retention;
live results establish the named observed operations, not general model accuracy.
Installed release `native-release-Jg5gMF` passes the same live repository-policy replay in
fixture `pi-hyper-rpc-allow-HTVK1s`, with four successful Luna calls, both commands executed, and no dialog.
Its persisted JSONL records contain 518 question answers and 515 clause-admission records,
linked to both complete assessment exchanges and the final approval.
The installed prohibition control `pi-hyper-rpc-prohibited-a1TH2T` blocks both proposed reads without a dialog.
The fallback, original-budget, ownership, public API, type, and focused zero-error lint checks pass.
Complete installed receipts are indexed in [the current handover](../handover/pi-auto-mode-current-frontier.md).

### Upstream filing decision

Nothing to add upstream for this incident:
the demonstrated missing fallback and diagnostics are owned application behavior.
The original model interpretation cannot be attributed to a specific clause from its retained logs.
The upstream-fault gate therefore fails;
upstream fixability, contribution acceptance, maintainer intent, and an upstream prototype do not arise.
The existing provider supports the Decisions request shape exercised by the retained live HTTP 200 receipts.
No upstream message or speculative provider patch is prepared.

## Local read falsely acquires an unrelated prerequisite on 2026-10-15

After restoring selected conversation context,
the configured host received complete HTTP 200 assessments for a local `proof.txt` read,
but emitted:

> An independent authorization prerequisite remains unsatisfied or unresolved;
> session confirmation cannot satisfy it

Luna selected `prerequisite` with probability `0.60` for `PX2`,
whose authorization condition concerns destructive or external actions.
The observed local read does neither.
The first live host attempt and both unchanged-body repetitions admitted that meaning.
The adopted inclusive `0.60` threshold explains why the consumer retained the prerequisite;
its independent confidence field was `0.53` and was not the admission field.
This is a false positive for this operation,
not evidence that the Gateway ignored input or that the confirmation guard should be removed.

### Source trace and bounded repair

The question supplied the selected clause while retaining the operation only in shared input.
The replacement repeats the complete original operation beside each clause,
without changing source content,
question identity,
rubrics,
admission threshold,
or authorization behavior.
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/wire.mjs:56`
decodes the original native JSON snapshots:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/wire.mjs:56
const proposedOperation = Object.freeze({
  parent: JSON.parse(envelope.parentJSON),
  prepared: JSON.parse(envelope.preparedJSON)
});
```

`package/pi-plugin/auto-mode/native/contract/integration/action-policy/wire-clause.mjs:59`
adds `proposedOperation` to each categorical clause question.
The original shared input remains complete.
The tradeoff is repeated operation bytes per clause;
conversation archives remain excluded.
The measurements establish changed model outputs,
not the provider's internal cause or general semantic accuracy.

### Verification

The unchanged requests and repeat responses are retained under
`${HOME}/temp/agent/pi-hyper-rpc-allow-e9HmAE/`
and `${HOME}/temp/agent/pi-core-repeat-fgODvB/`.
Two changed-format observations under `${HOME}/temp/agent/pi-core-operation-ygPhES/`
retained permission and fresh-confirmation estimates while no independent prerequisite met admission.
These diagnostic observations executed no proposed tool operation.

After commit `67a2d7cfe3`,
the installed CLI was exercised in disposable homes with all 17 configured packages,
copied workspace instructions,
authored primary-model responses,
and live Luna assessments:

- `pi-hyper-rpc-allow-xBd3tp`:
  one scoped confirmation and one completed native file read.
- `pi-hyper-rpc-prohibited-jdUv2C`:
  an actual file-read prohibition, no confirmation, and no child execution.
- `pi-hyper-rpc-generic-L5IYbS`:
  required approval from an external records office, no confirmation, and no child execution.

Each assessment used three HTTP calls under the original fifteen-second budget.
The permission case retained 516 questions and request sizes of 370631,
377120,
and 289751 bytes.
Its slowest response completed in 5760.916 milliseconds.
These receipts prove the named live consumer cases,
not human keyboard delivery or general classifier accuracy.
The maintained `test:native:decision-context` also checks exact operation equality in every clause question,
complete selected messages,
absence of the old prefix,
follow-up selection,
and reload.
No upstream defect or filing is established by these application measurements.

## Historical admission investigation

The remaining sections describe the 2026-10-14 input,
threshold,
and experiments.
Their unresolved result does not replace the newer configured-host receipts.

## Symptom and version

On 2026-10-14,
`openai/gpt-6-luna-decisions` through `POST /v1/decisions` accepted the installed native assessment
and returned all 84 named answers.
The maintained consumer nevertheless could not admit a complete interpretation at the adopted `0.80` cutoff.
This is an application admission boundary,
not an HTTP error or established upstream defect.

The initial baseline contains 584,007 bytes and reports 126,129 input tokens.
Repeated baseline responses admit four supported meanings,
including the primary permission meaning at `0.83`.
Independent confidence remains a separate returned field.
The model name is the returned Gateway identity;
the provider's immutable model revision and hosted Gateway commit are not established.

## Root cause and limits of the source trace

The observed application hold is consistent with unresolved categorical estimates.
No experiment establishes why the model assigns those distributions.
Neither batching invariance nor repeated identical answer data proves caching,
independent model evaluation,
or a provider implementation defect.

There is also a separate missing native consumer for an admitted whole-request
`potential-instruction` answer.
The baseline selects that answer at `0.96`.
An actual built native contrast admits all clause meanings and changes only the authored whole-request relation:
`none` executes three prepared children;
`potential-instruction` executes zero.
Both runs retain five accounted partial source associations and 33 original sources.
Each uses one local HTTP response and zero provider requests.
`${HOME}/temp/agent/pi-luna-request-use-control-20261014.log` retains the complete contrast.

The owned consumer in `package/pi-plugin/auto-mode/src/request-material-obligations.ts:65`
keeps that governing-use obligation:

```ts
// package/pi-plugin/auto-mode/src/request-material-obligations.ts:65
if ((interpretation.admission === 'admitted') && (interpretation.relation === 'potential-instruction')) {
```

Its emitted issue at line 69 is `request-material-instruction-use`.
Existing partial source contributions do not supply the missing complete interpretation.
That gap must be implemented through original source ownership;
prompting the model to report absence would not repair it.

The fresh read-only Gateway clone is
`${HOME}/temp/agent/llmgateway-decisions-20261014/`,
revision `8d7a6f3a65af6a11d7646cc3e927a0acd0a2b961`.
Its `apps/gateway/src/decisions/decisions.ts:315` forwards the parsed input and question array:

```ts
// apps/gateway/src/decisions/decisions.ts:315 in the cited read-only clone
model: upstreamModel,
input: request.input,
questions: request.questions,
```

The same file at line 326 retains the upstream body while assigning the Gateway response model:

```ts
// apps/gateway/src/decisions/decisions.ts:326 in the cited read-only clone
? { ...upstream, model: responseModel }
```

These paths explain the public forwarding and response-normalization contract.
They do not identify the deployed revision or expose the model's interpretation process.
The [official Decisions guide][openai-guide] distinguishes choice probabilities from confidence
and recommends application-labeled examples for threshold selection.
The current cutoff is the user's adopted operating choice,
not an accuracy measurement.

## Verification

The [frozen experiment record][experiments] contains schedules,
request hashes,
durations,
and private artifact locations.
Every experiment preserves complete shared input,
question membership,
original criteria,
and the five-second deadline.
No represented tool operation executes.
The built maintained adapter validates model identity,
question identity,
complete distributions,
and independent confidence before comparison.

Working controls:

- The six-call baseline and explicit-rubric comparison returns complete valid answers without refusals.
- Replaying all six responses through the built adapter preserves all 504 distributions and confidence values.
- Partitioning the original questions into three groups of 28 returns every original answer.
  All three repetitions exactly match the baseline answer data.
- Repeating exact selected text beside its original JSON binding returns valid answers in every observation.
  All three repetitions have identical answer data.

Incomplete operating results:

- The baseline admits four answers at `0.80`.
- Explicit existing rubrics beside each question admit six;
  four absence estimates newly cross the cutoff and two formerly admitted estimates fall below it.
- Question batching admits four and reports 246,783 input tokens per complete assessment.
- Readable selected text admits eight and reports 178,834 input tokens per assessment.
  Its primary permission estimate is `0.82`,
  with independent confidence `0.78`.

Every format still has unresolved interpretation obligations.
The explicit-rubric and readable-text variants provide positive controls:
the comparison detects distribution changes when they occur.
Admission counts are not accuracy results.
The latency ranges overlap across observations and do not establish a speed improvement.

The original network namespaces are consumed and must not be rerun in place.
Local built-consumer verification needs no provider credential:

```sh
# Repository root; no external provider calls
mise run //package/pi-plugin/auto-mode:test:decision-wire
mise run //package/pi-plugin/auto-mode:test:semantic-operating-policy
```

Those commands verify the adapter and admission rule;
they do not reproduce provider distributions.
A new provider reproduction requires a fresh finite schedule retaining the complete frozen request.

## Verified workarounds

There is no verified workaround yielding complete native admission for this captured input.
The maintained lossless single-call adapter remains the selected implementation.
It preserves every original obligation and may withhold execution when estimates remain unresolved.
That behavior does not implicitly open an approval dialog.

## What does not work

Question batching supplies no answer change for this input and repeats shared-input tokens.
Repeating rubrics or readable selected text changes estimates but does not establish complete admission or accuracy.
Further representation-only trials on the same input are deferred in favor of source-boundary investigation
and the maintained native quality queue.
Removing received source material,
treating unknown as absence,
or weakening independent authority requirements would change the accepted policy.
None is an adopted remedy.

## Upstream filing decision

The repository's `.out-of-scope/` filenames were checked;
none names Luna or this Gateway Decisions behavior.
No upstream issue or comment is being sent.

1.  Upstream fault is unestablished:
    valid probabilistic output with unresolved application admission is not a demonstrated defect.
2.  Fixability is undetermined because no upstream cause has been identified.
    The source does expose the expected forwarding boundary.
3.  Typed shared-input questions are supported by the public Decisions schema and guide.
    Those documents do not promise this application's categorical certainty.
4.  The cloned `README.md` and `.github/ISSUE_TEMPLATE/` were inspected.
    No AI-report ban was found there;
    the README's new-provider contribution restriction is unrelated.
    No root `CONTRIBUTING.md` exists in this clone.
5.  `gh search issues --repo theopenco/llmgateway 'decisions confidence' --limit 30`
    and `gh search prs --repo theopenco/llmgateway 'decisions questions' --limit 30`
    returned no matches.
    That limited search supplies no maintainer position on this application.
6.  No upstream patch is proposed or prototyped because the upstream-fault gate fails.
    The measured consumer experiments remain local and unadopted.

Nothing to add upstream:
the available evidence supports an application investigation,
not a defect report or speculative provider patch.

[openai-guide]: https://developers.openai.com/api/docs/guides/decisions
[experiments]: ../planning/pi-auto-mode-gateway-decisions-current-contract.md
