# LLM Gateway System One source does not establish model identity, retry count, or payload-free error logs

## Status and symptom

The auto-mode migration requires narrow axiom estimates,
complete current policy input,
qualified model identity,
and a five-second total assessment budget.
The user permits one client transport retry and accepts gateway-internal retries.
Necessity-based retention without a fixed deletion deadline is accepted.
Neither zero retention nor an end-to-end two-attempt cap is required.
The user subsequently authorized all task-relevant assessment content through LLM Gateway/Jev,
including private or sensitive content,
and declined dashboard access.
Metadata-only logging is user-reported configuration,
not independent verification of every diagnostic field.
Production implementation remains unauthorized.

A successful `/v1/systemone` response does not establish all these boundaries.
The public source normalizes the model name,
rotates credentials after retryable failures,
and retains upstream error diagnostics separately from ordinary payload fields.
These are source findings reproduced with an offline mocked-provider harness,
not observations of model substitution or private-data retention in the hosted service.

## Full-rule request rejected with `max_tokens_exceeded`

The [2026-10-14 contract and workaround assessment](../planning/pi-auto-mode-jev-current-contract.md)
updates the current operating scope and vendor documentation.
Historical statements that production adoption was pending are superseded by the
[adopted semantic operating rule](../decision/pi-auto-mode-semantic-admission.md).
The source and live results in this record retain their original scope.

### Symptom and retained scope

On 2026-10-05,
the private copied-input action-relation canary sent one 454,012-byte request
containing 205 independent rule questions.
The response was HTTP 400:

```text
# Private program-rule-interface/dispatch-private/model-private/failure.json: retained response
{"detail":{"error_type":"max_tokens_exceeded"}}
```

`proc_48e7` verified the retained request hash,
original five-second accounting,
one initiated client attempt,
zero completed samples,
worker exit,
and complete private streams.
No scores or usage were returned.
The failed namespace remains consumed;
there was no retry,
input truncation,
SDK session,
or native tool execution.
Billing is unknown,
not proven zero.

### Documented limits and forwarding source

The [TypeSafe model reference][typesafe-model-limits]
specifies 64k tokens for `state` plus all questions combined,
and 32k tokens for `state` plus the longest question.
The public Gateway catalogue exposed only `context_length: 64000`.
The error did not identify which constraint failed or provide an exact token count.
Do not turn serialized byte counts into asserted tokenizer measurements.

The source paths in this subsection refer to the fresh read-only clone
`~/temp/agent/llmgateway-systemone.sDAGWZTZ`,
revision `cdfb922062f73f24e3ab457720b2afce14d0a458`.
The hosted deployment's matching revision is unestablished.
The forwarding object preserves state and questions:

```typescript
// apps/gateway/src/systemone/systemone.ts:635-639 in the 2026-10-05 clone
requestBody: {
  model: upstreamModel,
  state,
  questions,
},
```

The error branch can return a structured upstream body unchanged:

```typescript
// apps/gateway/src/systemone/systemone.ts:1056-1059 in the 2026-10-05 clone
upstreamJson && typeof upstreamJson === "object"
  ? upstreamJson
  : normalizedUpstreamError,
status as 400 | 401 | 403 | 404 | 410 | 429 | 500 | 502 | 503 | 504,
```

This source trace explains why a TypeSafe-shaped detail can reach the client.
It does not reveal exact backend token accounting or establish zero hosted retries or charges.

### Local encoding revision and remaining verification

The rejected encoding duplicated native binding projections in `state.bindings`
as well as supplying the selected rule directly in each question.
`proc_ba2b` removed only that duplicate model-facing bookkeeping in a private fork:

- Entire request:
  454,012 bytes to 315,353 bytes.
- State:
  208,680 bytes to 70,021 bytes.
- Question map:
  unchanged at 245,279 bytes.
- Longest question:
  1,280 bytes.

The check requires deep equality of every question and every remaining state field.
Native IDs,
source bindings,
and answer correlation remain with the original code owner.
Full policy,
selected-rule text and headings,
parent/prepared programs,
and original main request remain intact.
The separate 315,353-byte representation trial also returned HTTP 400
with the same `max_tokens_exceeded` detail.
`proc_e51c` verified one attempt,
no returned scores or usage,
and no replay.
Both failed requests have unknown billing.
Thus metadata removal is a verified local representation change,
not a working provider workaround or semantic-equivalence result.

The [TypeSafe API reference][typesafe-question-schema]
says question-map keys are not sent to inference.
Keep the selected rule explicitly inside `instructions`;
a key such as `rule_LN7` alone is insufficient.
The [Jev 1.13 limitations][typesafe-jaggedness]
also warn about indirection,
irrelevant state,
and adversarial content.
The metadata-only revision preserved all question wording and criteria.
After its rejection,
the next local prototype retained the exact question sentence and criteria in each head
while moving 373 bytes of remaining guidance verbatim to shared state.
Every question names `assessmentGuidance` explicitly.
`proc_0c8c` checked that concatenating the question sentence and shared guidance
reconstructs the original instruction text exactly,
and measured 256,710 total bytes,
70,418 state bytes,
and a 992-byte longest question.
A separate frozen trial using that exact representation was admitted and succeeded once.
`proc_ca7b` verified 205 estimates,
62,053 reported input tokens,
3,729 output tokens,
and 1,001.814279 ms including preparation.
The modeled charge was US$0.002606226.
The known semantic subtotal became US$0.058122608 plus the two failed requests with unknown billing.
This verifies acceptance of one bounded input,
not a universal capacity workaround or semantic equivalence.
The documentation's indirection warning still applies.
Neither failed response identified the failed token bound;
the successful request does not retrospectively identify those causes.

### Rejected remedies and filing decision

Do not drop rules,
truncate policy or complete operations,
reset an original budget,
replay a consumed namespace,
or present unknown failed-request billing as free.
Do not treat general JSON structure support as proof that a new encoding preserves calibration.
A separately frozen trial is required before claiming the revised representation is accepted.

No upstream defect is established:
the provider documents bounded context,
and the observed request was rejected with a token-limit error.
Upstream fault therefore fails the filing gate;
fixability,
support,
contribution acceptance,
and willingness are not grounds for filing this as a bug.
The consumer-side encoding revision has local content-preservation measurements only.
No upstream issue or comment is warranted.

[typesafe-model-limits]: https://docs.typesafe.ai/models
[typesafe-question-schema]: https://docs.typesafe.ai/api
[typesafe-jaggedness]: https://docs.typesafe.ai/model-jaggedness/jev-1.13

## Installed Jev request returns HTTP 400 without retained provider detail

On 2026-10-14,
the approved disposable Qwen host test reached the Jev route through LLM Gateway.
Its retained cause was `AssessmentTransportError: Assessment request returned HTTP 400`.
The raw provider error was not retained in
`${HOME}/temp/agent/pi-native-installed-live-NB6KVz/host-stderr.txt`.
The governed file read did not complete successfully.

The owned transport explains the missing detail,
not the upstream rejection.
`package/pi-plugin/auto-mode/src/assessment-transport.ts:238` contains:

```ts
// package/pi-plugin/auto-mode/src/assessment-transport.ts
if (response.status !== limits.successfulStatus)
  throw new AssessmentTransportError(`Assessment request returned HTTP ${response.status}`,);
```

The complete bounded response was consumed before this branch,
then replaced with a status-only diagnostic.
Unlike the historical overflow controls,
this observation contains no `max_tokens_exceeded` code.
The separate installed run at `${HOME}/temp/agent/pi-native-installed-live-9wruOV/host-stderr.txt`
stopped at the owned `RuleAssessmentWireError: Complete mixed assessment exceeds the one MiB bound`
before provider inference.
Neither incident proves the other's cause.

Historical accepted 205-answer requests provide working provider controls,
but do not reproduce this current installed payload.
No current-request workaround is verified.
The proposed diagnostic correction is to retain recognized error codes only,
without echoing provider free text or input content.
Blind retries,
raising the local byte guard,
or treating the unknown HTTP 400 as a confirmed context overflow are not established remedies.

The existing upstream filing decision remains unchanged:
upstream fault and fixability are unestablished,
typed questions are documented,
no contribution or willingness inquiry was made,
and no hosted-source fix can be prototyped from this status-only evidence.
No issue or external message is drafted or sent.

### Recognized capacity diagnostic verified on 2026-10-14

The user resumed implementation after the Jev assessment.
The built sender regression reproduced the loss of the exact
`{"detail":{"error_type":"max_tokens_exceeded"}}` envelope.
Commit `7f0c910ee4` retains the failing test;
`520cfe39ca` adds the diagnostic correction.
The scoped transport tests and TypeScript check pass after rebuilding.
Scoped lint reports zero warnings and zero errors on the changed files.

`package/pi-plugin/auto-mode/src/assessment-http-failure.ts:63` now recognizes only this exact code:

```ts
// package/pi-plugin/auto-mode/src/assessment-http-failure.ts
if (detail.error_type === 'max_tokens_exceeded') {
  rl.debug('recognized provider token capacity failure',);
  return `${message}; provider code max_tokens_exceeded`;
}
```

Malformed JSON,
unknown codes,
wrong types,
lookalikes,
and codes outside the recognized envelope retain the status-only diagnostic.
Provider free text is excluded;
the existing credential-echo check runs before classification.
This improves diagnosis without changing the request or admission result.

The approved installed disposable rerun at
`${HOME}/temp/agent/pi-native-installed-live-pygBoi/host-stderr.txt:75`
then reported
`AssessmentTransportError: Assessment request returned HTTP 400; provider code max_tokens_exceeded`.
Qwen proposed `return await tools.read({path:"proof.txt", offset:1, limit:2000});`.
The governed read remained withheld.
This establishes a capacity rejection for this rerun,
without retroactively recovering the discarded body from the previous run.
No payload reduction or capacity workaround is yet verified for the installed host.
The local replay and representation checks are recorded in the capacity subsection.
The upstream filing decision remains unchanged because a documented capacity rejection is not an established bug.

### Complete native state and lossless representation checks

The network-isolated replay at `${HOME}/temp/agent/pi-native-installed-live-UFBJsm`
captured 73 source occurrences,
84 questions,
and a 504,946-byte request.
The state occupied 251,214 bytes and questions occupied 253,679 bytes.
There were 51 distinct source texts;
the repeated source-text array occupied 216,561 bytes versus 174,262 for distinct texts.
These are serialized byte measurements,
not token counts.
Native JSON receipts also repeat values within their original records.

Private local prototypes recovered every original state field and every question instruction by exact deep comparison.
The explicit-node graph measured 194,593 bytes.
The shared-content representation measured 165,373 bytes,
including 125,822 bytes of state and 39,498 bytes of questions.
A simpler source-record representation measured 305,929 bytes.
All retained every original source occurrence and question;
none was installed.
Exact reconstruction does not establish equivalent model interpretation.

The frozen shared-content Jev request at
`${HOME}/temp/agent/pi-capacity-shared-check-20261014`
has SHA-256 `4b0f35e9721c53aba83b905b9332df8d742e06e903b723311f16e123d78aa16b`.
Its single five-second request returned HTTP 400 with `max_tokens_exceeded` after 1,269.832 ms.
No retry occurred.
This tested representation is therefore not a capacity workaround.
Dropping source occurrences,
relaxing the adopted cutoff,
or assuming byte reduction proves token fit remains unjustified.

The separate state-fit diagnostic at `${HOME}/temp/agent/pi-jev-shared-state-fit-20261014`
first passed a literal-label serving control with HTTP 200.
The identical shared state with only one original question then returned HTTP 400 `max_tokens_exceeded`
in 731.176 ms.
That failure rules out splitting the unchanged shared-state request as a sufficient remedy.
The server did not identify a tokenizer count or which limit fired.
The next local candidate moves all original question instructions back to their respective question entries;
exact reconstruction passes for the original state and every original question.
Its state is 105,339 bytes and the complete request is 140,618 bytes.
The complete placement candidate returned HTTP 400 `max_tokens_exceeded` in 1,049.183 ms.
Its single-question control retained the entire candidate state and failed with the same code in 518.147 ms.
Both calls are terminal;
neither returned answers or established semantic behavior.
Moving instructions alone is therefore not a capacity workaround.

The subsequent single-use-reference inlining candidate also returned HTTP 400 `max_tokens_exceeded`
in 1,233.578 ms.
The experimental row-encoding candidate failed earlier in gateway schema validation:
`invalid_union_discriminator`,
`Invalid discriminator value. Expected 'noul' | 'choice' | 'score'`,
at the questions' `type` fields.
Its encoder had wrapped whole question envelopes in `{$row: ...}` objects.
Exact reconstruction does not prove that a request satisfies the API wire schema.
Preserve each question's envelope and encode only its instruction payload;
preflight the unchanged type and criteria against the original request.
No token-fit conclusion follows from that rejected row request.

The corrected offline prototype rejects that captured malformed input,
then proves unchanged outer question fields and exact original reconstruction for every corrected candidate.
The combined representation is 117,254 bytes with 87,989 state bytes.
Its frozen one-call provider check then returned HTTP 400 `max_tokens_exceeded` in 997.329 ms.
This valid envelope still did not fit;
no semantic answers were returned.

## Source identity

Repository:
 <https://github.com/theopenco/llmgateway>.
Revision:
 `4affe8bf02559880fea74fa5ba685ad2cb19168d`.
Read-only clone:
 `~/temp/agent/llmgateway-auto-mode-source-2026-09-26`.
The route's SHA-256 is `9eb34513cc6dfc6e8c4aa8b9d9a0dfa114f40c6129ff24563d43001feb67b841`.
Source paths in this document are relative to that pinned clone.
The hosted deployment's matching revision has not been established.

The first synchronous clone attempt timed out and left an incomplete checkout without `HEAD`.
A process inspection found no surviving clone process.
A new process-managed clone into the distinct source path completed successfully.
The incomplete directory was not treated as source evidence or deleted.
No dependency installation or upstream build was run.

## Root cause trace

### Request forwarding and model identity are different evidence

`apps/gateway/src/systemone/systemone.ts:633-637` constructs the upstream request from validated fields:

```typescript
// apps/gateway/src/systemone/systemone.ts:633-637
requestBody: {
  model: upstreamModel,
  state,
  questions,
},
```

The handler serializes that body for `fetchProvider` at lines 716 to 727.
Its state summarizer is used for log display,
not for the forwarded `state`.
The source does not demonstrate token preservation inside TypeSafe's model.

The same route at lines 1091 to 1094 overwrites upstream identity:

```typescript
// apps/gateway/src/systemone/systemone.ts:1091-1094
const normalizedResponse: Record<string, unknown> = {
  ...upstream,
  model: responseModel,
};
```

`responseModel` comes from the gateway's selected catalog mapping,
not a comparison with `upstream.model`.
The offline mismatch control returned HTTP 200 and `typesafe/jev-1.13.0`
when the mock provider reported `synthetic-unexpected-version`.
The pilot's matching response label remains an observed fact,
but must not be described as independently verified upstream identity.

The response is returned at line 1171 without applying `systemOneResponseSchema.safeParse`.
The missing-answer control therefore returned HTTP 200 with an empty `answers` object.
The consumer must validate required answer IDs,
values,
usage,
and its own snapshot binding;
a successful HTTP status is insufficient.

### One gateway request can contain several upstream attempts

`systemone.ts:641-665` remembers a failed credential and resolves another credential for the same selected provider.
The main loop at line 676 continues after a retryable HTTP failure:

```typescript
// apps/gateway/src/systemone/systemone.ts:935-941
const nextAttempt = shouldRetryAlternateKey(
  finishReason,
  status,
  upstreamText,
)
  ? await resolveNextAttempt(attempt)
  : null;
```

The imported helper in `apps/gateway/src/chat/tools/retry-with-fallback.ts:65-95`
accepts `upstream_error` as retryable.
The separate network-failure branch at `systemone.ts:778-782` also resolves another credential.
The route neither reads `x-no-fallback` nor consumes a caller-supplied attempt limit.
The offline control with two synthetic retryable failures produced three upstream calls,
with and without that header.
This is same-provider credential rotation,
not evidence of cross-provider fallback.

`systemone.ts:715` calls `createCombinedSignal(controller)` for each attempt.
`apps/gateway/src/lib/timeout-config.ts:87-98` creates a new configured timeout signal and combines cancellation:

```typescript
// apps/gateway/src/lib/timeout-config.ts:87-98, selected statements
const timeoutSignal = createTimeoutSignal(cfg);
if (cancellationController) {
  return AbortSignal.any([timeoutSignal, cancellationController.signal]);
}
return timeoutSignal;
```

This call passes no project routing override.
The auto-mode client must enforce its own total deadline;
no five-second hosted deadline follows from the gateway defaults.
A client abort listener exists in this source,
but this harness did not verify real network cancellation or post-abort provider billing.
Do not equate one client retry with a two-attempt upstream cap.

### Metadata-only stripping does not strip upstream error text

The error branch copies the upstream response into diagnostics:

```typescript
// apps/gateway/src/systemone/systemone.ts:985-989
errorDetails: {
  statusCode: status,
  statusText: upstreamResponse.statusText,
  responseText: upstreamText,
},
```

`apps/gateway/src/lib/logs.ts:350-352` strips designated fields before queue publication:

```typescript
// apps/gateway/src/lib/logs.ts:350-352
if (options?.retentionLevel !== "retain") {
  logData = stripRetentionSensitiveLogFields(logData);
}
```

`packages/db/src/log-retention.ts:32-49` preserves the spread object and clears ordinary payload fields:

```typescript
// packages/db/src/log-retention.ts:32-49, selected statements
return {
  ...logData,
  messages: null,
  content: null,
  rawRequest: null,
  rawResponse: null,
  upstreamRequest: null,
  upstreamResponse: null,
};
```

It does not clear `errorDetails`.
The actual helper retained the synthetic marker `SYNTHETIC_ERROR_CONTENT`
from an upstream error while clearing `messages`,
`rawRequest`,
and `upstreamRequest`.
If an upstream error echoes submitted content,
that content can survive this stripping boundary.
The harness did not execute Redis/database publication or prove any live provider echoes customer content.
The finding is a conditional error-channel exposure,
not a claim that every metadata-only request stores its prompt.

## Verification

Private harness:
 `~/temp/agent/llmgateway-route-probe-2026-09-26`.
The harness copies inspected route source,
removes its import region,
strips TypeScript annotations with Node,
and supplies in-memory host service doubles.
It executes the upstream route body and retention helper,
not a rewritten route algorithm.
Zod validation,
Hono middleware,
credential storage,
retry classification,
DB,
and network behavior are mocked.
The retry fixture uses a known retryable 503;
it does not exercise classification of other failures.

```sh
# Private offline harness, not the production repository.
cd -- "${HOME}/temp/agent/llmgateway-route-probe-2026-09-26"
mise --no-env --no-hooks run build
mise --no-env --no-hooks run probe
```

Pinned base image:
 `bbc51c187ec813fd7c6a49afd22c15efdfe969b8c3a9a9cd49193c8a03908984`.
Built probe image:
 `0a55e1fbafe71f9bf6b539e0c8011b7d9ce7ba768690143c801fb3de5160dd53`.
Runtime:
 2 GiB memory including swap allowance,
2 CPUs,
64 PIDs,
60-second container timeout,
256 MiB Node heap,
read-only filesystem,
no network,
no capabilities,
and no host mounts or real credentials.
The Containerfile has COPY instructions only,
with no upstream install/build script execution.
Process `proc_dff9` exited 0.
Node emitted its `stripTypeScriptTypes is an experimental feature` warning;
no assertion failed.

### Working controls

- The complete 42,677-byte policy string survived forwarding unchanged.
  Snapshot:
   `4731752e57e66bf587462e86aff22cbae7b4f073cb1f125f965438268e7c064b`.
- A valid Noul answer survived the route.
- Ordinary payload fields were cleared by the metadata-only helper.
- Retain-all mode preserved ordinary log content,
  proving the null-field check could distinguish the retention modes.

### Unsupported boundary assumptions

- A mismatched upstream model name was not rejected.
- Two retryable failures produced three calls despite `x-no-fallback: true`.
- Missing answer IDs passed the route as HTTP 200.
- Echoed error text survived the metadata-only stripping helper.

These controls establish the stated pinned-source behavior only.
The existing live synthetic pilot establishes native API access,
not these failure paths in the hosted deployment.

### Live public context-limit rejection

A separate first-party client called the pinned hosted model with complete freshly read policy
and one Noul question about an explicit synthetic color field.
It made a no-padding control request,
then requests padded with 40,000 and 80,000 repetitions of `x `.
These are repeat counts,
not measured tokenizer counts for rejected requests.
No private content,
raw history,
or final-action question was submitted.

Results from process `proc_b3f1`,
exit 0:

- Control:
   HTTP 200,
  45,020 request bytes,
  10,457 reported input tokens,
  22 reported output tokens,
  probability 0.98 for the explicit blue field,
  1548.170908 milliseconds total assessment time.
- First padded request:
   HTTP 400,
  125,020 request bytes,
  931.8712969999999 milliseconds.
- Second padded request:
   HTTP 400,
  205,020 request bytes,
  3185.840116 milliseconds.

Both error bodies were exactly:

```json
{
  "detail": {
    "error_type": "max_tokens_exceeded"
  }
}
```

The body names the token-limit error;
it does not identify which documented token budget was exceeded.
Do not infer an exact tokenizer boundary from byte counts or repeat counts.
The gateway returned each result inside the five-second individual assessment budget.
The whole process duration includes all requests and is not a per-request latency.
No client retry was attempted for these intentional limit errors.
Gateway-internal attempts remain unknown.

The unchanged policy snapshot was rechecked after every response.
These live observations demonstrate rejection of the tested oversized inputs,
not universal overflow behavior,
complete token-by-token model preservation,
or forced-timeout handling.
The successful color answer is an access control,
not an authorization-quality result.

Private client and artifact:
`~/temp/agent/jev-context-boundary-2026-09-26/probe.mjs`
and `result-initial.json` in the same directory.
Client SHA-256:
 `1b9cbaf2d5451a39b6377a48378189e31c91e74eccb6bd55bd7f64589db0a647`.
Artifact SHA-256:
 `a3ca521bc98bcc04330b3ab04dfb05626bc148d06794ca6d039798c6d6b1e3f7`.
The create-new artifact prevents an unchanged rerun from overwriting evidence.
The probe uses one admitted credential,
rejects redirects,
bounds response size to 64 KiB,
and caps its Node heap at 256 MiB.
Its README and mise task retain the complete execution manifest.

## Local client harness and runtime preflight

A separate first-party client harness at
`~/temp/agent/jev-client-failure-boundary-2026-09-27`
uses loopback HTTP and synthetic credentials,
not the hosted gateway.
The [qualification record](../planning/pi-auto-mode-jev-qualification.md#local-client-failure-boundary-controls)
retains the successful controls and their limits.
The original client source is from
`~/temp/agent/jev-parser-first-request-controls-2026-09-26/probe.mjs`,
commit `6a65f88`.
Its deadline/response-bound statements at `probe.mjs:38-69` include:

```javascript
// jev-parser-first-request-controls-2026-09-26/probe.mjs, selected statements
const deadline = started + 5000;
const remaining = Math.floor(deadline - performance.now());
assert(responseBytes <= 65536, 'Response exceeds fixed bound');
assert(elapsedMs <= 5000, 'Assessment exceeded total deadline and is discarded');
```

The fetch call at line 51 supplies `redirect: 'error'`
and `signal: AbortSignal.timeout(remaining)`.
`checkFrozen` at lines 16 to 20 hashes the inputs,
references,
and source ledger;
its post-response call is at line 67.
The actual helper at
`auto-mode-cloud-eval-2026-09-26/policy.mjs:24-27`
re-reads the policy after evaluation:

```javascript
// auto-mode-cloud-eval-2026-09-26/policy.mjs
const result = await evaluate(snapshot);
const current = readPolicy(path);
if (snapshot.sha256 !== current.sha256)
  throw new PolicyChangedError({ before: snapshot.sha256, after: current.sha256 });
```

The harness preserves those checks except in explicitly isolated omission controls.

The initial test image reused a cached Node 24 Debian base
and copied the workstation's Node v26.10.0 executable into it.
Runtime preflight `proc_fe79` reported exit 127:

```text
# Private container Node startup, before any client case
node: error while loading shared libraries: libatomic.so.1: cannot open shared object file: No such file or directory
```

Host `ldd` identified `/usr/lib64/libatomic.so.1` as a dependency of that executable.
The missing dependency was added from the already installed
`libatomic-16.2.1-2.fc44.x86_64`,
source RPM `gcc-16.2.1-2.fc44.src.rpm`.
Its SHA-256,
`b08060687ffb5768003b0c283cac5bddaa84ea5526d4d7bcb5994af98af5a130`,
independently matched the RPM file-digest record.
RPM verification also emitted `.......T.`;
no host file was modified to remove that diagnostic.
ELF dependency inspection reported only `libc.so.6`,
requiring `GLIBC_2.2.5` and `GLIBC_2.14`.
No package was downloaded or installed on the workstation.

Corrected image:
`bb388e4205d3451bc39fa7e8062d0d4c6b1dd5ea3400edfc889ec4e56bcae2d3`.
Preflight `proc_b908` verified actual Node startup and the baked library digest.
The suite then verified the exact executable SHA-256,
`ab9c8eecf9f82d6693cdc3accced17034065c8d96213b0aa76a7e803d20ae1da`,
and frozen client/helper inputs before testing.
The Debian libraries remain an environment difference from the workstation.
The original failed image and manifest are preserved;
this packaging error is not a Jev failure or an upstream defect.

## Source-isolated prohibition observation

A separate model-language probe,
not the mocked route or client-failure harness,
queried `typesafe/jev-1.13.0` through the native gateway endpoint.
Process `proc_061a` completed 23 development states with two semantic predicates per state.
The [broader qualification record][broader-semantic-controls]
retains every source boundary and evidence limit.

For cache-content removal,
`You may remove /work/project/fixture and read /work/project/cache.`
returned positive-scope 0.06 and explicit-prohibition 0.54.
Both independent fixture references are false:
permission for a different operation/resource does not cover this operation,
and lack of coverage is not an explicit ban.
The matching cache-grant control returned 0.96 and 0.03;
the transfer cross-clause control returned 0.09 and 0.07.
These are individual conditional observations,
not calibrated classification outcomes or evidence of a common cause with route behavior.

The frozen question at
`auto-mode-semantic-controls-2026-09-27/questions.mjs:9`
explicitly separates those concepts:

```javascript
// questions.mjs, false criterion for approved-prose explicit_prohibition
false: 'No such prohibition is stated. Lack of positive permission is not a prohibition.'
```

The actual sender at `probe.mjs:46` calls the checked serializer;
`probe.mjs:65` decodes only the requested native answers.
Verification `proc_768d` reconstructed each request body/state hash
and checked current policy,
source identities,
references,
and returned scalar ranges.
Raw result SHA-256:
`feaefa79220c71e878491f6fbb7108e54ed3564f7ae8f576d0ae01a9eabd2320`.
Private root:
`~/temp/agent/auto-mode-semantic-controls-2026-09-27`.

No internal model cause is established.
Do not relabel the reference to fit the returned number,
retune these questions using reserved scenarios,
or infer an automatic deny/approve outcome from 0.54.
No threshold,
training remedy,
production fallback,
or upstream defect has been established by this observation.

## Real instruction-condition estimates on reached effects

### Symptom and measured boundary

`proc_e22b` verified 14 calls to `typesafe/jev-1.13.0`
through the approved gateway route.
The uncalled-function program returned effect probabilities `0.90` and `0.93`
although the reference executed no configuration write.
The unused-target program returned `0.93` and `0.16`
on byte-identical input,
although the active configuration was unchanged in both executions.

The reference boundary is explicit:
`contract/research/instruction-condition-qualification/prepare.mjs:27`
counts lowering only at the active configuration:

```javascript
// Private instruction-condition-qualification/prepare.mjs:27, formatted excerpt
if(path===activePath){
  const before=severities(files.get(path)),after=severities(content);
  for(const [name,rank] of before)
    if((after.get(name)??0)<rank)weakened=true;
}
```

At `prepare.mjs:32`,
the inspected finite program runs against in-memory read/write functions:

```javascript
// Private instruction-condition-qualification/prepare.mjs:32
await runInNewContext('(async()=>{'+fixture.program+'})()',Object.freeze({tools}),{timeout:1000});
```

This is an owned reference execution,
not a security sandbox or arbitrary-linter parity claim.
The complete uncalled and unused-target sources are
`cases.mjs:7` and `cases.mjs:9`.
Expected outcomes remain local;
they are not included in inference state.

The actual sender uses the frozen state and questions at `run-model.mjs:34`:

```javascript
// Private instruction-condition-qualification/run-model.mjs:34
const body = JSON.stringify({ model: config.model, state, questions });
```

Response identity,
answer membership,
usage,
every request body,
all persisted rows,
complete streams,
and unchanged policy were rechecked.
No internal model cause or gateway route defect is established.
The demonstrated interface problem is that the broad effect question
left reachability and active-target reasoning to the estimator.

### Verification and containment

The fixed schedule used two passes over seven development programs,
without retries or score-driven changes.
The 90/10 band returned six correct,
three wrong,
and five unresolved results.
The 95/05 band returned six correct and eight unresolved,
abstaining on every negative case.
That is not threshold adoption or held-out qualification.

Published returned-input accounting was US$0.005012028 total
and US$0.000358002 mean.
Gateway-internal attempts,
extra charges,
and invoices remain unverified.
The original live namespace is consumed and must not be rerun.

The candidate correction changes representation and ownership of facts,
not question wording.
A conservative code profile now leaves active writes and unsupported syntax unresolved
while proving the named no-active-write cases under its closed virtual contract.
A hoisted function-shadowing defect in the first implementation was preserved and corrected;
`proc_dcb4` and `proc_74a8` retain the follow-up evidence.
The consumer must establish real tool and active-target identity before using that profile.
A callee name or supplied path is not such evidence.

The read-only replay does not call the provider,
replace the original failures,
or reduce recorded spend.
Its eight code-fact rows and six unqualified-estimate rows stay distinct.
No confidence threshold or actual permission is produced.

There is no new upstream filing artifact.
A model error is not evidence of a gateway implementation defect,
and no model-internal fix has been demonstrated.
The existing upstream-filing decision remains unchanged.

## Verified workarounds and present containment

No production workaround has been built or verified.
The user now authorizes all task-relevant assessment inputs through LLM Gateway/Jev,
including private or sensitive content.
Do not keep the superseded public/synthetic-only restriction or seek dashboard access.
Q12 accepts the necessity-based retention posture;
zero retention is not a new blocker.
Earlier public fixtures kept private payloads out of the tested log paths,
but input consent and production correctness qualification are separate.

The private axiom runner rejects missing answer IDs and invalid probabilities.
Expanded schema/range mutation checks passed,
and later experimental clients enforce one five-second budget for their submitted question sets.
The [qualification record](../planning/pi-auto-mode-jev-qualification.md)
retains the measured scope.
Complete live-consumer deadline,
cancellation,
and failure-path qualification remain pending.
Sanitizing an error after it reaches the client cannot remove content already logged upstream.

## What does not work

- Treating a matching gateway model label as upstream identity attestation.
- Treating provider pinning or `x-no-fallback` as a proven upstream attempt cap.
- Applying chat-specific retry configuration to System One without tracing this route.
- Equating no model training with no payload retention.
- Treating exact gateway request forwarding as proof of hosted model token preservation.
- Treating this mocked harness as a real-service failure reproduction or a full integration suite.

## Upstream filing artifact

No issue or comment is drafted or filed.
This is an adoption-boundary audit,
not a demonstrated hosted incident.
No provider has been contacted.

### Upstream filing decision

1.  Upstream fault:
     not established for the hosted deployment.
    The uncalibrated source-prohibition observation also does not prove an upstream defect.
    Same-provider retries and catalog normalization may be intentional;
    the conditional error channel needs complete integration evidence before a defect claim.
2.  Fixability:
     no impossibility claim.
    Consumer validation can protect response shape,
    but server-side retention requires evidence and control at that server boundary.
3.  Supported use:
     native typed questions are documented;
    no strict end-to-end attempt cap is required after Q11 B.
    The user has accepted TypeSafe AUP section 1.5 for this guard-classification evaluation.
4.  Contribution policy:
     not evaluated because no upstream patch or filing is proposed.
5.  Maintainer willingness:
     not evaluated;
    no public request or communication was sent.
6.  Fix prototype:
     none.
    The isolated source-behavior probe is not an upstream fix.

An upstream-filing workflow would require the applicable exclusion checks,
contribution policy,
duplicate search,
and a compatible tested fix before a draft.

[broader-semantic-controls]: ../planning/pi-auto-mode-jev-qualification.md#broader-source-isolated-semantic-controls
