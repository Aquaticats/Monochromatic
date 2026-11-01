# NeuralWatt 400s the developer role pi-ai sends for reasoning models (pi-ai 1.3.0, dsh-llm-pi-ai 0.2.1-rc.2)

A provider route whose models declare `reasoningEfforts` gets its leading
system prompt sent as `role: "developer"`,
 and NeuralWatt's `mimo-v2.6-pro`
answers `400 unsupported_message_role`,
 failing every turn on the route.

## Symptom

A turn on the `neuralwatt` route fails at the API boundary,
 while the same
model id served through other providers works:

```text // web GUI turn failure, 2026-10-22
Failed
This turn failed400: {"message":"Model 'mimo-v2.6-pro' does not support messages with role 'developer'. Use role 'system' instead.","type":"invalid_request_error","param":"messages","code":"unsupported_message_role"}
```

Surface pattern:
 any custom `openai-completions` route that
 (a) declares `reasoningEfforts` on its models,
 so pi-ai treats them as
reasoning-capable,
 and
 (b) leaves `compat.supportsDeveloperRole` unset,
 so
URL auto-detection decides.
 The failing variant names the model in the message
(`mimo-v2.6-pro` here);
 the passing variant is the same route with the
`system` role on the wire.

This appeared only after an accurate `reasoningEfforts` mapping landed.
 The
same model id served fine before,
 because a route declaring no `reasoningEfforts`
keeps `model.reasoning` false and pi-ai then always sends `system`.

## Root cause

The call chain,
 walked end to end.

Our listing mapping declares thinking levels per model
 (`package/dsh-plugin/neuralwatt/src/listing.ts`,
 `mappedServiceableModels`),
 which the shipped adapter turns into
`reasoning: true`:

```js // @deepseek-ai/dsh-llm-pi-ai/lib/index.js:587-605
if (efforts === void 0) return { reasoning: base?.reasoning ?? false };
if (efforts === false) return { reasoning: false };
// ...declared levels collapse here...
    reasoning: true,
```

pi-ai's OpenAI-completions serializer then chooses the instruction role,
and its compat schema documents the unset flag as auto-detected from the URL:

```js // @earendil-works/pi-ai/dist/api/openai-completions.js:901
const instructionRole = model.reasoning && compat.supportsDeveloperRole ? "developer" : "system";
```

```js // @earendil-works/pi-ai/dist/providers/compat-schema.js:112-114
supportsDeveloperRole: optionalCompatBoolean({
    description: "Whether the provider supports the developer role instead of system. Default: auto-detected from URL.",
}),
```

So on the custom host `https://api.neuralwatt.com/v1/` the auto-detection
resolved `supportsDeveloperRole` truthily,
 the model was reasoning-capable,
and the leading instruction message went out as `role: "developer"`.
 NeuralWatt
validates message roles per model and rejects it.
 pi-ai's own README names
this exact case and the switch that fixes it
 (`@earendil-works/pi-ai/README.md:1303`:
 set `compat.supportsDeveloperRole` to `false` for servers that do not
understand the `developer` role).

An earlier reading was wrong and is worth recording:
 the `developer` string
is not carried through from dsh's internal `developer` message role.
 The
adapter bridge maps both `developer` and `system` history to pi-ai `system`
messages
 (`@deepseek-ai/dsh-llm-pi-ai/lib/index.js:1332-1341`),
 and pi-ai
re-upgrades only the leading instruction role.
 The Responses variant applies
the same rule
 (`@earendil-works/pi-ai/dist/api/openai-responses-shared.js:131`).

## Verification

Versions under test:

- `@earendil-works/pi-ai@1.3.0`.
- `@deepseek-ai/dsh-llm-pi-ai@0.2.1-rc.2`.
- Harness `dsh` next build `v11/1483ad-18e0966c47886467-0`,
  and the plugin build at commit `589ded0b87` in `Monochromatic`.

Runnable harness:
 a fixture profile under the real `$DSH_HOME`
 (so the stored
`NEURALWATT_API_KEY` record in `~/.dsh/.credentials.yaml` resolves),
 created
from the `headless` template,
 with the bundle linked from the workspace and the
`credentials` entry enabled
 (the headless template ships it `disabled: true`,
which silently drops stored-credential resolution):

```sh // fixture harness, run in Monochromatic
dsh --profile nwfix --from-default-profile headless --dump-config
dsh plugin --profile nwfix add /var/home/user/Monochromatic/package/dsh-plugin/neuralwatt
dsh --profile nwfix --patch ~/temp/agent/nw-fix.patch.yml --json "reply with exactly: pong"
```

`~/temp/agent/nw-fix.patch.yml` enables `credentials` and points
`agent-default-model` at `neuralwatt/<model>`.

Measured results with the compat pin in place
 (the recorded runs are
`~/temp/agent/nw-fix*-out.txt`):

- `neuralwatt/mimo-v2.6-pro` (reasoning with an off level):
 `"text":"pong"`.
- `neuralwatt/glm-5.3` (mandatory reasoning):
 `"text":"pong"`.
- `neuralwatt/kimi-k3-fast` (non-reasoning):
 `"text":"pong"`.

The failing case before the pin is the verbatim GUI error in Symptom
 (same
host,
profile,
model,
 and stored credential;
 only the mapped `compat` changed).

Patterns that work cleanly:

- `compat: { supportsDeveloperRole: false }` on the provider route,
  sending
the classic `system` role for every model it serves.
- Leaving the flag unset on a host the URL auto-detection classifies
  correctly;
  the generated pi-ai catalogs pin `supportsDeveloperRole: false`
  explicitly for OpenAI-compatible hosts
  (`@earendil-works/pi-ai/dist/providers/data/nvidia.json`,
  `zai.json`,
  `together.json`,
  `deepseek.json`),
  which is the ecosystem norm.

Patterns that fail:

- Declaring `reasoningEfforts` while leaving `supportsDeveloperRole`
  to URL
auto-detection on a custom host
  (the failure in Symptom).
- Relying on dsh's internal `developer` history role to explain the wire
  role;
  the bridge normalizes it to pi-ai `system` first.

## Verified workarounds

Pin `compat: { supportsDeveloperRole: false }` in the provider profile
 (the
fix that landed in `package/dsh-plugin/neuralwatt/src/listing.ts`,
 commit
`589ded0b87`).
 Tradeoffs:
 the pin is provider-level,
 so every model on the route
gets the `system` role;
 three capability classes verified fine,
 but a future
NeuralWatt model that required `developer` would need the pin revisited.

Drop `reasoningEfforts` from the mapping.
 Tradeoffs:
 the wire role returns to
`system` for free,
 but the Models page loses context windows and thinking
levels and dispatch stops sending `reasoning_effort`,
 which is the regression
this provider exists to avoid.

## What does not work

- Per-model role control:
  `compat` is provider-level in the profile schema
  (`@deepseek-ai/dsh-llm-pi-ai/lib/index.js:1050-1053`);
  no per-model role
  switch exists.
- Interception at the plugin layer:
  a settings-feed plugin writes route
  profiles only;
  it cannot wrap serialization
  (the shipped adapter owns the
  request build).
- Swallowing the 400 and retrying:
  the rejection is deterministic per
  message shape,
  so retries repeat it.

## Upstream filing artifact

### Upstream filing decision

- `.out-of-scope/`:
  no exemption names `@earendil-works/pi-ai`;
  the decision
  below stands on constraint 1 alone.
- Duplicate search:
  not needed after constraint 1;
  pi-ai documents the switch
  for exactly this failure (`@earendil-works/pi-ai/README.md:1303`),
  so
  there is no defect to report or comment on.
- Constraint 1,
  really upstream's fault:
  no.
  The compat schema declares
  `supportsDeveloperRole` with URL auto-detection as the documented default
  (`compat-schema.js:113`),
  the README prescribes `false` for these servers,
  and the generated catalogs pin it for OpenAI-compatible hosts.
  The custom
  route simply did not pin it.
- Constraint 2,
  can upstream fix it:
  not applicable after constraint 1.
- Constraint 3,
  supporting this use case:
  yes,
  through the documented
  `compat.supportsDeveloperRole` switch,
  which is the landed fix.
- Constraint 4,
  welcome contribution:
  fails with constraint 1;
  a
  friendlier auto-detect heuristic is speculative,
  not a defect fix.
- Constraint 5,
  likely fix:
  not applicable after constraints 1 and 4.
- Constraint 6,
  prototyped minimal fix:
  not applicable.
  The compat pin is
  the fix and is verified end to end in this repository.

Decision:
 do not file.
 No draft is kept,
 because there is no defect to
describe;
 the pin and its verification are the durable record.
