# @monochromatic-dev/agent-harness-shared-model-selection

Shared model-selection logic for pi plugins and harness-neutral agent packages.

This package lives under `package/agent-harness-shared/` because multiple agent harnesses consume it:
 pi plugins (advisor, goal, thinking-defaults) and the harness-neutral advisor core
 shared by the pi and DSH advisor variants.
 It moved here from `package/pi-shared/model-selection` on 2026-10-20;
 that category required two pi consumers,
 while this one serves pi plus DSH.
 The `/pi-coding-agent` tier keeps its optional pi peers
 and is the deliberate exception to this category's zero-pi-import habit.
Actual pi extensions stay under `package/pi-plugin/`.

## Export tiers

The package uses subpath exports so lightweight consumers can import string and type helpers
without loading platform-coupled modules.

- `@monochromatic-dev/agent-harness-shared-model-selection` and `/core` export dependency-free helpers and types.
- `/scope` exports scope pattern,
   settings,
   and effective-scope helpers.
- `/cost` exports cost scoring and ranking helpers.
- `/budget` exports fast judge-model strategy and override selection helpers with injected auth callbacks.
- `/pi-coding-agent` exports wrappers that directly import `@earendil-works/pi-coding-agent`.

`@earendil-works/pi-ai` and `@earendil-works/pi-coding-agent` are optional peer dependencies.
Root and `/core` modules must not import either peer.

## Consumer boundaries

Advisor and goal can import heavier `/scope`,
 `/cost`,
 `/budget`,
 and `/pi-coding-agent` modules.
Thinking-defaults imports only root or `/core` helpers.

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
