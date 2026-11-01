# Charm Hyper forced Qwen tool response on 2026-09-01 ended with unparseable JSON

## Symptom

A streamed Charm Hyper Anthropic Messages request used:

- route `qwen3.8-27b`;
- one forced `immutable_shell_slots` client tool;
- one exact WebP image;
- `max_tokens: 32000`;
- no nondefault reasoning,
  effort,
  temperature,
  or thinking parameter.

The endpoint returned HTTP 200 and a terminal `message_stop`.
Metadata reported one thinking block,
one tool-use block,
`stop_reason: "end_turn"`,
and 51,468 output tokens.
The accumulated tool-input string was not parseable JSON,
so the Candidate K consumer persisted:

```text
failureType = schema-mismatch
failureDetailType = unparseable-json
```

The raw generated response was intentionally not retained.
The evidence therefore does not expose corpus wording or model output,
and it cannot identify the malformed token or assign the defect to the routed model versus Hyper's adapter.

## Root cause

### Hyper promises Anthropic Messages compatibility

Hyper's Anthropic Messages documentation,
accessed on 2026-09-01,
describes `POST /v1/messages` as an “Anthropic Messages API compatible endpoint” and says:

```text
All standard Anthropic parameters are accepted.
```

The cited searches identified no public Hyper service source.
Published documentation and the observed endpoint are therefore the available upstream evidence.

Anthropic's [streaming documentation][anthropic-streaming] says `input_json_delta.partial_json` values are partial JSON
strings.
It also says final `tool_use.input` is always an object.
Anthropic's [stop-reason documentation][anthropic-stop] says:

```text
A response that leaves a client tool_use block waiting on you never has a stop_reason of pause_turn:
when Claude stops to call your tools, stop_reason is tool_use.
```

Both pages were accessed on 2026-09-01.
The observed combination of a client tool-use block,
`end_turn`,
and unparseable accumulated input does not match that documented successful-tool shape.

[anthropic-streaming]: https://platform.claude.com/docs/en/build-with-claude/streaming
[anthropic-stop]: https://platform.claude.com/docs/en/build-with-claude/handling-stop-reasons

This is an observed compatibility failure,
not enough evidence to locate an internal provider cause.

### The consumer preserves provider fragments in arrival order

At `package/translation-repair/provider/src/anthropic-completion.ts:271-278`,
the consumer appends each tool fragment without rewriting it:

```ts
if (kind === 'input_json_delta')
  fold
    .toolParts
    .push(stringField({
    fields: delta,
    name: 'partial_json',
  },),);
```

At `package/translation-repair/provider/src/anthropic-completion.ts:554-580`,
the consumer joins those fragments and returns joined tool input as answer:

```ts
const toolAnswer = fold
  .toolParts
  .join('',);

return {
  text: (toolAnswer === '') ? prose : toolAnswer,
  ...((stopReason === '') ? {} : { finishReason: stopReason, }),
  ...usageOf({ counts: fold, },),
};
```

The parser does not repair malformed generated JSON.
`package/translation-repair/provider/src/chat-json-outcome.ts:197-227` parses once and fails closed:

```ts
const attempt = parseModelJson({
  text: (marker === '') ? content : stripCodeFence({ text: content, },),
},);

if (!attempt.parsed) {
  // ...
  return {
    kind: 'schema-mismatch',
    rawText: reply.text,
    detail: `content is not valid JSON: ${attempt.detail}${stopped}`,
    ...usageSpread,
  };
}
```

The consumer result follows directly from these boundaries:
provider fragments formed a terminal but invalid tool-input string,
and the JSON gate gave it no candidate effect.

An earlier assumption treated HTTP 200 plus a forced tool block as proof of parseable arguments.
Candidate K disproves that assumption.
Transport success,
forced tool selection,
stream termination,
and JSON validity are separate predicates.

## Verification

The affected calibration used:

- prototype commit `399d0d686524818cd78e91bdb9417496901f9880`;
- built Candidate K artifact SHA-256
  `bc64696ca4d7e1066e099718f5c83624eafc1be388d635191214f1c6f4678a83`;
- harness SHA-256 `41094dded81174b49bec30b5b7c14966362470524180c6fe44daadb77897886a`;
- manifest digest `5d7b817948a1927128dd828dfcccaa5b0a6cc684e6c621db54b0b4f6ca92c09d`;
- live model-catalog digest `d68a71501ffa7c67825467ef935c177ad0abb01560f81f17c3fe65fe88742ab7`.

Read the metadata-only reproduction without redispatching the spent prompt:

```bash
node --input-type=module - <<'NODE'
import { readFile } from 'node:fs/promises';
const root = '/var/home/user/temp/agent/prototype-Carena-K-review-unit-20260901';
const exchange = JSON.parse(await readFile(`${root}/network-exchanges/01.json`, 'utf8'));
const node = JSON.parse(await readFile(`${root}/node-review-unit-author-0.json`, 'utf8'));
console.log(JSON.stringify({
  status: exchange.status,
  blockCounts: exchange.blockCounts,
  stopReasons: exchange.stopReasons,
  outputTokens: exchange.outputTokens,
  nodeState: node.state,
  failureType: node.failureType,
  failureDetailType: node.failureDetailType,
}, null, 2));
NODE
```

Expected output fields are:

```json
{
  "status": 200,
  "blockCounts": {
    "thinking": 1,
    "tool_use": 1
  },
  "stopReasons": [
    "end_turn"
  ],
  "outputTokens": 51468,
  "nodeState": "spent-unusable",
  "failureType": "schema-mismatch",
  "failureDetailType": "unparseable-json"
}
```

### Clean patterns

- Candidate I used the same Qwen route,
  image,
  forced tool name,
  and response-format digest.
  Its author exchange ended with `end_turn`,
  parsed successfully,
  and produced a mechanically admitted candidate.
  This is a positive control for route and schema capability only;
  it does not make the Candidate K failure anomalous or identify its provider-side cause.
- Candidate I used the same forced author tool and schema with MiniMax M3.
  That exchange ended with `tool_use`,
  parsed successfully,
  and produced a mechanically admitted candidate.

These successful patterns show that neither the tool schema nor `end_turn` alone predicts malformed JSON.

### Failing patterns

- Candidate K Qwen author:
  terminal HTTP 200 with thinking plus tool use,
  `end_turn`,
  and unparseable accumulated tool input.
- Candidate K GLM and MiniMax authors:
  terminal HTTP 200 with `max_tokens` and no admissible candidate.
  Those truncations are covered by
  [Charm Hyper Anthropic Messages `max_tokens`](charm-hyper-max-tokens-tool-json.md).

## Verified workarounds

### Fail closed at JSON admission

Keep unparseable tool input as `spent-unusable`,
persist only noncontent diagnostics and digests,
and skip dependent verifier nodes.

Tradeoff:
a provider formatting defect removes that candidate even if recoverable meaning exists in the malformed stream.
This preserves atomic candidates and prevents guessed repair.

### Keep complete candidate production separate from audit plans

For a successor calibration,
give authors only the source,
archive wording evidence,
images,
and exact mutable output slots.
Keep readable review plans in verifier requests rather than making producers reason over audit bookkeeping.

Tradeoff:
this reduces producer instruction load but does not guarantee valid JSON or semantic quality.
Verifier coverage remains a separate gate.

### Retain only metadata needed to classify the boundary

Persist status,
block kinds,
stop reasons,
counts,
durations,
request and response digests,
and consumer failure category.

Tradeoff:
privacy is preserved,
but a malformed byte or adapter rewrite cannot be diagnosed after the response is discarded.
A future public synthetic reproduction would be needed for provider-level attribution.

## What does not work

- Treating forced tool choice as strict JSON enforcement:
  Candidate K received a tool-use block whose accumulated input did not parse.
- Treating HTTP 200 and `message_stop` as candidate completeness:
  both transport predicates passed.
- Reusing the same output schema as proof of repeatability:
  Candidate I succeeded with response-format digest
  `62c24c97df4f769830b8a62703db52aaf861ec068b8eda455c349c83badecd7c`,
  while Candidate K failed with the same digest.
- Extending only the deadline:
  the Qwen exchange completed before 900,000 milliseconds and still failed JSON admission.
- Retrying the prompt:
  it would violate zero retry,
prompt uniqueness,
and spent-node restart semantics.

## Upstream filing decision

No `.out-of-scope/` entry matches Hyper or Anthropic Messages.
A web search found Hyper documentation and the public `charmbracelet/pi-hyper-provider` client integration,
but these searches identified no public Hyper service repository or service issue tracker.
Correctly formed open and closed GitHub issue searches on 2026-09-01 found no matches for
`hyper.charm.land` plus `tool_use`,
Anthropic Messages plus `end_turn` and `tool_use`,
or `tool_use end_turn` and `unparseable JSON` in the client repository.
The client repository is not the service source.
No raw response may be published,
and the private corpus prompt must not be replayed.

1.  Upstream fault:
    **unresolved**.
    The observed shape differs from documented Anthropic tool-use behavior,
    but available evidence cannot distinguish routed-model output from adapter handling.
2.  Upstream can fix:
    **unknown** without service source or an attributable public reproduction.
3.  Supported use case:
    **yes** at the API level.
    Hyper advertises Anthropic Messages compatibility and accepts standard Anthropic parameters.
4.  Contribution welcome:
    **unknown**.
    No public service source or contribution policy was found.
5.  Likely fix:
    **unknown**.
    No public tracker exists to show maintainer direction.
6.  Minimal compatible prototype:
    **no upstream prototype is possible** without source and an attributable reproduction.
    The consumer fail-closed behavior is already verified.

### Upstream filing artifact

Nothing to file or comment upstream.
The retained evidence is deliberately insufficient for a public attribution,
and no public service tracker was identified.
