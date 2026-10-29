# DSH Advisor 2026-10-29: configured reviewer not tried, first-ranked route 400s, review truncated

## Symptom

One `advisor` tool call with no `model` argument reported
 `1 usable review; 2 attempts; complete`
 with `opencode-go/mimo-v2.6-pro #1 failed`,
 `neuralwatt/deepseek-v4-flash #1 succeeded (output limit reached)`,
 and no mention of the configured `defaultReviewerModel`.
The same signature repeats in three operations of
 `session-2edd714c-fb4a-4c6c-a0d6-808b5ee7ca4b`
 on 2026-10-29 at 06:04, 06:15, and 06:17 UTC.

## Root cause

### The configured reviewer names a model no scoped route serves

The merged config `~/.pi/agent/extensions/pi-advisor.json` sets
 `defaultReviewerModel: "opencode-go/kimi-k3.1-pro"`.
A scoped-slug probe lists 44 scoped slugs,
 of which `opencode-go` contributes exactly
 `qwen3.8-max`, `gemini-4-argon-contributor`, `deepseek-v4.1-flash`, `mimo-v2.6-pro`.
That route list is whatever `~/.dsh/profiles/web/cordis.patch.yml` last declared for
 `llm-pi-ai.providers.opencode-go.models`
 (two models at 06:11 UTC on 2026-10-29, edited to the four above during this investigation).
No `kimi-k3.1-pro` id exists in the `@earendil-works/pi-ai` built-in `opencode-go` catalog
 (which holds `kimi-k3` and `kimi-k2.7-code`) or in any scoped route.
The scoped kimi seats are `synthetic/hf:moonshotai/Kimi-K3`,
 `neuralwatt/kimi-k3`, `neuralwatt/kimi-k3-fast`, and `neuralwatt/kimi-k3-flex`.
Resolution needs an exact scoped match under the accepted explicit-model forms,
 so the configured key cannot resolve
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:100-113`).

### The resolution miss falls through silently

`resolveConfiguredDefaultReviewer` catches the resolution error and returns no entry
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:115-120`).
`applyConfiguredInitialReviewer` then rewrites the selection reason to
 `defaultReviewerModel "..." did not resolve among output-eligible scoped models,
 so the default ranking applies`
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:210-213`).
That reason lives only in the operation ledger field `reason`
 and a debug log line.
The rendered summary carries attempts, reviews, and a progress tail,
 never the reason
 (`package/agent-harness-shared/advisor/src/run-advisor.ts:240-260`),
 so the operator sees a reviewer they never chose with no explanation.

### The first-ranked route rejects every Advisor dispatch

Default ranking orders candidates by largest `contextWindow`,
 then name, then slug
 (`package/dsh-plugin/advisor/README.md`,
 `package/dsh-plugin/advisor/src/ranking-policy.ts`).
`opencode-go/mimo-v2.6-pro` advertises 1,048,576
 and `neuralwatt/deepseek-v4-flash` advertises 1,048,560,
 which matches the observed attempt order.
A focused probe against the first-ranked route returns:

```text
Error: advisor: no usable review; opencode-go/mimo-v2.6-pro attempt 1:
400: {"type":"MissingSessionID","message":"Request is missing x-opencode-session
and cannot be routed efficiently. Please see
https://opencode.ai/v2/docs/console/go/#where-can-i-use-it"}
```

`@earendil-works/pi-ai` attaches `x-opencode-session` only when the stream options
 carry `sessionId`
 (`dist/providers/opencode-headers.js`, `withSessionHeader`).
The Advisor request path sets `maxTokens`, `reasoning`, and auth fields
 but never `sessionId`
 (zero matches in `package/agent-harness-shared/advisor/src/advisor-client.ts`,
 `package/dsh-plugin/advisor/src/request-map.ts`,
 `package/dsh-plugin/advisor/src/provider.ts`),
 so every `opencode-go` dispatch under Advisor fails at the endpoint.
Main-agent calls carry the harness session id and succeed on the same route.

### The response budget is shared with reasoning at a pinned effort

The request sets `maxTokens: options.config.maxAdvisorOutputTokens`
 (`package/agent-harness-shared/advisor/src/advisor-client.ts:341`),
 32,000 in the live config.
pi-ai folds reasoning tokens into output usage,
 and the endpoint counts thinking and answer in one pool.
The advisor always dispatches the greatest reasoning effort below `max`
 (`package/agent-harness-shared/advisor/src/advisor-client.ts:270-291`),
 so both attempts ran `reasoning=high` with no tool-level knob to lower it
 (`model` and `question` are the only tool arguments).
A provider `stopReason: "length"` marks the review truncated
 (`package/agent-harness-shared/advisor/src/operation-attempt.ts:204`)
 and renders the `(output limit reached)` marker
 (`package/agent-harness-shared/advisor/src/run-advisor.ts:249`).
The truncated review body measured 2,060 characters,
 while a focused follow-up operation on the same model,
 effort, and budget completed fully at 168,621 context characters.
The measured precedent for this failure class is
 [Pi Advisor 0.0.1 with Pi 0.84.1: Kimi K3 reasoning exhausts the 16K response budget](pi-advisor-reasoning-output-exhaustion.md),
 where `output: 16384, reasoning: 15664, textChars: 3362`.

### The output budget doubles as an eligibility floor

`filterAdvisorScopeByOutputCapacity` keeps only entries whose
 `model.maxTokens >= maxAdvisorOutputTokens`
 (`package/agent-harness-shared/advisor/src/output-eligibility.ts:46-53`).
The catalog fills that field as
 `resolved.defaultMaxTokens ?? maxAdvisorOutputTokens`
 (`package/dsh-plugin/advisor/src/catalog.ts:117-124`).
`@deepseek-ai/dsh-llm` defines `defaultMaxTokens` as
 "Adapter-configured per-request output cap materialized when callers omit one"
 (`lib/types/types.d.ts`),
 which is not the endpoint maximum.
The listing `maxTokens` and `max_output_tokens` disclosures
 (for example neuralwatt `deepseek-v4-flash` at 393,216) are ignored here.
Raising `maxAdvisorOutputTokens` to buy more output
 therefore also shrinks the eligible model set
 and rejects explicit requests for routes whose adapter declares a modest default cap
 (`package/agent-harness-shared/advisor/src/output-eligibility.ts:140-157`).

### Non-current avoidance compares route slugs, not model identity

The session default model is `provider: neuralwatt`, `model: mimo-v2.6-pro`
 (`~/.dsh/profiles/web/cordis.patch.yml`, entry `agent-default-model`),
 and the tool description promises the largest-context-window
 "eligible non-current model".
`scopeAvoidingCurrentMainModel` compares the slug
 `${provider}/${id}`
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:424-431`),
 so `opencode-go/mimo-v2.6-pro` counts as non-current
 while being the same model the session runs.
Avoidance is skipped when `agent.options.provider` is absent
 (`package/dsh-plugin/advisor/src/run-review.ts:171-175`)
 and is a no-op when the current slug is not in scope
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:454-456`).

## Verification

1. Explicit `advisor` request for `opencode-go/mimo-v2.6-pro` with a trivial question
   reproduces the 400 `MissingSessionID` failure above,
   so the historical `#1 failed` line has this cause and not a credential fault
   (the request reached the endpoint and was routed far enough to return 400).
1. Explicit `advisor` request for `opencode-go/kimi-k3` fails before dispatch with
   `model slug "opencode-go/kimi-k3" was not found in scoped models`
   and lists the 44 allowed slugs,
   proving the route list gates the configured reviewer.
1. The session store `session.v4.jsonl.zstd` of the matrix session
   holds the three operations with identical attempt signatures.

## Evidence limits

- The operation ledger (attempt diagnostics, aggregate usage, selection reason)
  is not persisted for completed operations.
  Session records store rendered text only,
  and `advisor/operation` events (record name renamed to `plugin:advisor/operation` on 2026-10-29)
  persist only when the whole operation fails,
  so historical failure diagnostics are unrecoverable.
  The 400 cause above was reconstructed by probe on 2026-10-29.
- The exact reasoning versus text token split of the truncated call is unrecoverable
  for the same reason.
  A 2,060-character body against a 32,000-token pool implies the remainder
  was thinking output, matching the measured precedent,
  but the split itself was not measured.

## Verified workarounds

1. Point `defaultReviewerModel` at a scoped kimi seat,
   `neuralwatt/kimi-k3` or `synthetic/hf:moonshotai/Kimi-K3`.
2. Pass `model` explicitly on each `advisor` call;
   scoped slugs outside `opencode-go` dispatch normally.
3. Narrow the `question` to reduce the thinking draw under the shared pool;
   the focused follow-up completed within the same budget.

## What does not work

- Raising `maxAdvisorOutputTokens` alone.
  The pool stays shared with reasoning and the eligibility floor rises with the value.
- Naming any `opencode-go` slug.
  The dispatch returns 400 `MissingSessionID` regardless of reviewer.

## Resolution and remaining work

- Config: replace `defaultReviewerModel: opencode-go/kimi-k3.1-pro`;
  that id exists in no catalog and no route.
- Plugin: send a `sessionId` on Advisor stream options so `opencode-go` routes
  can attach `x-opencode-session`.
- Plugin: split `maxAdvisorOutputTokens` from any eligibility floor
  and read the endpoint `maxTokens` disclosure for capacity checks.
- Plugin: surface the configured-reviewer fallthrough in the rendered summary,
  not only in the ledger reason and debug log.
- Plugin: persist the operation ledger and usage for completed operations.
- Plugin: treat same-model-different-route as current when avoiding the main model.
