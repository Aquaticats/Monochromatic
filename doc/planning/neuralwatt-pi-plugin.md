# NeuralWatt plugin in pi

Proposal for the owner,
written 2026-11-07T22:30:07-05:00
from the grilling interview opened 2026-11-07T22:02:14-05:00,
its repo and installed-API measurements,
and two cross-vendor design reviews folded in before confirmation.
The owner accepted this proposal 2026-11-07T22:33:58-05:00,
which unlocked implementation.

## Contents

- [Purpose](#purpose)
- [What lands](#what-lands)
- [Settled decisions](#settled-decisions)
- [Dependencies](#dependencies)
- [Identity and behavior invariants](#identity-and-behavior-invariants)
- [Install and credential migration](#install-and-credential-migration)
- [Verification](#verification)
- [Pre-ship probes](#pre-ship-probes)
- [Accepted risks and limits](#accepted-risks-and-limits)
- [Out of scope](#out-of-scope)
- [Pending doc edits](#pending-doc-edits)
- [Interview record](#interview-record)
- [Next action](#next-action)

## Purpose

Make the NeuralWatt LLM provider usable in the pi coding agent
(`@earendil-works/pi-coding-agent` and `@earendil-works/pi-ai`,
both 1.5.0)
with behavioral parity to the DSH plugin [`package/dsh-plugin/neuralwatt`](../../package/dsh-plugin/neuralwatt),
which today serves the `neuralwatt` route only inside DSH.
pi currently carries no NeuralWatt provider at all,
measured across `~/.pi/agent/settings.json`,
`models.json`,
and `models-store.json`.

## What lands

The mapping logic is owned once in a harness-neutral core,
with a thin projection per harness.

- [`package/agent-harness-shared/neuralwatt-listing`](../../package/agent-harness-shared/neuralwatt-listing)
  (`@monochromatic-dev/agent-harness-shared-neuralwatt-listing`):
  the listing-to-facts mapping
  (skip rules,
  display-name fallback,
  context and max-output fallback chains,
  input modalities,
  the offered efforts dict and its `off` rule),
  `expandThinkingLevelMap` for the pi projection,
  the reduced snapshot with its fetch-date header and pricing fields,
  the bounded listing fetch
  (`GET https://api.neuralwatt.com/v1/models`,
  `NEURALWATT_LISTING_TIMEOUT_MS` in `src/constants.ts`),
  shared constants
  (listing URL,
  base URL,
  timeout,
  effort table,
  and the `NEURALWATT_API_KEY` credential name),
  and a `refresh-snapshot` mise task that rewrites the chunks and prints drift.
  It carries no pi types and no DSH types.
- [`package/dsh-plugin/neuralwatt`](../../package/dsh-plugin/neuralwatt):
  behavior and export surface unchanged.
  It keeps the Cordis `inject`/`apply` entry (`src/index.ts`),
  the `llm-pi-ai` settings seam with its `sameJsonShape` idempotence
  (`src/providers.ts`),
  and a facts-to-`PiAiModelProfile` projection;
  its re-exports point at the shared core.
- [`package/pi-plugin/neuralwatt`](../../package/pi-plugin/neuralwatt)
  (`@monochromatic-dev/pi-plugin-neuralwatt`):
  the pi extension.
  A default-export factory registers a hand-built `Provider`
  through `pi.registerProvider`,
  the shape [`package/pi-plugin/openai-fast`](../../package/pi-plugin/openai-fast)
  already runs (`src/index.ts`).
  Snapshot baseline models first;
  `refreshModels(context)` performs the bounded fetch and
  `publish({ update })`,
  never `persist`;
  refresh on `session_start` for startup and reload;
  teardown on `session_shutdown`;
  auth resolves pi credential,
  then the encrypted store,
  then `process.env.NEURALWATT_API_KEY`,
  then guidance logging;
  cold start registers the provider anyway.
  Package boilerplate mirrors siblings:
  `README.md` with a fresh trust-rule code,
  `mise.toml`,
  rolldown config,
  `tsconfig.json`,
  and the `"pi": { "extensions": [...] }` manifest.

## Settled decisions

### Owner answers

Recorded under the interview question ids.

- `home`:
  a pi extension in this repo,
  installed in the owner's pi.
  No upstreaming and no npm publishing in this plan.
- `parity`:
  full behavioral parity.
  Snapshot first,
  bounded live listing fetch,
  every mapping rule of `package/dsh-plugin/neuralwatt/src/listing.ts`,
  and the compat pins.
- `sharing`:
  extract the shared core into
  `package/agent-harness-shared/neuralwatt-listing`,
  consumed by both plugins.
  The advisor port is the precedent:
  `package/dsh-plugin/advisor/package.json` and
  `package/pi-plugin/advisor/package.json` both consume
  `@monochromatic-dev/agent-harness-shared-advisor`.
- `persist`:
  never persist.
  Publication is `publish({ update })` with `persist` omitted
  (`@earendil-works/pi-ai` 1.5.0 `ModelsPublication` documents
  "Omit to leave storage unchanged"),
  and user stores stay byte-identical.
- `spend`:
  verification may run a bounded live smoke against the NeuralWatt API,
  within the owner's "use NeuralWatt normally" ruling
  ([`doc/decision/translation-repair-neuralwatt-provider.md`](../decision/translation-repair-neuralwatt-provider.md)).
- `peers`:
  `@earendil-works/*` peers declare `"*"`,
  with the characterization fixture and `verify:host`
  as the drift tripwire and the accepted risk recorded in this plan.
- `cred-home`:
  the pi extension sources the credential
  from the sops/age-encrypted `.env.local.json`
  through `@monochromatic-dev/env-local-json-core`,
  the accepted one-credential-home decision
  ([`doc/decision/env-local-json-store-loading.md`](../decision/env-local-json-store-loading.md)),
  with pi's `--api-key` and `auth.json` overriding as pi-standard.
- `cold-start`:
  with no credential from any source,
  register the provider anyway and log guidance,
  matching DSH's `MISSING_CREDENTIAL` at request time.
- `snapshot-off`:
  the snapshot reducer keeps `reasoning.default_enabled`
  so the next `refresh-snapshot` run closes the offline `off`-level gap
  (concrete case `gemma-4-31b`),
  while today's checked-in snapshot stays byte-identical
  and DSH behavior is unchanged until that refresh.

### Recorded adoptions

Veto-open;
each was recorded during the interview and none was objected to.

- Provider key `neuralwatt`,
  model ids,
  and effort wire values stay
  byte-identical across harnesses,
  so `neuralwatt/<model>` slugs work in both.
- The extension never writes `enabledModels`,
  `models.json`,
  or `models-store.json`.
- No extras beyond parity:
  no slash commands,
  no quota surfaces,
  no UI.
  DSH `locale/` and `icon.svg` stay DSH-bundle-only.
- Compat pins:
  `supportsDeveloperRole: false`,
  `thinkingFormat: "openai"`,
  `supportsReasoningEffort: true`,
  and `maxTokensField` decided by probe.
  The listing itself corroborates the pins
  (`capabilities.developer_role: false`,
  `system_role: true`,
  `reasoning_effort: true`).
- Thinking-level expansion mirrors `dsh-llm-pi-ai`
  (`resolveModelReasoning` in `lib/index.js`):
  declared levels carry their wire spelling,
  declared `off` with a value sends it,
  declared `off` with `null` stays absent ("send nothing"),
  undeclared levels pin `null`.
  `package/dsh-plugin/neuralwatt/src/listing.ts` keeps offering `off`
  only when the listing says thinking starts disabled.
- pi's required `Model` fields:
  `cost` maps from `metadata.pricing` per-million fields
  (zeros where the listing says `null`),
  and `contextWindow` and `maxTokens` fall back to pi's schema defaults
  128000 and 16384 where the listing omits them.
- Imports use `@earendil-works/pi-ai/models` and
  `@earendil-works/pi-ai/api/openai-completions.lazy`,
  never `@earendil-works/pi-ai/compat`,
  whose installed `dist/compat.d.ts` calls itself temporary
  and slated for deletion.
- valibot is rejected for the shared core;
  the mapping keeps manual `isRecord` narrowing.
- sinon needs no new pick:
  `package/test-harness/core` already ships it.
- Payload capture reuses
  `package/pi-plugin/openai-fast/src/host-fixture-http.ts`.
- Each compat pin gets a fail-first guard test:
  remove the pin,
  observe the failure,
  restore it.
- Logger tags are renamed per the tagging rule when modules move.
- The credential seam remembers its store read
 (`package/pi-plugin/neuralwatt/src/credential-memory.ts`):
 one decryption per
 `CREDENTIAL_REVALIDATION_INTERVAL_MS`,
 revalidation behind the request,
 the last good key kept when a read fails,
 one shared read for concurrent resolutions,
 and a first read retried inside `STORE_READ_DEADLINE_MS`.
 Measured on 2026-11-09:
 one warm decryption of the repository store costs 17.6 to 23.4 ms over
 eleven reads and 1.1 s on the first read of a process,
 while pi resolves the credential once per request and twice per
 availability refresh,
 so per-request decryption spent hundreds of sops invocations in a
 142-turn session and one timeout ended it with
 `Provider is not configured`.
 The failure mode and its verification live in
 [`doc/troubleshooting/pi-neuralwatt-encrypted-store-timeout.md`](../../../doc/troubleshooting/pi-neuralwatt-encrypted-store-timeout.md).
 Veto-open:
 the revalidation window is what a rotated key waits for,
 because pi-ai calls `onResponse` only after a successful status,
 so no provider callback sees a rejected key.
- Snapshot staleness stays visible through the fetch-date header
  and `refresh-snapshot` drift printing;
  no hard freshness limit ships,
  since no measured cadence justifies one.

## Dependencies

- `@earendil-works/pi-ai` and `@earendil-works/pi-coding-agent`:
  peers `"*"`,
  dev-dependencies `catalog:`
  (`pnpm-workspace.yaml` pins `>=1.5.0`).
- `@monochromatic-dev/env-local-json-core`:
  workspace;
  the credential store reader,
  already proven inside a pi extension by
  `package/search-fetch/pi-plugin`.
- `@monochromatic-dev/logger-core` and
  `@monochromatic-dev/caught-value-core`:
  workspace;
  logging and value narrowing,
  matching the DSH plugin.
- `@monochromatic-dev/test-harness-core`,
  `config-rolldown`,
  and `config-typescript`:
  workspace dev tooling,
  matching siblings.
- Native `fetch` with `AbortSignal.timeout` for the listing request.

Rejected dependencies:

- valibot:
  bounded listing input needs no schema library.
- HTTP client libraries:
  native fetch suffices.
- `@earendil-works/pi-ai/compat`:
  a temporary entrypoint.

## Identity and behavior invariants

- Route key `neuralwatt` in both harnesses.
- Model ids and display names follow the listing mapping exactly.
- Effort wire values are the listing spellings;
  `none` dispatches as `reasoning_effort: "none"`.
- The system prompt travels as role `system`,
  fixing the `400 unsupported_message_role` documented in
  [`doc/troubleshooting/neuralwatt-rejects-developer-role.md`](../troubleshooting/neuralwatt-rejects-developer-role.md).
- Offline behavior:
  the checked-in snapshot serves whenever the fetch fails,
  times out,
  or maps to zero serviceable models.

## Install and credential migration

- Install with
  `pi install /var/home/user/Monochromatic/package/pi-plugin/neuralwatt`,
  last in the `packages` array of `~/.pi/agent/settings.json`,
  through a guarded task that snapshots settings
  and fails on any change outside `packages`,
  tolerating pi's home-relative path normalization
  (conventions recorded in
  [`package/pi-plugin/model-retirement/README.md`](../../package/pi-plugin/model-retirement/README.md)).
- One-time credential migration:
  satisfied by the owner,
  who generated a new NeuralWatt key
  and added `NEURALWATT_API_KEY` to `.env.local.json` themselves
  (store entry first written 2026-11-08T03:29:38Z per `sops.lastmodified`).
  The key is a dedicated mint,
  distinct from the DSH credentials service record
  in `~/.dsh/.credentials.yaml`
  ([`doc/agent/drive-dsh-as-a-subagent.md`](../agent/drive-dsh-as-a-subagent.md))
  and from `TRANSLATION_REPAIR_NEURALWATT_API_KEY`,
  so pi draws on its own credential.
  Validated read-only against `GET https://api.neuralwatt.com/v1/quota`
  (status 200,
  JSON body).
  The store is gitignored (`.gitignore`,
  `*.local.*`),
  and no value is ever logged,
  echoed,
  or committed.
- The store is re-read lazily on each resolve,
  so a key added later takes effect without restarting pi.
- Store resolution for the extension:
  `readEnvLocalJson` is called without an explicit path,
  so the library honors `ENV_LOCAL_JSON_PATH`
  (host-wired at `~/.bashrc:57`)
  and then its own repository-root discovery;
  an `import.meta.url`-derived repo-store path serves as the fallback
  when pi is launched without a shell environment,
  used only when it validates as a file,
  never as a hard path error.

## Verification

Named suites and gates.

- Unit suites:
  the mapping,
  fetch,
  and snapshot tests move to the shared core;
  the settings-seam tests stay with the DSH plugin;
  the pi plugin gains projection and registration tests.
- `verify:extension` against a fake `ExtensionAPI`,
  as `package/pi-plugin/thinking-default` runs it.
- Characterization fixture plus `verify:host`
  (pi RPC in a throwaway home),
  the recorded mitigation for pi's extension API carrying
  no stability guarantee
  ([`doc/decision/pi-model-retirement.md`](../decision/pi-model-retirement.md)).
- Publication replacement assertion:
  a model removed from the live listing disappears from the catalog
  after refresh.
- Expansion parity characterization:
  one offered dict through the real `dsh-llm-pi-ai` adapter
  and through `expandThinkingLevelMap`,
  asserting identical maps.
- Never-persist guard:
  `models-store.json`,
  `models.json`,
  and `enabledModels`
  byte-identical before and after refresh.
- Register-anyway probe:
  a provider whose auth resolves nothing still lists models in pi 1.5.0.
- Store-discovery test:
  the credential resolves from an arbitrary pi working directory.
- DSH-inert assertion:
  the DSH projection ignores snapshot pricing,
  so the idempotence guard never rewrites on upgrade.
- Live smoke on `neuralwatt/mimo-v2.6-pro` with a thinking level selected,
  payload-captured,
  asserting the `reasoning_effort` wire value and role `system`,
  plus one offline run proving snapshot fallback.

## Pre-ship probes

- `maxTokensField`:
  probed 2026-11-08T03:39:59Z on `nw-small` with the owner-minted key:
  the API accepts both `max_completion_tokens` and `max_tokens`
  (status 200 for each),
  so pi's auto-detected default stands unpinned
  and the pin list keeps only verified differences.
- With-pin and without-pin reasoning dispatch positive control,
  settling the discrepancy between the DSH README's silent-drop claim
  and pi-ai 1.5.0 auto-detecting `thinkingFormat: "openai"`
  for unknown hosts.
- `off`-level rule re-proof on pi-ai 1.5.0
  (`thinking_level_select` and `clampThinkingLevel` era).
  The rule ships as conservative parity now;
  a successful re-proof can only relax it later.

## Gate results (2026-11-08T00:06:46-05:00)

Measured by the lead after the teammates' commits,
each run named with its backing.

- Suites,
  lead-run in cgroup bounds:
  shared core 53 cases in 7 suites (`d65e987c30` adds the reduction cases),
  DSH plugin 17 cases plus the expansion-parity characterization
  (`4354eeffc8`) with the settings-seam tests unedited,
  pi extension 47 cases in 13 suites plus the without-pin probe case
  (`c6701d0112`).
- `verify:extension`:
  `PASS neuralwatt extension: provider:neuralwatt, event:session_start,
  event:session_shutdown (route neuralwatt)`.
- Install:
  guarded `install:global` passed its non-package-settings and
  unrelated-packages assertions,
  and the entry landed last in `packages`
  as pi's home-relative path.
- Live smoke:
  `pi --print --model neuralwatt/mimo-v2.6-pro:medium` answered the prompt,
  which the documented `400 unsupported_message_role` makes the role-pin
  acceptance test;
  the `reasoning_effort` wire values,
  the off dispatch,
  and `system` versus `developer`
  are asserted against pi's real serialization
  in the dispatch-payload suite.
  In-flight body capture through pi's `http-dispatcher` was not achieved:
  a `globalThis.fetch` wrapper,
  a `node:http`/`node:https` request wrapper,
  and `--mode rpc` all stayed silent (their exact failures recorded here
  rather than claimed away).
- Offline gate,
  in a network namespace with the positive control `curl-exit=7`:
  `pi --list-models` serves the 27 neuralwatt models from the snapshot,
  and a completion fails gracefully with
  "live listing refresh failed,
  so the served catalog stands".
  A first attempt used `IPAddressDeny=any`,
  which the same positive control proved ineffective on this host
  (`curl-exit=0`) and is discarded as evidence.
- Probes:
  `maxTokensField` accepts both spellings (no pin);
  the two reasoning pins measured redundant-but-kept-for-parity
  (`c6701d0112`),
  the role pin load-bearing under its positive control;
  the `off`-level diagnosis confirmed on pi-ai 1.5.0
  (an unselected level sends the off wire value),
  so the rule stays needed,
  and the DSH README's silent-drop claim was retracted (`3fb3feed65`).
- `env-local-json-core` fix `5c80f0fd9e`:
  the store reader joined sops stdout at `exit` and returned empty
  documents under load (8 of 110 reads in one pi startup);
  it now waits for `close`,
  with a deterministic guard shown failing then passing,
  and three pi censuses went from 4/9/4 failures to 0/0/0
  after the extension bundle rebuilt.
- Open gate:
  `verify:host` (throwaway-home pi RPC) was not run as its own gate;
  the real-home boundary runs in `Gate results (2026-11-08T00:06:46-05:00)` exercised the host path instead.
- Blocked elsewhere:
  typecheck gates whose graph includes `test-harness/core-matrix` fail on
  `TS2307` for `container-limit-core`,
  a locked-but-uninstalled workspace dependency outside this work,
  filed as [#6269](https://github.com/Aquaticats/Monochromatic/issues/6269).

## Accepted risks and limits

- pi's extension API carries no stability guarantee
  and its changelog records 59 breaking changes
  ([`doc/decision/pi-model-retirement.md`](../decision/pi-model-retirement.md)).
  Peers stay `"*"`;
  the fixture and `verify:host` detect drift on dependency bumps.
- The snapshot can serve stale models during an outage
  until someone runs `refresh-snapshot`.
  The fetch-date header makes the staleness visible.
- Cost display shows zeros where the listing prices a field `null`,
  unless `ModelCostSchema` offers an explicit unknown marker,
  probed in the cost gate.

## Reload-transaction deferral fix (2026-11-11T16:29:00-05:00)

The DSH plugin's mount refresh wrote its route fire-and-forget from `apply`
(`void refreshNeuralWattRoute({ settings })`),
so a manager re-trigger (`set_plugin`, `install_bundle`, `set_bundle`)
hit the same nested reload-transaction refusal
the qwen-token-plan-individual provider documented
(`HMR transactions cannot be nested`,
`@deepseek-ai/dsh-hmr` `runExclusive`, `lib/index.js:749`).
Commit `f0d569b5cd` re-roots the write the same way:
`NEURALWATT_SETTINGS_WRITE_DEFERRAL_MARKERS`,
`isNeuralWattSettingsWriteDeferral`,
and `onWriteDeferral` on `refreshNeuralWattRoute`
hand a whole-mount retry to `apply`,
which re-runs it from `plugin-manager/changed`
(`dsh-plugin-manager` `lib/index.js:3185`),
emitted outside the reload transaction.

Measured on the live web profile (`~/.dsh/profiles/web/cordis.patch.yml`),
a `set_plugin` disable/enable cycle of `include:neuralwatt`
was traced through every write of the file:
the checked-in snapshot mapping (27 models)
and then the live listing mapping (26 models)
both landed,
the 27-model intermediate state
byte-equal to `mapNeuralWattProviders(neuralWattListingSnapshot)`.
Before the fix the identical re-trigger refused both writes
(the qwen provider's pre-fix run logged two
`HMR transactions cannot be nested` refusals at 2026-11-11T15:54:00Z).

Suites: the DSH plugin's unit tests cover the deferral classifier,
both write sites,
the whole-mount retry ordering,
the no-handler fallback,
and the `apply` wiring
(`apply.unit.test.ts`).

## Out of scope

- npm publishing,
  the pi.dev gallery,
  and upstreaming into pi.
- Extras:
  slash commands,
  quota surfaces,
  UI,
  model auto-enabling.
- Any DSH behavior change.

## Pending doc edits

These land with the port unless the owner wants them sooner.

- `AGENTS.md`,
  new rule,
  code `QCR` (checked against both forbidden-strings appendixes;
  extending the EVL text would exceed the rule length cap):

```text
# AGENTS.md
QCR: Credential and config seams: read the resolver implementation
and its stores before porting or questioning them; declared field
names like `apiKeyEnv` hide the source.
```

- `package/dsh-plugin/neuralwatt/README.md`,
  beside the route's `apiKeyEnv` description:
  the name addresses a record in the DSH credentials service
  (`$DSH_HOME/.credentials.yaml`),
  with the launching environment as fallback;
  it is not an environment requirement.
- New package READMEs carry fresh trust-rule codes,
  the same "Minimize Pi dependencies" text under per-package codes,
  as every sibling README does.

## Interview record (2026-11-07T22:02:14-05:00 to 2026-11-07T22:30:07-05:00)

- Round one settled `home` and `parity`.
- Round two settled `sharing`.
- The pi-API research and the repo fact-map supplied the measurements
  the later rounds rest on,
  including the installed pi-ai 1.5.0 type reading that replaced
  docs-only API claims.
- Round five settled `persist`,
  `spend`,
  and `peers`.
- The `missing-key` question was withdrawn after the owner challenged
  its premise.
  Measurement showed the credential lives in the DSH credentials service
  (`~/.dsh/.credentials.yaml` holds the `NEURALWATT_API_KEY` record;
  no `NEURALWATT_*` name exists in the environment),
  and the corrected questions `cred-home` and `cold-start` replaced it.
- Cross-vendor reviews:
  Moonshot Kimi (`neuralwatt/kimi-k3`) named the installed-API,
  pin-evidence,
  and off-level blockers now closed above;
  Z.ai GLM (`neuralwatt/glm-5.3`) returned READY
  with the named gates folded into [Verification](#verification).
- The owner confirmed the shared understanding (`confirm`).
- Implementation start:
  the owner generated a new NeuralWatt key
  and installed `NEURALWATT_API_KEY` in `.env.local.json` themselves,
  satisfying the migration step with a dedicated pi credential.

## Next action

Owner acceptance of this proposal unlocks implementation.
The first step is the shared-core extraction (`sharing`),
since both plugins depend on it;
the DSH plugin's suite must stay green without test edits
(`buildAndTest` in `package/dsh-plugin/neuralwatt/mise.toml`).
