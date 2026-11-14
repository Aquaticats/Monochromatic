# pi-plugin-qwen-token-plan-individual

Pi extension that registers the Qwen Token Plan Individual route as a
first-class pi provider,
replacing the builtin catalog while it is loaded.

It serves the checked-in listing snapshot immediately,
then refreshes the catalog from the bounded live listing fetch at session
start,
projecting each model through the shared listing core so pi sees the same ids,
effort wire values,
and limits the DSH plugin reports.
The builtin provider's static 10-model list stops serving as soon as this
extension registers,
and `unregisterProvider` restores it on removal.

Design,
measurements,
and every rejected alternative:
[`doc/planning/qwen-token-plan-individual-provider.md`](../../../doc/planning/qwen-token-plan-individual-provider.md).
The mapping facts live in
[`package/agent-harness-shared/qwen-token-plan-individual-listing`](../../agent-harness-shared/qwen-token-plan-individual-listing/README.md).

## Pi dependency trust

PJT:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package takes the rule as two Pi peers and no others:
`@earendil-works/pi-ai` and `@earendil-works/pi-coding-agent`,
both at `"*"`.
Every other dependency is a workspace package,
and the provider is hand-built from pi's `Provider` shape instead of pi's
`createProvider`,
which would persist model metadata this plugin refuses to write.

## What it does

- Registers provider `qwen-token-plan-individual` (API `openai-completions`) at
 extension load and lists the projected snapshot right away,
 so the model picker shows the full live set before any network.
- On `session_start` with reason `startup` or `reload`,
 asks pi's model registry to refresh the route with network access;
 every other reason,
 and `allowNetwork: false`,
 leaves the catalog untouched.
- On `session_shutdown`,
 aborts the in-flight refresh and keeps the served catalog.
- Projects each mapped listing entry into pi's `Model`:
 the facts table's context and output limits (pi schema defaults where no
 family covers an id),
 input modalities,
 the zero cost rates the builtin rows carry,
 reasoning as the three recorded states,
 and the efforts dict as pi's `thinkingLevelMap`.
- Pins the OpenAI-compatible dispatch the builtin rows pin:
 `thinkingFormat: "qwen"`,
 `supportsDeveloperRole: false`,
 `supportsStore: false`,
 `supportsStrictMode: true`,
 and `supportsReasoningEffort` true exactly when the model offers efforts.
- Skips the media ids the listing carries (image,
 video,
 TTS,
 ASR,
 realtime),
 which refuse chat completions.
- Hands pi this provider's passing token-rate limit in the one form pi's own
 automatic retry takes,
 so a burst of runs slows down and asks again instead of each ending on its
 provider's first answer
 ([The provider's token-rate limit](#the-providers-token-rate-limit)).

## The provider's token-rate limit

The provider answers a request that outstrips the tokens it allows a second or
 a minute with status `429` and OpenAI's `insufficient_quota` code.
Its own error-code page
([`error-code`](https://www.alibabacloud.com/help/en/model-studio/error-code),
 "429 - Throttling.AllocationQuota/insufficient_quota",
 read 2026-11-13)
gives two messages under that heading,
 each caused by "Token consumption per second or minute (TPS/TPM) triggered
 rate limiting":

- `Allocated quota exceeded, please increase your quota limit.`
- `You exceeded your current quota, please check your plan and billing details.`

So the answer passes by itself,
and the layer that answers a `429` here is pi's
 own automatic retry
 (`package/pi-shared/subagent-driver/README.md`,
"Which layer retries what",
 owner decision of 2026-11-11).
Pi 1.5.2 does not retry it:
`isRetryableAssistantError` in `@earendil-works/pi-ai/dist/utils/retry.js`
 tests `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN` before the list holding
 `429`,
and that first list carries `insufficient_quota` and `quota exceeded`
 for OpenAI's billing answer and for budgets nothing refills inside a turn.
On 2026-11-13 that ended fifteen review runs on `qwen4-max-preview` after one
 attempt each.

This package therefore translates the answer where it hands pi the stream.
Both `stream` and `streamSimple` forward every event as the provider's own
 stream produced it,
 and only the terminal error of that one limit is rebuilt:

- the error text becomes
 `429: token rate limit reached: the token allowance this provider gives a
 second or a minute, which passes by itself. Pi retries it, and the provider's
 own answer is carried in this message's diagnostics and in the log.`,
 which names the status and the limit in words
 `isRetryableAssistantError` accepts and holds none of the words it tests
 first;
- the provider's own answer stays word for word in a diagnostic on the
 message (`type` `qwen-token-plan-individual.passing-token-rate-limit`,
 with its `message` and `details` holding the status and code),
 so it survives in pi's session and in any transcript that carries the
 message,
 and one `warn` record per attempt names it on standard error,
 which is where a run's `stderr.log` keeps it.

What matches is narrow on purpose,
 so every answer the provider documents as spent,
 overdue or unauthenticated ends a run at once:

- the error text has to start with `429: `,
 which is where pi writes the status it received;
- what follows has to parse as a JSON object;
- its `code` or `type` has to be `insufficient_quota`,
 and neither may disagree;
- its `message` has to open with one of the two documented TPS/TPM messages,
 in any letter case and ended at a word boundary,
 so the provider's own trailing details still fit and
 `Free allocated quota exceeded.` never does.

The second documented message,
`You exceeded your current quota, please check your plan and billing details.`,
is also OpenAI's answer for a spent balance,
and this is the one answer here that costs a budget instead of a decision:
if this provider ever sends those words for a plan that is really spent,
the run is retried for the whole of pi's budget
(eight waits,
about three minutes at this machine's settings)
and then still ends `model-error` on it.
No recorded answer on 2026-11-13 carried it,
and the provider's page gives it under the TPS/TPM cause,
which is why it is read as the passing limit.

Everything else passes through exactly as pi received it.
The answers that look alike and stay final:

- `Free allocated quota exceeded.`
 is a spent free quota under the same status and code,
  and its opening word
 keeps it apart from `Allocated quota exceeded, ...`;
- `PrepaidBillOverdue` and `PostpaidBillOverdue` are unsettled bills;
- `AllocationQuota.FreeTierOnly` is a spent free tier,
  which the provider's page lists under its `403` answers;
- a body that is not JSON,
 a body cut short,
  and any status other than `429`
 are answers this package cannot read,
  so they are left as they arrived.

The same `insufficient_quota` code is OpenAI's own spent-credit answer,
and
 that answer stays final:
 this translation lives in this plugin's provider,
 whose `baseUrl` is this route's,
 and OpenAI never reaches this module's matcher with a message this page
 documents.

A person reading a run sees it in two places:

- `feed.jsonl` carries pi's own `auto_retry_start` line before each wait,
 with this package's text as its `errorMessage`,
 then `auto_retry_end` with `success: true` when a later reply worked;
 that run ends `idle` with the reply and `rateLimitErrors` counting the failed
 attempts.
- A limit that outlasts pi's retry budget ends the run `model-error` after
 those attempts:
 `modelError` in `summary.json` then carries this package's text (not the
 provider's,
  which is the one departure from the driver's word-for-word rule),
 `piRetryGaveUp` names how many attempts pi made and its `finalError`,
 and the provider's own words are in the session message's `diagnostics` and
 in the run's `stderr.log`.

This translation is a workaround for a pi release,
not for this provider:
when pi classifies the recorded answer as retryable itself,
it is deleted.
Upstream has the report
 ([issue 10656](https://github.com/earendil-works/pi/issues/10656)) and a pull
 request its contributor gate closed unmerged
 ([#10677](https://github.com/earendil-works/pi/pull/10677)),
 so the fix would arrive in a pi-ai release newer than 1.5.2.
A reader checks it in one step:
`src/passing-token-rate-limit.unit.test.ts`,
 "pi leaves the recorded answer alone and takes the rewritten one",
 asserts that pi refuses the recorded text;
 when that assertion starts failing on a newer pi,
 the release carries the fix and this module,
 its tests and this section can go.

## Credential homes

The credential resolves in this order:

1. Pi's own credential (passed key,
 `--api-key`,
 or the `qwen-token-plan-individual` entry in pi's `auth.json`,
 which keeps resolving because the override keeps the provider id).
2. `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY` from the environment pi hands the auth
 context.
3. `QWEN_TOKEN_PLAN_API_KEY` from the same environment,
 the builtin provider's own name kept as a fallback.
4. Nothing:
 the provider registers anyway,
 lists every model,
 serves the snapshot,
 and logs guidance naming the homes above.

There is no encrypted store in this chain (owner decision,
`doc/planning/qwen-token-plan-individual-provider.md`,
"Install and credential seams"),
unlike `package/pi-plugin/neuralwatt`.
No secret value is ever logged.

## Parity and settings invariants

- Provider id `qwen-token-plan-individual`,
 model ids,
 and effort wire values stay byte-identical to the DSH plugin's and to the
 builtin rows',
 so `qwen-token-plan-individual/<model>` slugs keep working and a user sees one
 catalog spelling in both harnesses.
- For every id the builtin catalog also serves,
 the projected row equals the builtin row value for value
 (id,
 name,
 contextWindow,
 maxTokens,
 input,
 reasoning,
 thinkingLevelMap,
 compat,
 cost,
 type,
 api,
 baseUrl,
 provider,
 inputLimits).
 The characterization fixture pins those rows so a pi-ai upgrade cannot change
 one silently.
- The plugin never writes `models-store.json`,
 `models.json`,
 or `enabledModels`,
 and never auto-enables a model:
 its registry publications carry an `update` and no `persist`.
- A failed,
 empty,
 or media-only refresh keeps the last served catalog;
 it never demotes the snapshot.
- The `install:global` task is the only settings seam:
 it runs `pi install` and fails on any `~/.pi/agent/settings.json` change
 outside the `packages` array,
 so nothing else the owner configured is touched.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/qwen-token-plan-individual"
  ]
}
```

Then supply the credential through one of the homes in
[credential homes](#credential-homes):
 pi's `auth.json`,
 `--api-key`,
 `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`,
 or `QWEN_TOKEN_PLAN_API_KEY`.
The `install:global` task applies the same settings edit with a guard that
refuses to touch unrelated settings or packages.

A settings entry names the **package directory**,
and pi loads the file that
entry's `package.json` lists under `pi.extensions`,
which here is `./dist/final/node/index.mjs`.
So a pi run on a machine whose settings name a checkout loads that checkout's
**built bundle**,
never `src/`:
a change reaches the runs only after
`mise run //package/pi-plugin/qwen-token-plan-individual:build` in that
checkout.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks:

- `build`,
 `lint`,
 `lint:types`,
 `format:oxlint`
- `test:unit`
 (builds first,
 runs every `src/*.unit.test.ts` against the built entry)
- `verify:extension`
 (imports `dist/final/node/index.mjs`,
 registers against a fake Pi API,
 and checks the expected registrations)
- `verify:host`
 (gated by `QWEN_TOKEN_PLAN_INDIVIDUAL_VERIFY_HOST=1`,
 boots pi RPC and `pi --list-models` in throwaway homes and asserts the served
 set,
 the never-persist guard,
 and the projection facts)
- `verify:token-rate-limit-retried`
 (builds first,
 then runs a real pi twice against a loopback stand-in,
 once with this checkout's bundle and once with a bundle built from the commit
 before this translation,
 and asserts that the first retried and the second did not;
 no request reaches the real provider)
- `fixture:builtin-catalog`
 (regenerates `src/builtin-catalog.fixture.json` from the installed pi-ai
 catalog)
- `install:global`
 (pi CLI install plus a settings guard)

## Tests

`src/*.unit.test.ts` covers the projection
 (golden equality with the builtin rows,
 the three reasoning states,
 the compat pins,
 and the ids the builtin lacks),
 the credential chain and its trimming rules,
 the provider refresh lifecycle and its listing credential,
 the serialized dispatch payload
 (including a positive control where the role pin is lifted),
 registration and session lifecycle,
 the never-persist byte-identity guard over `models-store.json`,
 `models.json`,
 and `enabledModels`,
 the characterization fixture over the installed builtin rows,
 and the passing token-rate limit:
 every answer the matcher accepts and refuses,
 the rewritten text accepted by pi's own `isRetryableAssistantError`,
 the stream wrapper's forwarding and its guard for a stream that reports no
 answer,
 and both dispatch entry points over the recorded `429` and a reply.

`verify:token-rate-limit-retried` proves the same boundary with a real pi:
 its `retried` run shows `auto_retry_start` and ends on the stand-in's reply,
 and its `unchanged` run ends on the provider's own `429` with no attempt,
 and it keeps both feeds under the proof folder it prints.
