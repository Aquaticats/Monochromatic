# `api.openai.com` (ChatGPT OAuth, pi `openai` provider, codex-cli 0.162.1):
every credentialed request returns Cloudflare `520`,
while unauthenticated requests return clean `401` JSON

Observed 2026-10-13 between 20:50 and 21:05 UTC from this workstation.
OpenAI's own status page read "We're fully operational" for the whole window,
so the headline status is not evidence that the OpenAI-backed models in pi work.

## Symptom

Advisor calls pinned to an OpenAI model fail immediately:

```text
advisor: no usable review; openai/gpt-6-astra attempt 1: OpenAI API error (520): 520
{"type":"https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-520/",
"title":"Error 520: Web server is returning an unknown error","status":520,
"detail":"The origin web server sent a response that Cloudflare could not parse. This may indicate the origin
returned an empty response, malformed HTTP headers, or an otherwise invalid response.",
"instance":"a4a13fb09ce3c730","error_code":520,"error_name":"unknown_origin_error","error_category":"origin",
"ray_id":"a4a13fb09ce3c730","timestamp":"2026-10-13T20:56:03Z","zone":"api.openai.com",
"cloudflare_error":true,"retryable":true,"retry_after":60,"owner_action_required":true}
```

Raw requests against the same host with the live ChatGPT OAuth access token from `~/.pi/agent/auth.json`
return Cloudflare's HTML error page instead of any OpenAI JSON:

```text
GET  https://api.openai.com/v1/models    -> HTTP 520 html :: api.openai.com | 520: Web server is returning an unknown error
POST https://api.openai.com/v1/responses -> HTTP 520 html :: api.openai.com | 520: Web server is returning an unknown error
```

`codex exec` still answers prompts (it uses the `chatgpt.com` Codex endpoint),
but logs a catalog failure and MCP transport failures on the way:

```text
2026-10-13T20:55:21.008561Z ERROR codex_models_manager::manager: failed to refresh available models: request timed out
2026-10-13T20:55:26.286496Z ERROR rmcp::transport::worker: worker quit with fatal: Transport channel closed, when
  Client(HttpRequest(HttpRequest("http/request failed: error sending request for url
  (https://chatgpt.com/backend-api/ps/mcp)")))
```

Surface trigger:
 any request to `api.openai.com` that carries a **valid** ChatGPT OAuth bearer token.
Requests with no credential or an invalid credential are unaffected.

## Root cause

OpenAI's origin behind `api.openai.com` is closed source,
so no upstream file and line can be cited for the failure itself.
What the differential probes establish is where the failure sits.

Requests that never reach token validation get a correct OpenAI JSON error from the origin:

```text
no auth header      -> HTTP 401 json :: {"error":{"message":"Missing bearer authentication in header", ...}}
bearer garbage      -> HTTP 401 json :: {"error":{"message":"Incorrect API key provided: sk-inval********-000. ..."}}
POST chat/completions, bearer garbage -> HTTP 401 json (220ms)
```

Those `401` bodies are produced by OpenAI's API,
not by Cloudflare,
which proves DNS,
TLS,
edge routing,
and the origin's HTTP stack were healthy from this network during the window.
The `520` appears only once a genuine token must be resolved,
and Cloudflare classifies it as an origin-side fault
(`"error_category":"origin"`,
`"error_name":"unknown_origin_error"`,
`"detail":"The origin web server sent a response that Cloudflare could not parse"`).
So the broken component is OpenAI's OAuth-token validation path behind `api.openai.com`.

Local causes were excluded by measurement,
not assumption:

- No proxy environment variables were set (`HTTP_PROXY`,
   `HTTPS_PROXY`,
   `http_proxy`,
   `https_proxy`,
   `ALL_PROXY`,
  `NO_PROXY`,
   `OPENAI_BASE_URL` all unset).
- Other hosts handle an `Authorization` header normally from the same process:
  `api.anthropic.com/v1/models` returned `401` JSON with an `x-api-key` header,
  and `httpbin.org/bearer` returned `200` with the token echoed.
- The credential was fresh and correctly scoped,
  not expired or mis-issued.
  Its JWT claims read `iss=https://auth.openai.com`,
  `aud="https://api.openai.com/v1"`,
  `iat` 9 minutes before the probe,
  `exp` 51 minutes after it,
  scopes including `resource.invoke`.
  `codex login status` printed `Logged in using ChatGPT`.
- No `520` appears in any pi session log for this repository before 2026-10-13
  (searched `~/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-10-0*.jsonl` and `2026-10-1[0-2]*.jsonl`),
  so this is a change in the remote service,
  not a stale local configuration.

Which pi code path issues those requests:
the `openai-fast` plugin registers a native OpenAI target and a Codex target,
`package/pi-plugin/openai-fast/src/constants.ts:30` and `package/pi-plugin/openai-fast/src/constants.ts:10`:

```ts
export const OPENAI_PROVIDER = 'openai';
export const CODEX_PROVIDER = 'openai-codex';
```

with `package/pi-plugin/openai-fast/src/constants.ts:35` naming the API
`'openai-responses'`.
The native target posts to `api.openai.com` with the ChatGPT OAuth bearer token,
asserted at `package/pi-plugin/openai-fast/src/native-openai.unit.test.ts:105`
and `package/pi-plugin/openai-fast/src/native-openai.unit.test.ts:109`:

```ts
expect(request.url,).toBe('https://api.openai.com/v1/responses',);
expect(native.requests[1]?.headers.get('authorization',),).toBe(`Bearer ${HOST_TOKEN}`,);
```

The Codex target posts to `<baseUrl>/codex/responses` on the ChatGPT backend instead,
per `package/pi-plugin/openai-fast/src/stream-fixture-http.ts:124`:

```ts
if (url !== `${model.baseUrl}/codex/responses`)
```

That split explains why `codex exec` kept working while the `openai` provider died:
they use different hosts.

Two earlier readings were wrong and are recorded so they are not re-derived.
"OpenAI is down" is disproved by the clean `401` JSON controls and by `codex exec` returning `pong`.
"Our OAuth token expired or was revoked" is disproved by the `iat`/`exp` claims and `codex login status`.

Context from OpenAI's own status history:
incident `01M4T4ZCM54D3ZPTQXRSWFG0XS`,
"Degraded performance in GPT-6.1 Sol Codex and Work mode",
ran from 08:10 UTC (investigating 08:22 UTC,
mitigation applied 09:30 UTC) to a 16:36 UTC resolution,
covering ChatGPT Work,
Codex Web,
Codex API,
CLI,
and the VS Code extension.
The observations in this doc start 4 hours after that resolution marker,
so this is either residual failure of the same OAuth path or a separate one;
nothing published distinguishes them.
An independent monitor recorded the same window as 08:24:03 to 16:42:03 UTC
with 164 of 166 checks degraded and now reports `Operational · 241ms`;
that site is an aggregator that times a plain request and reads the published status feed,
so treat it as corroboration of timing only,
not of cause.

## Verification

Versions under test:
 `codex-cli 0.162.1`,
Node `v26.11.1`,
pi `@earendil-works/pi-coding-agent` 1.2.0,
repository `/var/home/user/Monochromatic`.

Runnable harness.
It reads the OAuth token from pi's own auth store and never prints it.
Save as `${HOME}/temp/agent/oai-520-probe.mjs` and run `node ${HOME}/temp/agent/oai-520-probe.mjs`:

```js
// oai-520-probe.mjs: separate an OpenAI-side origin fault from local credential and proxy problems.
import { readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const auth = JSON.parse(readFileSync(join(homedir(), '.pi/agent/auth.json'), 'utf8')).openai;
const claims = JSON.parse(Buffer.from(auth.access.split('.')[1], 'base64url').toString('utf8'));
console.log('token exp in', claims.exp - Math.floor(Date.now() / 1000), 's | aud', claims.aud);

async function probe(label, url, init) {
  const started = Date.now();
  try {
    const response = await fetch(url, { ...init, signal: AbortSignal.timeout(20_000) });
    const body = await response.text();
    const html = body.trimStart().startsWith('<');
    const marker = html ? (body.match(/<title>([^<]*)<\/title>/i)?.[1] ?? '') : body.replace(/\s+/g, ' ').slice(0, 140);
    console.log(`${label}\n    -> HTTP ${response.status} ${Date.now() - started}ms cf-ray=${response.headers.get('cf-ray') ?? '-'} :: ${marker}`);
  } catch (error) {
    console.log(`${label}\n    -> THREW ${error.name}: ${error.message}`);
  }
}

const models = 'https://api.openai.com/v1/models';
// Positive control: must produce OpenAI JSON. If it does not, the failure is local, not upstream.
await probe('control no-auth', models, {});
await probe('control garbage bearer', models, { headers: { authorization: 'Bearer sk-invalid-probe-000' } });
await probe('control other host with auth header', 'https://api.anthropic.com/v1/models', {
  headers: { 'x-api-key': 'sk-ant-invalid-probe', 'anthropic-version': '2023-06-01' },
});
// Subject: the request shape pi's native openai path sends.
await probe('subject real token GET /v1/models', models, { headers: { authorization: `Bearer ${auth.access}` } });
await probe('subject real token POST /v1/responses', 'https://api.openai.com/v1/responses', {
  method: 'POST',
  headers: { authorization: `Bearer ${auth.access}`, 'content-type': 'application/json', originator: 'codex_cli_rs' },
  body: JSON.stringify({
    model: 'gpt-6-astra',
    input: [{ role: 'user', content: [{ type: 'input_text', text: 'Reply with the single word: pong' }] }],
    stream: false,
    store: false,
    max_output_tokens: 16,
  }),
});
```

Interpretation rule:
the three controls must stay clean (`401`,
`401`,
`401`) while the two subject lines return `520`.
If a control also fails,
the fault is local (proxy,
DNS,
egress filtering) and this doc does not apply.

### Requests that worked during the window

- `GET https://api.openai.com/v1/models` with no auth header:
   `HTTP 401` JSON,
   6 of 6 over two probe runs,
   plus one `curl` request,
   92ms to 198ms,
  `cf-ray=a4a1399fbea2a3f1-YYZ`.
- `GET https://api.openai.com/v1/models` with `Bearer sk-invalid-probe-000`:
   `HTTP 401` JSON naming the bad key,
   113ms.
- `POST https://api.openai.com/v1/chat/completions` with `Bearer sk-invalid-probe-000`:
   `HTTP 401` JSON,
   220ms.
- `GET https://api.anthropic.com/v1/models` with an invalid `x-api-key`:
   `HTTP 401` JSON with a `request_id`,
   90ms.
- `GET https://httpbin.org/bearer` with a garbage bearer:
   `HTTP 200` JSON echoing the token,
   145ms.
- `codex exec --sandbox read-only --skip-git-repo-check 'Reply with the single word: pong'`:
   answered `pong`
  (chatgpt.com Codex path,
   model `gpt-6.1-sol`,
   15,048 tokens used).
- `GET https://api.llmgateway.io/v1/models` with the `llmgateway-devpass` key:
   `HTTP 200`,
   145 models,
  including both `gpt-6.1-sol` and `gpt-6-astra`.
- `POST https://api.llmgateway.io/v1/chat/completions` for `gpt-6.1-sol`:
   `HTTP 200` with `"pong"` in 2224ms.
- `POST https://api.llmgateway.io/v1/chat/completions` for `gpt-6-astra`:
   `HTTP 200` with `"pong"` in 2337ms.

### Requests that failed during the window

All failures were the same variant:
 Cloudflare HTML `520` from zone `api.openai.com`,
`error_category: origin`,
`retryable: true`,
`retry_after: 60`.

- `GET /v1/models` with the real OAuth bearer:
   `HTTP 520`,
   7 of 7 attempts,
   72ms to 249ms,
  rays `a4a139a0587fa3f1-YYZ`,
   `a4a139a18c7ba3f1-YYZ`,
   `a4a139a35b27a3f1-YYZ`,
   `a4a139a47f81a3f1-YYZ`,
   `a4a14c2aabf45163-YYZ`;
  two earlier attempts were not recorded with their rays.
- `POST /v1/responses` with the real OAuth bearer,
   `model: gpt-6-astra`:
   `HTTP 520`,
   228ms then 144ms on a second attempt,
   ray `a4a14c2b9f7c5163-YYZ`.
- `POST /v1/responses` with the real OAuth bearer,
   `model: gpt-6.1-sol`:
   `HTTP 520`,
   77ms.
- Advisor pinned to `openai/gpt-6-astra`:
   `OpenAI API error (520)`,
   ray `a4a13fb09ce3c730`,
   20:56:03Z.
- `codex` model catalog refresh:
   `request timed out`,
   twice in one run.
- `https://chatgpt.com/backend-api/ps/mcp`:
   transport failure,
   twice in one run.

Not evidence of anything:
`GET https://chatgpt.com/backend-api/models` with the real token returned `HTTP 403` HTML,
and `GET https://platform.openai.com/` returned `HTTP 403` from `curl`.
Both endpoints require headers or a session that the probes did not send,
so a `403` there is the expected rejection.

## Verified workarounds

### Route the same model ids through the `llmgateway-devpass` provider

`~/.pi/agent/models.json` already registers `gpt-6.1-sol` and `gpt-6-astra` under
`llmgateway-devpass` (`baseUrl: https://api.llmgateway.io/v1`,
`api: openai-completions`,
API key,
header `x-source: pi`).
Both answered `pong` over `POST /chat/completions` during the window,
so selecting that provider restores the model ids without waiting on OpenAI.

Tradeoffs:
 a different operator and billing path,
so ChatGPT-plan quota does not apply and provider-side logging differs;
the served weights are whatever that gateway fronts,
so behavior can drift from first-party;
and the API is `openai-completions`,
not `openai-responses`,
which drops Responses-only features such as stored previous-response ids,
reasoning item replay,
and `service_tier: priority`.

On 2026-10-13 the user declined re-routing pi's OpenAI model ids through this provider,
so it stays a documented fallback rather than an applied configuration change.

### Use the Codex backend path for one-off checks

`codex exec --sandbox read-only --skip-git-repo-check '<prompt>'` reaches the model through
`chatgpt.com/backend-api/codex/responses` and worked during the window.

Tradeoffs:
 it is a separate harness,
and `.out-of-scope/codex-harness.md` keeps this repository from investing in Codex harness integration,
so it is a manual probe rather than a pi routing fix;
its model catalog refresh was timing out,
so any model list it shows may be stale;
and pi's `openai` provider does not use this host,
so nothing in pi is repaired by it.

### Retry with a bound

Cloudflare marks the error `retryable: true` with `retry_after: 60`.
Ten credentialed requests spread over 15 minutes all returned `520`,
so short backoff did not recover inside the observation window.

Tradeoff:
 unbounded retrying burns wall time and hides the diagnosis;
cap the attempts and re-run the differential harness before concluding anything changed.

## What does not work

- Reading `status.openai.com` as proof of health.
   It printed "We're fully operational" and
  "We're not aware of any issues affecting our systems."
   for the entire window while every credentialed
  `api.openai.com` request failed.
   Its 90-day component view also lists resolved incidents under a
  green headline,
   so a green page cannot distinguish "no incident" from "incident closed,
   path still broken".
- Re-authenticating or refreshing the OAuth token.
   Not attempted here,
   because `iat`/`exp` showed a
  nine-minute-old token with 51 minutes of validity and `codex login status` reported a healthy login;
  a fresh token cannot fix an origin that fails while parsing its own auth response.
   If a future session
  does refresh,
   record whether the `520` persists,
   since that is the one local variable this doc leaves untested.
- Switching endpoints on the same host.
   Both `/v1/models` and `/v1/responses` returned `520`
  with the valid token,
   so the fault is not endpoint-specific.
- Sending the real credential to a third-party echo service to test header propagation.
   Use a garbage
  token for that control;
   leaking a live ChatGPT OAuth token to `httpbin.org` or similar buys nothing.

## Upstream filing artifact

### Out-of-scope check

`.out-of-scope/` was read.
`.out-of-scope/codex-harness.md` exempts the Codex harness plugin surface and states it
"does not apply to Codex models,
model providers,
APIs,
benchmarks,
or other non-harness Codex technology",
so it does not cover this failure.
`.out-of-scope/pi-gpt55-long-context.md` concerns context-window metadata,
not transport.
No exemption matched.

### Duplicate search

`gh search issues --repo openai/codex '"520"'` returned openai/codex issue 45569,
"Repeated HTTP 520 from Codex responses endpoint interrupts desktop tasks and reviewers",
opened 2026-09-15,
state open,
labels `bug`,
`app`,
`connectivity`,
zero comments.
That report describes intermittent `520` responses from
`https://chatgpt.com/backend-api/codex/responses` in the Codex desktop app,
states "The exact origin/edge cause is unknown" and "no minimal deterministic reproducer is available",
and notes its ray ids were supplied privately through the Help Center.
A phrase search for `"Web server is returning an unknown error"` and a search for `unknown_origin_error`
across the `openai` organization returned no other matching issue.
No new issue should be opened.

### Upstream filing decision

1. Really upstream's fault:
    yes.
    Cloudflare reports `error_category: origin` on zone `api.openai.com`,
   and the origin answers invalid-credential requests correctly,
    so the fault is in OpenAI's
   OAuth validation path.
2. Can upstream fix it:
    yes,
    and only they can.
    It is their origin infrastructure.
3. Are they supporting this use case:
    yes.
    The token's own `aud` claim is `https://api.openai.com/v1`,
   and pi's native OpenAI target is documented by
   `package/pi-plugin/openai-fast/src/native-openai.unit.test.ts:105`.
4. Would the repository welcome the contribution:
    unknown,
    and not the deciding factor here.
   `openai/codex` accepts bug reports (issue 45569 carries the `bug` and `connectivity` labels and
   stayed open),
    but the failing component is closed-source service infrastructure that no pull request
   in that repository can change.
    No policy banning AI-assisted reports was found in the issue template
   fields reproduced in issue 45569.
5. Will they likely fix it:
    unknown.
    They resolved a related Codex degradation the same day
   (incident `01M4T4ZCM54D3ZPTQXRSWFG0XS`),
    which shows the area is actively worked,
    and issue 45569
   has sat open without a maintainer response since 2026-09-15.
6. Prototyped minimal fix:
    no,
    and it is not achievable.
    There is no source to patch for
   `api.openai.com`'s origin,
    so the auto-prototype step has no target.
    This is the constraint that
   fails the gate.

Decision:
 do not file a new issue.
The comment draft below is additive to issue 45569:
that thread has no deterministic reproducer and no isolation between the credential path and the
transport path,
and this one supplies both.
Posting to `openai/codex` is an external action on a repository this project does not own,
so it was held for authorization;
the user authorized it on 2026-10-13,
and the comment went up as
[comment 6155282654][posted-comment]
at 21:11:37 UTC.
The posted text differs from the draft kept here only in recomputed attempt counts,
the ray ids added by a second probe run,
and an observation window widened to 20:50 through 21:05 UTC.
The draft stays as the audited artifact this decision was made from.

~~~md
Additive evidence: deterministic HTTP 520 on `api.openai.com` for valid ChatGPT OAuth tokens,
with invalid-credential controls staying healthy

Posted 2026-10-13 as openai/codex issue 45569 comment 6155282654 with the user's authorization.
Kept here as the audited draft. The posted text carries recomputed attempt counts, the ray ids
from a second probe run, and a window of 20:50 to 21:05 UTC.

Observed 2026-10-13 20:49 to 20:56 UTC, codex-cli 0.162.1, Node v26.11.1, Linux workstation,
ChatGPT-plan login (`codex login status` reports `Logged in using ChatGPT`).

This report describes intermittent 520s from `chatgpt.com/backend-api/codex/responses` with no
deterministic reproducer. A different host shows the same status code deterministically, and the
differential isolates it to the OAuth validation path rather than the transport:

- `GET https://api.openai.com/v1/models` with a valid ChatGPT OAuth access token: HTTP 520,
  Cloudflare HTML, 5 of 5 attempts, 72ms to 249ms. Rays `a4a139a0587fa3f1-YYZ`,
  `a4a139a18c7ba3f1-YYZ`, `a4a139a35b27a3f1-YYZ`, `a4a139a47f81a3f1-YYZ`.
- `POST https://api.openai.com/v1/responses` with the same token: HTTP 520 for both `gpt-6-astra`
  (228ms) and `gpt-6.1-sol` (77ms).
- Same URL, no auth header: HTTP 401 JSON `Missing bearer authentication in header`, 4 of 4,
  92ms to 198ms, ray `a4a1399fbea2a3f1-YYZ`.
- Same URL, `Bearer sk-invalid-probe-000`: HTTP 401 JSON `Incorrect API key provided`, 113ms.
- `POST /v1/chat/completions` with the invalid bearer: HTTP 401 JSON, 220ms.

The 401 bodies come from the API, not the edge, so DNS, TLS, edge routing, and the origin HTTP stack
were healthy throughout. The 520 appears only when a genuine token must be resolved. The Cloudflare
body reports `"error_category":"origin"`, `"error_name":"unknown_origin_error"`, and
`"detail":"The origin web server sent a response that Cloudflare could not parse"`.

Local causes excluded: no proxy environment variables set; `api.anthropic.com/v1/models` returned
401 JSON and `httpbin.org/bearer` returned 200 from the same process with auth headers; the token was
9 minutes old with 51 minutes of validity, `iss=https://auth.openai.com`, `aud=https://api.openai.com/v1`,
scope `resource.invoke`.

During the same window `codex exec --sandbox read-only` completed a prompt successfully, while logging
`codex_models_manager::manager: failed to refresh available models: request timed out` twice and two
transport failures for `https://chatgpt.com/backend-api/ps/mcp`. `status.openai.com` read
"We're fully operational" throughout, and incident `01M4T4ZCM54D3ZPTQXRSWFG0XS` had been marked
resolved at 16:36 UTC, four hours earlier.

Suggested direction: the origin returns an unparseable or empty response when validating a ChatGPT
OAuth token on the `api.openai.com` `/v1` surface, so the auth path there warrants the same look the
Codex responses endpoint got, and the model-catalog refresh timeout suggests one shared dependency.
~~~

## Related

- [doc/troubleshooting/README.md](README.md),
  troubleshooting family index.
- OpenAI status history: <https://status.openai.com/history>.
- Incident cited in this doc: <https://status.openai.com/incidents/01M4T4ZCM54D3ZPTQXRSWFG0XS>.
- Duplicate report: <https://github.com/openai/codex/issues/45569>.
- Comment posted to that report:
   [comment 6155282654][posted-comment].
- Independent monitor used for window timing only: <https://cutoffreport.com/down/openai>.

[posted-comment]: https://github.com/openai/codex/issues/45569#issuecomment-6155282654
