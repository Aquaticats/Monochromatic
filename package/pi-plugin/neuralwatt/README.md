# pi-plugin-neuralwatt

Pi extension that registers the NeuralWatt route as a first-class pi provider.

It serves the checked-in NeuralWatt listing snapshot immediately,
then refreshes
the catalog from the bounded live listing fetch at session start,
projecting each
model through the shared listing core so pi sees the same ids,
effort wire values,
and limits the DSH plugin reports.

Design,
measurements,
and every rejected alternative:
[`doc/planning/neuralwatt-pi-plugin.md`](../../../doc/planning/neuralwatt-pi-plugin.md).
The mapping facts live in
[`package/agent-harness-shared/neuralwatt-listing`](../../agent-harness-shared/neuralwatt-listing/README.md).

## Pi dependency trust

PJT:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package takes the rule as two Pi peers and no others:
`@earendil-works/pi-ai` and `@earendil-works/pi-coding-agent`,
both at `"*"`.
Every other dependency is a workspace package,
and the provider is hand-built from
pi's `Provider` shape instead of pi's `createProvider`,
which would persist model metadata this plugin refuses to write.

## What it does

- Registers provider `neuralwatt` (API `openai-completions`) at extension load and
 lists the projected snapshot right away,
 so the model picker works before any network.
- On `session_start` with reason `startup` or `reload`,
 asks pi's model registry to refresh
 the route with network access;
 every other reason,
 and `allowNetwork: false`,
 leaves the catalog untouched.
- On `session_shutdown`,
 aborts the in-flight refresh and keeps the served catalog.
- Projects each listing entry into pi's `Model`:
 declared context and output limits (pi schema defaults when the listing is silent),
 input modalities,
 pricing as the four pi cost rates (zero where the listing is silent),
 and reasoning efforts as pi's `thinkingLevelMap`.
- Pins the OpenAI-compatible dispatch:
 the system prompt travels as `system`,
 `reasoning_effort` takes the mapped wire value,
 and a declared `off` spelling is sent when no thinking level is selected.

## Credential homes

The credential resolves in this order:

1. Pi's own credential (passed key,
 `--api-key`,
 or pi's `auth.json`).
2. The sops/age-encrypted `.env.local.json` store through `env-local-json-core`,
 at `ENV_LOCAL_JSON_PATH` when set,
 otherwise the repository store derived from this package's own location.
3. `NEURALWATT_API_KEY` from the environment pi hands the auth context.
4. Nothing:
 the provider registers anyway,
 lists every model,
 and logs guidance naming the homes above.

The store read hides `NEURALWATT_API_KEY` from the reader's environment slice,
so a
stored value wins over a stale environment value.
No secret value is ever logged.

## Parity invariants

- Provider id `neuralwatt`,
 model ids,
 and effort wire values stay byte-identical to the DSH plugin's,
 so a user sees one catalog spelling in both harnesses.
- The plugin never writes `models-store.json`,
 `models.json`,
 or `enabledModels`,
 and never auto-enables a model:
 its registry publications carry an `update` and no `persist`.
- A failed or empty refresh keeps the last served catalog;
 it never demotes the snapshot.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/neuralwatt"
  ]
}
```

Then supply the credential through one of the homes in
[credential homes](#credential-homes):
 the repository `.env.local.json` store,
 pi's `auth.json`,
 `--api-key`,
 or `NEURALWATT_API_KEY`.
The `install:global` task applies the same settings edit with a guard that refuses
to touch unrelated settings or packages.

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
- `install:global`
 (pi CLI install plus a settings guard)

## Tests

`src/*.unit.test.ts` covers the projection,
 the credential chain and store discovery,
 the provider refresh lifecycle,
 the serialized dispatch payload
 (including a positive control where the role pin is lifted),
 registration and session lifecycle,
 and a never-persist byte-identity guard over
`models-store.json`,
`models.json`,
and `enabledModels`.
