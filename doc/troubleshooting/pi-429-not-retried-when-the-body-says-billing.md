# `@earendil-works/pi-coding-agent` 1.5.2 never retries a 429 whose body carries the word `billing`, so a Neuralwatt concurrency limit ends the run

## Symptom

A Pi run started through `package/pi-shared/subagent-driver` ends with `"endReason": "model-error"` in
`summary.json` after one assistant message failed, and the same summary reports `"retries": 0` and
`"modelErrorAttempts": 1`: Pi scheduled no retry at all, so the 90-second-to-hours of work in that
session is left where it stopped.

The recorded failure, from `${HOME}/temp/agent/pi-runs/release-wait-reports-a-passed-run-as-passed-1/summary.json`
(field `modelError`, 2026-11-11T01:34Z):

```text
429: {"type":"rate_limit_error","code":"concurrent_budget_exceeded","message":"Concurrent limit reached for
deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. Your subscription is
in overage with auto-renew off, so pay-as-you-go limits apply until it renews. Turn auto-renew back on to restore
your account level's limits: https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off",
"retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,
"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,...}
```

The same body says `"retryable":true` and tells the client to wait and retry, and the run still did not.
Sixteen runs ended this way between 01:19Z and 02:00Z on 2026-11-11; over the whole history of
`${HOME}/temp/agent/pi-runs`, measured on 2026-11-11T03:10Z, 114 runs ended `model-error` on
`concurrent_budget_exceeded` and every one of them reports `"retries": 0`.

Synthetic's 429 has the same shape and the same cause, and its body is quoted in the root cause
below. Six runs ended on it over the same history, each also with `"retries": 0`. What differs is
only the answer the driver owes it: a concurrency limit passes when another run finishes, and a spent
quota does not pass by waiting on the same provider at all.

## Root cause

Pi classifies a failed assistant message before it retries it. The test a message must pass to be
retried at all is `isRetryableAssistantError`, and it first excludes everything that matches
`NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN`:

`node_modules/@earendil-works/pi-ai/dist/utils/retry.js:208-215`, installed from
`@earendil-works/pi-ai@1.5.2`:

```text
sha512-2tWPSnV3qzfiDIY3aWluBag3vC++Vs+VsXt/UquSU5onPMohPih72FUhIhaqmDdMYubOgB3Byl9uF4Vk40JLvQ==
```

```js
export function isRetryableAssistantError(message) {
    if (message.stopReason !== "error" || !message.errorMessage)
        return false;
    const errorMessage = message.errorMessage;
    if (NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN.test(errorMessage))
        return false;
    return RETRYABLE_PROVIDER_ERROR_PATTERN.test(errorMessage);
}
```

That pattern is an alternation of substrings, tested against the whole error text with no word
boundary and no URL awareness. One alternative is the bare word `billing`:

`node_modules/@earendil-works/pi-ai/dist/utils/retry.js:4,17-18`:

```js
const NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN = buildProviderErrorPattern([
    ...
    "quota exceeded",
    "billing",
```

The Neuralwatt body carries `billing` inside a documentation URL
(`https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off`), so the pattern
matches, `isRetryableAssistantError` answers `false`, and the retry machinery is never entered. The
provider's own `"retryable":true` is not read by Pi at all: nothing in `retry.js` parses the body as
JSON.

The caller consults that function first and only then asks the retry policy, so no retry setting can
compensate. `node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.js:1604`:

```js
if (this._isRetryableError(message) && (await this._prepareRetry(message))) {
```

and `agent-session.js:3142-3147`:

```js
    _isRetryableError(message) {
        // Context overflow is handled by compaction, not retry.
        if (isContextOverflow(message, (this._modelForMessage(message) ?? this.model)?.contextWindow ?? 0))
            return false;
        return isRetryableAssistantError(message);
    }
```

`_prepareRetry` (`agent-session.js:3192-3202`) is where `retry.maxRetries` is read
(`if (this._retryAttempt > settings.maxRetries)`), and it is unreachable for this message. The other
entrance is `_willRetryAfterAgentEnd` (`agent-session.js:858-872`), which checks the same function at
line 868 after its own budget check, so it is unreachable too.

The second recorded body, Synthetic's 429, carries `billing` the same way, in
`https://synthetic.new/billing`:

```text
429 "You've exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at
https://synthetic.new/billing"
```

So Pi never retried that one either. That the quota it names is spent is a separate fact the driver
acts on: no wait on Synthetic refills it, so the run moves to another provider instead of retrying.

### The settings this was first blamed on

`${HOME}/.pi/agent/settings.json` holds `retry.enabled` true, `retry.baseDelayMs` 1000 and
`retry.provider.maxRetryDelayMs` 0 (read with `jq '.retry'`). None of them can change the outcome:

- `retry.enabled` and `retry.maxRetries` are read inside `_prepareRetry`, which the classifier's
  `false` never reaches.
- `retry.baseDelayMs` and `retry.maxAgentDelayMs` only shape a delay after a retry is scheduled
  (`retryDelayMs`, `pi-ai/dist/utils/retry.js:114-118`).
- `retry.provider.*` belongs to the provider-SDK layer, `retryProviderRequest`
  (`pi-ai/dist/utils/provider-retry.js:76-96`), whose `maxRetries` defaults to 0 and which retries
  HTTP statuses rather than assistant-message texts.
- The settings schema has no key that narrows or extends
  `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN`
  (`pi-coding-agent/dist/core/settings-schema.js:32-53`).

So this is not a settings problem and cannot be fixed by editing settings.

## Verification

Under test: `@earendil-works/pi-coding-agent` 1.5.2 and `@earendil-works/pi-ai` 1.5.2, as resolved
by `pnpm-lock.yaml:9100-9111`:

```text
'@earendil-works/pi-ai@1.5.2':
  resolution: {integrity: sha512-2tWPSnV3qzfiDIY3aWluBag3vC++Vs+VsXt/UquSU5onPMohPih72FUhIhaqmDdMYubOgB3Byl9uF4Vk40JLvQ==, ...}
'@earendil-works/pi-coding-agent@1.5.2':
  resolution: {integrity: sha512-mUWI04rqF1Vd/m1q+Ps1XeWsTXEVvRUnsmRa83I3Dzua0Tx5bYS93imeqtIfrXf7PkkaR6ERz+AuQhIJlom5yA==, ...}
```

No model provider is called. The harness imports the installed classifier and hands it the recorded
texts, which is the whole decision Pi makes:

```ts
// ${HOME}/temp/agent/pi-429/classify.ts
import { isRetryableAssistantError } from
  '/…/package/agent-harness-shared/advisor/node_modules/@earendil-works/pi-ai/dist/utils/retry.js';

const synthetic =
  `429 "You've exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"`;

const transient = '503 {"error":{"message":"Service Unavailable"}}';
const plainThrottle = '429 Too many requests';

function probe (label: string, message: string) {
  const withoutUrl = message.replace(/https?:\/\/\S+/gu, '<url>');
  console.log(label, '| as-is:', isRetryableAssistantError({ stopReason: 'error', errorMessage: message }),
    '| without url:', isRetryableAssistantError({ stopReason: 'error', errorMessage: withoutUrl }));
}
probe('neuralwatt concurrency 429', neuralwatt);
probe('synthetic quota 429    ', synthetic);
probe('plain 429              ', plainThrottle);
probe('503 server error       ', transient);
```

Output, 2026-11-11T03:11Z:

```text
neuralwatt concurrency 429 | as-is: false | without url: true
synthetic quota 429     | as-is: false | without url: true
plain 429               | as-is: true | without url: true
503 server error        | as-is: true | without url: true
```

Catalogs:

- Fails, so no retry: the two recorded 429 bodies above, one from each provider, since each carries a
  `/billing` URL; and any other text carrying `billing`, `quota exceeded`, `insufficient_quota`,
  `out of budget`, `GoUsageLimitError`, `FreeUsageLimitError`, `Monthly usage limit reached`,
  `available balance`, `subscription_sharing_usage_limit_exceeded`,
  `subscription_sharing_workplace_individual_usage_limit_exceeded` or `"code":1308`.
- Works, so it retries: both recorded bodies with their URLs removed, as the second column shows,
  plus `429 Too many requests` and `503 {"error":{"message":"Service Unavailable"}}`.

The second column is the positive control for the first: the only difference between `false` and
`true` is the URL, so the URL is what decides it, on both providers.

## Verified workarounds

1. Read the failure in the driver and resume the session there. This is what
   `package/pi-shared/subagent-driver` does: `kindOfModelError` in
   `package/pi-shared/subagent-driver/src/retries-of-a-model-error.ts` reads the limit from the
   provider's own words (`"limit_type":"concurrent`, `concurrent limit`, `slots in use`) before the
   quota shapes, waits with the `suggested_initial_delay_s` and `max_delay_s` the body names
   (`src/waits-a-model-error-names.ts`), and sends the continuation prompt to the same Pi, which
   keeps working in the same session. Tradeoff: the classification is the driver's own list of
   phrases, so a provider that names a concurrency limit in wording this list does not hold still
   ends the run, and the driver must be told about that wording.
2. Move the run to another provider when the failure cannot pass. For a spent Synthetic quota the
   driver ends that Pi and starts a new one on
   `--provider=neuralwatt --model=kimi-k3.1-pro-flex --session <the session Pi reported>`
   (`src/switch-provider-on-a-spent-quota.ts`). Tradeoff: the new provider has a different context
   window and price, the session's history stays but its model does not, and a session that used
   provider-specific thinking blocks loses them (`transformMessages` drops a thinking block of
   another model).
3. Restarting the run by hand from its worktree. Tradeoff: it is the work the driver now does
   automatically, it spends a queue position, and the run's feed then shows two runs rather than one.

## What does not work

- Raising `retry.maxRetries`, or any other `retry.*` setting: the classifier answers before the
  policy is read, so the run never reaches it.
- `set_auto_retry` over RPC: it writes `retry.enabled` into the shared `settings.json`
  (`README.md` of the driver package records the measurement), and it is the same setting that
  cannot help.
- Adding `billing`-free wording to the request: the text comes from the provider's response body,
  which the client does not control.
- Treating the body's `"retryable":true` as authoritative: Pi never parses the body as JSON, so that
  field is unread by anything in `pi-ai`.
- Retrying on the same provider after a Synthetic 429: the quota is spent, so each attempt reaches the
  same answer until the quota returns, which no backoff spans.

## Upstream filing decision

Not filed. Default policy, and constraint 6 does not hold.

1. **Is it really upstream's fault?** Yes, behaviour rather than wording: a substring meant to catch
   billing exhaustion is matched inside a documentation URL, which turns a transient limit into a
   permanent one.
2. **Can upstream fix it?** Yes, and cheaply: a word boundary, an exclusion for URLs, or reading the
   provider's `retryable` field would each do it. No architectural limit is involved.
3. **Are they supporting this use case?** Unknown. Pi documents automatic retry and provider
   configuration, and the pattern list's comments show the exclusions were each added for a reported
   case, so a concurrency limit from a new provider is plausibly in scope, but nothing documents that
   specific combination.
4. **Would the repo welcome our contribution?** Not checked for this report: no contribution policy,
   issue template or maintainer statement was read. No ban was found either, because nothing was
   searched.
5. **Will they likely fix it?** Unknown: no upstream tracker, commit history or release delta in this
   path was examined.
6. **Have we prototyped a minimal fix compatible with their architecture?** No. No patch against
   `pi-ai` was written or tested, so the filing gate is not met.

The next investigator who wants to file it should prototype the pattern change against
`pi-ai/dist/utils/retry.js` with the harness above, confirm `false` becomes `true` for the recorded
body and stays `false` for the true quota texts, and then walk constraints 4 and 5 against the
upstream repository.
