# Pi 0.85.1 Advisor: exhausted-credit responses need call-local provider exclusion

Note 2026-11-09T16:51:44-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/advisor/src/advisor-completion.ts` is now
`package/agent-harness-shared/advisor/src/advisor-completion.ts`,
moved 2026-10-20 in `093f32832a`.
`package/pi-plugin/advisor/src/operation-attempt.ts` is now
`package/agent-harness-shared/advisor/src/operation-attempt.ts`,
moved 2026-10-20 in `093f32832a`.

## Symptom

During the #413 design interview,
 the user reported this Advisor error:

```text
advisor: provider call failed for hyper/deepseek-v4-flash-0731 on attempt 1:
402: {"message":"You're out of credits. Add more at https://hyper.charm.land","type":"billing_error","code":null}
```

The requested correction is just-in-time provider exclusion within the current Advisor operation.
Do not preflight credit balances,
 persist the exclusion into another call,
 or silently switch an explicitly requested model.
The interview decisions are recorded in
 `doc/handover/pi-advisor-413-grilling.md`.
The user confirmed the design and authorized implementation.
The resolution and verification evidence are recorded in the Resolution section.

## Root cause and integration boundary

### The provider adapter returns a failed message rather than the original HTTP error

Installed `@charmland/pi-hyper-provider@0.3.2` registers Pi's OpenAI-compatible adapter.
Its `src/index.ts:112` starts the provider registration,
 and `src/index.ts:138` selects:

```typescript
api: openAICompletionsApi(),
```

The installed `src/index.ts` matched the read-only clone at commit
 `ac3ed634636b9e8eddad3e02e358943a5a829737` by `diff --brief`.

Pi source references in this document use tag `v0.85.1`,
 commit `d981de1229ef899957bbe968bc8dcda02a21f477`.
In `packages/ai/src/api/openai-completions.ts:708`,
 the adapter records the terminal state and a formatted string:

```typescript
output.stopReason = options?.signal?.aborted ? "aborted" : "error";
output.errorMessage = formatProviderError(normalizeProviderError(error));
```

`packages/ai/src/utils/error-body.ts:128` formats the normalized HTTP status and body:

```typescript
return prefix !== undefined ? `${prefix} (${norm.status}): ${norm.body}` : `${norm.status}: ${norm.body}`;
```

The surrounding function preserves an existing message when it already carries the body.
Consequently,
 the classifier must tolerate supported terminal-message forms rather than assume an SDK error object survives.

### The response callback is not the rejection boundary

In `packages/ai/src/api/openai-completions.ts:366`,
 the adapter awaits the SDK request before invoking `onResponse` at line 374:

```typescript
const { data: openaiStream, response } = await retryProviderRequest(
  () => client.chat.completions.create(params, requestOptions).withResponse(),
  // retry options omitted from excerpt
);
await options?.onResponse?.({ status: response.status, headers: headersToRecord(response.headers) }, model);
```

The fixture confirms that HTTP 402 does not reach this callback on this adapter.
Classification based only on `onResponse` would miss the user-reported response.
This observation does not describe every Pi adapter or transport.

### The retained exact-model helper throws terminal provider errors

`package/pi-plugin/advisor/src/advisor-completion.ts:246` handles a provider error as follows:

```typescript
if (response.stopReason === 'error') {
  throw new AdvisorCompletionError(
    `advisor: provider call failed for ${modelSlug} on attempt ${String(attempt,)}: ${responseFailureText(response,)}`,
  );
}
```

`package/pi-plugin/advisor/src/advisor-completion.ts` moved on 2026-10-20 to
`package/agent-harness-shared/advisor/src/advisor-completion.ts` in `093f32832a`
(refactor(agent-harness-shared-advisor):
move the advisor core files).
`package/pi-plugin/advisor/src/operation-attempt.ts` moved on 2026-10-20 to
`package/agent-harness-shared/advisor/src/operation-attempt.ts` in `093f32832a`
(refactor(agent-harness-shared-advisor):
move the advisor core files).

Tool and slash-command operations now use the raw-response observer instead.
`package/pi-plugin/advisor/src/operation-attempt.ts:196` applies the credit-specific classification:

```typescript
if ((response.stopReason === 'error') && isAdvisorCreditExhaustion(diagnostic,))
  ledger.blockProvider(candidate.provider,);
```

The operation ledger retains the provider identity and exclusion.
The final `onDispatch` gate rereads that exclusion after authentication,
 before another provider request can start.
Successful review text is never used to infer exhausted credits.

## Verification

The installed adapter was exercised without network requests or real credentials,
 using an injected `fetch` and synthetic SSE or HTTP-error responses.
Installed versions were `@earendil-works/pi-ai@0.85.1` and `openai@6.40.0`.

Run the following from the repository root:

```bash
# doc/troubleshooting/pi-advisor-provider-billing.md
node --input-type=module <<'JS'
import { streamSimple } from './package/pi-plugin/advisor/node_modules/@earendil-works/pi-ai/dist/api/openai-completions.js';
const model = {
  id: 'fixture', name: 'Fixture', provider: 'fixture-provider', api: 'openai-completions',
  baseUrl: 'https://example.invalid/v1', reasoning: false, input: ['text'],
  cost: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 }, contextWindow: 4096, maxTokens: 256,
};
const usage = { prompt_tokens: 7, completion_tokens: 2, total_tokens: 9 };
const content = { choices: [{ index: 0, delta: { content: 'fixture review' }, finish_reason: null }], usage };
const terminal = { choices: [{ index: 0, delta: {}, finish_reason: 'stop' }] };
const sse = `data: ${JSON.stringify(content)}\n\ndata: ${JSON.stringify(terminal)}\n\ndata: [DONE]\n\n`;
const exhausted = { message: "You're out of credits. Add more at https://hyper.charm.land", type: 'billing_error', code: null };
const cases = [
  { name: 'success', status: 200, body: sse },
  { name: 'credit-exhaustion', status: 402, body: JSON.stringify({ error: exhausted }) },
  { name: 'unrelated-402', status: 402, body: JSON.stringify({ error: { message: 'payment authorization denied', type: 'payment_error' } }) },
  { name: 'bare-body-control', status: 402, body: JSON.stringify(exhausted) },
  { name: 'abort-after-usage', status: 200, body: sse, abort: true },
];
for (const test of cases) {
  const controller = new AbortController();
  const callbacks = [];
  const requests = [];
  const stream = streamSimple(model, { messages: [{ role: 'user', content: 'fixture', timestamp: 0 }] }, {
    apiKey: 'fixture-only', maxRetries: 0, timeoutMs: 2000,
    signal: AbortSignal.any([controller.signal, AbortSignal.timeout(2000)]),
    fetch: async () => {
      requests.push(1);
      return new Response(test.body, { status: test.status, headers: {
        'content-type': test.status === 200 ? 'text/event-stream' : 'application/json',
      } });
    },
    onResponse: (response) => { callbacks.push(response.status); },
  });
  for await (const event of stream) {
    if (test.abort && event.type === 'text_delta' && event.partial.usage.input === 7) controller.abort();
  }
  const result = await stream.result();
  console.log(JSON.stringify({ name: test.name, requests: requests.length, callbacks,
    stopReason: result.stopReason, errorMessage: result.errorMessage, usage: result.usage }));
}
JS
```

### Successful controls

- The successful SSE response returns `stop` with input `7`,
   output `2`,
   and total `9` tokens.
  `onResponse` observes `200`.
- Aborting after the usage-bearing text event returns `aborted`
   and retains the same available usage.
  This proves retention only for usage received before cancellation,
   not completeness of provider billing.

### Failure controls

- Wrapped exhausted-credit JSON returns `error` and reproduces the reported
   `402: {"message":"You're out of credits...","type":"billing_error",...}` shape.
  The response callback list is empty.
- An unrelated HTTP 402 returns `payment_error`,
   not credit-exhaustion evidence.
  The response callback list is empty.
- Bare JSON without the OpenAI-style `error` wrapper returns
   `402 status code (no body)` in this fixture.
  This does not establish the raw network envelope of the user's request.

Each fixture made exactly one injected fetch call with retries explicitly disabled.
The private exploratory harness also checked adapter event sequences.
No live Hyper balance or production request was queried.

## Verified integration approach

Read terminal `errorMessage` at the provider-completion boundary.
The wrapped fixture preserves the exhausted-credit message and billing type needed for call-local classification.
Use provider identity from the selected model,
 not a string inside the provider's body.

Tradeoff:
 classification depends on supported diagnostic forms until the adapter exposes structured error identity.
An unrecognized error stays an ordinary provider failure;
 do not claim it proves exhausted credits.
The operation block and routing tests are implemented.
The resolution deliberately remains call-local and does not add persistent provider health or balance preflights.

## What does not work

- Relying only on `onResponse` misses the tested HTTP 402 failures.
- Treating every HTTP 402 as exhausted credits conflates the positive fixture with the unrelated-payment control.
- Treating the displayed error body as the full raw HTTP envelope made the first probe lose the body.
  Adding the OpenAI error wrapper reproduced the user's displayed diagnostic.
- Canceling a request does not establish its final billed usage.
  The abort fixture proves only that already-received usage can survive.
- A global cooldown or eager balance query does not implement the user's call-local,
   just-in-time requirement.

## Resolution

The implementation adds call-local provider exclusion at candidate selection
 and again immediately before provider dispatch.
Default calls can recover serially through other eligible scoped models without enabling overlap.
Explicit requests remain exact.
Already-running calls can still contribute usable reviews;
 one failed request does not retroactively discard their results.

The full Advisor unit suite passed,
 including getter-backed scope changes during authentication and the reported billing-error shape.
The guarded `verify:host` task exercised the built extension in real Pi hosts with disposable homes
 and only faux providers.
Serial credit recovery,
 explicit-model failure,
 collected overlap,
 straggler cancellation,
 and successful and failed slash-command persistence passed.
Host assertions checked top-level tool usage,
 per-attempt accounting,
 metadata-only progress,
 and empty child stderr.

Guard controls ran in a separate disposable worktree:
 removing the final provider gate allowed an additional fixture `hyper` dispatch
 and failed the regression;
 removing candidate filtering selected a blocked model;
 removing the live-scope gate failed scope-change regressions.
Restoring the guards and rebuilding made the affected suites pass.
The main-worktree credit suite remained green while the separate mutated artifact existed.
No real provider was called by these controls.

## Upstream filing decision

The repository's `.out-of-scope/` inventory was checked.
`pi-gpt55-long-context.md` concerns context metadata and does not cover this behavior.

Searches for `billing error` in open and closed Pi issues and pull requests found
 [Pi issue #7234](https://github.com/earendil-works/pi/issues/7234)
 about preserving structured HTTP status and error kind,
 and [Pi issue #6025](https://github.com/earendil-works/pi/issues/6025)
 about another provider's credit-exhaustion classification.
Both threads and comments were read.
The first search invocation used an incorrectly quoted repository qualifier;
 rerunning with `--repo earendil-works/pi` produced the cited results.

1. **Upstream fault:**
    The requested call-local Advisor routing policy is our extension's responsibility.
   The tested adapter preserves the reported diagnostic;
    no upstream fix is required to implement this request.
2. **Upstream fixability:**
    Structured error fields could improve integration,
    but this task does not depend on them.
3. **Supported use case:**
    Pi documents custom providers,
    nested tools,
    cancellation signals,
    and top-level nested usage in its extension documentation.
4. **Contribution policy:**
    Pi's `CONTRIBUTING.md` requires human-voiced issue prose
    or clearly AI-labelled follow-up comments and prior approval for pull requests.
   This does not authorize an upstream filing from this interview.
5. **Maintainer direction:**
    The related threads contain automated new-contributor closure comments,
    not a technical rejection of this task's consumer-side solution.
6. **Prototype:**
    The network-free consumer probe verifies the relevant contract.
   No upstream patch is needed because the fault criterion does not hold for the requested routing policy.

Nothing additive is being filed upstream.
The structured-error concern already has an issue;
 this task proceeds at the Advisor boundary.
