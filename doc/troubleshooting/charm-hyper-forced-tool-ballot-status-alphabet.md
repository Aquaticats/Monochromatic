# Charm Hyper forced-tool ballots can violate the requested status alphabet

## Symptom

The one zero-retry Candidate L Carena calibration sent candidate-scoped verifier requests through Hyper's
Anthropic-compatible `POST /v1/messages` endpoint.
Each request forced one `candidate_review_unit_ballot` tool,
carried complete source,
archive,
candidate,
and image evidence,
and requested one bounded ballot.

Qwen3.8-27B and MiniMax M3 each returned HTTP 200 with a tool-use block.
Both tool inputs parsed as JSON and passed the strict transport schema,
but the caller guard rejected them with
`failureCategory: "status-alphabet"`.
Neither ballot had selection effect.
Raw reviewer wording was intentionally not retained,
so the evidence does not reveal which status field or character differed.

## Root cause

### JSON shape and review semantics are separate boundaries

`package/module/translation-repair/src/prototype-review-unit-schema.ts` (removed before 2026-10-18)
constrains exact object keys,
status-string lengths,
array cardinality,
indexes,
and candidate-binding values.
It does not encode the per-field status alphabet as a JSON Schema pattern.
The substantive prompt requires:

- `p` or `d` for front matter,
  clauses,
  and relations;
- `c` or `d` for language and global statuses.

`package/module/translation-repair/src/prototype-review-unit-guard.ts` (removed before 2026-10-18)
checks those alphabets after JSON parsing.
It returns `status-alphabet` when any character falls outside the field-specific set.

The observed outputs therefore demonstrate semantic instruction failure after syntactic tool success.
They do not demonstrate a JSON parser,
forced-tool,
or Hyper transport defect.

### Live evidence

The calibration used:

- harness SHA-256
  `6d050811e82a3156e1da5a0524b0bdb6a595d27fbfb814296ed854168b85a0fc`;
- manifest digest
  `14e5b100a5bddfc4426b7c0d5dbee89255e90d3a787644e5f115f08c6ed39fd3`;
- Qwen verifier duration 568,639 milliseconds,
  `stop_reason: "end_turn"`,
  thinking,
  text,
  and tool-use blocks;
- MiniMax verifier duration 141,327 milliseconds,
  `stop_reason: "tool_use"`,
  thinking and tool-use blocks.

Hyper's Qwen usage metadata reported 39,834 output tokens despite the request's 32,000-token `max_tokens` value.
The retained metadata cannot determine whether the count includes reasoning under a different accounting rule.
That count is not used to classify the alphabet failure.

The no-network post-audit summary is retained at
`/var/home/user/temp/agent/prototype-Carena-L-lean-realization-20260901/calibration-post-audit-summary.json`
with SHA-256
`55deaafeb67a22a11a0e07ed512efe54e37d306e3ecead39cd303e09e1f3c8c0`.

## Verified workarounds

### Fail closed after transport-schema admission

Keep the runtime caller guard after JSON parsing and schema checks.
Persist only the neutral failure category and response digest,
mark the node spent-unusable,
and exclude it from evidence.

Tradeoff:
no meaning is recovered from an almost-correct ballot.
This preserves atomic evidence and avoids guessing reviewer intent.

### Keep private fallback ineligible

If every verifier abstains,
retain an admitted author candidate only as private evidence with
`evidenceFloorMet: false` and `productionEligible: false`.

Tradeoff:
a readable candidate cannot publish without independent clean evidence.
Complete-page review may still reject it for defects that no valid ballot captured.

## What does not work

- Treating HTTP 200 plus forced tool use as semantic ballot validity.
- Rewriting unexpected status characters into the allowed alphabet.
  The intended verdict cannot be inferred.
- Accepting partial fields from an atomically invalid ballot.
- Retrying either Candidate L verifier prompt.
  Both potentially transmitted prompts are spent.
- Adding a correction loop.
  It violates the statically finite graph and prompt-uniqueness contract.

## Upstream filing decision

Hyper documents Anthropic-compatible forced tools,
but the requests reached those tools and returned parseable objects.
The rejected condition belongs to the consumer's substantive ballot contract,
not a documented provider guarantee.
Raw outputs cannot be published,
and no attributable synthetic reproduction exists.

1.  Upstream fault:
    **not established**.
2.  Upstream can fix:
    **not established**;
    the consumer prompt and semantic guard define the alphabet.
3.  Supported use case:
    **yes at the transport level**.
4.  Contribution welcome:
    **unknown** because Hyper service source is not public.
5.  Likely fix:
    **unknown** and not justified by this private workload.
6.  Minimal compatible prototype:
    **the local fail-closed guard already handles the condition**.

No upstream issue or comment is retained.
