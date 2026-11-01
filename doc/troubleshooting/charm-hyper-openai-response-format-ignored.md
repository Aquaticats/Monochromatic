# Charm Hyper 2026-08-31 Chat Completions accepts `response_format` but returns nonconforming Kimi content

## Symptom

`POST https://hyper.charm.land/v1/chat/completions` returns HTTP 200 and terminal `stop` after receiving
`response_format.type: "json_schema"`,
but caller cannot trust response as schema-conformant.

Equal-depth one-attempt probes produced:

- `kimi-k2.6`:
  140 completion tokens and 366 content characters that did not parse as JSON;
- `kimi-k2.7-code`:
  139 completion tokens and 364 content characters that did not parse as JSON;
- `kimi-k3`:
  parseable JSON after 5,853 completion tokens and 17,139 reasoning characters,
  but exact caller guard failed.

All carried one synthetic image,
the same strict schema,
and no thinking,
effort,
temperature,
or retry parameter.
All three responses were unusable despite HTTP success.

## Root cause

Hyper documentation says Chat Completions is OpenAI-compatible and all standard parameters are accepted:

```text
https://hyper.charm.land/docs/api/openai-chat-completions.html
OpenAI-compatible chat completions. All standard parameters are accepted
(temperature, max_tokens, tools, top_p, stop, etc.).
```

That statement covers request acceptance but does not establish server-side schema enforcement.
The page lists common fields without documenting `response_format` behavior.

Repository transport investigation had already measured decisive gateway behavior.
`package/translation-repair/shared-contract/src/hyper-catalog.ts:21-25` records:

```ts
// SCHEMA'D OUTPUT ONLY WORKS OVER ANTHROPIC MESSAGES HERE. The OpenAI Chat
// Completions endpoint accepts `response_format` and ignores it: a deliberately
// invalid `{type: 'not_a_real_mode_xyz'}` returns 200, an unknown top-level
// field returns 200, and every mode answers with markdown-fenced JSON carrying
// invented keys. Nothing in this file supports that endpoint.
```

`package/translation-repair/shared-contract/src/hyper-catalog.ts:35-42` binds structured output to Anthropic route:

```ts
/**
 * Endpoint one call is POSTed to, measured live on 2026-08-24.
 *
 * THE MESSAGES API RATHER THAN CHAT COMPLETIONS, for the reason the module
 * note records: the OpenAI-shaped endpoint accepts `response_format` and
 * ignores it, so structured output has nowhere else to go.
 */
export const HYPER_MESSAGES_URL = 'https://hyper.charm.land/v1/messages';
```

The 2026-08-31 Kimi probe independently reproduced this contract gap.
HTTP success proved route acceptance only.
K2.6 and K2.7 Code did not supply standalone JSON;
K3 supplied JSON whose values failed caller semantics.
Provider-side `strict: true` therefore had no admission authority at this route.

An earlier Candidate I plan was wrong to infer Hyper enforcement from Moonshot's model-side structured-output documentation.
Model capability and gateway request enforcement are separate layers.
The repository's measured gateway source was deciding evidence and should have stopped that route before spend.

Hyper service source is not public.
No source clone can trace internal request forwarding or schema compilation.
Observed route behavior,
provider documentation,
and repository transport measurements are available boundary evidence.

## Verification

- Access date:
  2026-08-31.
- Hyper catalog digest:
  `d68a71501ffa7c67825467ef935c177ad0abb01560f81f17c3fe65fe88742ab7`.
- Probe script:
  `~/temp/agent/probe-hyper-kimi-candidate-i-json-schema-20260831.mjs`.
- Script SHA-256:
  `223f6cb6d17d39af6ff5e40b8e715d7a4a037e82a5656f138bde833e39d5795c`.
- Metadata-only summary:
  `~/temp/agent/probe-hyper-kimi-candidate-i-json-schema-20260831/summary.json`.
- Summary SHA-256:
  `f2fd14972da243503e7078197965ecd87d7ee98715d5549adad783bff0cc6ee1`.
- Attempt count:
  exactly three,
  one per model,
  with no retry.

Run from worktree carrying Hyper credential injection:

```bash
cd -- ~/temp/agent/translation-repair-finite-prototypes
node ~/temp/agent/probe-hyper-kimi-candidate-i-json-schema-20260831.mjs
```

The runner refuses an existing output directory.
Use a fresh dated copy only when testing a genuinely different canonical substantive prompt;
do not replay spent prompt and model pairs.

### Clean patterns

Hyper Anthropic Messages with forced tool choice is repository's supported structured-output route.
`package/translation-repair/shared-contract/src/hyper-catalog.ts:138-148` records twenty schema-conformant forced-tool
attempts for each
then-current allowlisted model.
Caller validation remains mandatory.

A Kimi K3 Anthropic probe with one image,
forced tool,
and low effort returned valid tool input.
That proves exact field combination acceptance only;
it is not usable production workaround because owner policy forbids effort parameter.

### Failing patterns

Same strict OpenAI JSON Schema and synthetic image failed exact caller admission for:

- `kimi-k2.6`:
  non-JSON content;
- `kimi-k2.7-code`:
  non-JSON content;
- `kimi-k3`:
  JSON syntax passed,
  exact semantics failed.

A 200 status,
`stop` finish reason,
and `strict: true` request were present in every failing case.

## Verified workarounds

### Use Anthropic Messages forced tools for vetted Hyper models

Send structured work through `POST /v1/messages` with forced tool and validate raw tool input for duplicate members,
shape,
binding,
and semantics.
This is current package boundary and has historical 20-of-20 conformance evidence on allowlisted roster.

Tradeoff:
model must support forced tools on Anthropic route and fit its output ceiling.
This does not make K2.6 or K2.7 Code eligible because those exact ids have not passed owner allowlist and route validation.

### Treat OpenAI Chat Completions as unstructured text

Use route only where free-form assistant text is contract and no schema enforcement claim is made.

Tradeoff:
caller must not infer JSON admission or publication eligibility from `response_format`,
HTTP status,
or finish reason.
This is unsuitable for Candidate I ballots.

### Keep deterministic caller validation

Reject every malformed,
truncated,
raw-duplicate,
stale,
or semantically invalid response without repair or retry.
Record only privacy-safe diagnostics and digests.

Tradeoff:
valid-looking provider responses can abstain,
so architecture needs enough independent finite evidence without relying on route repair.

## What does not work

### Trusting HTTP 200

HTTP 200 means request reached accepted route.
It does not prove `response_format` was forwarded or enforced.

### Trusting model-vendor structured-output documentation

Moonshot documents Kimi model support,
but Hyper gateway behavior is separate.
All three Kimi ids failed on Hyper route under test.

### Adding `strict: true`

The probe already used strict JSON Schema.
K2.6 and K2.7 Code still returned non-JSON content;
K3 still failed caller semantics.

### Parsing markdown fences as schema success

Repository's earlier route probe observed fenced JSON with invented keys.
Removing fences would not establish schema or semantic compliance and would weaken raw duplicate-member boundary.

### Retrying same prompt

Retry would violate one-model and one-canonical-substantive-prompt uniqueness,
spend indeterminate work again,
and turn finite abstention into correction loop.
All three probe attempts remain spent evidence.

## Upstream filing decision

No `.out-of-scope/` entry matches Charm Hyper or this route behavior.
Web search for a Charm Hyper `response_format` issue found unrelated OpenAI-compatible servers but no public Hyper tracker or
repository.

1.  Upstream fault:
    unresolved.
    Behavior is potential OpenAI-interoperability gap,
    but Hyper's field table does not explicitly claim `response_format` support.
    Caller guard correctly rejects content.
2.  Upstream can fix:
    probably,
    by rejecting unsupported parameter or forwarding and enforcing schema,
    but service source is unavailable.
3.  Supported use case:
    ambiguous.
    Hyper says all standard parameters are accepted and links full OpenAI specification,
    but its field table does not name `response_format`.
4.  Contribution welcome:
    unknown.
    No public service source,
    contribution policy,
    issue template,
    or tracker was found.
5.  Likely fix:
    unknown.
    No public maintainer response or matching tracker history was found.
6.  Minimal compatible prototype:
    no.
    Provider implementation is unavailable,
    so consumer cannot produce source-compatible patch.

Constraints three through six do not all pass.
Default is not to file.

### Draft issue, do not file as-is

~~~md
Title: Chat Completions accepts `response_format` without enforcing JSON Schema

On 2026-08-31,
`POST /v1/chat/completions` returned HTTP 200 and `finish_reason: "stop"` for strict JSON Schema requests to three Kimi
models,
but none passed caller schema semantics:

- `kimi-k2.6` and `kimi-k2.7-code` returned content that did not parse as JSON;
- `kimi-k3` returned JSON but violated required exact values.

All requests used one synthetic image,
model-default reasoning,
and no retry.
The same route had previously accepted deliberately invalid `response_format.type` and unknown top-level fields with HTTP
200.

Could unsupported `response_format` requests return 400,
or could the route forward and enforce JSON Schema before returning 200?

Expected:
strict JSON Schema request either yields schema-conformant content or explicit unsupported-parameter error.

Observed:
HTTP success and terminal stop with content rejected by caller guard.
~~~
