# Rate limit fallback for Pi

`@monochromatic-dev/pi-plugin-rate-limit-fallback` adds the keyless
`rate-limit-fallback` provider to Pi. Its `writer` and `reviewer` models are
Pi native virtual models: a router for each resolves the pool model from Pi's
catalog on every request and sends that request to the package's pool
provider. Select one with `--provider=rate-limit-fallback --model=writer` or
`--provider=rate-limit-fallback --model=reviewer`.

The provider also lists two internal physical models, `writer-pool` and
`reviewer-pool`, which the routers dispatch through. They exist because a
virtual model must route to a physical model with credentials, and Pi refuses
a virtual model whose id a physical model of the same provider already holds.
Selecting either pool model directly bypasses the router; the difference it
makes is described under "What the route gives Pi".

The extension registers its catalog immediately and takes the live model
registry from `session_start`. It does not install itself or change your
settings.

## What an install needs

Pi loads this package the way `~/.pi/agent/settings.json` lists it: a directory
under `packages`, whose `package.json` names the built entry under
`pi.extensions` (`./dist/final/node/index.mjs`). A checkout that is not built
loads nothing. Add the package directory to `packages`, or name it for one run
with `-e <package directory>`.

The sessions the pool dispatches to are physical models of other providers, so
those providers' extensions must load too, as the settings package list does
for the members this package asks. A session started with `--no-extensions`
loads neither this package nor the member providers; it must name every
extension it needs explicitly, and a session with no member registered can
answer nothing (see "When no member answers").

Do not select `rate-limit-fallback/writer-pool` or
`rate-limit-fallback/reviewer-pool` directly: that is the pool without the
router, and it loses Pi's context-overflow recovery.

## Order and refusal policy

The project chose these writers on 2026-11-19 in [its subagent skill][writers]
(section "Who writes and who reviews"), which dropped GPT 6.2 Sol as a
writer. The `writer` pool asks these members in order:

- `neuralwatt/deepseek-v4.1-flash`
- `neuralwatt/deepseek-v4.1-flash-flex`
- `opencode-go/deepseek-v4.1-flash`

The `reviewer` pool tries these members in order:

- `qwen-token-plan-individual/deepseek-v4.1-flash-1017`
- `synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`
- `opencode-go/deepseek-v4.1-flash`
- `neuralwatt/deepseek-v4.1-flash-flex`

No request is sent to the provider named `deepseek`. An unregistered member
is skipped. A registered member's authentication error is not skipped.

A terminal HTTP 429 **before any text, thinking, or tool output** moves the
request on using the same transcript. A bare stream `start` is held back
until the attempt commits or successfully finishes. Once a content event
has been exposed, or a terminal message already contains content, the
plugin returns that member's error instead of replacing its partial
answer or mixing events. Cancellation, transport errors, bad inputs, and
other errors that are neither a 429 nor a transient provider status do not
trigger fallback. When an adapter does not expose a
response status, the pool reads it from the adapter's own wording: the
Responses adapter's `<provider> API error (429): …` label, which spells the
label `OpenAI` only for the provider named `openai`, or the Completions
adapter's `429: …`. Recorded provider refusals also appear in a quoted form,
`429 "…"`, which the same leading-status reading accepts; a number that merely
starts with `429`, such as `4290 tokens`, is not a refusal.

The same pre-output rule moves the request to the next member for HTTP 503 and
for Cloudflare's origin statuses 520 to 526, which the owner decided on
2026-11-19 ("Move on (Recommended)"). NeuralWatt documents 503 as fleet
capacity that a retry may clear, and Cloudflare documents 520 to 526 as its own
origin failures. Those statuses are read the same way as a 429: from the
response the adapter exposed when it exposed one, and otherwise from the
status the refusal text opens with. Cloudflare's 522 page heads itself
"connection timed out", and Pi's own retry classifier matches `timed out`, so
the pool consumes that refusal before Pi sees it whenever another member
answers. When every member refuses transiently, the pool returns the last
refusal unchanged, as described under "When no member answers", and Pi's own
retry may then apply to it. A
transient refusal's `Retry-After`, when a response carries one, holds that
member for the time it names. A status that arrives after output was exposed
ends the request at that member, like a 429.

A policy stop follows the same exposure rule, and only the OpenAI provider can
move a request on. A member registered as provider `openai` whose refusal opens
with `This content was flagged for possible cybersecurity risk`, and which shows
no text, thinking, or tool output before it, moves the request to the next
member with the same transcript. Any other member's content stop ends the
request there, and so does a refusal that merely quotes the sentence or carries
it inside a 429 body. Once output was shown, the stop ends the request and the
message names the member and keeps the advisory. A move is logged with the
member's name and the stop's text, worded apart from a rate-limit move, and the
assistant record names the member that answered; a stop that ends the request is
not logged. Neither pool has an OpenAI member since 2026-11-19, so this move
cannot fire unless one is added, and the rule stays for that case.

The project decided this on 2026-11-19, recorded with its owner answers in
[the handover that carries the provider work][handover]: the stop moves the
request on automatically ("Automatic (Recommended)"). The decision was about
OpenAI's filter alone. The rule that once made the same move by hand is retired on `main`, so the
pool carries it.

These rules rest on unit tests that drive the built plugin against stand-in
members, and on the installed Responses adapter reporting the recorded stop
text byte for byte over an in-memory stream. The recorded advisory carries no
HTTP status, which is the form the installed adapter reports when a stream
fails without one; the capture fixes the text alone, not the wire event that
carried it. No policy move has been observed on a real provider yet; the
extension check covers the 429 handoff only.

OpenAI's `This content was flagged for possible` sentence takes precedence over
a 429 or a transient provider status wherever it sits in the text, even if the
advisory following it is truncated or changes punctuation, so a 429 or 503 body
that quotes it ends the request instead of moving it. The shorter
`possible risk` wording and generic policy labels identify a stop only when
there is no 429 and no transient provider status, and they never move a request
on: such a stop ends it. A 503 or Cloudflare body carrying only those words
therefore moves the request on like any other transient refusal; one that also
carries OpenAI's opening ends the request at that member, like a 429 quoting it.
A stop that ends the request names the member
and preserves the advisory, including OpenAI's remediation. It replaces only the
spans matched by Pi's transient-error classifier, such as standalone retryable
status numbers, `rate limit`, and `please retry your request`, with `[omitted]`.
The surrounding wording and punctuation are kept. This prevents Pi's outer retry
from restarting a policy stop; a move to the next member never reaches that
classifier at all.

When no member answers, the pool returns a failure rather than a fallback. A
policy stop it set aside comes first, attributed to the member that stopped the
request: that run ends there, even though the later members were only
rate-limited, because returning their 429 would let Pi's outer retry restart at
the member that refused. With no stop set aside, the pool returns the last
refusal it set aside, as it arrived: a 429 or a transient status. When no member
was attempted, the pool names the held members and their state: a published
concurrency ceiling of zero, a local concurrency ceiling with the count in use,
or a hold until a provider-derived time. These local admission results do not carry a fabricated
429 and are not retried by Pi: generic retry
backoff cannot be aligned to a local slot freeing, a provider reset, or a plan change. Each member
attempt runs with provider-level retries off, so a 429 reaches the pool on the first attempt and the
pool owns fallback; Pi's own agent retry layer still applies to whatever terminal error the pool
returns. This plugin does not change the subagent driver's retry or pool accounting.

## What the route gives Pi

Pi asks the router for a physical model before every request made with a
virtual selection, including a retry, a tool continuation and a compaction
summary. Each router answers with the pool model from the catalog it reads at
that moment, so every request goes through the pool's own order and refusal
rules, and the pool still decides which member answers.

The virtual models declare, and the pool models carry, the smallest window and
output limit every registered member of that pool reports, read from the session
registry and re-registered when the session starts. Pi compacts on a threshold
taken from those limits, so a compaction it decides fits every member. Before a
session binds, both carry Pi's own fallback for a model that declares no window,
128,000 tokens and 16,384 output.

After the first answer, Pi measures context usage, and compacts on the threshold,
against the model the router returned: the pool model, which carries the
smallest registered member's limits, so a compaction Pi decides still fits every
member. Each request's output limit is sized from that routed model and then
clamped to the member that receives it. The declared pair is therefore a floor
for the selection, not a promise about the member that answers.

The route is what restores Pi's context-overflow recovery. An assistant message
records the member that produced it. Under a virtual selection Pi resolves that
recorded model in the catalog, recognises the member's overflow as the selected
model's overflow, compacts the transcript and retries the turn once; the retry
asks the router again, which returns the pool, and the pool starts from its head
with the smaller transcript. The same overflow under `writer-pool` selected
directly records a model Pi cannot match to the selection (the message names the
member, not `writer-pool`), so Pi neither compacts nor retries and the turn ends
on the member's error. The extension check measures both arms.

A consequence for callers that drive Pi's SDK: Pi marks a keyless provider's
availability in a background pass, so a host that prompts immediately after
binding extensions should await that pass (`ModelRuntime.refresh({ allowNetwork:
false })`) first, as Pi's own command line does before its first request.

## Changing limits

The pool starts from its head on every request. It holds no fixed provider
request counts. When a provider supplies a retry or reset time, that
member is eligible for reevaluation at the time given. With no timing
evidence, it can be retried on the next request. The plugin neither
sleeps on a refused member nor permanently demotes it.

A `Retry-After` reaches the plugin only if an adapter exposes the failed
response through its response hook, and none of the three adapters Pi 1.5.2
installs for these members does: each calls the hook after a successful
response alone. The plugin therefore takes a refusal's timing from the delay the
refusal body names, `retry_strategy.suggested_initial_delay_s`, as NeuralWatt's
guide documents, and otherwise from a provider's documented wording. Header
timing stays in place for an adapter that starts exposing failed responses, and
it now covers a transient refusal's `Retry-After` as well as a 429's.

Authenticated quota endpoints are read when a member refuses: a 429 before
output triggers one read of that member's endpoint, bounded by the read's own
deadline. Nothing is read on admission, on the first eligible attempt, or on
reentry; the evidence a read produces governs later admissions. The owner
decided this on 2026-11-19 ("Read at refusal, quota endpoints have separate
extremely high rate limits"). The sources read do not price the three endpoints
the same way. NeuralWatt documents a numeric limit on its rate-limit endpoint:
`/v1/rate_limits` is "rate-limited to 2 requests per second per customer" and
clients are asked to poll at most once per second. Synthetic documents an
accounting exemption rather than a number: its quota page says "/quotas requests
do not count against your subscription limits". No source read states a limit
for NeuralWatt's `/v1/quota`, which a refusal's read also asks for its
subscription facts. The plugin therefore keeps its own floor: one read of a
member at most once per second, so a burst of refusals does not become a burst
of reads, and concurrent requests in one session share a pending lookup. A failed or timed-out read leaves existing
evidence in place, and the next refusal reads again. A transient 503 or
Cloudflare status is not a quota signal and reads nothing.

A reading that reports a ceiling of zero, such as a plan with no capacity for
that lane, excludes that member for the rest of the session: admission refuses
it without dispatching, so no later refusal can trigger the fresh reading that
would lift it, and the plugin logs the exclusion at `info` with the member, the
reported ceiling and the reading's time. When another member answers the request
instead, the turn itself says nothing about the exclusion. The next session
reads again at its first refusal. A 429's own hold is never permanent: it
carries the time the provider named, and once that time passes the member is
asked again.

The quota read has its own deadline of two seconds,
capped at half a positive caller-supplied model timeout (rounded down).
The read stops holding up dispatch at that deadline even if the transport is slow
to acknowledge cancellation.
At that deadline the read is abandoned and routing continues without its evidence
(`readQuotaWithinDeadline` in `src/member-capacity.ts`),
so a slow endpoint costs only the reading,
never the request.
A request joining a shared lookup bounds its own wait without canceling that lookup.
The model request retains its own timeout.
That bound comes from a live endpoint measurement on 2026-11-19:
twenty alternating rounds against Synthetic's `/v2/quotas` and NeuralWatt's
`/v1/rate_limits` and `/v1/quota`,
with NeuralWatt's flex lane measured over six rounds.
Every quota read was paired with a same-host listing request that reads no quota:
the read's median ratio to that listing was 1.32 on Synthetic (1.01 to 1.80),
and 1.45 on NeuralWatt (0.35 to 5.12).
The ratio is not what the value rests on,
because the listings return bodies tens of times larger than the quota documents,
and NeuralWatt's own spread spans that whole range.
The absolute readings are what it rests on:
the highest read took 464 ms and the nearest-rank 95th percentile 403 ms,
so two seconds covers the highest reading four times over,
and still covers it once the host's control spread after the warm-up round
(325 ms) is added on top (790 ms).
`doc/quota-read-latency.md` records the method and the per-arm figures.

### Provider sources

- **OpenAI.** The [API rate-limit guide][openai-limits] documents
  `Retry-After` and request/token remaining and reset headers. The plugin
  uses a `Retry-After` when an adapter exposes the failed response, which no
  installed adapter does, and can use zero-remaining headers on a successful
  response. Pi's adapter does not expose every unsuccessful
  HTTP response through its response hook. The [organization API][openai-admin]
  has project rate-limit administration, not a live remaining-quota
  lookup for the inference credential. The [Codex app-server][openai-codex]
  documents `account/rateLimits/read` for ChatGPT subscriptions, which is
  not a Pi registry HTTP endpoint. The plugin does not start a Codex server
  or use organization admin credentials.
- **Qwen Token Plan Individual.** Alibaba Cloud's [Token Plan FAQ][qwen]
  says TPM, RPM, and concurrency are adjusted based on load. Its error table
  names the refusal `429 Requests rate limit exceeded`, its "Rate limiting and
  concurrency" section separates that class from `insufficient_quota`, and the
  advice is "Wait about 1 minute before retrying". The plugin uses explicit
  response timing first and the published wait only for that documented wording.
  No refusal recorded on this machine carries that wording (the recorded Qwen
  refusals are quota exhaustion), so this rule rests on the documentation alone,
  and a change in the provider's wording would silently disable it. The
  documentation checked did not identify a personal-plan quota API or
  plan-specific limit headers for this inference credential, so this provider
  has no endpoint for the refusal-time read.
- **Synthetic.** The plugin reads `/v2/quotas` and accepts the current
  and previous weekly-block names. [Its quota page][synthetic-quotas] says those
  requests "do not count against your subscription limits", which is why a
  refusal can afford the read. A depleted window holds the member
  only when it also reports a reset time. See the
  [recorded quota schema change][synthetic-schema]. A positive reading
  does not guarantee the next request will be admitted.
- **OpenCode Go.** [Go's usage guide][go] publishes window rules and
  estimates, with actual usage tracked in its console. The
  [model list API][go-models] publishes plan/model monthly limits, not
  remaining account allowance. No documented account quota endpoint
  or Go-specific quota headers were found in these references, so this provider
  has no endpoint for the refusal-time read. The plugin uses a `Retry-After`
  only when an adapter exposes the failed response, rather than guessing room
  from a published estimate or starting a console session.
- **NeuralWatt.** [Rate limits][nw-rates] documents
  `/v1/rate_limits?model=…`, whose row names the canonical model, and states that
  the endpoint is "rate-limited to 2 requests per second per customer" and asks
  clients to poll at most once per second: the interval the plugin keeps between
  reads of one member. The `-flex` member takes the reported `flex_floor`, the ceiling for
  low-priority flex-lane requests, and the standard member takes
  `effective_concurrency`, the resolved cap for the model; the lane comes from
  the id the pool asks for. The plugin enforces it over in-flight calls in this
  Pi session; it cannot count another process's calls or reserve seats.
  [Quota][nw-quota] exposes `subscription.in_overage`, `auto_renew` and
  `current_period_end`. The [account-level guide][nw-account] confirms the
  pay-as-you-go exception when a subscription is in overage with auto-renew off.
  If a 429 names that exception and reports a lower concurrency, the cap that
  refusal reports governs until the period ends or a quota read confirms the
  exception cleared, even when it sits below a published ceiling: the refusal's
  own number describes the account as it stands, while the published number is
  the plan's promise. The owner decided this on 2026-11-19 ("Refusal's number
  (Recommended)"). The program does not hard-code the historical cap or
  extrapolate it to another provider or lane.

## Credentials and turn records

Each inference attempt uses the live model and request-time authentication
from `ModelRegistry.streamSimple`. The virtual models forward no credential at
all; their routers only read the catalog. The pool never forwards its
caller-supplied key, headers, or scoped environment. For quota GETs it uses
`ModelRegistry.getApiKeyAndHeaders`, only sends the bearer to the same
official origin as the configured model and resolved authentication, and
forbids redirects. Registry-resolved bearer headers are accepted; other resolved headers are not
forwarded to quota endpoints. No production code reads credential files or logs credential values,
raw quota bodies, or transcripts. What it does log is a routing decision's member and outcome, and
the text of a policy stop that moved a request on. The plugin's logger writes those records as
JSONL to the nearest `node_modules/.monochromatic/<day>/` folder, or to
`~/.local/state/monochromatic/log` when no project tree is above the working folder.

Pi records the physical provider and model in the assistant message's
`provider` and `model` fields, which for this route is the member that answered,
not the pool and not the virtual model. A summary can read those fields for every
assistant response, including tool continuations, rather than reading
the current selection. Discarded attempts, a 429, a transient status or a policy
stop that moved on, have no assistant record.
While the session is open, its active selection remains the virtual model, and a
reopened session restores it from the `model_change` entry: Pi keeps a virtual
change even though the answers after it name physical members, as long as that
virtual model is still registered. The pool models selected directly do not
survive a resume that way, because Pi sees only the physical answer; select
`rate-limit-fallback/writer` or `rate-limit-fallback/reviewer` again in that
case.

The pool advertises text input and no price. Its context window and output
limit, and the virtual models' declared pair, are the smallest its registered
members report, read from the session registry each time Pi asks for the
catalog, so Pi's compaction threshold fits every member. Before a session binds,
both advertise Pi's own fallback for a model that declares no window, 128,000
tokens and 16,384 output. A model captured in `--models` or `enabledModels`
before a session binds keeps that provisional window, because Pi composes a
scoped list from the catalog it read then. Members' own output caps and usage
are used on dispatch. If a member rejects a context, the error is passed through
as that member's own; under the virtual selection Pi then compacts and retries
the turn once, and under a directly selected pool model the turn ends in that
error.

## Verification

- `mise run //package/pi-plugin/rate-limit-fallback:test:unit` runs the
  unit suites against the built entry with stand-in providers and quota endpoints.
  It also checks the installed adapters' refusal strings through an in-memory transport,
  including the recorded OpenAI policy stop, which reaches Pi without a status prefix.
- `mise run //package/pi-plugin/rate-limit-fallback:verify:extension` runs two
  verifiers in one disposable, offline home. The first loads the built extension
  in a real Pi session through the SDK and checks request-time registry auth,
  persisted physical identity, that each pool's advertised window is its
  smallest registered member's, that a long transcript leaves no compaction
  record, and that a pool model held from before the session bound takes that
  window once it does. The second starts real `pi` command lines whose
  `settings.json` `packages` list holds this package directory and a stand-in
  member extension, which is how `~/.pi/agent/settings.json` installs it; it
  checks that the selection stays `rate-limit-fallback/writer`, that a 429, a 503
  and a Cloudflare 523 each move the request on inside one turn with no
  `auto_retry_start`, and that three padded turns make Pi compact and retry a
  member's context overflow, while the same turns with `writer-pool` selected
  directly recover nothing.
- `mise run //package/pi-plugin/rate-limit-fallback:lint:types` and
  `mise run //package/pi-plugin/rate-limit-fallback:lint:oxlint` check the package.
- `doc/captures.md` records every fixture text the suites replay, with its
  capture, the installed Pi source that produces it, or the statement that it is
  made up and the boundary it tests.

[openai-limits]: https://developers.openai.com/api/docs/guides/rate-limits
[handover]: ../../../doc/handover/cli-git-rust-implementation.md
[writers]: ../../../.agents/skills/pi-as-subagent/SKILL.md
[openai-admin]: https://developers.openai.com/api/reference/resources/admin/subresources/organization
[openai-codex]: https://developers.openai.com/codex/app-server
[qwen]: https://www.alibabacloud.com/help/en/model-studio/token-plan-faq
[go]: https://opencode.ai/docs/console/go.md
[go-models]: https://opencode.ai/docs/console/api/models
[synthetic-schema]: ../../../doc/troubleshooting/synthetic-quota-weekly-block-renamed.md
[synthetic-quotas]: https://dev.synthetic.new/docs/synthetic/quotas
[nw-rates]: https://docs.neuralwatt.com/guides/rate-limits.md
[nw-quota]: https://docs.neuralwatt.com/api/quota
[nw-account]: https://docs.neuralwatt.com/billing/account-levels.md
