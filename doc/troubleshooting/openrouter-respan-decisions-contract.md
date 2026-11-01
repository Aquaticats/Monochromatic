# OpenRouter Respan Decisions needs a model-specific span and version-aware validation

## Symptom

On 2026-09-28,
`POST https://openrouter.ai/api/alpha/decisions` with `respan/span-01`
rejected a generic Decisions state object with HTTP `400`:

> Respan state must be a string or an object with only input (a message array) and output (a message),
> where each message has a string content and a role of system,
> user,
> assistant or tool,
> and the output role is assistant

A separately frozen span-shaped request returned HTTP `200`,
but the experimental client's exact-alias validator rejected returned model `respan/span-01-20260925`.
The requested alias was `respan/span-01`.
These are separate input-contract and client-validation findings,
not model-quality failures.
They are also distinct from the [direct Respan output-role refusal](respan-span-output-role.md).

## Root cause and evidence boundary

The [Decisions reference][decisions] permits string,
object,
or array state in its general schema.
The observed Respan-specific request restriction is narrower.
Owned `openrouter-pro/pilot-contract.mjs:12` supplied a generic state with `evidence_text`,
`supplied_operation`,
and `loaded_project_context` rather than a span.
No public server implementation was inspected;
the hosted response establishes the observed restriction,
not an internal source trace.

The corrected `openrouter-corrected/pilot-contract.mjs:12` instead uses the native span:

```javascript
// Private canary: openrouter-corrected/pilot-contract.mjs
state: native.span,
```

It maps each existing behavior ID to a Noul question,
using the unchanged behavior definition as `instructions`.
No criteria or reference labels are added.
Preflight compared this span and these definitions with the retained accepted Lite request.

The subsequent client failure originates at `openrouter-pro/pilot-contract.mjs:19`:

```javascript
// Private canary: openrouter-pro/pilot-contract.mjs
const parsed = parseResponse({ value, expectedModel: 'respan/span-01', questionIds: expectedIds });
```

The shared `parseResponse()` rejects any different model string before releasing estimates.
Its check is in private `auto-mode-axiom-fixtures-2026-09-26/response.mjs:11`:

```javascript
// Exact model-comparison expression inside parseResponse()'s rejection condition
value.model !== expectedModel
```

[Public endpoint metadata][endpoints],
read anonymously after the call,
listed model ID `respan/span-01` with endpoint name `Respan | respan/span-01-20260925`.
This corroborates the versioned response label;
it does not independently verify hosted weights or routing internals.

## Verification

Private evidence root is `~/temp/agent/respan-auto-mode-eval-2026-09-28`.

- `d0a0585` and `9ceeb90` retain the initial source and input freeze.
- `dc479ae` retains the HTTP `400` response.
- `477d830` and `8f3ec7e` retain the corrected source and input freeze.
- `016a630` retains the HTTP `200` response and original client rejection.
- `d442ec4` retains anonymous endpoint metadata and offline verification source.
- `8a4e6c5` retains the offline observation summary.

Each live attempt was separately bounded to one call,
five seconds,
and zero retries.
Requests restricted the provider to Respan,
disabled fallback,
requested `data_collection: deny`,
and capped published input/output/request prices.
Only `AUTO_MODE_OPENROUTER_API_KEY` was loaded through scoped root mise.
No dashboard,
account-setting,
credit-top-up,
or paid direct Respan operation occurred.

The corrected request carried the complete 42,677-byte policy,
SHA-256 `4731752e57e66bf587462e86aff22cbae7b4f073cb1f125f965438268e7c064b`.
It totaled 48,397 request bytes.
The response reported 13,151 input tokens,
zero output tokens,
and usage cost `0.00026302` USD.
That is response accounting,
not an independently checked invoice.

The retained response contains only `type` and `noul` for each answer:

- `positive_relation`:
   `0.6700895`.
- `explicit_prohibition`:
   `0.027236922`.

These equal the direct Lite present-probability fields on this one matched span.
The offline comparison detected a deliberately changed scalar as different.
This checks the comparison mechanism,
not provider independence,
caching behavior,
equal weights,
correct tier routing,
or general quality parity.
No absent/not-observable probability is inferred from a Noul scalar.
The router's general mapping remains unestablished.

The original client stopped at `529.101426` ms because of the model-label check.
It did not run its remaining live freshness checks.
A later offline source/policy/body recheck passed,
but cannot retroactively turn that run into a complete five-second assessment.
The original result remains rejected and unchanged.

The executed offline check was:

```bash
# Run from the main repository root; reads retained evidence and makes no model call
mise --no-env --no-hooks exec -- node \
  /home/user/temp/agent/respan-auto-mode-eval-2026-09-28/recheck-pro-result.mjs
```

It writes create-new evidence and is not intended to overwrite or silently rerun an existing receipt.

## Verified workarounds

Use a Respan span as Decisions `state`,
with the selected user source in input and the required assistant output retained as synthetic padding.
The corrected live request verifies this input shape.
The tradeoff is provider-specific state handling rather than a generic Decisions object.

For offline interpretation,
validate the exact versioned model label only after catalogue corroboration.
`recheck-pro-result.mjs:26` fixes the accepted label;
line 28 checks its endpoint association;
line 31 parses with that exact expected model.
The same check rejects an unrelated model label.
This does not install a production alias policy or broaden the frozen live validator.
No further Pro call was made to repair the original record.

## What does not work

- A generic Decisions state object failed for Respan despite fitting the general schema.
- Comparing a versioned response model only with the requested alias rejected the observed response.
- Filling in absent/not-observable mass from the Noul scalar is unsupported.
- Treating equal scalar values on one span as proof that Pro and Lite are the same model is unsupported.
- Treating the original response duration as a completed freshness-checked assessment would overstate the evidence.
- Public `context_length: 0` and `max_prompt_tokens: null` metadata do not establish a zero-token limit
  or a measured context capacity.

## Upstream filing decision

### Span 02 verification, 2026-10-19

The user subsequently requested trying `respan/span-02`.
Current anonymous metadata lists endpoint `Respan | respan/span-02-20261015`.
Eight bounded Decisions calls return HTTP `200`, that exact versioned model label,
and complete categorical distributions.
The `state` is the original Gateway input string;
question instructions remain unchanged and choice descriptions become keyed `criteria`.
This verifies string-state categorical requests for this observed Span 02 endpoint.
It does not establish support for arbitrary object state or revise the historical Span 01 failures.

The owned trial serializer is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/run-span02-trial.ts`:

```javascript
// Private run-span02-trial.ts, request construction
const request = {
  model: manifest.model,
  state: input,
  questions: sourceQuestions,
  provider: manifest.provider,
};
```

Response model validation admits only the requested alias and exact versioned labels in the retained metadata.
The trial checks explicit authorization and prohibition controls,
three unchanged repeats of the recorded escalation,
two recorded approvals and a new prohibition appended to the original conversation.
All routes match the authored expectations.
The [package trial evidence](../../package/pi-plugin/auto-mode/doc/span-02-trial.md)
records distributions, measurements, receipt locations and limits.
The hosted implementation remains uninspected; no server-source cause or general accuracy claim follows.
No new failing pattern or upstream filing arises from this successful trial.

### Historical Span 01 filing decision

No issue,
comment,
or vendor message was sent.
No Respan/OpenRouter-specific entry appeared in the `.out-of-scope/` filename inventory.
The user authorized a bounded evaluation,
not vendor contact or a server-source audit.
No upstream draft or duplicate search was started.

1.  Upstream fault is not established;
    one request used an unsupported model-specific shape and the other failure was in the owned validator.
2.  The general reference could document model-specific restrictions;
    no server implementation change has been assessed.
3.  Respan on the Decisions API is documented,
    but arbitrary object state for this particular model is not explicitly promised.
4.  Contribution policies were not investigated;
    no filing is authorized.
5.  Maintainer willingness is unknown.
6.  The input correction and offline label check are verified client-side work,
    not an upstream patch.

Nothing is fileable from this bounded finding.
No top-up recommendation follows from these observations.

[decisions]: https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request.md
[endpoints]: https://openrouter.ai/api/v1/models/respan/span-01/endpoints
