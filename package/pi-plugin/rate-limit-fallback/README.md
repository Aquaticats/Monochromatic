# Rate limit fallback for Pi

`@monochromatic-dev/pi-plugin-rate-limit-fallback` adds the keyless
`rate-limit-fallback` provider to Pi. Select it with
`--provider=rate-limit-fallback --model=writer` or
`--provider=rate-limit-fallback --model=reviewer`. The extension registers
its catalog immediately and takes the live model registry from `session_start`.
It does not install itself or change your settings.

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
other non-429 errors do not trigger fallback. When an adapter does not expose a
response status, the pool reads it from the adapter's own wording: the
Responses adapter's `<provider> API error (429): …` label, which spells the
label `OpenAI` only for the provider named `openai`, or the Completions
adapter's `429: …`.

A policy stop follows the same exposure rule, and only OpenAI's own sentence
moves a request on. A member whose refusal opens with `This content was
flagged for possible cybersecurity risk` and shows no text, thinking, or tool
output before it moves the request to the next member with the same
transcript. Once output was shown, the stop ends the request and the message
names the member and keeps the advisory. Either way the plugin logs the
member's name and the stop's text, worded apart from a rate-limit move, and
the assistant record names the member that answered. If every member stops
before output, the request ends with the last stopped member's attributed
advisory. The sentence is matched at the start of the refusal, so a refusal
that merely quotes it does not move a request. Neither pool has an OpenAI
member since 2026-11-19, so this move cannot fire unless one is added, and
the rule and its tests stay for that case.

The project decided this on 2026-11-19 for [issue 6314][issue-6314]: the stop
moves the request on automatically ("Automatic (Recommended)"). That makes
rule `RKD` of [`AGENTS.md`][agents-md] automatic for runs on this provider;
resuming such a run on DeepSeek is the same move, done by hand.

These rules rest on unit tests that drive the built plugin against stand-in
members, and on the installed Responses adapter reporting the recorded stop
text byte for byte over an in-memory stream. The recorded advisory carries no
HTTP status, which is the form the installed adapter reports when a stream
fails without one; the capture fixes the text alone, not the wire event that
carried it. No policy move has been observed on a real provider yet; the
extension check covers the 429 handoff only.

OpenAI's `This content was flagged for possible`
sentence takes precedence over a 429 wherever it sits in the text, even if the
advisory following it is truncated or changes punctuation, so a 429 body that
quotes it ends the request instead of moving it. The shorter `possible risk`
wording and generic policy labels identify a stop only when there is no 429
status, and they never move a request on: such a stop ends it, as it did
before the move existed.
A stop that ends the request names the member and preserves the advisory,
including OpenAI's
remediation. It replaces only the spans matched by Pi's transient-error
classifier, such as standalone retryable status numbers, `rate limit`, and
`please retry your request`, with `[omitted]`. The surrounding wording and
punctuation are kept. This prevents Pi's outer retry from restarting a policy stop; a
move to the next member never reaches that classifier at all.

When no member answers, the pool returns a failure rather than a fallback. A
policy stop it set aside comes first, attributed to the member that stopped the
request: that run ends there, even though the later members were only
rate-limited, because returning their 429 would let Pi's outer retry restart at
the member that refused. With no stop set aside, the pool returns the last
member's 429 as it arrived. When no member was attempted, the pool names the held
members and their state: a published zero flex
floor, a local concurrency ceiling with the count in use, or a hold until
a provider-derived time. These local admission results do not carry a
fabricated 429 and are not retried by Pi: generic retry backoff cannot be
aligned to a local slot freeing, a provider reset, or a plan change. Pi's
configured SDK and agent retry layers still apply to physical errors they see. This plugin does not
change the subagent driver's retry or pool accounting.

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
timing stays in place for an adapter that starts exposing failed responses.

Authenticated quota endpoints are read on the first eligible attempt
and before returning to the member after a refusal, a known reset, or
a local concurrency rejection. There is no background polling. Probing
immediately on a 429 would make fallback wait on another request to the
same provider; the plugin uses response timing first and probes on reentry
instead. Concurrent requests in one session share a pending lookup.
A failed or timed-out read leaves existing evidence in place and calls for
another read on the next eligible attempt.

The quota read has its own provisional deadline of two seconds, capped at
half a positive caller-supplied model timeout (rounded down). The read
stops holding up dispatch at that deadline even if the transport is slow
to acknowledge cancellation. A request joining a shared lookup bounds its
own wait without canceling that lookup. The model request retains its own timeout.
Set the provisional bound from the live endpoint check before installing
this extension. The offline verification does not measure endpoint latency.

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
  plan-specific limit headers for this inference credential.
- **Synthetic.** The plugin reads `/v2/quotas` and accepts the current
  and previous weekly-block names. A depleted window holds the member
  only when it also reports a reset time. See the
  [recorded quota schema change][synthetic-schema]. A positive reading
  does not guarantee the next request will be admitted.
- **OpenCode Go.** [Go's usage guide][go] publishes window rules and
  estimates, with actual usage tracked in its console. The
  [model list API][go-models] publishes plan/model monthly limits, not
  remaining account allowance. No documented account quota endpoint
  or Go-specific quota headers were found in these references. The
  plugin uses a `Retry-After` only when an adapter exposes the failed response,
  rather than guessing room from a published estimate or starting a console
  session.
- **NeuralWatt.** [Rate limits][nw-rates] documents
  `/v1/rate_limits?model=…`, whose row names the canonical model. The
  `-flex` member takes the reported `flex_floor`, the ceiling for low-priority
  flex-lane requests, and the standard member takes `effective_concurrency`,
  the resolved cap for the model; the lane comes from the id the pool asks for.
  The plugin enforces it over in-flight calls in this Pi session; it
  cannot count another process's calls or reserve seats. [Quota][nw-quota]
  exposes `subscription.in_overage`, `auto_renew` and `current_period_end`.
  The [account-level guide][nw-account] confirms the pay-as-you-go
  exception when a subscription is in overage with auto-renew off.
  If a 429 names that exception and reports a lower concurrency,
  the lower of the published flex ceiling and the cap observed on that
  request governs until the period ends or a quota read confirms the
  exception cleared. The program does not hard-code
  the historical cap or extrapolate it to another provider or lane.

## Credentials and turn records

Each inference attempt uses the live model and request-time authentication
from `ModelRegistry.streamSimple`. The virtual provider never forwards its
caller-supplied key, headers, or scoped environment. For quota GETs it uses
`ModelRegistry.getApiKeyAndHeaders`, only sends the bearer to the same
official origin as the configured model and resolved authentication, and
forbids redirects. Registry-resolved bearer headers are accepted; other
  resolved headers are not forwarded to quota endpoints. No production code reads credential files or logs
credential values, raw quota bodies, or transcripts.

Pi records the physical provider and model in the assistant message's
`provider` and `model` fields. A summary can read those fields for every
assistant response, including tool continuations, rather than reading
the current selection. Discarded attempts, a 429 or a policy stop that moved
on, have no assistant record.
While the session is open, its active selection remains the pool. Pi
1.5.2 may restore the last physical answer in a reopened session instead;
explicitly select `rate-limit-fallback/writer` or `rate-limit-fallback/reviewer`
again when resuming into the pool.

The pool advertises text input and no price. Its context window and output
limit are the smallest its registered members report, read from the session
registry each time Pi asks for the catalog, so Pi's compaction threshold fits
every member; a provisional window stands in before a session binds. Members'
own output caps and usage are used on dispatch. If a member rejects a context,
the error is passed through, not replayed to another vendor.

## Verification

- `mise run //package/pi-plugin/rate-limit-fallback:test:unit` runs the
  unit suites against the built entry with stand-in providers and quota endpoints.
  It also checks the installed adapters' refusal strings through an in-memory transport,
  including the recorded OpenAI policy stop, which reaches Pi without a status prefix.
- `mise run //package/pi-plugin/rate-limit-fallback:verify:extension` loads
  the built extension in a real Pi session, in a disposable offline home,
  and checks request-time registry auth and persisted physical identity.
- `mise run //package/pi-plugin/rate-limit-fallback:lint:types` and
  `mise run //package/pi-plugin/rate-limit-fallback:lint:oxlint` check the package.

[openai-limits]: https://developers.openai.com/api/docs/guides/rate-limits
[issue-6314]: https://github.com/Aquaticat/Monochromatic/issues/6314
[agents-md]: ../../../AGENTS.md
[writers]: ../../../.agents/skills/pi-as-subagent/SKILL.md
[openai-admin]: https://developers.openai.com/api/reference/resources/admin/subresources/organization
[openai-codex]: https://developers.openai.com/codex/app-server
[qwen]: https://www.alibabacloud.com/help/en/model-studio/token-plan-faq
[go]: https://opencode.ai/docs/console/go.md
[go-models]: https://opencode.ai/docs/console/api/models
[synthetic-schema]: ../../../doc/troubleshooting/synthetic-quota-weekly-block-renamed.md
[nw-rates]: https://docs.neuralwatt.com/guides/rate-limits.md
[nw-quota]: https://docs.neuralwatt.com/api/quota
[nw-account]: https://docs.neuralwatt.com/billing/account-levels.md
