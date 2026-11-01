# Pi OpenAI accounts extension design

## Status

Design interview requested with `/grill-me` and worked through five rounds.
The frontier is closed: every branch of the design tree has an answer recorded in the decision ledger.
Implementation is **not** authorized yet.
Per the repo's deliberation rules, only explicit acceptance of this document unlocks the code,
and until then the main worktree receives this document and nothing else.

Two packages will change once accepted:
`package/pi-plugin/openai-accounts` (new) and `package/pi-plugin/openai-fast` (edited).

Upstream reminder issue: [#605](https://github.com/Aquaticat/Monochromatic/issues/605) holds a finished draft
for `earendil-works/pi`. Posting it upstream is the user's action, not an agent action.

## Goal

One pi installation holds several of the user's own ChatGPT plan logins,
selectable individually and usable in sequence so that work continues when one account's allowance is spent.

Concretely, from the interview:

- Interactive sessions stop being blocked when the active account's usage window closes.
- Unattended runs (`pi -p`, RPC, overnight lanes) survive an exhausted account without a human at the terminal.
- Around nine accounts, of which eight are new slots and the native `openai` login is the ninth participant.
- ChatGPT plan OAuth only. API keys are already unlimited through `models.json` and environment variables.

## Non-goals

- No multi-tenant service, no shared or resold access, no selling of capacity.
- No quota probing or scraping of ChatGPT Settings → Usage. OpenAI's spec forbids inferring a reset time
  from the exhaustion code, and the route exposes no usage introspection.
- No warm-up or prestart inference traffic. It would spend real allowance and is the exact behavior
  [openai/codex#41664](https://github.com/openai/codex/issues/41664) asks OpenAI to rule on with no answer.
- No changes to pi settings, `enabledModels`, `defaultModel`, model scopes, or the statusline package.
- No takeover of the built-in `openai` provider id.

## Accepted risk, recorded once

OpenAI's Sign in with ChatGPT documentation for open-source clients explicitly requires apps to
"manage multiple account registrations so users can choose an existing ChatGPT account or add another"
and describes an account picker with continue, switch, and sign out
([Accounts and sessions](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions)).
Holding and switching between one operator's own accounts is documented behavior.

Automatic failover when an account's allowance is spent is **undocumented** rather than prohibited.
For the exhaustion code, the documented recovery is "Pause new requests that use the user's ChatGPT plan
and link to ChatGPT Settings → Usage"
([Errors and recovery][errors-and-recovery]).
The user owns this decision and made it with that text in front of them.
The design keeps the switch as the primitive and failover as a policy layered on it, so removing the policy
later leaves a documented multi-account selector behind.

## Decision ledger

Answers are cited by the interview question that produced them.

- Q1: drivers are (a) interactive work stopping at a usage window and (b) unattended lanes dying at exhaustion.
- Q2: all accounts are the user's own, one operator.
- Q3: ChatGPT plan OAuth credentials only.
- Q4: around ten accounts; Q33 and Q46 settled on eight new slots as a constant in code.
- Q5: two real accounts available for live verification, on demand.
- Q6: interactive TUI and headless both, with `openai-fast` in scope.
- Q7: design document first, implementation after acceptance.
- Q8: no takeover of the `openai` id. Register `openai2` through `openai9` as separate providers.
- Q18 and Q32: pi-owned credentials only, with lazy refresh at failover time. No pool file for credentials,
  no proactive background refresh, no warm-up requests.
- Q19: native `openai` participates read-only. Its access token is used when unexpired and never refreshed
  by this extension.
- Q20 and Q33: fixed slot set `openai2`..`openai9`, eight slots, nine accounts with native.
- Q21 and Q39: trigger set accepted; the chain walks every account, one attempt per account per request.
- Q22: retry on mid-stream exhaustion too, discarding the partial output.
- Q23 and Q35: parallel lanes exist, so soft-preference leasing with heartbeat renewal and stale reclaim.
- Q24: explicit ordered list with least-recently-used as tiebreak.
- Q25: implement the documented login flow, including ID token retention, JWKS validation, reuse of each
  account's issued `client_id`, and pi's device id as `ext_agent_host_id`.
- Q26: clone the live native `openai` catalog per slot; no up-front plan gating; record per-account
  rejections as they surface.
- Q27: leave the model picker and scopes alone. No suggested `enabledModels` snippet is written to settings.
- Q28 and Q36: `openai-fast` is edited to enumerate account providers instead of its two hardcoded constants.
- Q30: package is `package/pi-plugin/openai-accounts`, `@monochromatic-dev/pi-plugin-openai-accounts`.
- Q31: upstream draft filed as reminder issue #605, labeled `ready-for-human`; the user posts it.
- Q34: non-secret state lives in `<agent-dir>/openai-accounts.json` at `0600`, with leases in a separate
  ephemeral file that carries no meaning after a crash.
- Q37 and Q43: the switch is announced through a tagged logger from `@monochromatic-dev/logger-core`
  at `warn`. No `pi.notify`, no footer status. Interactive visibility is an open probe, recorded below.
- Q38 and Q44: `/accounts` renders a transcript-visible, non-model-visible status list. No switch command,
  no reorder command, no rename command.
- Q40: both live accounts are ChatGPT Pro.
- Q41: the allowance that closes is a **weekly** one.
- Q42: spent accounts persist a `spent_at` timestamp with reset unknown, are skipped for the rest of that
  process, and are retried by a later process.
- Q45: adopted with one correction, recorded in the corrections section.
- Q46: the slot ceiling is a constant, not a setting. Slot ids are retired permanently and never reused.

## Evidence

### Pi's credential architecture

- `CredentialStore` in `@earendil-works/pi-ai/dist/auth/types.d.ts` is documented as "App-owned credential
  storage, keyed by `Provider.id`, one credential per provider", with `modify` as the only write path and
  OAuth refresh run inside `modify` so concurrent requests cannot double-refresh a rotated token.
- `ProviderId = KnownProvider | string` in `pi-ai/dist/types.d.ts:22`, so arbitrary provider ids are legal.
- `ProviderConfig.oauth` in `pi-coding-agent/dist/core/extensions/types.d.ts:1413` accepts
  `login(callbacks)`, `refreshToken(credentials, signal)`, `getApiKey(credentials)`, and
  `modifyModels(models, credentials)`, and pi stores the result in `auth.json` under that provider id and
  lists it in `/login`.
- `readStoredCredential(providerId, authPath?)` is exported from `@earendil-works/pi-coding-agent`
  (`dist/index.d.ts:4`) as a one-off synchronous read of a stored credential. No write path is exported.
- Pi's auth backend locks the whole `auth.json` with `proper-lockfile` using `realpath: false` and
  `stale: 30_000`, and reads the file inside the lock (`pi-coding-agent/dist/core/auth-storage.js`).
  Files are created with mode `0600`.
- Deep-importing pi's built-in ChatGPT flow fails with `ERR_PACKAGE_PATH_NOT_EXPORTED`, verified from
  `package/pi-plugin/openai-fast`. `@earendil-works/pi-ai/api/openai-responses` **is** importable and
  exports `stream` and `streamSimple`; `@earendil-works/pi-ai/providers/openai` exports `openaiProvider`.
- `ProviderRequestOptions.apiKey` (`pi-ai/dist/types.d.ts`) lets a caller supply the bearer token per request.
- `pi-agent-core/dist/agent-loop.js:143` ends the turn when `message.stopReason` is `error` or `aborted` and
  returns before the tool-call filter at line 156. A failed request therefore executed no tools, which is
  what makes retrying it against another account side-effect free.
- `ModelRuntime.resolveModel` (`pi-coding-agent/dist/core/model-runtime.js:727`) passes the failed response to
  a virtual model's `route()` as `options.failed`, but only when a retry happens.
- `subscription_sharing_usage_limit_exceeded` is a member of `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN` in
  `pi-ai/dist/utils/retry.js`. `subscription_sharing_usage_unavailable` and
  `subscription_sharing_user_unavailable` are members of `RETRYABLE_PROVIDER_ERROR_PATTERN`.
- `ExtensionUIContext` exposes `notify`, `setStatus`, `select`, `confirm`, and `input`
  (`pi-coding-agent/dist/core/extensions/types.d.ts:70`), each implemented per mode.
- `takeOverStdout()` in `pi-coding-agent/dist/core/output-guard.js:38` redirects `process.stdout.write` to
  stderr while the TUI owns the terminal. Stderr is not renderer-managed.

### Pi's ChatGPT login flow

From `pi-ai/dist/auth/oauth/openai-chatgpt.js`:

- Public client, PKCE, `client_id=dynamic_agent_client` on every login, `agent_name_hint: "Pi"`,
  `ext_agent_host_id` as `urn:uuid:<pi device id>`.
- Scope `openid profile email offline_access resource.invoke chatgpt.tokens.use.direct`, resource
  `https://api.openai.com/v1`, authorize at `https://auth.openai.com/api/accounts/authorize`, token at
  `https://auth.openai.com/api/accounts/oauth/token`.
- Callback fixed at `127.0.0.1:1455/auth/callback`; host overridable with `PI_OAUTH_CALLBACK_HOST`, port not.
  Concurrent logins collide, and pi fails with an explicit `EADDRINUSE` message.
- Stored credential is `{type, access, refresh, expires, clientId, scopes}` with a three-minute expiry margin
  baked into `expires`. The `id_token` presence is checked and then discarded.
- Refresh sends `grant_type=refresh_token`, the stored issued `client_id`, the stored `refresh_token`, and
  `resource`, omitting `scope`. `toAuth` returns `{apiKey: credential.access}`, so the access token is a plain
  bearer with no per-account header.
- Requests set `store: false` and `stream: true` (`pi-ai/dist/api/openai-responses.js:242` and `:246`),
  matching the SIWC preview requirements, and send a `session_id` header (line 207).

### OpenAI's specification for this flow

- Access tokens last one hour (`expires_in: 3600`); refresh tokens last 30 days and are replaced on every
  refresh, with no fixed limit on successive replacements
  ([Token reference](https://developers.openai.com/siwc/token-sharing-open-source/token-reference)).
- Refreshes must be serialized per session "so two processes do not race a rotating token".
  Terminal refresh codes are `invalid_grant`, `invalid_refresh_token`, `token_expired`,
  `refresh_token_expired`, `refresh_token_invalidated`, and `refresh_token_reused`
  ([Errors and recovery][errors-and-recovery]).
- Reauthorization for a saved account reuses that account's issued `client_id` with fresh `state`, `nonce`,
  and PKCE values, omits `agent_name_hint`, includes the stable `ext_agent_host_id`, and may send a retained
  `id_token` as `id_token_hint`. The ID token is validated against JWKS for issuer, audience, expiry, and
  nonce, and its `sub` is the account identity
  ([Registration and sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in)).
- The documented credential record carries `email`, `issuer`, `subject`, `client_id`, `ext_agent_host_id`,
  `id_token`, `access_token`, `refresh_token`, `token_type`, `expires_in`, `scopes`, and `saved_at`, written
  atomically at `0600` and never committed or logged.
- Sign-out revokes at the `revocation_endpoint` from `https://auth.openai.com/.well-known/openid-configuration`
  with `token=<refresh token>`, `token_type_hint=refresh_token`, and the issued `client_id`. An empty HTTP 200
  is success, including for an already-invalid token.
- One issued `client_id` per user and workspace is shared across hosts, and plan usage settings and limits are
  shared per issued client across those hosts
  ([ChatGPT plan usage overview](https://developers.openai.com/siwc/token-sharing-open-source)).
- Preview limits reject `temperature`, `max_output_tokens`, `prompt_cache_retention`, `previous_response_id`,
  and similar, and name a service-tier override as a cause of
  `subscription_sharing_unsupported_capability` (400)
  ([Preview limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations)).
- "Pro plans currently have no five-hour limit" and "Weekly limits may also apply". Work already in progress
  may continue past the limit within an active turn, subject to fair use
  ([ChatGPT and Codex pricing](https://learn.chatgpt.com/docs/pricing)).

### The failure this design exists for, observed

Five assistant records in the user's own pi transcripts carry `stopReason: "error"` with this exact message:

```text
The ChatGPT user has reached their Subscription Sharing usage limit. Ask the user to try again after their usage limit resets or use an API key instead.
```

Verified in a session that predates this interview, at
`~/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-09-04T17-44-33-328Z_01a06d85-b130-7e13-ae20-6fbb47504c8c.jsonl`,
twice within nine seconds, with `provider: "openai"`, `api: "openai-responses"`, `model: "gpt-6-astra"`,
and response ids `resp_02076eebda34c52e016ac35f4d499c87d2be74748790cf869f` and
`resp_02076eebda34c52e016ac35f55c35487d2a032dc676567a301`.
Other occurrences sit in sessions dated 2026-09-26, 2026-10-04 (two), and 2026-10-05.

Two properties of that text drive the design:

- It does **not** contain `subscription_sharing_usage_limit_exceeded`, so a trigger keyed on the machine code
  alone would never fire on the user's real failure. Pi's own URL hint at
  `pi-ai/dist/api/openai-responses.js:168` is appended only when the message includes that code, and the
  observed messages have no appended hint, which confirms the code was absent from the text pi received.
- It matches neither `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN` nor `RETRYABLE_PROVIDER_ERROR_PATTERN`, so
  pi schedules no retry and the run ends.

## Corrections to claims made during the interview

Recorded per the repo's correction rule, because both claims shaped a recommendation.

- **Trigger set.** The interview recommendation keyed failover on `subscription_sharing_usage_limit_exceeded`,
  taken from OpenAI's error table. The observed failure carries a human message without that code. The trigger
  is therefore defined in the failover state machine below as a three-part disjunction: the raw code captured
  from the provider stream event, an HTTP 429 status captured from the response hook, and the observed message
  text as a fallback.
- **Non-retryable classification.** The interview stated that pi deliberately classifies the exhaustion signal
  as non-retryable and that this is why routers never see the failure. The pattern membership is real, but for
  the user's actual message the reason no retry happens is that neither pattern matches. Both routes end the
  run, so the architectural conclusion stands and the upstream draft's request is unchanged; issue #605 carries
  a corrective comment with the observed text.
- **`model-retirement`.** The interview said new namespaces would be "absent from those lists" until added.
  `package/pi-plugin/model-retirement` has no provider list. Its README states it groups entries into families
  "by provider, API, and name shape" and re-registers or wraps affected providers, preserving the owner's
  `apiKey`, `oauth`, and `streamSimple`. So it processes `openai2`..`openai9` automatically and identically.
  The user's challenge was correct. What remains is an interaction to verify, not a list to update: this
  extension's providers get re-registered or wrapped at every session start, so a test must prove that a
  wrapped account provider still fails over and still refreshes.

## Architecture

### Provider topology

Eight new providers, `openai2` through `openai9`, registered through `pi.registerProvider(name, config)` with:

- `baseUrl: https://api.openai.com/v1` and `api: "openai-responses"` at provider level.
- `models` cloned from the live native `openai` catalog at startup, so the user's `models.json` overrides are
  inherited rather than duplicated from a second source.
- `oauth` implementing the documented SIWC flow, with pi storing the credential in `auth.json` under the slot
  id and listing the slot in `/login`.
- `streamSimple` owning the failover chain and delegating the Responses protocol to
  `@earendil-works/pi-ai/api/openai-responses`, passing `options.onPayload`, `options.onResponse`, and
  `options.onProviderStreamEvent` through untouched so request inspection and `openai-fast` keep working.

Native `openai` is never re-registered, never refreshed, and never written. It joins the chain read-only
through `readStoredCredential('openai')`, used only while its stored `expires` is in the future.

Unconfigured slots should contribute no selectable models, based on the `filterModels` contract in
`pi-ai/dist/models.d.ts` ("`Models.getAvailable()` applies this filter after confirming that provider auth is
configured") and the credential check at `model-runtime.js:748`. This is listed as a probe in the verification
plan rather than assumed, because it is what makes eight fixed empty slots cheap.

### Credentials and refresh

Credentials stay exactly where pi puts them: one entry per slot in `auth.json`. There is no credential pool
file and no second copy of any refresh token.

Refresh has one implementation per account, supplied as that slot's `refreshToken` hook, so pi's own refresh
calls and this extension's failover-time refresh calls run the same code. When the chain reaches a slot whose
stored token is expired, the extension:

1. Acquires `proper-lockfile` on `auth.json` with the same options pi uses (`realpath: false`,
   `stale: 30_000`), so the write serializes against pi's own read-modify-write in every process.
2. Reads the current entry inside the lock and re-checks expiry, because another process may have refreshed it.
3. Calls the token endpoint with the stored issued `client_id` and `refresh_token`.
4. Writes the rotated pair back atomically at mode `0600`, preserving every other field of that entry.

A terminal refresh code marks the account dead for the process and moves the chain on. Transient network
failures never erase a credential, per the spec's disconnection guidance.

Lock ordering is fixed: pi's `auth.json` lock is always the outermost, and this extension's own state and
lease locks are acquired inside it, never the reverse.

### Failover state machine

Classification of a failed request:

- **Switch accounts.** Raw `error.code` equal to `subscription_sharing_usage_limit_exceeded` from a
  `response.failed` provider stream event, or HTTP 429 from the response hook, or the observed message text
  "reached their Subscription Sharing usage limit". Record `spent_at` for that account, log at `warn`, and try
  the next account in order. One attempt per account per request.
- **Mark dead, keep walking.** `subscription_sharing_invalid_user` (401),
  `subscription_sharing_user_not_eligible` (403), `chatpass_v2_scope_not_authorized` (403), and every terminal
  refresh code. Dead means skipped for the rest of this process; a later process retries it, because a weekly
  window may have rolled and no reset time may be inferred.
- **Back off, do not switch.** `subscription_sharing_usage_unavailable` (503) and
  `subscription_sharing_user_unavailable` (503). These are left to pi's existing retryable path.
- **Not this extension's business.** `subscription_sharing_unsupported_capability` (400),
  `subscription_sharing_route_not_supported` (403), context overflow, transport failures, and aborts. They
  propagate unchanged, and rate limits and transient failures are never rewritten as context overflow.

Mid-stream exhaustion is retried the same way, discarding the partial output, because
`pi-agent-core/dist/agent-loop.js:143` proves no tool from the failed request executed.

When every account is spent or dead, the request fails with a diagnostic that names each account tried, its
classification, and the possibility that the limit is app-scoped rather than account-scoped, since the spec
warns an app-specific limit can produce the same 429. Per-app limits attach to an issued `client_id`, and each
account authorization gets its own issued client, so walking the chain can escape an app limit as well as an
account limit. The diagnostic also links ChatGPT Settings → Usage and states that no reset time is known.

### Ordering and leasing

Order is an explicit list the user edits in `<agent-dir>/openai-accounts.json`, with least-recently-used as
tiebreak. Native `openai` sits last, per Q19.

Because lanes run in parallel, each run takes a **soft** lease: a preference for one account, recorded with a
pid and a heartbeat, reclaimed when stale. Soft rather than exclusive because an exclusive lease strands
capacity whenever a lane idles between turns, and because two lanes spending the same account at once is
survivable while a crashed lane holding an exclusive lease is not. The lease never blocks the chain; it only
orders preference.

### Non-secret state

`<agent-dir>/openai-accounts.json`, mode `0600`, atomic write, holding per slot: alias, masked email,
validated `subject` prefix, issued `client_id` reference, order position, `spent_at`, `dead` marker with
reason, priority-capable marker, models this account rejected, and `last_used`. Leases live in a separate
ephemeral file with no meaning after a crash.

The file stores no access token and no refresh token. Those exist only in `auth.json`, owned by pi.
The retained `id_token` needed for `id_token_hint` is a credential and stays in `auth.json` as an extra field
on that slot's entry, which `OAuthCredentials`' index signature permits.

Tokens, `id_token`, `ext_agent_host_id`, and authorization URLs containing an `id_token_hint` never reach a
log line, per the spec's credential-security section and the repo's logging rules.

### Login and logout

`/login openai5` runs this extension's flow for that slot: PKCE, `client_id=dynamic_agent_client` for a first
registration or the slot's saved issued `client_id` for reauthorization, `id_token_hint` when retained,
`ext_agent_host_id` from pi's device id, callback on `127.0.0.1:1455`, and the manual paste fallback for
headless hosts. The ID token is validated against OpenAI's JWKS using `node:crypto` JWK import, with issuer,
audience, expiry, and nonce checked, and its `sub` becomes the account identity. Logins are sequential, since
the callback port is fixed.

`/logout openai5` revokes at the `revocation_endpoint` before the credential is cleared, retries revocation with
backoff on network failure or 5xx while the refresh token is still available, and reports when remote
revocation could not be confirmed. Slot ids are retired permanently and never reused, because order, aliases,
markers, and `spent_at` all key on the slot id.

### Announcement channel

The switch is logged at `warn` through a tagged logger from `@monochromatic-dev/logger-core`, tagged per the
repo convention with the function name, naming the account that served the turn and the account that was left.
No `pi.notify`, no footer status, no transcript message, and nothing model-visible.

Known properties of that channel: the console sink maps `warn` to `console.warn` and therefore stderr, and the
Node file sink appends JSONL to `<nearest ancestor node_modules>/.monochromatic/<timestamp>.log.jsonl`,
marking itself unavailable when no ancestor `node_modules` exists. Headless visibility is straightforward.
Interactive visibility is **unprobed**: pi redirects stdout to stderr under `takeOverStdout()` but renders
through its own raw-stdout path, so a bare stderr write during a fullscreen TUI may be overdrawn by the next
repaint. The probe is in the verification plan. If it shows the line is not visible interactively, that finding
goes back to the user rather than being fixed by silently adding a second channel.

### `/accounts`

One command, status only, rendered as a transcript list that is not model-visible. Per slot: id, alias,
masked email, state, token expiry, and last used. State is marked with two visible channels per the repo's
interface rule, for example an icon plus a word: active, leased with pid, idle, spent with timestamp, dead
with reason, and never logged in.

### `openai-fast` interaction

`package/pi-plugin/openai-fast/src/constants.ts` hardcodes `OPENAI_PROVIDER = 'openai'` and
`CODEX_PROVIDER = 'openai-codex'`. The edit replaces those two constants with an enumeration that includes
occupied account slots, so companions exist for `openai2`..`openai9` and not for empty ones. Its existing
no-silent-downgrade behavior is preserved, and a `subscription_sharing_unsupported_capability` (400) response
to a priority request records that account as not priority-capable so the companion stops being offered for it.
The README notes that Fast and Ultrafast modes draw subscription usage at different rates from Standard, so
priority use spends an account's weekly allowance faster.

## Rejected alternatives

- **Take over the `openai` provider id.** Rejected by Q8. It would have kept one catalog and let `openai-fast`
  work untouched, at the cost of owning everything pi's built-in provider does.
- **One pooled provider with an extension-owned credential store.** Rejected by Q18. It solves standby expiry
  cleanly but introduces a second credential copy, and a duplicated rotating refresh token is a documented
  terminal failure (`refresh_token_reused`).
- **Projecting the active account into pi's native `openai` slot.** Rejected for the same rotation hazard, and
  because the slot is process-global while lanes run in parallel.
- **Local loopback proxy holding all accounts.** Rejected: a daemon lifecycle plus a second faithful Responses
  SSE implementation, and no surface in Q6 needs a non-pi client. It remains the right answer if Codex CLI
  ever needs the same accounts.
- **Per-account `PI_CODING_AGENT_DIR` with a retrying wrapper.** Rejected: the wrapper replays a whole
  invocation, so tool side effects run twice when the failure lands mid-run. Zero code, but it cannot satisfy
  the unattended requirement safely.
- **Virtual-model router failover.** Not reachable today. `resolveModel` hands a router the failed response
  only on a retry, and no retry is scheduled for this failure. Upstreamed as reminder issue #605; if it lands,
  the `streamSimple` chain is deleted and failover moves to the route layer.
- **Proactive background refresh of standby accounts.** Rejected by Q32 in favor of lazy refresh: about 200
  token rotations a day across eight accounts to save one round trip during a failure that is already waiting.
- **Warm-up inference requests.** Rejected outright; spends real allowance and is unresolved upstream.
- **Exclusive leases.** Rejected by Q35; strands capacity and wedges accounts when a lane crashes.
- **Quota-aware ordering.** Rejected by Q24; no data source exists on this route.
- **`pi.notify` plus footer status.** Rejected by Q43 in favor of the warn log alone.
- **A setting for the slot ceiling.** Rejected by Q46 in favor of a constant.

## Verification plan

Live work runs first, on the user's two Pro accounts, on a throwaway `PI_CODING_AGENT_DIR` created with
`mktemp -d` and mode `700`. The real agent directory is never a fixture.

Probes and checks:

- **Empty-slot probe.** Confirm with `pi --list-models` and the interactive picker that a registered slot with
  no credential offers no selectable models. If it does offer them, the fixed-slot decision needs revisiting.
- **Interactive stderr probe.** Run a temporary extension that logs one `warn` line at session start, inside a
  pseudo-terminal, and establish whether the line survives the TUI repaint. Report the result before relying on
  Q43's choice.
- **Live login.** First registration for slot two, then reauthorization reusing the saved issued `client_id`
  and `id_token_hint`, confirming in ChatGPT Settings that no second app registration appears.
- **Live refresh race.** Two processes refresh the same slot concurrently against the real token endpoint,
  asserting exactly one rotated pair survives and no `refresh_token_reused` occurs.
- **Live switch and chain.** Manual selection of each slot, then a forced chain walk against the stub.
- **Live revocation.** Logout revokes at the `revocation_endpoint`, and a subsequent request with the old token
  fails.
- **Stub server.** An offline OpenAI stub returning the documented error bodies and stream shapes, with
  positive controls proving the detector can show a difference: a plain 429 without the exhaustion code does
  not switch, a 503 backs off without switching, the observed human message text does switch, and the raw
  `response.failed` code does switch. A detector that never fires on a case that must move is not evidence.
- **Mid-stream exhaustion.** The stub emits partial text and tool-call deltas, then fails; assert the partial
  output is discarded, no tool executes, and the retry lands on the next account.
- **`model-retirement` interaction.** With that extension loaded, assert an account provider it wrapped or
  re-registered still fails over and still refreshes.
- **`openai-fast` interaction.** Companions appear for occupied slots only, and a priority 400 marks that
  account not priority-capable.
- **Guard tests.** Each guard is committed, then removed, rebuilt, and run to prove the test fails without it,
  then restored.
- **Branch coverage.** Every classification branch, both refresh paths, lease acquire and stale reclaim,
  atomic write and crash-between-writes, login first-registration and reauthorization, revocation success and
  unconfirmed, and the all-accounts-spent diagnostic.
- **Exhaustion coverage, live.** Recorded as stub-proven and live-pending. Both accounts are Pro, Pro plans
  have no five-hour limit, and the weekly window cannot be forced on demand, so no live exhaustion check is
  claimed until one happens naturally.

## Open items

- The interactive stderr probe result, which decides whether Q43's warn-only channel is actually visible to the
  user in a TUI session.
- The empty-slot probe result, which decides whether eight fixed empty slots cost anything in the picker.
- Credits exhaustion was asked and not answered. The design adopts "switch, and say so in the diagnostic",
  since credits are per account too. Veto welcomed.
- Whether accounts span plans was asked and not answered. Both live accounts are Pro, so per-account rejection
  recording is implemented regardless and plan markers stay best-effort.
- Posting the #605 draft upstream, which is the user's action.

## Implementation sequence

Each step is committed before the next begins, with its own scope.

1. `docs(planning)`: this document.
2. `feat(pi-plugin-openai-accounts)`: package skeleton, `mise.toml` mirroring sibling packages, README, state
   file schema and atomic write, logger tagging.
3. `feat(pi-plugin-openai-accounts)`: SIWC login and reauthorization with JWKS validation, refresh under pi's
   lock protocol, logout with revocation.
4. `feat(pi-plugin-openai-accounts)`: provider registration for the eight slots with cloned native catalog.
5. `feat(pi-plugin-openai-accounts)`: failover chain in `streamSimple`, classification, soft leases, warn
   announcements, `/accounts`.
6. `fix(pi-plugin-openai-fast)`: enumerate account providers instead of the two constants.
7. Verification: stub suite, live suite, probes, guard proofs, then the README completeness pass.

[errors-and-recovery]: https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery
