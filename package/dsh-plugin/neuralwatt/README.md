# dsh-plugin-neuralwatt

NeuralWatt provider for DSH:
the plugin owns the `neuralwatt` provider route
over the shipped `@deepseek-ai/dsh-llm-pi-ai` adapter,
with the model list
fetched live from the NeuralWatt API at activation.
Each fetched model carries
its accurate context window and its selectable thinking levels,
and a
checked-in snapshot of the same listing serves while the API is unreachable.

## Pi dependency trust

PJN:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## How it serves

Activation is one settings write plus one bounded async fetch:

1.  The plugin maps the checked-in snapshot and merges it into the `llm-pi-ai`
    entry's volatile `providers` field through the settings service,
    keyed as
    `neuralwatt`.
    The shipped pi-ai adapter reads that field and registers the
    route with its adapter,
    so the default model keeps streaming without
    waiting on the network.
2.  The mount fetch reads the public listing endpoint
    [the NeuralWatt listing endpoint][neuralwatt-listing] within a fixed time
    bound,
    maps it,
    and merges the fresh route the same way.
3.  A fetch failure,
    a non-listing body,
    or a listing mapping to no serviceable
    model logs the reason and keeps the served snapshot,
    so an unreachable
    endpoint never takes the route down.
4.  A write whose stored route already matches the mapped one is skipped,
    so an
    unchanged profile document is never rewritten.
5.  A write the host refuses because its reload transaction is still open
    (`HMR transactions cannot be nested`)
    is deferred, not failed: the whole mount refresh is re-run from the
    plugin-manager's `plugin-manager/changed` dispatch, which lands outside
    that transaction, so a `set_plugin`, `install_bundle`, or `set_bundle`
    re-trigger still lands the full route.

The plugin never applies a second pi-ai adapter.
The shipped `llm-pi-ai`
instance owns every route,
and its activation re-declares the whole
configurable provider catalog,
which a second instance cannot survive.
The
loader merges `providers` per provider key instead,
so hand-declared routes
beside `neuralwatt` keep standing untouched.

## Model mapping

The mapping itself lives in the shared core
`@monochromatic-dev/agent-harness-shared-neuralwatt-listing`
(`package/agent-harness-shared/neuralwatt-listing`),
extracted from this package on 2026-11-07
so the pi plugin projects the same facts;
this section documents the behavior the shared mapping serves.

Each listing entry maps to one pi-ai model profile:

- Entries the listing marks deprecated,
  or without an id,
  are skipped rather
  than served with wrong facts.
- `name` comes from `metadata.display_name`,
  falling back to the id.
- `contextWindow` comes from `metadata.limits.max_context_length`,
  then the
  entry's `max_model_len`;
  both absent leaves the field unset.
- `maxTokens` comes from `metadata.limits.max_output_tokens`;
  a null value
  leaves the field unset instead of guessing.
- `input` is `['text']`,
  or `['text', 'image']` when `capabilities.vision` is
  true.
- `reasoningEfforts` is `false` for a model `capabilities.reasoning` marks
  non-reasoning.
  Otherwise the keys are the thinking levels the listing's
  `metadata.reasoning.supported_efforts` spellings map to,
  each level sending
  its own spelling as its wire value.
  The spelling `none` maps to the `off`
  level and sends `reasoning_effort: "none"` when thinking is off.
  A model
  whose efforts include no level beyond `off` also declares `false`,
  since
  nothing is selectable.
- `metadata.reasoning.accepted_efforts` serves as the effort source only when
  `supported_efforts` is missing,
  minus the spellings
  `metadata.reasoning.effort_aliases` marks as aliases of another effort.
- An effort spelling this mapping cannot express is skipped instead of
  guessed,
  so a listing it does not know yet degrades to fewer offered levels.

## Thinking level dispatch

The route pins `compat.supportsReasoningEffort` so a selected thinking level
dispatches as the documented top-level `reasoning_effort` parameter.
The pin
stays for parity and determinism,
not necessity:
 a dispatch-capture probe on
2026-11-07T23:45:20-05:00
(commit `c6701d0112`)
measured pi-ai 1.5.0 auto-detection
still sending the mapped `reasoning_effort` wire value for
`https://api.neuralwatt.com/v1/`
with the `supportsReasoningEffort` and `thinkingFormat` pins removed.
The earlier claim that URL detection
finds no known effort format and silently drops reasoning from requests
was measured against the DSH adapter's older pi-ai context
and is retracted for pi-ai 1.5.0
(correction recorded 2026-11-07T23:46:44-05:00).
The pin remains as insurance because `detectCompat` is a URL heuristic:
 a host it special-cases,
such as `api.together.ai`,
would flip `supportsReasoningEffort` to `false`.

The route also pins `compat.supportsDeveloperRole: false` so the system prompt
travels as the classic `system` role.
`mimo-v2.6-pro` rejects the `developer`
role pi-ai otherwise picks for reasoning-capable models,
answering
`400 unsupported_message_role`;
the diagnosis lives in
[`neuralwatt-rejects-developer-role.md`](../../doc/troubleshooting/neuralwatt-rejects-developer-role.md).

The level dict carries the `off` level only when the listing says the model
thinks off by default (`default_enabled: false`).
The dispatch seam cannot
tell a default level apart from the off level:
pi-ai sends the model's off
wire whenever the caller names no effort,
so an offered `off` level on a
thinking-by-default model silently turned every untouched default into
`reasoning_effort: "none"`.
A thinking-by-default model therefore keeps the
`off` level out of its dict,
which makes naming no effort reach NeuralWatt as
the parameter's absence and the model run its own default effort;
the
diagnosis lives in
[`pi-ai-unset-reasoning-effort-dispatches-off.md`](../../doc/troubleshooting/pi-ai-unset-reasoning-effort-dispatches-off.md).

## Deliberately omitted scope

The listing's `metadata.pricing` block exists but is never read.
This provider
serves models and their capabilities;
it does not estimate cost,
price
requests,
or meter energy.
Request-level token accounting stays the harness's
concern.

## Wiring

`engines.dsh` declares the verified host version `0.2.2-rc.1` explicitly.
The exact prerelease limits the compatibility claim to the tested host;
revalidate activation before changing it.
DSH reads this field before peer dependencies,
so local directory links can retain the workspace's `catalog:` dependency specifications.
See the [startup diagnosis](../../../doc/troubleshooting/dsh-rc5-profile-startup.md).

The bundle is `@monochromatic-dev/dsh-plugin-neuralwatt`,
installed through
`plugin_manager` `install_bundle` like any workspace bundle.
The `neuralwatt`
route has exactly one owner,
so a profile `cordis.patch.yml` that still
declares a hand-written `neuralwatt` block under `llm-pi-ai` `providers` must
drop that block;
the plugin's route replaces it.
Credentials resolve the
`NEURALWATT_API_KEY` credential reference exactly as the hand-written block did.
That name is a credential reference,
not an environment requirement:
 it addresses a record in the DSH credentials service
(`$DSH_HOME/.credentials.yaml`,
written from the web Models page),
and the launching environment is only the fallback source
(`doc/agent/drive-dsh-as-a-subagent.md`).

The plugin writes its `neuralwatt` block into the profile's own
`cordis.patch.yml`,
the hand-edit surface,
under the `llm-pi-ai` row's
`providers` key.
The loader merges that field per provider key,
so the other
routes in the file survive,
and comments outside the row's `config` node
survive too.
The block is machine-managed but not machine-removed:
uninstalling the bundle leaves it behind,
and dropping it is a hand edit.

```ts
// package/agent-harness-shared/neuralwatt-listing/src/constants.ts
export const NEURALWATT_API_KEY_ENV = 'NEURALWATT_API_KEY';
```

`@monochromatic-dev/agent-harness-shared-neuralwatt-listing`'s `src/constants.ts` names the route key,
wire protocol,
endpoint base,
listing
endpoint,
credential reference,
and the listing fetch bound in milliseconds.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one from this directory with `mise run <task>`.

Unit tests import package behavior from the built `dist` entry,
so run `build`
before `test:unit` after source changes.

[neuralwatt-listing]: https://api.neuralwatt.com/v1/models
