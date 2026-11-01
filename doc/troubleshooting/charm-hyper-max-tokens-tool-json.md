# Charm Hyper Anthropic Messages on 2026-09-01 can end forced tool work at `max_tokens`

## Symptom

Charm Hyper accepts Anthropic-compatible `POST /v1/messages` requests with required `max_tokens`.
Structured translation responses travel as forced tool input.
A response that reaches the token limit reports `stop_reason: "max_tokens"`.

The repository currently carries that value as `reply.finishReason`,
but `readJsonOutcome()` returns `kind: "ok"` whenever the accumulated tool JSON parses and passes the caller guard.
A truncation that happens at a syntactically complete boundary can therefore be admitted despite the provider saying it stopped at the output ceiling.

Zero-spend reproduction on 2026-08-31:

```text
input finishReason = max_tokens
input text = {"x":1}
observed outcome = {"kind":"ok","value":{"x":1},"rawText":"{\"x\":1}"}
```

A truncation inside an unfinished JSON token or container becomes a schema mismatch.
The gap concerns a parseable prefix that is not proof of natural completion.

## Root cause

### Hyper makes `max_tokens` the output boundary

Hyper's Anthropic Messages documentation says `max_tokens` is required and describes it as “Max output tokens”:

```text
POST /v1/messages
max_tokens integer yes Max output tokens
All standard Anthropic parameters are accepted.
```

Hyper service source is not public,
so the published compatibility documentation and observable endpoint are the upstream authority.

The consumer applies its project-wide answer bound at
`package/translation-repair/shared-contract/src/hyper-catalog.ts:214-243`:

```ts
const MEASURED_ANSWER_BOUND = 32_000;

export function answerCeilingFor(
  { modelId, }: { readonly modelId: HyperServedId; },
): number {
  const { maxOutputLength, } = HYPER_MODELS[modelId];

  return Math.min(
    MEASURED_ANSWER_BOUND,
    maxOutputLength,
  );
}
```

`package/translation-repair/provider/src/anthropic-request.ts:433-458` sends the lower of that bound and a caller
override:

```ts
return {
  model: modelId,
  max_tokens: Math.min(
    ceiling,
    maxTokens ?? ceiling,
  ),
  stream: true,
  // ...
};
```

The 32,000-token value is therefore the current Hyper request ceiling across this package,
not only the frozen roster-expansion runner.

### Tool arguments are streamed as model-produced partial JSON

Anthropic's Streaming Messages documentation says `input_json_delta.partial_json` contains partial JSON strings,
while final `tool_use.input` is an object.
Its documented stream includes whitespace inside model-produced fragments:

```json
{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":" {\"location\":"}}
```

The same documentation says current models emit one complete key and value at a time,
but does not promise canonical whitespace or compact serialization.
Compact JSON measurement is therefore one witness serialization,
not a bound over every textual tool-input stream.

### The stream preserves the stop reason

`package/translation-repair/provider/src/anthropic-completion.ts:280-324` records `message_delta.delta.stop_reason`.
Lines 543-580 return the last reason as `finishReason`:

```ts
const stopReason = fold
  .stopReasons
  .at(-1,)
  ?? '';

return {
  text: (toolAnswer === '') ? prose : toolAnswer,
  ...((stopReason === '') ? {} : { finishReason: stopReason, }),
  ...usageOf({ counts: fold, },),
};
```

Anthropic's stop-reason documentation states that `max_tokens` means the response reached the requested limit and was cut off.
Its streaming documentation also states that `message_delta.usage.output_tokens` is cumulative.

### JSON admission consults the stop reason only after parse failure

`package/translation-repair/provider/src/chat-json-outcome.ts:93-102` can format the reason:

```ts
const { finishReason, } = reply;

if (finishReason === undefined)
  return '';
return ` (model stopped with finish_reason=${finishReason})`;
```

That helper is called only in the unparseable branch at lines 200-227.
The successful parse path at lines 230-253 checks the caller guard and returns `ok` without examining `finishReason`:

```ts
const candidate = attempt.value;

if (!validate(candidate,)) {
  return {
    kind: 'schema-mismatch',
    // ...
  };
}

return {
  kind: 'ok',
  value: candidate,
  rawText: reply.text,
  ...usageSpread,
};
```

The source therefore explains the zero-spend reproduction exactly:
parse and guard success bypass the truncation marker.

An earlier envelope reading treated every provider-returnable parseable object as potentially complete.
That reading is wrong for `finishReason: "max_tokens"`.
Transport boundedness and completion validity are separate facts.

## Verification

Accessed on 2026-08-31:

- Hyper Anthropic Messages documentation at
  `https://hyper.charm.land/docs/api/anthropic-messages.html`;
- Anthropic Streaming Messages documentation at
  `https://platform.claude.com/docs/en/build-with-claude/streaming`;
- Anthropic Stop reasons documentation at
  `https://platform.claude.com/docs/en/build-with-claude/handling-stop-reasons`.

Reproduction command:

```bash
cd /var/home/user/temp/agent/translation-repair-finite-prototypes
node --experimental-strip-types --input-type=module -e '
  import { readJsonOutcome } from "./package/translation-repair/provider/src/chat-json-outcome.ts";
  const outcome = readJsonOutcome({
    modelId: "hf:Qwen/Qwen3.8-27B",
    reply: { text: "{\"x\":1}", finishReason: "max_tokens" },
    validate(value) {
      return typeof value === "object" && value !== null && "x" in value && value.x === 1;
    },
  });
  console.log(JSON.stringify(outcome));
'
```

### Clean patterns

- Natural tool completion with valid JSON and non-truncating finish reason:
  parse and caller guard may return `ok`.
- `max_tokens` ending inside JSON:
  parsing fails and returns `schema-mismatch` with stop reason in detail.
- Any parsed object rejected by caller guard:
  returns `schema-mismatch`.

### Failing pattern

- `finishReason: "max_tokens"` plus parseable guard-valid JSON:
  the unhardened consumer returns `ok`.

### Candidate K live confirmation

The zero-retry pinned-Carena Candidate K calibration used harness SHA-256
`41094dded81174b49bec30b5b7c14966362470524180c6fe44daadb77897886a`
and manifest digest
`5d7b817948a1927128dd828dfcccaa5b0a6cc684e6c621db54b0b4f6ca92c09d`.
Its metadata-only evidence is retained under
`/var/home/user/temp/agent/prototype-Carena-K-review-unit-20260901`.

Two author exchanges independently confirmed truncating completion behavior:

- `glm-5.3-flash` returned HTTP 200 after 648,545 milliseconds,
  reported 32,000 output tokens and `stop_reason: "max_tokens"`,
  and emitted only a thinking block;
- `minimax-m3` returned HTTP 200 after 184,139 milliseconds,
  reported 32,000 output tokens and `stop_reason: "max_tokens"`,
  and emitted thinking plus a tool-use block.

The hardened prototype consumer classified both terminal nodes as
`schema-mismatch` with detail type `truncated-completion`.
Neither output produced a candidate or caused verifier dispatch.
The result also proves that extending the local request deadline from 360,000 to 900,000 milliseconds did not prevent
these GLM and MiniMax exchanges from exhausting the fixed output ceiling.
It does not prove that either route always truncates.

### Candidate L live confirmation

The one zero-retry Candidate L calibration used harness SHA-256
`6d050811e82a3156e1da5a0524b0bdb6a595d27fbfb814296ed854168b85a0fc`
and manifest digest
`14e5b100a5bddfc4426b7c0d5dbee89255e90d3a787644e5f115f08c6ed39fd3`.
Its GLM 5.3 Flash verifier returned HTTP 200 after 564,987 milliseconds,
reported 32,000 output tokens and `stop_reason: "max_tokens"`,
and emitted one thinking block without a tool-use block.
The consumer classified the node as `truncated-completion` and spent-unusable.
It had no ballot or selection effect.

This request carried the smaller Candidate L verifier protocol rather than Candidate K's author protocol.
The result shows that moving GLM to verifier-only duty did not avoid the fixed output ceiling on this complete-page workload.
It does not establish that GLM always truncates or that default reasoning is the cause.
The request used Hyper's default reasoning and sent no reasoning,
thinking,
effort,
or temperature override.

## Verified workarounds

### Reject truncating finish reasons before JSON admission

At the consumer boundary,
return schema mismatch before parsing or voting whenever `finishReason` is `max_tokens`.
Persist raw-response digest and finish reason as operational evidence.

Tradeoff:
a provider could emit a complete object exactly at the limit and still be rejected.
That is fail-closed and preserves the contract that truncated output has no effect.

### Keep the provider completion cap as transport bound

Continue sending the manifested `max_tokens` value.
Treat it as maximum provider-returnable completion tokens,
not as proof that every response schema instance can fit.

Tradeoff:
overlarge or pathological model output terminates unusably rather than producing a candidate.
A working-model calibration still needs measured complete responses with explicit framing headroom.

## What does not work

- Parsing first and mentioning `finishReason` only on syntax failure:
  a complete JSON prefix bypasses the marker.
- Continuing a truncated tool call in another request:
  it violates the finite one-payload prompt contract.
- Treating final `tool_use.input` as canonical serialized JSON:
  the stream exposes model-produced partial JSON strings and no compact-whitespace guarantee.
- Treating `max_tokens` as semantic completeness:
  it is only a transport generation limit.
- Raising `max_tokens` above 32,000 without owner decision:
  current package policy deliberately applies that measured answer bound to every Hyper model.
- Extending only the local deadline:
  Candidate K's GLM and MiniMax author calls reached `max_tokens` before the 900,000-millisecond deadline.

## Upstream filing decision

`.out-of-scope/` has no matching Hyper or Anthropic exemption.
No upstream issue search is warranted because the provider behavior is documented;
the fault is consumer admission.

1. Upstream fault:
   **no**.
   Hyper documents `max_tokens` as output limit,
   and Anthropic documents `stop_reason: max_tokens` as truncation.
2. Upstream can fix:
   **not applicable** to this incident;
   consumer must honor the supplied stop reason.
3. Supported use case:
   **yes**.
   Hyper advertises Anthropic Messages compatibility and standard parameters.
4. Contribution welcome:
   **unknown for Hyper service source**;
   no public source or contribution policy was found.
5. Likely fix:
   **no upstream fix is needed**.
6. Minimal compatible prototype:
   **not upstream**.
   The minimal fix belongs in `chat-json-outcome.ts` with a unit test.

### Upstream filing artifact

Nothing to file or comment upstream.
The actionable change is local consumer hardening.
