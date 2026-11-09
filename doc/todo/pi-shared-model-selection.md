# Extract model selection logic to agent-harness-shared/model-selection

## Status 2026-10-20

The package moved on 2026-10-20:
 original `package/pi-shared/model-selection`
 (`@monochromatic-dev/pi-shared-model-selection`),
 current `package/agent-harness-shared/model-selection`
 (`@monochromatic-dev/agent-harness-shared-model-selection`),
 per Q9 in [the advisor DSH variant grilling](../handover/pi-advisor-dsh-variant-grilling.md).
 Identifiers in this document were updated in place to the current values;
 the original values are the ones named in this note.
 The advisor DSH variant series also extracts a harness-neutral advisor core
 (`package/agent-harness-shared/advisor`) that consumes this package,
 and moves the advisor-specific selection wrapper into that core.
 Planned moves below that are not part of that series stay open.

The absence convention changed on 2026-05-31
 (commit `3bb4ce7f8c`,
`2026-05-31T11:38:59-04:00`):
 the generic `ABSENT` symbol and `Maybe<T>` alias,
 which the original plan collected in `src/maybe.ts`,
 became one exported `unique symbol` per expected miss,
 and `src/maybe.ts` was deleted.
 The decisions here and the `Absence convention` section state the current convention.

## Decisions captured here

- Keep the package at `package/agent-harness-shared/model-selection/` with package name
  `@monochromatic-dev/agent-harness-shared-model-selection`.
- Use one shared package whose manifest exposes `.` and `./ts` only,
   with the feature modules kept internal to the single entry.
- Move the full planned surface,
   including budget-model strategy orchestration.
- Leave only registry authentication and extension-host adapters in consumers.
- Signal a non-exceptional absence in shared APIs with one exported `unique symbol` per expected miss,
   returning `T | typeof SENTINEL` and narrowing with `=== SENTINEL`;
   this replaced the `ABSENT` and `Maybe<T>` pair on 2026-05-31 (commit `3bb4ce7f8c`).
- Preserve lightweight consumers by keeping dependency-free helpers out of platform-coupled modules.

## Problem

Three pi plugins contain related model-selection logic,
 but the overlap is uneven.
Advisor owns most of the current surface;
 auto-mode overlaps through budget selection and version ranking;
thinking-defaults overlaps through model-id leaf parsing.
The value of the extraction is therefore a reusable pi model-selection home first,
 and duplication removal second.

Measured whole-file inventory on 2026-05-26:
 3,431 lines across 16 source files.
This is inventory size,
 not a moved-line estimate.
Some files contain consumer-specific adapters that become thinner rather than disappearing.

- Advisor:
   2,261 whole-file lines.
  - `model-slug.ts`,
     260 lines:
     canonical slugs,
     scoped-slug matching,
     global registry existence checks,
    and allowed-slug formatting.
  - `scope-exact.ts`,
     135 lines:
     exact model reference matching by canonical slug,
     provider/model,
     and bare id.
  - `scope-match.ts`,
     372 lines:
     exact plus fuzzy pattern matching,
     thinking-level suffix parsing,
    alias versus dated-version detection,
     and scoped model construction.
  - `scope-patterns.ts`,
     198 lines:
     glob pattern resolution through `zeptomatch`,
     literal pattern resolution,
    and deduplication.
  - `scope-resolver.ts`,
     277 lines:
     effective scope resolution from live scope,
     argv,
     settings,
     and available models.
  - `model-cost.ts`,
     304 lines:
     request token estimation,
     expected-cost scoring,
     tie-breaking,
    and default model selection.
  - `advisor-selection.ts`,
     75 lines:
     advisor-specific explicit-or-default selection wrapper.
  - `argv-scope.ts`,
     82 lines:
     `--models` parsing.
  - `settings-scope.ts`,
     241 lines:
     `enabledModels` loading from global and project pi settings.
  - `maybe.ts`,
     44 lines:
     `ABSENT` and `Maybe<T>` sentinel.
     The file was removed on 2026-05-31 by commit `3bb4ce7f8c`,
      which replaced it with the per-purpose sentinels the absence section lists;
      the line count is the 2026-05-26 measurement.
  - `types.ts`,
     273 lines:
     advisor config,
     scope,
     selection,
     context,
     tool,
     and result types.
- Auto-mode:
   1,050 whole-file lines.
  - `budget-model.ts`,
     370 lines:
     same-provider and any-provider strategies,
     cheapest-first candidate ordering,
    and sequential auth walk.
  - `budget-model-auth.ts`,
     312 lines:
     auth resolution through `ModelRegistry.getApiKeyAndHeaders`,
    `NoBudgetModelError`,
     and candidate reporting.
  - `budget-model-override.ts`,
     102 lines:
     override resolution from `provider/id` or `{ model, auth }`.
  - `budget-model-version.ts`,
     266 lines:
     version extraction,
     major-version grouping,
    and cost-then-version sorting.
- Thinking-defaults:
   120 whole-file lines.
  - `model-policy.ts`,
     120 lines:
     model-id leaf extraction,
     GPT detection,
     and thinking policy.

## Package placement

`agent-harness-shared` is a top-level category for reusable harness and extension infrastructure
 that is not itself an extension.
A package belongs there when two or more harnesses or packages consume it and its public API is reusable outside
one extension's command,
 rendering,
 or config surface.
Actual pi extensions stay under `package/pi-plugin/`.

The repository has a nested-category precedent in `package/claude-code-plugin/hook-type/`,
 consumed by
`package/claude-code-plugin/source/`.
This plan intentionally chooses a top-level shared category instead,
 so the category rule above is part of
the extraction and should be kept with the package README.

- Directory:
   `package/agent-harness-shared/model-selection/`
- Package name:
   `@monochromatic-dev/agent-harness-shared-model-selection`
- Description:
   shared model selection logic for pi plugins,
   including slug resolution,
   pattern matching,
  scope resolution,
   cost ranking,
   budget-model strategy selection,
   and version extraction.

## Public export policy

The shipped manifest has two entries,
 read from `package/agent-harness-shared/model-selection/package.json` on 2026-11-08:

- `.`:
   the built bundle `dist/final/node/index.mjs`,
   which re-exports every feature module through the single `src/index.ts` entry:
   core,
   scope,
   cost,
   budget,
   and text token estimation.
- `./ts`:
   the TypeScript source entry `src/index.ts`.

`rolldown.node.config.ts` records why the per-feature bundles are gone:
 no consumer imports a feature subpath.
The feature modules (`core.ts`,
`scope.ts`,
`cost.ts`,
`budget.ts`,
`token-estimate.ts`) stay inside the package,
 and none of them is an export subpath,
 so `@monochromatic-dev/agent-harness-shared-model-selection/core` fails with
`ERR_PACKAGE_PATH_NOT_EXPORTED`.

Every workspace consumer imports the `/ts` entry (rule ST3):
`package/agent-harness-shared/advisor`,
`package/dsh-plugin/advisor`,
`package/pi-plugin/advisor`,
`package/pi-plugin/goal`,
and `package/pi-plugin/thinking-default`.
Thinking-defaults imports only `getModelIdLeaf` from that entry.

## Dependency policy

One physical package,
 one dependency tier.
Commit `54cb3ef443` (2026-11-05) removed both Pi peer dependencies and both Pi development
 dependencies when the former `./pi-coding-agent` wrapper became the Pi-free
`src/token-estimate.ts`.
The manifest now declares no `peerDependencies` and no `peerDependenciesMeta`,
 so the earlier policy "mark both platform peers optional" is retired:
 no peer is left to mark,
 and nothing in the package imports Pi.

Runtime dependencies,
 as the manifest declares them:

- `@monochromatic-dev/caught-value-core: workspace:*`,
   error text for failed settings validation.
- `@monochromatic-dev/ownership-marker-foreign-borrowed: workspace:*`,
   foreign-borrowed markers on the shared structural types.
- `type-fest: catalog:`,
   used for readonly helper types where needed.
- `valibot: catalog:`,
   used at runtime by settings validation.
- `zeptomatch: catalog:`,
   used by scope glob resolution.

Development dependencies:

- `@monochromatic-dev/config-rolldown: workspace:*`.
- `@monochromatic-dev/config-typescript: workspace:*`.
- `@monochromatic-dev/test-harness-core: workspace:*`.
- `@types/node: catalog:`.
- `typescript: catalog:`.

Consumer dependencies:

- `@monochromatic-dev/agent-harness-shared-model-selection: workspace:*` in
   `package/agent-harness-shared/advisor`,
   `package/dsh-plugin/advisor`,
   `package/pi-plugin/advisor`,
   `package/pi-plugin/goal`,
   and `package/pi-plugin/thinking-default`.
- Consumers declare `@earendil-works/pi-ai` or `@earendil-works/pi-coding-agent` only for their own
  Pi imports;
  this package needs neither.

## Type strategy

Use structural model types and generic return types rather than forcing all consumers through one widened
`ReadonlyModel` type.

- `ModelIdentity`:
   `provider`,
   `id`,
   and `name`.
- `ModelPricing`:
   model identity plus `cost`,
   `contextWindow`,
   and `maxTokens`.
- `ReadonlyModel`:
   full pi-like readonly model shape for callers that need a complete model record.
- `ScopedModel<TModel extends ModelIdentity = ReadonlyModel>`:
   selected model entry retaining the caller's
  original model type.
- `EffectiveModelScope<TModel extends ModelIdentity = ReadonlyModel>`:
   scope entries retaining the caller's
  original model type.

Functions should be generic over the minimum shape they need.
For example,
 `canonicalSlug` needs only `provider` and `id`,
 while cost ranking needs pricing fields.
If a module must widen a `Model<Api>` to `ReadonlyModel`,
 expose a named conversion helper so the loss of
narrowed `api` typing is explicit at the call site.

## Absence convention

Shared APIs report an expected miss with one exported `unique symbol`,
 returning `T | typeof SENTINEL` and narrowing with `=== SENTINEL`.
This avoids a banned `T | undefined` union while keeping no-match flows
 non-exceptional.
Name each sentinel after the miss it reports;
 a word that fits any miss,
such as `ABSENT`,
names none.
Declare it in the module that first returns it,
 unless a callback crosses the package boundary,
 where it belongs in a module both sides import without a cycle,
 as `NO_AUTH` does in `types.ts`.

Sentinels the package exports today:

- `NO_EXACT_MATCH`:
   no unambiguous exact model reference.
- `MALFORMED_SLUG`:
   a slug is not a `provider/model` pair.
- `NO_ARGV_MODELS`:
   argv holds no `--models` value.
- `NO_LIVE_SCOPE`:
   the runtime exposes no usable live model scope.
- `NO_CANDIDATE`:
   no budget candidate survives version filtering.
- `NO_MAJOR_VERSION`:
   a model id holds no parseable major version.
- `NO_AUTH`:
   the budget auth callback finds no usable credentials for a model.
- `NO_OVERRIDE_MODEL`:
   a registry lookup finds no model for a budget override slug.

Three more stay internal to the module that raises them:
 `NO_PATTERN_MATCH`,
 `NO_THINKING_LEVEL`,
 and `NO_SETTINGS_FILE`.

Use thrown errors for invalid inputs,
 impossible internal states,
 and user-facing selection failures.
Use optional object fields for optional metadata,
 such as a thinking-level suffix or source path.

## Shared modules

### Core modules

- one exported `unique symbol` per expected miss,
   declared in the module that returns it,
   as the `Absence convention` section requires;
   the planned `maybe.ts` was dropped on 2026-05-31 (commit `3bb4ce7f8c`).
- `types.ts`:
   define structural model,
   scope,
   cost,
   budget,
   and selection types.
- `model-id.ts`:
   provide `canonicalSlug`,
   `parseProviderModelSlug`,
   `getModelIdLeaf`,
   and allowed-slug helpers.
- `exact-match.ts`:
   move exact model reference matching with structural type parameters.
- `pattern-match.ts`:
   move model pattern parsing,
   thinking suffix parsing,
   glob detection,
   alias detection,
  date-suffix detection,
   and scoped model construction.
- `version.ts`:
   move version extraction,
   version-vector comparison,
   and major-version filtering.

Core extraction also replaced the linear string recursion in the advisor's `scope-match.ts`,
 which the shared `pattern-match.ts` now holds:
`parseModelPattern` and `isAsciiDigitString` scan iteratively rather than recurse over flat strings.

Promote these currently private helpers to public exports because consumers and tests need them after extraction:

- `isThinkingLevel`.
- `isAlias`.
- `hasDateSuffix`.

### Scope modules

- `argv-scope.ts`:
   move `parseArgvModelPatterns`.
- `settings-scope.ts`:
   move settings scope loading and replace `AdvisorSettingsFileSchema` with
  `PiSettingsFileSchema`,
   validating only `enabledModels`.
- `scope-patterns.ts`:
   move glob and literal scope resolution.
- `scope-resolver.ts`:
   move effective-scope resolution,
   but expose a narrow context interface instead of
  requiring `ExtensionContext` in the public signature.

The narrow scope context should include only:

- current working directory.
- available-model provider.
- optional live scope getter or live scope property.
- optional argv and home overrides for tests.

`pi-ai`'s `modelsAreEqual` was replaced by canonical-slug deduplication after the source check
 recorded in `src/scope-patterns.ts`:
provider and id equality is the same predicate.
Any surviving `modelsAreEqual` use stays in the consumer that owns a Pi dependency;
 this package has none.

### Cost modules

- `cost-ranking.ts`:
   move advisor expected-cost scoring and default highest-cost selection.
- Export `scoreModel`,
   `compareCostScores`,
   and `buildCostRanking` as newly factored public helpers.
  Mark them as new decompositions,
   not direct moves.
- Keep advisor-specific `AdvisorModelSelection` in advisor.
- Keep advisor-specific reason text either in advisor or as an injectable reason formatter.

Token estimation must not force every consumer to import pi-coding-agent.
Commit `54cb3ef443` (2026-11-05) settled this in the package instead:
`src/token-estimate.ts` counts UTF-16 code units and never loads a Pi SDK.

### Budget modules

Move budget selection strategy into shared while keeping host-specific auth in auto-mode.

- `budget-selection.ts`:
   same-provider and any-provider strategies,
   major-version filtering,
   candidate ordering,
   and sequential auth walk.
- `budget-override.ts`:
   override parsing and selection through injected model lookup and auth callbacks.
- `budget-report.ts`:
   candidate reporting and `NoBudgetModelError` if the error message can stay generic.
  If the fix text remains auto-mode-specific,
   keep only the formatter in auto-mode.

Budget selection must accept callbacks instead of importing auto-mode context directly:

```typescript
// package/agent-harness-shared/model-selection/src/types.ts
export type ResolveBudgetAuth<TModel> = (
  options: { readonly model: TModel; },
) => Promise<BudgetModelAuth | typeof NO_AUTH>;

export type BudgetModelSelectionOptions<TModel extends ModelPricing> = {
  readonly activeModel: TModel;
  readonly allModels: readonly TModel[];
  readonly strategy: 'same-provider' | 'any-provider';
  readonly majorVersions: number;
  readonly resolveAuth: ResolveBudgetAuth<TModel>;
  readonly hasConfiguredAuth: (options: { readonly model: TModel; }) => boolean;
};
```

Auto-mode keeps the adapter that calls `ctx.modelRegistry.getApiKeyAndHeaders`,
 returns `NO_AUTH` when the registry reports no usable credentials,
and passes registry-backed callbacks into shared budget selection.

## Consumer responsibilities after migration

### Advisor retains

- `advisor-selection.ts` as a thin wrapper that returns `AdvisorModelSelection`.
- `tool-context-selection.ts`,
   because it pairs model selection with advisor context budgeting.
- `advisor-client.ts`,
   API calls,
   and provider request handling.
- `context.ts` and `context-user.ts`,
   context serialization.
- `config.ts` and advisor-specific fields in `config-schemas.ts`.
- `rendering.ts` and `rendering-summary.ts`.
- `commands.ts`,
   `tool.ts`,
   and `tool-params.ts`.
- Advisor-specific types:
   `AdvisorConfig`,
   `AdvisorModelSelection`,
   `AdvisorDetails`,
   `AdvisorContext`,
  and tool result types.

### Auto-mode retains

- Registry auth adapter around `ModelRegistry.getApiKeyAndHeaders`.
- Extension config loading and defaults.
- Judge,
   signals,
   command parsing,
   trust handling,
   and non-model-selection code.
- Thin wrapper that passes active model,
   registry model list,
   strategy options,
   and auth callbacks to shared
  budget selection.

### Thinking-defaults retains

- Policy constants:
   GPT-shaped models use `xhigh`,
   other models use `high`.
- GPT detection if it remains policy-specific.
- Extension hook code in `apply-thinking-default.ts`.

Thinking-defaults imports `getModelIdLeaf` from the package's `/ts` entry only.

## Test strategy

Move or rewrite tests by behavior,
 not by file name.
Each shared module gets tests that describe the public API after extraction.
Consumer tests remain for consumer-specific wrappers.

Move and generalize from advisor:

- argv parsing tests into `argv-scope.unit.test.ts`.
- canonical,
   bare-id,
   display-name,
   ambiguous,
   out-of-scope,
   and unknown slug tests into `model-id.unit.test.ts`.
- exact matching tests into `exact-match.unit.test.ts`.
- pattern,
   thinking suffix,
   alias,
   dated-version,
   fuzzy,
   and glob tests into `pattern-match.unit.test.ts`
  and `scope-patterns.unit.test.ts`.
- settings precedence and invalid settings tests into `settings-scope.unit.test.ts`.
- live,
   argv,
   settings,
   and available fallback tests into `scope-resolver.unit.test.ts`.
- expected-cost and tie-break tests into `cost-ranking.unit.test.ts`.

Move and generalize from auto-mode:

- version extraction,
   date skipping,
   version-vector sorting,
   and major-version filtering tests into
  `version.unit.test.ts`.
- same-provider strategy,
   any-provider strategy,
   auth failure,
   override selection,
  and candidate-report tests into `budget-selection.unit.test.ts` and `budget-override.unit.test.ts`.

Keep or add consumer tests:

- Advisor keeps tests for `AdvisorModelSelection` shape,
   advisor context budgeting,
   tool details,
  rendering,
   and extension status.
- Auto-mode keeps tests for registry auth adapter behavior,
   config defaults,
   judge behavior,
   and signal handling.
- Thinking-defaults keeps policy tests for GPT versus non-GPT thinking defaults.

Add characterization tests before rewiring imports:

- Advisor default model selection returns the same slug and ranking for fixed model fixtures.
- Advisor explicit slug resolution returns the same errors for ambiguous,
   out-of-scope,
   and unknown slugs.
- Auto-mode budget selection returns the same model for same-provider and any-provider fixtures.
- Auto-mode emits the same user-facing error message shape for no-auth cases.
- Thinking-defaults returns the same level for slash-prefixed and bare GPT ids.

## Migration steps

1.  Scaffold `package/agent-harness-shared/model-selection/` with `package.json`,
     `mise.toml`,
    `rolldown.node.config.ts`,
     `README.md`,
     and its export map.
2.  Add `@monochromatic-dev/agent-harness-shared-model-selection: workspace:*` to advisor,
     auto-mode,
    and thinking-defaults.
3.  Add characterization tests in the three consumers while imports still point at local code.
4.  Extract core types plus `maybe.ts`;
     update advisor imports or re-exports.
5.  Extract `model-id.ts`;
     update advisor and thinking-defaults.
6.  Extract `exact-match.ts`;
     update advisor.
7.  Extract `pattern-match.ts`;
     rewrite linear recursion during the move;
     update advisor.
8.  Extract `version.ts`;
     update auto-mode.
9.  Extract `scope-patterns.ts`;
     update advisor.
10. Extract `argv-scope.ts`;
     update advisor.
11. Extract `settings-scope.ts`;
     generalize `AdvisorSettingsFileSchema` to `PiSettingsFileSchema`.
12. Extract `scope-resolver.ts`;
     use the narrow scope context interface and update advisor.
13. Extract cost ranking helpers;
     expose newly factored `scoreModel`,
     `compareCostScores`,
    and `buildCostRanking`;
     update advisor wrappers.
14. Extract budget selection and override strategy with injected auth callbacks;
     update auto-mode to a thin
    registry-auth adapter.
15. Move or rewrite unit tests for every shared module.
16. Run `mise run //package/agent-harness-shared/model-selection:buildAndTest`.
17. Run `mise run //package/agent-harness-shared/advisor:buildAndTest`.
18. Run `mise run //package/pi-plugin/auto-mode:build` and `mise run //package/pi-plugin/auto-mode:test:unit`.
19. Run `mise run //package/pi-plugin/thinking-default:build` and
    `mise run //package/pi-plugin/thinking-default:test:unit`.
20. Run type checks for the shared package and three consumers.
21. Run advisor and thinking-defaults extension verification tasks.
22. Remove local files that became empty wrappers only after import rewiring and tests pass.

## Risks and mitigations

- New top-level category:
   a shared category differs from the nested `hook-types` precedent.
  Mitigation:
   document the category rule in the package README and keep actual extensions in `package/pi-plugin/`.
- Single package with a broad root entry:
   one root barrel could make lightweight consumers pay for a heavy runtime.
  Mitigation:
   the package keeps no Pi peer and no Pi import,
   so its single root entry stays free of the Pi SDK;
  Pi-coupled helpers live in the consumers.
- Full budget extraction:
   auto-mode selection is strategy,
   ratio gating,
   and auth sequencing,
  not an inverse advisor cost sort.
  Mitigation:
   shared budget selection accepts injected auth and registry callbacks;
   auto-mode keeps host adapters
  and characterization tests.
- `ReadonlyModel` widening:
   widening `Model<Api>['api']` to `string` can erase useful type information.
  Mitigation:
   use generic structural model types and return the caller's original model type.
- Settings validation:
   `valibot` is runtime code,
   not test-only code.
  Mitigation:
   keep `valibot` in `dependencies`.
- `modelsAreEqual` coupling:
   scope dedup may require pi-ai at runtime.
  Mitigation:
   verify pi-ai equality semantics before replacing it;
   otherwise keep it in the consumer that already depends on pi-ai.
- Promoted helpers:
   `isThinkingLevel`,
   `isAlias`,
   and `hasDateSuffix` move from private helpers to public API.
  Mitigation:
   document them as low-level helpers and cover them with direct tests.
- Existing recursion over strings:
   direct-moving `scope-match.ts` would preserve a stack-risk pattern.
  Mitigation:
   rewrite recursive flat-string scans to iterative scans during extraction.
- Regex in version extraction:
   current version helpers use guarded regexes.
  Mitigation:
   carry scoped disable comments and tests,
   or replace with iterative token scanning if the move touches
  those parsers substantially.
