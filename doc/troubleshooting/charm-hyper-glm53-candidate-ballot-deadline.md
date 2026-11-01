# Charm Hyper full-document requests can hit local stream deadlines

## Symptom

Candidate I sent two concurrent,
candidate-scoped verifier requests to Hyper model `glm-5.3-flash` through `POST /v1/messages`.
Each request contained complete source,
archive,
candidate,
manifest,
and the sole page-referenced image,
plus one forced compact-ballot tool.

Both streams remained active until the local call deadline and then raised `StreamCutShortError`:

- candidate ordinal zero:
  cut after 359,987 milliseconds,
  first byte after 7,415 milliseconds,
  maximum observed inter-frame gap 1,113 milliseconds,
  zero content characters,
  and 33,277 reasoning characters;
- candidate ordinal one:
  cut after 359,989 milliseconds,
  first byte after 8,851 milliseconds,
  maximum observed inter-frame gap 1,118 milliseconds,
  zero content characters,
  and 40,883 reasoning characters.

Neither call returned a ballot.
The failure is not a provider HTTP status and is not evidence that Hyper closed the streams.
Both streams were still delivering reasoning frames when the configured local deadline fired.

The run correctly treated both calls as spent-unusable evidence.
It did not retry them,
and deterministic restart made zero transport calls.

## Root cause

Prototype commit `28d548d84fe1fcb51765b0e9a845c6361bf6359c` sets a six-minute default evaluation deadline in
`package/module/translation-repair/src/prototype-hyper-expansion-client.ts:36-39` (removed before 2026-10-18):

```ts
/**
 * Default bounded evaluation deadline.
 */
const EVALUATION_TIMEOUT_MS = 360_000;
```

The same file arms that deadline for every request and allows an explicit request value to replace it at
`package/module/translation-repair/src/prototype-hyper-expansion-client.ts:287-294` (removed before 2026-10-18):

```ts
/**
 * Joined per-call deadline.
 */
using deadline = armCallDeadline({
  signal: request.signal,
  timeoutMs: request.exchangeTimeoutMs ?? EVALUATION_TIMEOUT_MS,
  label: model.id,
},);
```

`package/translation-repair/provider/src/call-deadline.ts:112-130` arms the timer.
It aborts the call-owned controller with `CallTimeoutError` when it fires:

```ts
/**
 * Timer forfeiting the call at its deadline.
 */
const deadline = setTimeout(
  function onDeadline() {
    callController.abort(new CallTimeoutError({
      label,
      timeoutMs,
    },),);
  },
  timeoutMs,
);
```

The client passes that deadline-owned signal to the stream transport at
`package/module/translation-repair/src/prototype-hyper-expansion-client.ts:298-311` (removed before 2026-10-18):

```ts
const reply = await transport({
  url: HYPER_MESSAGES_URL,
  label: model.id,
  method: 'POST',
  headers,
  bodyJson: JSON.stringify(evaluationBody({
    request,
    model,
  }),),
  signal: deadline.callSignal,
  wireFormat: 'anthropic',
```

`package/translation-repair/provider-stream/src/stream-drain.ts:321-358` reports progress,
passes through only self-ended stream errors,
and otherwise retains partial text in `StreamCutShortError`:

```ts
const isSelfEnded = isSelfEndedStream({ error, },);

reportStreamProgress({
  label,
  progress: guard.progress(),
  unreadableFrames: watch.unreadableFrames(),
  outcome: endedOutcome({ error, },),
  openingText: watch.openingText(),
  generatedChars: watch.generatedChars(),
},);

if (isSelfEnded)
  throw error;

throw new StreamCutShortError({
  label,
  partialText,
  progress: guard.progress(),
  cause: (guardSignal.aborted && (!callerSignal.aborted)) ? guardSignal.reason : error,
},);
```

The persisted failure type was `StreamCutShortError` for both calls.
The metadata record intentionally omits nested error cause,
so it does not directly prove which error the stream drain wrapped.
The measured cut times align with the 360,000-millisecond client deadline,
and the process logs show active reasoning frames up to that boundary.
Those facts establish that the local deadline fired before either ballot completed.
The live catalog's `max` default-reasoning value is context,
not a proven cause.
No evidence establishes whether either response would eventually have completed under a longer deadline.

Hyper documents `POST /v1/messages` as Anthropic Messages compatible,
accepting standard Anthropic parameters and SSE streaming.
The documentation does not promise completion latency for this model or workload.
The managed inference service source is not public;
there is no provider-side source path to clone or trace.

## Verification

### Version and pins

- Hyper catalog snapshot:
  2026-08-31,
  SHA-256 `d68a71501ffa7c67825467ef935c177ad0abb01560f81f17c3fe65fe88742ab7`;
- live model:
  `glm-5.3-flash`,
  vision true,
  maximum output 131,072,
  default reasoning `max`;
- prototype commit:
  `28d548d84fe1fcb51765b0e9a845c6361bf6359c`;
- corpus commit:
  `a80634a674f94861ea3b7056fba054ca9eab1a2c`;
- harness:
  `~/temp/agent/run-candidate-i-carena-20260831.mjs`,
  SHA-256 `90455077ef109c50750728dc9cb975acde33459d91a4d55ca6ace5dc881e37ff`;
- summary:
  `~/temp/agent/prototype-Carena-I-candidate-ballot-20260831/calibration-summary.json`,
  SHA-256 `48c4607d29a90aebacc27f5130e7e45d8d83f4529958419bfd937b35afe3e115`.

The exact invocation was:

```bash
env CANDIDATE_I_HARNESS_DIGEST=90455077ef109c50750728dc9cb975acde33459d91a4d55ca6ace5dc881e37ff \
  node ~/temp/agent/run-candidate-i-carena-20260831.mjs
```

The harness refuses an existing output root,
a different harness digest,
a different source or artifact pin,
more than eight exchanges,
a retrying client,
a request missing the page image,
or a restart transport call.

### Working catalog

- A synthetic one-pixel image and small Candidate-I-shaped forced tool returned one exact `tool_use` in 1,607
  milliseconds.
- Both full-document author calls completed through the same Hyper Anthropic transport.
- Other verifier routes produced two admissible ballots in the same second wave.

### Failing catalog

- GLM verifier for candidate ordinal zero streamed reasoning until local deadline and returned no ballot.
- GLM verifier for candidate ordinal one streamed reasoning until local deadline and returned no ballot.

The synthetic route probe establishes forced-tool transport compatibility.
It does not predict completion of a full translation-review workload.

### Candidate L Qwen lean-author confirmation

The one zero-retry Candidate L calibration on 2026-09-01 used:

- Qwen route `qwen3.8-27b`;
- one forced 27-value lean-realization tool;
- complete source and archive evidence;
- the sole page-referenced image;
- a 32,000-token output ceiling;
- a 900,000-millisecond local deadline;
- no reasoning,
  thinking,
  effort,
  or temperature override.

The author request reached the local deadline after 900,022 milliseconds and persisted
`failureType: "CallTimeoutError"`.
The metadata-only exchange record retained no HTTP status,
response body,
block count,
stop reason,
or token count because the transport did not complete before the deadline.
The node was spent-unusable,
its three dependent verifiers were skipped,
and restart sent nothing.

Candidate I had previously proved that Qwen can complete a different full-document author request.
Candidate L therefore establishes workload-specific deadline failure,
not a stable Qwen incapability.
The spent Candidate L prompt cannot be repeated with a longer deadline.

## Verified workarounds

### Quarantine each cut response and preserve bounded settlement

Candidate I records each request as spent-unusable,
lets every manifested sibling settle,
and selects only from admitted evidence.
Restart reuses the terminal node records and sends nothing.

Tradeoff:
this preserves cancellation,
restart,
audit,
and no-retry semantics,
but it leaves both candidates below the required independent-family evidence floor.
It is a safety workaround,
not a way to make GLM produce a ballot.

### Keep private fallback ineligible

The runtime may retain one complete candidate privately,
but records `evidenceFloorMet: false` and `productionEligible: false`.
Complete-page review then rejected both candidates independently of the ballot result.

Tradeoff:
complete prose survives for diagnosis,
but normal publication still fails.

## What does not work

### Inferring consumer fit from a small forced-tool probe

The synthetic probe completed quickly,
but both complete consumer requests crossed the local deadline.
Transport acceptance is not workload completion evidence.

### Treating advertised output capacity as completion headroom

The catalog advertised 131,072 maximum output tokens.
The requests used a 32,000-token project ceiling,
yet both calls spent the full local time in reasoning and produced no tool content.
Maximum output metadata does not establish time-to-ballot.

### Depending on compact wire output alone

The required ballot was a compact pair of status strings plus bounded findings.
The prompt still required review of 134 clause-and-relation obligations plus ten global criteria.
Small expected JSON did not make the reasoning workload small.

### Repeating either request with a longer deadline

No such retry was run.
Each potentially transmitted canonical prompt is already spent,
and Candidate I's lifecycle forbids redispatch.
`exchangeTimeoutMs` can change the local deadline in source,
but using it on these same prompts would violate the experiment contract.
It still would not prove eventual completion in advance.
A future design must use a substantively different,
smaller review contract and receive its own validation.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?**
    No.
    Hyper delivered active streams;
    the local client aborted them at its configured deadline.
    Consumer fit failed,
    but no provider defect was established.
2.  **Can upstream fix it?**
    Not established.
    No service defect or deciding provider implementation path is known.
3.  **Are they supporting this use case?**
    Partly.
    Hyper documents Anthropic Messages,
    standard parameters,
    and SSE streaming,
    but makes no latency commitment for exhaustive translation ballots.
4.  **Would the repository welcome our contribution?**
    Not applicable.
    Hyper exposes support email and Discord,
    but no public service source repository or issue tracker was found.
    GitHub issue and pull-request searches for the endpoint,
    model id,
    and timeout behavior found no duplicate.
5.  **Will they likely fix it?**
    Not established because no upstream fault or requested change is identified.
6.  **Have we prototyped a minimal upstream fix?**
    No.
    There is no public provider implementation to patch,
    and changing the local deadline would not establish workload completion.

`.out-of-scope/` contains no Hyper or GLM exemption.
The six constraints do not pass.
There is nothing responsible to file or add upstream,
so no issue or comment draft is retained.
