# qwen-token-plan-individual provider in Pi and DSH

Design record for the owner's build order of 2026-11-11,
written from repo measurements,
installed-API reads,
two research subagent reports,
and two owner answers recorded under the question ids below.
The order was "Build our own qwen-token-plan-individual provider for Pi and dsh,
just like how we did it for neuralwatt.
The builtin qwen-token-plan-individual provider for Pi always provides outdated
model list."

## Contents

- [Purpose](#purpose)
- [What the staleness is, measured](#what-the-staleness-is-measured)
- [What lands](#what-lands)
- [Owner answers](#owner-answers)
- [Settled decisions](#settled-decisions)
- [Shared core export surface](#shared-core-export-surface)
- [Model facts table](#model-facts-table)
- [Identity and behavior invariants](#identity-and-behavior-invariants)
- [Install and credential seams](#install-and-credential-seams)
- [Verification](#verification)
- [Accepted risks and limits](#accepted-risks-and-limits)
- [Out of scope](#out-of-scope)
- [Next action](#next-action)

## Purpose

Serve the Qwen Token Plan Individual route
(`qwen-token-plan-individual`)
in Pi and DSH with the model list the endpoint actually serves today,
instead of the generated catalog pi-ai ships.
The architecture copies the NeuralWatt provider work
([`doc/planning/neuralwatt-pi-plugin.md`](neuralwatt-pi-plugin.md)):
one harness-neutral listing core,
a thin projection per harness.

## What the staleness is, measured

- pi-ai 1.5.2 ships `qwen-token-plan-individual`
  with a static 10-model catalog
  (`dist/providers/data/qwen-token-plan-individual.json`),
  auto-generated at release time from a hand-maintained allowlist
  (comment: "verified 2026-11-01") plus models.dev facts.
  No runtime provider fetches a model list;
  `refreshModels` exists only for Radius
  (upstream `packages/ai/src/` confirms it).
- The live listing
  (`GET https://token-plan.maas.qwencloudapi.com/compatible-mode/v1/models`,
  bearer key required,
  the `token-plan.ap-southeast-1.maas.aliyuncs.com` host serves byte-identical
  bodies from the same NLB)
  returned 21 ids on 2026-11-11,
  11 of them absent from the builtin catalog:
  `auto`,
  `deepseek-v4.1-flash`,
  `deepseek-v4.1-flash-1017`,
  `qwen4-max-preview`,
  `qwen-audio-3.0-asr-flash`,
  `qwen-audio-3.0-realtime-plus`,
  `qwen-audio-3.0-tts-plus`,
  `qwen-image-3.0-pro`,
  `wan2.7-image`,
  `wan2.7-image-pro`,
  `wan3.0-video`.
  The builtin catalog holds nothing the listing lacks.
- The listing is bare
  (id, `created`, `owned_by`, `object`;
  probes of `?detail=`, `?fields=`, `models/<id>` returned the same list
  or `InvalidParameter`),
  so per-model facts come from a facts table the shared core maintains.
- Chat probes classified the 21 ids:
  13 chat ids plus the `auto` router alias answer chat completions;
  the 7 image, video, audio, TTS, ASR, and realtime ids refuse them
  (400 or 500),
  matching the research report
  [`doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`](../audit/qwen-token-plan-individual-model-facts-2026-11-11.md),
  which also documents
  `qwen4-max-preview` (1M context,
  393,216 max output shared with thinking,
  efforts `low`/`medium`/`xhigh`)
  and the `deepseek-v4.1-flash` family
  (1M context,
  efforts `low`/`high`/`max`).

## What lands

The mapping logic is owned once in a harness-neutral core,
with a thin projection per harness,
exactly as NeuralWatt does.

- `package/agent-harness-shared/qwen-token-plan-individual-listing`
  (`@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing`):
  the facts table
  (display name,
  context window,
  max output,
  input modalities,
  reasoning efforts dict,
  wire-compat pins,
  model kind),
  the listing-to-models mapping
  (media skip rule,
  sibling-facts fallback,
  default fallback),
  the bounded listing fetch
  (`GET {baseUrl}/models`,
  `QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS`),
  the checked-in snapshot of listing membership with its fetch-date header,
  and a `refresh-snapshot` mise task that re-fetches the listing and the
  models.dev `alibaba-token-plan` slice and prints drift.
  It carries no pi types and no DSH types.
- `package/pi-plugin/qwen-token-plan-individual`
  (`@monochromatic-dev/pi-plugin-qwen-token-plan-individual`):
  the pi extension.
  A default-export factory registers a hand-built `Provider`
  with id `qwen-token-plan-individual` through `pi.registerProvider`,
  which pi's model runtime prefers over the builtin
  (`composeProvider` reads `nativeExtensionProviders` before `builtins`),
  so the stale builtin catalog stops serving while the extension is loaded
  and `unregisterProvider` restores it on removal.
  Snapshot baseline models first;
  `refreshModels(context)` performs the bounded fetch
  with `context.credential`,
  and `publish({ update })`,
  never `persist`;
  refresh on `session_start` for startup and reload;
  teardown on `session_shutdown`.
- `package/dsh-plugin/qwen-token-plan-individual`
  (`@monochromatic-dev/dsh-plugin-qwen-token-plan-individual`):
  the DSH plugin,
  the same shape as
  [`package/dsh-plugin/neuralwatt`](../../package/dsh-plugin/neuralwatt).
  It writes the full mapped route for key `qwen-token-plan-individual`
  into the `llm-pi-ai` settings entry
  at mount (snapshot first,
  live listing after),
  skipping a write whose stored route already matches.

## Owner answers

Recorded under the question ids.

- `dsh-model-list`:
  "Full live list in both".
  Both harnesses serve every serviceable chat id the live listing returns
  (14 today:
  the 13 chat ids and the `auto` router alias),
  and the DSH plugin write replaces the hand-declared
  2-model block in
  `~/.dsh/profiles/web/cordis.patch.yml`.
- `unknown-facts`:
  "Serve with fallback facts.
  For a newer model,
  check its older siblings, newer to older,
  assume the newer model would always have better capabilities than older."
  An id the facts table does not know inherits the nearest older sibling's
  facts as an understating floor,
  and only ids no sibling covers fall back to pi's schema defaults.

## Settled decisions

### Adopted from the NeuralWatt plan unasked

The owner's "just like how we did it for neuralwatt" adopts that plan's
answers where they carry over.

- Route key `qwen-token-plan-individual`,
  model ids,
  and effort wire values stay byte-identical across harnesses,
  so `qwen-token-plan-individual/<model>` slugs work in both.
- Snapshot first,
  bounded live fetch,
  zero serviceable models keeps the snapshot.
- `publish({ update })`,
  never `persist`;
  the extension never writes `enabledModels`,
  `models.json`,
  or `models-store.json`.
- No extras beyond parity:
  no slash commands,
  no quota surfaces,
  no UI.
- valibot is rejected for the shared core;
  manual `isRecord` narrowing stands.
- Logger tags follow the tagging rule
  (`@monochromatic-dev/logger-core`).
- Pi extension stability is watched the recorded way:
  a characterization fixture plus `verify:host`
  (pi RPC in a throwaway home),
  because pi's extension API carries no stability guarantee
  ([`doc/decision/pi-model-retirement.md`](../decision/pi-model-retirement.md)).

### New for this provider

- **Wire-compat pins** per model are carried by the facts table and copied by
  both projections:
  `thinkingFormat: "qwen"`,
  `supportsDeveloperRole: false`,
  `supportsStore: false`,
  `supportsStrictMode: true`,
  and `supportsReasoningEffort` true exactly when the model offers efforts.
  These are the builtin catalog's own pins
  (measured on all 10 builtin rows),
  so overlapping models dispatch identically to the builtin.
- **Three reasoning states**,
  because DSH's profile vocabulary cannot express the middle one for
  hand-declared models:
  non-reasoning (`reasoningEfforts: false`),
  reasoning with selectable efforts (the efforts dict),
  and reasoning with no selectable efforts
  (`qwen3.6-flash`, `qwen3.7-max`, `qwen3.7-plus`, `auto`).
  The third state projects in pi as
  `reasoning: true` with no `thinkingLevelMap`
  (the builtin's own shape for those three),
  and in DSH as an omitted `reasoningEfforts`
  (inheriting the installed catalog for catalog-known ids,
  meaning "does not reason" for `auto`,
  a recorded divergence).
- **Efforts dict to pi map expansion** pins every one of the seven thinking
  levels:
  a declared spelling carries it,
  everything else pins `null`.
  For the 10 builtin models the expansion must equal the builtin catalog's
  `thinkingLevelMap` byte for byte;
  that golden assertion is a unit test on the shared core.
- **Base URLs stay per harness**,
  the narrower change:
  Pi keeps `https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1`
  (the builtin's),
  DSH keeps `https://token-plan.maas.qwencloudapi.com/compatible-mode/v1`
  (the hand config's).
  Both resolve to the same NLB
  (measured `47.84.193.37` and `43.98.173.108`,
  identical listing bodies),
  so route identity is unaffected.
- **Credential seams stay per harness**:
  Pi resolves its stored credential
  (`~/.pi/agent/auth.json` already holds a `qwen-token-plan-individual` key,
  measured)
  then `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY` then `QWEN_TOKEN_PLAN_API_KEY`
  in the environment
  (the builtin's env name kept as fallback);
  DSH resolves the credential record
  `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY` through its credentials service,
  which `~/.dsh/.credentials.yaml` already holds
  (measured),
  exactly as the hand config names it.
- **Facts priority** when sources disagree:
  a pi-ai catalog row
  (builtin or the sibling `qwen-token-plan` catalog)
  wins,
  then documented facts,
  then sibling inheritance,
  then pi's schema defaults
  (128000 context,
  16384 max output).
  Known rows keep the builtin's numbers byte for byte
  so the override changes membership,
  not facts.
  The one recorded discrepancy:
  the research report documents 393,216 max output for the
  `deepseek-v4.1-flash` family
  while pi-ai's sibling catalog records 384,000;
  the catalog row wins.
- **Media skip rule**:
  a facts row whose kind is not chat is never served,
  and an id absent from both the facts table and the snapshot
  whose name marks it media
  (`image`, `video`, `audio`, `tts`, `asr`, `realtime`)
  is skipped rather than served as chat.
- **DSH credential seam is structural** (decided 2026-11-11T06:05:00-05:00):
  the fetch module types the credentials service by its one `resolve` call,
  mirroring `NeuralWattSettingsTarget`,
  instead of importing `@deepseek-ai/dsh-credentials`.
  Rationale measured then ruled:
  the sibling's production source imports `@deepseek-ai/*` as types only
  (its sole value import sits in a characterization test),
  `brandString` is identity at runtime,
  and the host's own `apiKeyEnv` seam reads `hit.value` with a fallback,
  which the structural call matches.
  `credentialRef` does validate the reference name and throws `TypeError`
  on a malformed one before branding;
  the shipped constant matches the grammar,
  so the branded call and the raw string resolve identically.
  The package keeps `peerDependencies` plus `devDependencies`
  for `@deepseek-ai/cordis`, `@deepseek-ai/dsh-llm-pi-ai`,
  and `@deepseek-ai/dsh-settings`,
  mirroring `package/dsh-plugin/neuralwatt/package.json`.

## Shared core export surface

Pinned so the two projections can be written against it.

```typescript
export const QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE = 'qwen-token-plan-individual';
export const QWEN_TOKEN_PLAN_INDIVIDUAL_API = 'openai-completions';
export const QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME =
  'QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY';
export const QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS = 10_000;

export type QwenTokenPlanModelKind =
  'chat' | 'router' | 'image' | 'video' | 'tts' | 'asr' | 'realtime';

export type QwenTokenPlanEffortsDict = {
  readonly off?: string | null;
  readonly minimal?: string | null;
  readonly low?: string | null;
  readonly medium?: string | null;
  readonly high?: string | null;
  readonly xhigh?: string | null;
  readonly max?: string | null;
};

export type QwenTokenPlanModelFacts = {
  readonly id: string;
  readonly name: string;
  readonly kind: QwenTokenPlanModelKind;
  readonly contextWindow?: number;
  readonly maxTokens?: number;
  readonly input: readonly ('text' | 'image')[];
  readonly reasoning: boolean;
  readonly efforts?: QwenTokenPlanEffortsDict;
  readonly compat: QwenTokenPlanCompatPins;
};

export type QwenTokenPlanListingEntry = {
  readonly id: string;
  readonly created?: number;
};

export type QwenTokenPlanListingFetcher = (options: {
  readonly baseUrl: string;
  readonly apiKey: string | undefined;
  readonly signal: AbortSignal;
}) => Promise<QwenTokenPlanListingResponse>;

export function fetchQwenTokenPlanIndividualListing(options: {
  readonly baseUrl: string;
  readonly apiKey: string | undefined;
  readonly fetcher?: QwenTokenPlanListingFetcher;
  readonly signal?: AbortSignal;
}): Promise<readonly QwenTokenPlanListingEntry[]>;

export const qwenTokenPlanIndividualFacts:
  readonly QwenTokenPlanModelFacts[];

export const qwenTokenPlanIndividualListingSnapshot: {
  readonly fetchedAt: string;
  readonly entries: readonly QwenTokenPlanListingEntry[];
};

export function mapQwenTokenPlanIndividualModels(options: {
  readonly listing: readonly QwenTokenPlanListingEntry[];
  readonly facts?: readonly QwenTokenPlanModelFacts[];
}): readonly QwenTokenPlanResolvedModelFacts[];

export function expandQwenTokenPlanThinkingLevelMap(options: {
  readonly efforts: QwenTokenPlanEffortsDict;
}): QwenTokenPlanThinkingLevelMap;
```

`QwenTokenPlanResolvedModelFacts` is `QwenTokenPlanModelFacts` with every
fallback already applied
(`contextWindow`, `maxTokens`, `input`, `efforts`, `name` all decided),
so each projection is a straight copy.

## Model facts table

Membership comes from the live listing;
facts come from the table.
Rows carried at launch,
each reproducing the builtin catalog byte for byte unless noted:

- `qwen3.8-max` and `qwen3.8-flash`:
  1,000,000 context,
  131,072 max output,
  text and image,
  efforts `low`, `medium`, `xhigh`.
- `qwen3.7-max`:
  1,000,000 / 131,072,
  text,
  reasoning without efforts.
- `qwen3.7-plus`:
  1,000,000 / 65,536,
  text and image,
  reasoning without efforts.
- `qwen3.6-flash`:
  1,000,000 / 65,536,
  text and image,
  reasoning without efforts.
- `glm-5.2`:
  1,000,000 / 131,072,
  text,
  efforts `high`, `max`.
- `glm-5.3`:
  1,000,000 / 131,072,
  text,
  efforts `low`, `high`, `max`.
- `deepseek-v4-flash-0731`,
  `deepseek-v4-pro`,
  `deepseek-v4-pro-0813`:
  1,000,000 / 384,000,
  text,
  efforts `high`, `max`.
- `deepseek-v4.1-flash` and `deepseek-v4.1-flash-1017`:
  1,000,000 / 384,000,
  text and image,
  efforts `low`, `high`, `max`
  (pi-ai's sibling catalog row,
  which also supplies the display names
  "DeepSeek V4.1 Flash" and "DeepSeek V4.1 Flash 1017").
- `qwen4-max-preview`:
  1,000,000 / 393,216,
  text and image,
  efforts `low`, `medium`, `xhigh`
  (documented;
  no catalog row exists).
- `auto`:
  no row;
  the sibling walk finds no family,
  so it serves with pi's schema defaults
  (128,000 / 16,384),
  text input,
  reasoning without efforts,
  display name from its id.

The seven media ids carry kind rows
(`image`, `video`, `tts`, `asr`, `realtime`)
so the skip rule is data driven,
and the name-pattern guard covers ids arriving between refreshes.

## Identity and behavior invariants

- Route key `qwen-token-plan-individual` in both harnesses,
  overriding the pi builtin while the extension is loaded.
- Model ids and display names follow the mapping exactly.
- For every model the builtin catalog also serves,
  the projected facts
  (id, name, contextWindow, maxTokens, input, reasoning, thinkingLevelMap,
  compat)
  equal the builtin row byte for byte.
- Offline behavior:
  the checked-in snapshot of listing membership plus the facts table serve
  whenever the fetch fails, times out, or maps to zero serviceable models.

## Install and credential seams

- Pi:
  `pi install` of the package directory,
  last in the `packages` array of `~/.pi/agent/settings.json`,
  through a guarded task that snapshots settings
  and fails on any change outside `packages`
  (the convention recorded in
  [`package/pi-plugin/model-retirement/README.md`](../../package/pi-plugin/model-retirement/README.md)).
  The stored credential already present in `~/.pi/agent/auth.json`
  under `qwen-token-plan-individual` keeps resolving,
  since the override keeps the provider id.
- DSH:
  the package joins the profile the same way
  `@monochromatic-dev/dsh-plugin-neuralwatt` already does
  (`~/.dsh/profiles/web/package.json` link plus its plugin list).
  The credential record `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`
  already present in the DSH credentials service keeps resolving,
  since the written route keeps the same `apiKeyEnv` pin
  as the hand block it replaces.
- No credential migration is required;
  both harnesses already authenticate this route.

## Verification

Named suites and gates,
mirroring the NeuralWatt gate list.

- Unit suites:
  mapping (media skip, sibling fallback walk, defaults fallback,
  golden builtin-row equality, thinking-level expansion),
  fetch (bounded, auth header, failure keeps snapshot),
  snapshot.
- `verify:extension` against a fake `ExtensionAPI`.
- Characterization fixture plus `verify:host`
  (pi RPC in a throwaway home).
- Override probe:
  `pi --list-models` shows the live set
  (including `deepseek-v4.1-flash-1017` and `qwen4-max-preview`),
  and the builtin's stale-only ids are gone.
- Never-persist guard:
  `models-store.json`, `models.json`, and `enabledModels`
  byte-identical before and after refresh.
- Offline gate:
  a network-denied run (with its positive control)
  serves the snapshot set and fails a completion gracefully.
- Live smoke on `qwen-token-plan-individual/qwen3.8-max`,
  plus one dispatch check that a selected effort level sends its wire
  spelling.
- DSH:
  the settings-seam unit tests
  (sameJsonShape skip, merge write),
  then the live route check that the served model list is the full set.

## Accepted risks and limits

- The facts table is curated;
  a model the endpoint adds before the table learns it serves with
  sibling-inherited or default facts
  (the owner's chosen freshness-over-accuracy trade),
  and `refresh-snapshot` prints the drift so the table catches up.
- `auto` is a router alias whose routed model decides its real limits;
  the defaults it serves are an understating floor,
  measured-safe on this endpoint
  (oversize `max_tokens` clamps silently,
  200 responses probed).
- `auto` loses its reasoning selector in DSH
  (the recorded divergence under "three reasoning states").
- The `deepseek-v4.1-flash` max-output discrepancy
  (384,000 catalog versus 393,216 documented)
  resolves to the catalog row;
  it truncates 2.4% of the documented output room at worst.

## Out of scope

- The sibling routes `qwen-token-plan`,
  `qwen-token-plan-cn`,
  and `qwen-token-plan-individual-cn`
  share the endpoint shape and could take the same treatment later;
  this build covers `qwen-token-plan-individual` only.
- Media generation dispatch
  (image, video, audio endpoints);
  those ids are skipped,
  not served.

## Next action

The shared core lands first,
since both projections depend on its export surface;
the pi and DSH plugins follow in parallel.
