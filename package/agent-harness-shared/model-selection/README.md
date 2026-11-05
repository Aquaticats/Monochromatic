# @monochromatic-dev/agent-harness-shared-model-selection

Shared model-selection logic for pi plugins and harness-neutral agent packages.

This package lives under `package/agent-harness-shared/` because multiple agent harnesses consume it:
 pi plugins (advisor, goal, thinking-defaults) and the harness-neutral advisor core
 shared by the pi and DSH advisor variants.
 It moved here from `package/pi-shared/model-selection` on 2026-10-20;
 that category required two pi consumers,
 while this one serves pi plus DSH.
 The package has no Pi runtime,
 peer,
 or development dependencies.
Actual pi extensions stay under `package/pi-plugin/`.

## Pi dependency trust

PJC:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Exports

The package root exports the built helpers;
workspace consumers use `/ts` for their source.
Both entries expose model identity,
scope resolution,
cost ranking,
budget selection,
and text token estimation without importing Pi.

The former `/pi-coding-agent` helper now lives in `src/token-estimate.ts`
(moved on 2026-11-05).
The earlier README's separate feature-subpath description was stale:
the manifest exposes only the root and `/ts`.

## Text token estimate

`estimateAdvisorInputTokens` estimates the combined system prompt,
the `\n\n` separator,
and serialized context.
It counts UTF-16 code units and rounds up once using `TOKEN_ESTIMATE_CHARS_PER_TOKEN`.
The implementation reads string lengths without constructing a combined message.
This preserves the former text-only Pi `1.5.0` wrapper's behavior;
it is a budgeting heuristic rather than a model tokenizer.

The built-entry regression rejects every Pi package during module resolution,
then imports the public entry and calls the estimator.

## Synchronous live scope

`readLiveScope()` uses the same live-scope precedence as `resolveEffectiveScope()`:
 a getter takes precedence over the property.
It accepts raw models and `{ model, thinkingLevel }` entries,
 normalizes their canonical slugs,
 and filters malformed entries.
An unavailable,
 non-array,
 or empty live value returns `NO_LIVE_SCOPE`.
Pi uses an empty cycle list for SDK sessions without an explicit live restriction.
`resolveEffectiveScope()` then checks argv,
 configured settings,
 and available models in that order.
Configured restrictions that match no models remain empty;
 a nonempty live list containing only malformed entries also remains empty.
Use this helper for a final dispatch check after asynchronous authentication.

## Fast judge-model ranking

Budget helpers retain their API names for compatibility,
 but automatic judge selection ranks by speed heuristic.
The selector first keeps the configured major-version families,
 then scores model id and display name tokens:
`highspeed` or `high-speed` > `fast` > `luna` > `flash` or `spark` > `terra` >
`turbo` > `nano` > `mini` > `haiku` > `lite` or `light` > no signal.
When no speed signal separates candidates,
 selection falls back to input cost and version.
