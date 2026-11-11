# Intentional nullish-union suppressions

## Rule and configuration

`package/config/oxlint/src/rule/restriction.ts` enables
`no-restricted-syntax/no-nullish-union` at error severity.
It rejects `T | null` and `T | undefined` unions
because this repo models absence with optional properties
under `exactOptionalPropertyTypes`
(`no-restricted-syntax/no-optional-escape` bans the `Partial` and
sentinel workarounds).

## Why the config cannot express this package

This package mirrors two upstream wire shapes exactly
(`doc/decision/pi-model-retirement.md` records the same
mirror-first posture):

- `QwenTokenPlanEffortsDict` and `QwenTokenPlanThinkingLevelMap` mirror
  pi-ai's `thinkingLevelMap` value shape
  (`@earendil-works/pi-ai` `providers/model-schema.ts`)
  and `dsh-llm-pi-ai`'s
  `PiAiReasoningEfforts = Partial<Record<ModelThinkingLevel, string | null>>`
  (`lib/types/catalog.d.ts`).
  In those shapes `null` is not absence:
  it is the instruction "level unsupported",
  distinct from an absent key ("supported,
  dispatch nothing").
  The expansion must keep both spellings reachable,
  or the pi projection would silently change dispatch behavior.
- `PiAiQwenTokenPlanIndividualCatalogRow['thinkingLevelMap']`
  (`src/test-fixtures/fixture.pi-ai-qwen-token-plan-individual-catalog.ts`)
  mirrors the same pi-ai map shape,
  so the golden comparison reads the builtin rows' own `null` spellings
  instead of normalizing them away.
- `QwenTokenPlanListingFetcher`'s `apiKey` field and
  `fetchQwenTokenPlanIndividualListing`'s `apiKey` option
  mirror the pinned export surface's `apiKey: string | undefined`
  (`doc/planning/qwen-token-plan-individual-provider.md`
  section "Shared core export surface"),
  where `undefined` is the explicit "holds no credential" instruction,
  distinct from an absent optional under `exactOptionalPropertyTypes`.

The statement-scoped suppression in `expandQwenTokenPlanThinkingLevelMap`
covers only the mutable scratch record that builds the exported map;
the exported declarations carry their own documented suppressions.
The facts table and the mapping need none,
since their dicts carry spellings only.
