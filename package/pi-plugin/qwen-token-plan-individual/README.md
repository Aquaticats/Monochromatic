# pi-plugin-qwen-token-plan-individual

Pi extension that registers the Qwen Token Plan Individual route as a
first-class pi provider,
replacing the builtin catalog while it is loaded.

It serves the checked-in listing snapshot immediately,
then refreshes the catalog from the bounded live listing fetch at session
start,
projecting each model through the shared listing core so pi sees the same ids,
effort wire values,
and limits the DSH plugin reports.
The builtin provider's static 10-model list stops serving as soon as this
extension registers,
and `unregisterProvider` restores it on removal.

Design,
measurements,
and every rejected alternative:
[`doc/planning/qwen-token-plan-individual-provider.md`](../../../doc/planning/qwen-token-plan-individual-provider.md).
The mapping facts live in
[`package/agent-harness-shared/qwen-token-plan-individual-listing`](../../agent-harness-shared/qwen-token-plan-individual-listing/README.md).

## Pi dependency trust

PJT:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package takes the rule as two Pi peers and no others:
`@earendil-works/pi-ai` and `@earendil-works/pi-coding-agent`,
both at `"*"`.
Every other dependency is a workspace package,
and the provider is hand-built from pi's `Provider` shape instead of pi's
`createProvider`,
which would persist model metadata this plugin refuses to write.

## What it does

- Registers provider `qwen-token-plan-individual` (API `openai-completions`) at
 extension load and lists the projected snapshot right away,
 so the model picker shows the full live set before any network.
- On `session_start` with reason `startup` or `reload`,
 asks pi's model registry to refresh the route with network access;
 every other reason,
 and `allowNetwork: false`,
 leaves the catalog untouched.
- On `session_shutdown`,
 aborts the in-flight refresh and keeps the served catalog.
- Projects each mapped listing entry into pi's `Model`:
 the facts table's context and output limits (pi schema defaults where no
 family covers an id),
 input modalities,
 the zero cost rates the builtin rows carry,
 reasoning as the three recorded states,
 and the efforts dict as pi's `thinkingLevelMap`.
- Pins the OpenAI-compatible dispatch the builtin rows pin:
 `thinkingFormat: "qwen"`,
 `supportsDeveloperRole: false`,
 `supportsStore: false`,
 `supportsStrictMode: true`,
 and `supportsReasoningEffort` true exactly when the model offers efforts.
- Skips the media ids the listing carries (image,
 video,
 TTS,
 ASR,
 realtime),
 which refuse chat completions.

## Credential homes

The credential resolves in this order:

1. Pi's own credential (passed key,
 `--api-key`,
 or the `qwen-token-plan-individual` entry in pi's `auth.json`,
 which keeps resolving because the override keeps the provider id).
2. `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY` from the environment pi hands the auth
 context.
3. `QWEN_TOKEN_PLAN_API_KEY` from the same environment,
 the builtin provider's own name kept as a fallback.
4. Nothing:
 the provider registers anyway,
 lists every model,
 serves the snapshot,
 and logs guidance naming the homes above.

There is no encrypted store in this chain (owner decision,
`doc/planning/qwen-token-plan-individual-provider.md`,
"Install and credential seams"),
unlike `package/pi-plugin/neuralwatt`.
No secret value is ever logged.

## Parity and settings invariants

- Provider id `qwen-token-plan-individual`,
 model ids,
 and effort wire values stay byte-identical to the DSH plugin's and to the
 builtin rows',
 so `qwen-token-plan-individual/<model>` slugs keep working and a user sees one
 catalog spelling in both harnesses.
- For every id the builtin catalog also serves,
 the projected row equals the builtin row value for value
 (id,
 name,
 contextWindow,
 maxTokens,
 input,
 reasoning,
 thinkingLevelMap,
 compat,
 cost,
 type,
 api,
 baseUrl,
 provider,
 inputLimits).
 The characterization fixture pins those rows so a pi-ai upgrade cannot change
 one silently.
- The plugin never writes `models-store.json`,
 `models.json`,
 or `enabledModels`,
 and never auto-enables a model:
 its registry publications carry an `update` and no `persist`.
- A failed,
 empty,
 or media-only refresh keeps the last served catalog;
 it never demotes the snapshot.
- The `install:global` task is the only settings seam:
 it runs `pi install` and fails on any `~/.pi/agent/settings.json` change
 outside the `packages` array,
 so nothing else the owner configured is touched.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/qwen-token-plan-individual"
  ]
}
```

Then supply the credential through one of the homes in
[credential homes](#credential-homes):
 pi's `auth.json`,
 `--api-key`,
 `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`,
 or `QWEN_TOKEN_PLAN_API_KEY`.
The `install:global` task applies the same settings edit with a guard that
refuses to touch unrelated settings or packages.

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
- `verify:host`
 (gated by `QWEN_TOKEN_PLAN_INDIVIDUAL_VERIFY_HOST=1`,
 boots pi RPC and `pi --list-models` in throwaway homes and asserts the served
 set,
 the never-persist guard,
 and the projection facts)
- `fixture:builtin-catalog`
 (regenerates `src/builtin-catalog.fixture.json` from the installed pi-ai
 catalog)
- `install:global`
 (pi CLI install plus a settings guard)

## Tests

`src/*.unit.test.ts` covers the projection
 (golden equality with the builtin rows,
 the three reasoning states,
 the compat pins,
 and the ids the builtin lacks),
 the credential chain and its trimming rules,
 the provider refresh lifecycle and its listing credential,
 the serialized dispatch payload
 (including a positive control where the role pin is lifted),
 registration and session lifecycle,
 the never-persist byte-identity guard over `models-store.json`,
 `models.json`,
 and `enabledModels`,
 and the characterization fixture over the installed builtin rows.
