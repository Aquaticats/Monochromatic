# Translation repair multi-provider: the second provider built and verified live on 2026-08-24

The pieces that landed on 2026-08-24 to put Charm Hyper behind the existing guards:
the system prompt,
the transport confirmed on the wire,
the client,
the budget view and the router,
the per-model concurrency measurement,
the saturation race,
the live check at the production seam,
the cross-provider re-ask and the wiring;
plus the one correction to the reader sub-roster.

Part of [Translation repair: two providers, and the quota reader that was never wired](translation-repair-multi-provider.md),
which lists every file of this record.

## The system prompt the owner asked for

The owner's instruction was that a detailed system prompt carrying the full tool schema is required,
because some model and provider pairs emit the wrong tool-call format without one.

`renderToolSystemPrompt` states the caller's instruction first,
then the whole schema as JSON,
then a list of format rules.
The rules are not filler.
Each line names a shape a model has been seen to emit instead of a tool call:
the answer as fenced text,
the arguments as a JSON string rather than an object,
the object wrapped in one more envelope key,
renamed properties,
a required field dropped because its honest value was empty,
a second call carrying the rest.

A test checks the tool name in BOTH renderings rather than each alone,
because two renderings of one schema can drift
and a drift teaches a model to call a tool that is not the one being offered.

## Correction: the reader sub-roster is five, not six

The option the owner accepted was worded "All 6 read,
and re-derive self-certification",
and the six in that wording was mine,
not a measurement.
Deriving the set from both catalogs gives FIVE:
`hf:zai-org/GLM-5.2`,
`hf:Qwen/Qwen3.8-27B`,
`hf:moonshotai/Kimi-K3`,
`qwen3.8-max`,
`minimax-m3`.
The substance of the instruction is unchanged and is what landed:
readers are derived from what the catalogs report rather than listed by hand,
and self-certification is re-derived.
The lane still goes from two readers to five.

## Reading is narrower than talking

`hf:zai-org/GLM-5.2` reads pictures on Charm Hyper and does NOT on Synthetic.
The same model,
the same weights,
a different serving stack,
and each catalog reports its own side correctly.

So a call carrying a picture reaches fewer providers than the same model's text call does,
and `visionReachOf` answers that question separately from `reachOf`.
Asking one question for both would either send a picture where it cannot be read,
or refuse one that can.

## The transport, confirmed live on 2026-08-24

These were measured but never written down,
which is how the second session came to need re-measuring:

-   `POST https://hyper.charm.land/v1/messages`,
    with `Authorization: Bearer <key>`,
    `content-type: application/json` and `anthropic-version: 2023-06-01`.
    An `x-api-key` header,
    which is what Anthropic's own API takes,
    answers `401 missing authorization`.
-   `GET https://hyper.charm.land/v1/credits` with the same auth returns `{"balance": N}`;
    read 243 on 2026-08-24,
    down from 249 earlier the same day.
-   The stream carries `{"type":"ping"}` keep-alive frames between real events.
    Both readers already ignore them and count zero unreadable frames over twenty of them,
    but that was accidental rather than intended,
    so both now pin it with a case taken off the wire.
-   A forced tool call arrives as one `input_json_delta` whose `partial_json` is the whole answer object,
    with `stop_reason: "tool_use"` on `message_delta` and usage there rather than on `message_start`.

## The client, the budget view, and the router: landed 2026-08-24

`createHyperClient` in `hyper-client.ts` assembles the parts that were built and unreachable:
`buildAnthropicBody` for the body,
`extractAnthropicCompletion` for the drained reply,
`wireFormat: 'anthropic'` on the exchange so the guards read the stream,
and the endpoint facts confirmed live.

It raises `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19),
named after the other provider,
DELIBERATELY.
`benchmark.ts` branches on `error instanceof SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
to read a status off a failed call,
and a fresh class here would make that site blind
to exactly the provider added to survive the other one's exhaustion.
The name is wrong and the behaviour is right;
renaming it is held with the `SyntheticClient` rename
(`SyntheticClient` was renamed `ModelClient` on 2026-10-19).

The JSON ladder moved to `chat-json-outcome.ts` rather than being copied.
None of it is provider-specific:
it reads text a model wrote and decides whether that text is an answer,
a refusal,
or a mismatch.
Every step in it stands for a defect found live,
and a copy would have taken none of the next ones.

`provider-budget.ts` is the first thing that has ever consumed the quota reader built on 2026-07-16.
It reads both meters,
caches the view,
and lets a refusal correct it.
AN UNREADABLE METER READS AS SPENDABLE:
a budget endpoint that times out is a monitoring failure,
and treating it as exhaustion converts that into an outage.
A refusal is stickier than a meter reading,
because a meter can lag a 429 by its own refresh interval;
the cooldown is one-directional and can only hold a provider out.

`provider-router.ts` is the client seam `routeProviderFor` now threads through.
A budget refusal marks that provider and asks the other one,
exactly once.
A failure that is NOT about budget is re-raised untouched,
because spending the second provider's money on someone else's fault
hides the fault and pays for it twice.
Both 429 and 402 count as budget refusals:
a subscription reports exhaustion as a rate limit and a balance reports it as payment due.

Pictures route by `visionReachOf` rather than `reachOf`,
which is what the "reading is narrower than talking" finding demands.

## Per-model concurrency: measured, not inherited

The second provider DOES NOT SERIALISE PER MODEL.
Measured live on 2026-08-24 against `minimax-m3`:

-    4 concurrent calls:
     all ok,
     burst 1512 ms.
-    8 concurrent calls:
     all ok,
     burst 2128 ms.
-   16 concurrent calls:
    all ok,
    burst 1589 ms.
-   32 concurrent calls:
    all ok,
    burst 2482 ms.

Single-call band over five runs:
994 to 1641 ms.
Every burst finished in about the time ONE call takes;
serialised,
the 32 would have taken some 40 seconds.
Zero refusals at any width.

The default moves from the other provider's 1 to 8.
It is held below the proven 32 because the probe sent a two-line prompt
and a corpus call carries orders of magnitude more:
what was measured is that the provider accepts the width,
not that 32 large bodies stream at once through our own drain and guards.

The balance did not move across roughly 50 calls,
so `hyperIsDry` reading `balance <= 0` detects exhaustion but not approach to it.

## A race the saturation test caught

The router's slot count rose at DISPATCH.
Two calls choosing at once both resume from the budget read before either has been sent,
so both saw a free slot and both went to the same provider.
The count now moves in the same synchronous step as the decision,
with nothing awaited between reading it and incrementing it,
and the slot is released through a `Disposable` rather than a `finally`
so a later early return cannot skip it.

## Verified live at the production seam, 2026-08-24

`createRunClient()` with both keys,
driven from a consuming script.
The wire label each call carried names the provider that served it,
because the two providers spell a shared model differently:

-   `hf:moonshotai/Kimi-K3` -> Synthetic,
    the roster spelling.
    A shared model with budget goes to the preferred provider.
-   `deepseek-v4-flash-0731` -> Charm Hyper,
    its own spelling.
    The only provider that serves it.
-   `hf:zai-org/GLM-5.2` -> Synthetic,
    the roster spelling.
    Shared,
    and this was a TEXT call;
    the same model carrying a picture routes to Hyper instead.
-   `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` -> Synthetic.
    The only provider that serves it,
    and the router did not try Hyper.

All four returned schema-valid answers.
The `quotas` passthrough answered five-hour 2735/2750,
weekly 99.797%.

The budget layer read each meter ONCE across the four calls,
one `quotas` and one `credits`,
which is the caching working rather than a coincidence:
a second `quotas` line appears only because the probe called the passthrough itself.

## The cross-provider re-ask: landed 2026-08-24

`chatJson` on the routing client reads its outcome,
and a non-conformant one from a model BOTH providers serve is re-asked on the other stack.

Why this is worth a second call rather than a retry:
the two providers extract structure by genuinely different mechanisms,
a forced tool on one and a `response_format` on the other,
so the same weights can conform on one serving stack and not the other.

It is not a budget failover and does not pretend to be one.
A bad answer marks nobody as refusing.
The re-ask is skipped where the other provider does not serve the model,
which is the invalid-candidate re-ask the policy names,
and skipped where the other provider has no budget.
When both stacks disagree with the schema,
the PREFERRED provider's answer is returned,
because the caller's own handling is written against it;
both are logged.

## Wiring: landed 2026-08-24

`createRunClient` is the one factory every corpus-run entrypoint calls,
so the routing client reaches all of them at once.
The returned surface is unchanged,
`quotas` included,
so no caller and not the bench recorder needed touching.

The routing client deliberately does NOT offer `quotas`:
the two providers meter differently and there is no single reading.
The wiring layer is where the knowledge that `quotas` means the Synthetic meter belongs.

The second key is OPTIONAL and its absence is LOUD.
Refusing to start would stop a run the first provider can serve alone;
starting silently would hide a setup mistake until a 429 storm read as a provider outage.

## Still to build

-   The calibration pass that picks the narrow writer set from the ten,
    which also settles the provisional third editor and refiner seat.
