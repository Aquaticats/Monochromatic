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
A re-probe after the harness restarted onto 0.2.1-rc.4
 (2026-10-29 04:22 America/New_York)
 still listed the same scoped slugs without `opencode-go/kimi-k3.1-pro`;
 the `@earendil-works/pi-ai` 1.4.0 built-in `opencode-go` catalog holds
 `kimi-k3`, never `kimi-k3.1-pro`, so the restart does not add the id.

### The resolution miss falls through silently

`resolveConfiguredDefaultReviewer` catches the resolution error and returns no entry
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:115-120`).
`applyConfiguredInitialReviewer` then rewrites the selection reason to
 `defaultReviewerModel "..." did not resolve among output-eligible scoped models,
 so the default ranking applies`
 (`package/agent-harness-shared/advisor/src/advisor-selection.ts:210-213`;
 the wording at incident time; the sentence now reads `among scoped models`).
That reason lives only in the operation ledger field `reason`
 and a debug log line.
The rendered summary carries attempts, reviews, and a progress tail,
 never the reason
 (`package/agent-harness-shared/advisor/src/run-advisor.ts:240-260`),
 so the operator sees a reviewer they never chose with no explanation.

Resolved on 2026-10-29:
 `renderAdvisorOperationSummary` now renders the selection-reason line
 whenever the call made a default selection,
 so the fallthrough names itself in the tool output.

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

Resolved on 2026-10-29:
 the agent session id now rides every Advisor request
 through `DshAdvisorAgentSurface.sessionId`,
 the catalog and provider shims,
 into `GenerateOptions.sessionId`
 (`package/dsh-plugin/advisor/src/request-map.ts`),
 so pi-ai attaches `x-opencode-session`.

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

### The output budget doubled as an eligibility floor

`filterAdvisorScopeByOutputCapacity` kept only entries whose
 `model.maxTokens >= maxAdvisorOutputTokens`.
The catalog fills that field from `resolved.defaultMaxTokens`
 (`package/dsh-plugin/advisor/src/catalog.ts:117-124`),
 which the pi-ai adapter populates from the profile's per-model `maxTokens`
 (the web UI value the discovery listing pre-fills from `max_output_tokens`),
 substituting the configured budget when a route declares none.

Correction to the first draft of this section:
 the field holds the declared capacity when a route declares one,
 so the listing disclosures do reach it through the profile.
The defects were the floor semantics and the substitution.
Raising `maxAdvisorOutputTokens` to buy more output shrank the eligible model
 set and refused explicit requests:
 the live profile's `neuralwatt/gemma-4-31b` declares `maxTokens: 16384`
 (dropped under a 32,000 budget),
 while `neuralwatt/kimi-k3` declares none
 (substituted to 32,000, making the gate vacuous for it and its request
 unclamped).

Resolved on 2026-10-29:
 `package/agent-harness-shared/advisor/src/output-eligibility.ts` is retired.
Every scoped model stays selectable,
 and the request clamps to `min(budget, advertised capacity)`
 (`package/agent-harness-shared/advisor/src/advisor-client.ts`);
 routes declaring a non-positive capacity receive the configured budget
 unclamped.

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

Resolved on 2026-10-29:
 exclusion now also matches provider-independent identity
 (model id plus display name),
 and the DSH run context passes the conversation model's display name from the
 catalog.
Ids repeat across unrelated models,
 so identity matching without a display name stays route-exact.

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
  was not persisted for completed operations at incident time.
  Session records store rendered text only,
  and the `plugin:advisor/operation` record
  (name and `advisor/operation` renamed on 2026-10-29)
  persisted only when the whole operation failed,
  so the incident's own failure diagnostics are unrecoverable.
  The 400 cause above was reconstructed by probe on 2026-10-29.
  Completed operations now persist their ledger the same way failures do.
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

## What does not work at incident time

- Raising `maxAdvisorOutputTokens` alone.
  The pool stays shared with reasoning and the eligibility floor rose with the value.
- Naming any `opencode-go` slug.
  The dispatch returned 400 `MissingSessionID` regardless of reviewer.

## Resolution

Landed on 2026-10-29 across `package/agent-harness-shared/advisor` and
 `package/dsh-plugin/advisor`:

- `fix(dsh-plugin-advisor): stamp the session identity onto every Advisor request`,
  so `opencode-go` routes attach `x-opencode-session`.
- `fix(*): clamp the advisor request output budget instead of gating scope on it`;
  `output-eligibility.ts` is retired.
- `fix(agent-harness-shared-advisor): surface the selection reason in the operation summary`.
- `fix(dsh-plugin-advisor): persist the operation ledger for completed operations`.
- `fix(*): avoid the current main model by identity across provider routes`.

Remaining work:

- The response pool is still shared with reasoning at the pinned greatest
  effort below `max`, and the tool schema offers no knob to lower it,
  so broad reviews can still end at `stopReason: "length"`.
- Config: `defaultReviewerModel: opencode-go/kimi-k3.1-pro` still resolves to
  nothing (re-probed 2026-10-29 04:22 after the rc.4 restart).
  The value is kept at user direction pending the id appearing;
  scoped kimi seats today are `neuralwatt/kimi-k3` and
  `synthetic/hf:moonshotai/Kimi-K3`.
