# @monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing

Harness-neutral Qwen Token Plan individual listing core shared by the DSH and
pi qwen-token-plan-individual plugins:
 the curated facts table,
 listing-to-models mapping with sibling-facts fallback,
 thinking-level expansion,
 the checked-in listing snapshot,
 and the bounded listing fetch.
It lives under `package/agent-harness-shared/` because two agent harnesses
consume it
(the DSH plugin `package/dsh-plugin/qwen-token-plan-individual`
and the pi plugin `package/pi-plugin/qwen-token-plan-individual`),
and it carries no Pi types and no DSH types:
 each harness projects the same facts
onto its own model profile shape.
The design record is
[`doc/planning/qwen-token-plan-individual-provider.md`](../../doc/planning/qwen-token-plan-individual-provider.md),
whose "Shared core export surface" pins the names below.

## Pi dependency trust

PJM:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package takes the rule at its strongest reading:
 it has zero runtime dependencies,
zero Pi peers,
and zero DSH peers.

## Exports

The package root exports the built core;
workspace consumers use `/ts` for their source
(`doc/decision/workspace-ts-source-imports.md`).

- `qwenTokenPlanIndividualFacts`:
 the curated facts table,
 one row per id the table knows
 (display name,
 kind,
 context window,
 max output,
 input modalities,
 reasoning state,
 efforts dict,
 wire-compat pins),
 with source provenance comments in `src/model-facts.data.ts`.
- `mapQwenTokenPlanIndividualModels`:
 listing entries to `QwenTokenPlanResolvedModelFacts`,
 applying the media skip rule,
 the sibling fallback walk,
 and pi's schema defaults,
 in listing order.
- `expandQwenTokenPlanThinkingLevelMap`:
 one efforts dict to the fully decided
`QwenTokenPlanThinkingLevelMap`,
 declared levels carrying their wire spelling,
 declared `null` staying absent ("send nothing"),
 undeclared levels pinning `null`,
 matching the builtin catalog's `thinkingLevelMap` value for value.
- `fetchQwenTokenPlanIndividualListing`:
 the bounded `GET {baseUrl}/models` with the bearer key,
 caller-abort aware,
 with `QwenTokenPlanListingError` as its one failure shape.
- `qwenTokenPlanIndividualListingSnapshot`:
 the checked-in membership snapshot
 (`{ fetchedAt, entries }`),
 the offline fallback the mount fetch keeps serving.
- `qwenTokenPlanModelFamilyKey`,
 `qwenTokenPlanModelAgeKey`,
 `compareQwenTokenPlanModelAgeKeys`:
 the family normalization and age ordering the sibling walk uses.
- Constants:
 `QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE`,
 `QWEN_TOKEN_PLAN_INDIVIDUAL_API`,
 `QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME`,
 `QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS`,
 `QWEN_TOKEN_PLAN_THINKING_LEVELS`,
 `QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS`,
 `QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS`,
 `QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS`,
 `QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW`,
 `QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS`.
- Types:
 `QwenTokenPlanModelKind`,
 `QwenTokenPlanEffortsDict`,
 `QwenTokenPlanThinkingLevel`,
 `QwenTokenPlanThinkingLevelMap`,
 `QwenTokenPlanCompatPins`,
 `QwenTokenPlanModelInput`,
 `QwenTokenPlanModelFacts`,
 `QwenTokenPlanResolvedModelFacts`,
 `QwenTokenPlanListingEntry`,
 `QwenTokenPlanListingFetcher`,
 `QwenTokenPlanListingResponse`,
 `QwenTokenPlanModelAgeKey`,
 `QwenTokenPlanListingError`.

## Facts table and fallbacks

Membership comes from the live listing;
facts come from the table,
then the fallback chain the owner chose
("Serve with fallback facts.
For a newer model,
check its older siblings,
newer to older"):

- A facts row whose kind is not chat never serves:
 the seven media rows exist so the skip rule is data driven.
 An id with no row whose name carries a media marker
 (`image`, `video`, `audio`, `tts`, `asr`, `realtime`)
 is skipped rather than served as chat.
- An id with no row walks its older siblings
 (same family key,
 nearest older first,
 ordering by the listing's `created` when present
 else parsed versions),
 and each empty field fills from the first sibling that has it:
 an understating floor.
 Media rows never join the walk.
- What no sibling discloses falls to pi's schema defaults
 (128,000 context,
 16,384 max output,
 text input,
 reasoning without selectable efforts),
 exactly what the planning doc serves for `auto`.
- A rowless id's display name is its id verbatim:
 names are identity and never inherited from a sibling.
 A rowless id serves with kind `chat`.
 A facts row's absent fields are deliberate
 (`qwen3.7-max` offers no efforts),
 so the walk covers only ids with no row.

## Snapshot

The snapshot is the listing membership fetched from
`https://token-plan.maas.qwencloudapi.com/compatible-mode/v1/models`
at 2026-11-11T05:00:34-05:00,
keeping each id and its `created` stamp.
The `refresh-snapshot` task re-fetches the live listing and the models.dev
`alibaba-token-plan` slice,
rewrites `src/snapshot.ts`,
and prints the drift it would change
(new ids without facts rows,
retired ids,
facts the models.dev slice contradicts).
It reads the API key from `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`
in the environment.

## Test fixtures

`src/test-fixtures/fixture.pi-ai-qwen-token-plan-individual-catalog.ts`
holds the 10 builtin `qwen-token-plan-individual` rows copied verbatim from
pi-ai 1.5.2
`dist/providers/data/qwen-token-plan-individual.json`
(extracted 2026-11-11),
every field included
(`inputLimits`, `cost`, `type`, `api`, `baseUrl`, `provider`, ...).
Workspace consumers reach it through the `test-fixtures` subpath
(`@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/test-fixtures`),
so the pi plugin can reuse it for full-row equality against its projection.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one from this directory with `mise run <task>`,
or from the repository root as
`mise run //package/agent-harness-shared/qwen-token-plan-individual-listing:<task>`.

Unit tests import package behavior from the built `dist` entry,
so run `build`
before `test:unit` after source changes,
or run `buildAndTest` for both.
`refresh-snapshot` stays out of `task.ts`
so no CI path runs its live fetch.
