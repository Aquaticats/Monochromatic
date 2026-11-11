/**
 Tests for the pi projection of shared Qwen Token Plan Individual listing facts.

 The golden assertions read pi-ai's own installed catalog rows
 (tests may read installed dependencies;
 production code must not),
 so a projection that drifts from the builtin row shape fails here even when
 both sides move together.

 @module
 */

import {
  readFileSync,
} from 'node:fs';
import {
  fileURLToPath,
} from 'node:url';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  expandQwenTokenPlanThinkingLevelMap,
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualFacts,
  qwenTokenPlanIndividualListingSnapshot,
  type QwenTokenPlanEffortsDict,
  type QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  hasResolvedEfforts,
  mapQwenTokenPlanIndividualModelCost,
  PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
  PI_SCHEMA_DEFAULT_MAX_TOKENS,
  projectQwenTokenPlanIndividualModel,
  projectQwenTokenPlanIndividualModels,
  QWEN_TOKEN_PLAN_COST_RATE_ZERO,
  QWEN_TOKEN_PLAN_INDIVIDUAL_IMAGE_INPUT_LIMITS,
} from '../dist/final/node/index.mjs';

//region Constants

/**
 Documented output ceiling of the `deepseek-v4.1-flash` family, the catalog
 row's winning number (`doc/planning/qwen-token-plan-individual-provider.md`,
 "Settled decisions": the catalog row wins over the documented 393,216).
 */
const DEEPSEEK_V41_FLASH_MAX_TOKENS = 384_000;

/**
 Documented output ceiling of `qwen4-max-preview`
 (`doc/planning/qwen-token-plan-individual-provider.md`, "Model facts table").
 */
const QWEN4_MAX_PREVIEW_MAX_TOKENS = 393_216;

/**
 Documented context window every named facts row carries
 (`doc/planning/qwen-token-plan-individual-provider.md`, "Model facts table").
 */
const DOCUMENTED_CONTEXT_WINDOW = 1_000_000;

/**
 Model id whose facts the sibling catalog supplies.
 */
const DEEPSEEK_V41_FLASH_1017 = 'deepseek-v4.1-flash-1017';

/**
 Model id whose facts only the research record documents.
 */
const QWEN4_MAX_PREVIEW = 'qwen4-max-preview';

/**
 Router alias id the listing carries and no facts row names.
 */
const ROUTER_ALIAS = 'auto';

/**
 A reasoning model that offers no selectable efforts, one of the three
 reasoning states (`doc/planning/qwen-token-plan-individual-provider.md`,
 "three reasoning states").
 */
const REASONING_WITHOUT_EFFORTS_ID = 'qwen3.7-max';

/**
 A reasoning model that offers selectable efforts.
 */
const REASONING_WITH_EFFORTS_ID = 'qwen3.8-max';

//endregion Constants

//region Fixtures

/**
 pi-ai's builtin rows for this route, keyed by their catalog row key.
 Read from the installed dependency so the golden assertion tracks the rows
 pi actually ships.

 @returns the `openai-completions` row table of the builtin catalog

 @example
 ```typescript
 const rows = readBuiltinCatalogRows();
 ```
 */
function readBuiltinCatalogRows(): Record<string, unknown> {
  /**
   Data directory beside pi-ai's bundled providers entry.
   */
  const dataDirectory = fileURLToPath(new URL(
    './data/',
    import.meta.resolve('@earendil-works/pi-ai/providers/all'),
  ),);
  /**
   Parsed builtin catalog document for this route.
   */
  const document = JSON.parse(readFileSync(
    `${dataDirectory}/qwen-token-plan-individual.json`,
    'utf8',
  ),) as Record<string, unknown>;
  return document['openai-completions'] as Record<string, unknown>;
}

/**
 Resolved facts the shared mapping earns from the checked-in listing.

 @returns one resolved row per serviceable listing id
 */
function resolvedSnapshotFacts(): readonly QwenTokenPlanResolvedModelFacts[] {
  return mapQwenTokenPlanIndividualModels({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  },);
}

/**
 The resolved facts row one id earned.

 @param id - model id to find

 @returns the resolved row

 @throws when the checked-in listing maps no such id
 */
function resolvedFactsFor(
  id: string,
): QwenTokenPlanResolvedModelFacts {
  /**
   Resolved row the mapping produced for this id, when it produced one.
   */
  const facts = resolvedSnapshotFacts()
    .find(function sameId(one,) {
      return one.id === id;
    },);
  if (facts === undefined)
    throw new Error(`the checked-in listing maps no id "${id}"`);
  return facts;
}

/**
 A minimal resolved facts entry the state tests vary.

 @param overrides - fields the entry changes

 @returns a fully decided non-reasoning text-only entry

 @example
 ```typescript
 const facts = resolvedFactsFixture({ reasoning: true });
 ```
 */
function resolvedFactsFixture(overrides: {
  readonly id?: string;
  readonly name?: string;
  readonly contextWindow?: number;
  readonly maxTokens?: number;
  readonly input?: readonly ('text' | 'image')[];
  readonly reasoning?: boolean;
  readonly efforts?: QwenTokenPlanEffortsDict;
} = {},): QwenTokenPlanResolvedModelFacts {
  return {
    id: overrides.id ?? 'fixture-model',
    name: overrides.name ?? 'Fixture Model',
    kind: 'chat',
    contextWindow: overrides.contextWindow ?? PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
    maxTokens: overrides.maxTokens ?? PI_SCHEMA_DEFAULT_MAX_TOKENS,
    input: overrides.input ?? ['text'],
    reasoning: overrides.reasoning ?? false,
    ...(overrides.efforts === undefined ? {} : { efforts: overrides.efforts, }),
    compat: {
      thinkingFormat: 'qwen',
      supportsDeveloperRole: false,
      supportsStore: false,
      supportsStrictMode: true,
      supportsReasoningEffort: overrides.efforts !== undefined,
    },
  };
}

//endregion Fixtures

//region Cost mapping

await describe({
  name: mapQwenTokenPlanIndividualModelCost.name,
  children: [
    it({
      name: 'serves the zero rate on all four pi cost fields',
      fn: async () => {
        expect(mapQwenTokenPlanIndividualModelCost(),).toEqual({
          input: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
          output: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
          cacheRead: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
          cacheWrite: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
        },);
      },
    },),
    it({
      name: 'builds a fresh cost object per call, never a shared one',
      fn: async () => {
        expect(mapQwenTokenPlanIndividualModelCost())
          .not
          .toBe(mapQwenTokenPlanIndividualModelCost(),);
      },
    },),
  ],
},);

//endregion Cost mapping

//region Efforts narrowing

await describe({
  name: hasResolvedEfforts.name,
  children: [
    it({
      name: 'is true when the resolved facts carry an efforts dict',
      fn: async () => {
        expect(hasResolvedEfforts(resolvedFactsFixture({
          reasoning: true,
          efforts: { high: 'high', },
        },),),).toBe(true);
      },
    },),
    it({
      name: 'is false when the resolved facts carry no dict',
      fn: async () => {
        expect(hasResolvedEfforts(resolvedFactsFixture({ reasoning: true, },),),)
          .toBe(false);
      },
    },),
  ],
},);

//endregion Efforts narrowing

//region Golden builtin equality

await describe({
  name: 'golden builtin row equality',
  children: [
    ...Object.entries(readBuiltinCatalogRows(),)
      .map(function goldenRowTest([rowKey, builtinRow,],) {
        /**
         Model id the builtin row carries.
         */
        const builtinId = (builtinRow as { id: string }).id;
        return it({
          name: `projects ${builtinId} to builtin row ${rowKey} value for value`,
          fn: async () => {
            /**
             Projected row the same id earned.
             */
            const projectedRow = projectQwenTokenPlanIndividualModel({
              facts: resolvedFactsFor(builtinId,),
            },);
            expect(projectedRow,).toEqual(builtinRow,);
            expect(Object.keys(projectedRow,)
              .toSorted(),)
              .toEqual(Object.keys(builtinRow,)
                .toSorted(),);
          },
        },);
      },),
    ...Object.entries(readBuiltinCatalogRows(),)
      .map(function imageLimitTest([rowKey, builtinRow,],) {
        /**
         Row fields the image-limit relation reads.
         */
        const row = builtinRow as {
          id: string;
          input: string[];
          inputLimits?: unknown;
        };
        return it({
          name: row.input
            .includes('image')
            ? `pins the image input limits on image-input row ${row.id}`
            : `omits inputLimits entirely on text-only row ${row.id} (row ${rowKey})`,
          fn: async () => {
            /**
             Projected row the same id earned.
             */
            const projectedRow = projectQwenTokenPlanIndividualModel({
              facts: resolvedFactsFor(row.id,),
            },);
            if (row.input
              .includes('image'))
              expect(projectedRow.inputLimits,)
                .toEqual(QWEN_TOKEN_PLAN_INDIVIDUAL_IMAGE_INPUT_LIMITS,);
            else
              expect(Object.hasOwn(projectedRow, 'inputLimits',),)
                .toBe(false);
          },
        },);
      },),
  ],
},);

//endregion Golden builtin equality

//region Three reasoning states

await describe({
  name: 'three reasoning states',
  children: [
    it({
      name: 'projects non-reasoning facts as reasoning false with no thinkingLevelMap',
      fn: async () => {
        /**
         Projected row for a non-reasoning entry.
         */
        const row = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFixture({ reasoning: false, },),
        },);
        expect(row.reasoning,).toBe(false);
        expect(Object.hasOwn(row, 'thinkingLevelMap',),).toBe(false);
      },
    },),
    it({
      name: 'projects reasoning with efforts as reasoning true plus the expanded map',
      fn: async () => {
        /**
         Efforts dict the fixture entry offers.
         */
        const efforts: QwenTokenPlanEffortsDict = {
          low: 'low',
          high: 'high',
        };
        /**
         Projected row for a reasoning entry that offers efforts.
         */
        const row = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFixture({
            reasoning: true,
            efforts,
          },),
        },);
        expect(row.reasoning,).toBe(true);
        expect(row.thinkingLevelMap,)
          .toEqual(expandQwenTokenPlanThinkingLevelMap({ efforts, },),);
      },
    },),
    it({
      name: 'projects reasoning without efforts as reasoning true and no thinkingLevelMap',
      fn: async () => {
        /**
         Projected row for the reasoning-without-efforts state.
         */
        const row = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFixture({ reasoning: true, },),
        },);
        expect(row.reasoning,).toBe(true);
        expect(Object.hasOwn(row, 'thinkingLevelMap',),).toBe(false);
      },
    },),
    it({
      name: 'serves the builtin reasoning-without-efforts shape for the ids that reason silently',
      fn: async () => {
        /**
         Projected row for a reasoning-without-efforts model the facts table
         names.
         */
        const row = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFor(REASONING_WITHOUT_EFFORTS_ID,),
        },);
        expect(row.reasoning,).toBe(true);
        expect(Object.hasOwn(row, 'thinkingLevelMap',),).toBe(false);
        expect(row.compat.supportsReasoningEffort,).toBe(false);
      },
    },),
    it({
      name: 'serves the expanded map for the ids that offer efforts',
      fn: async () => {
        /**
         Resolved facts for a reasoning-with-efforts model.
         */
        const facts = resolvedFactsFor(REASONING_WITH_EFFORTS_ID,);
        /**
         Projected row for the same id.
         */
        const row = projectQwenTokenPlanIndividualModel({ facts, },);
        expect(row.reasoning,).toBe(true);
        expect(hasResolvedEfforts(facts,),).toBe(true);
        expect(row.thinkingLevelMap,)
          .toEqual(expandQwenTokenPlanThinkingLevelMap({
            efforts: facts.efforts ?? {},
          },),);
      },
    },),
  ],
},);

//endregion Three reasoning states

//region Compat pins

await describe({
  name: 'compat pins',
  children: [
    it({
      name: 'copies the wire-compat pins the facts table declares',
      fn: async () => {
        /**
         Resolved facts carrying the declared pins.
         */
        const facts = resolvedFactsFixture({
          reasoning: true,
          efforts: { max: 'max', },
        },);
        /**
         Projected row for the same entry.
         */
        const row = projectQwenTokenPlanIndividualModel({ facts, },);
        expect(row.compat,).toEqual(facts.compat,);
        expect(row.compat,).toEqual({
          thinkingFormat: 'qwen',
          supportsDeveloperRole: false,
          supportsStore: false,
          supportsStrictMode: true,
          supportsReasoningEffort: true,
        },);
      },
    },),
    it({
      name: 'pins supportsReasoningEffort false exactly when the model offers no efforts',
      fn: async () => {
        /**
         Projected rows for one entry of each effort-offering shape.
         */
        const withEfforts = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFixture({
            reasoning: true,
            efforts: { high: 'high', },
          },),
        },);
        const withoutEfforts = projectQwenTokenPlanIndividualModel({
          facts: resolvedFactsFixture({ reasoning: true, },),
        },);
        expect(withEfforts.compat.supportsReasoningEffort,).toBe(true);
        expect(withoutEfforts.compat.supportsReasoningEffort,).toBe(false);
      },
    },),
    ...Object.entries(readBuiltinCatalogRows(),)
      .map(function compatPinsTest([, builtinRow,],) {
        /**
         Builtin row fields the pin relation reads.
         */
        const row = builtinRow as { id: string; compat: unknown };
        return it({
          name: `keeps the builtin compat pins on ${row.id}`,
          fn: async () => {
            /**
             Projected row the same id earned.
             */
            const projectedRow = projectQwenTokenPlanIndividualModel({
              facts: resolvedFactsFor(row.id,),
            },);
            expect(projectedRow.compat,)
              .toEqual(row.compat,);
          },
        },);
      },),
  ],
},);

//endregion Compat pins

//region Listing ids the builtin lacks

await describe({
  name: 'listing ids the builtin lacks',
  children: [
    it({
      name: 'projects deepseek-v4.1-flash-1017 from the core resolved facts',
      fn: async () => {
        /**
         Resolved facts the shared mapping earns for the id.
         */
        const facts = resolvedFactsFor(DEEPSEEK_V41_FLASH_1017,);
        /**
         Projected row for the same id.
         */
        const row = projectQwenTokenPlanIndividualModel({ facts, },);
        expect(row.id,).toBe(DEEPSEEK_V41_FLASH_1017);
        expect(row.name,).toBe(facts.name);
        expect(row.contextWindow,).toBe(DOCUMENTED_CONTEXT_WINDOW);
        expect(row.maxTokens,).toBe(DEEPSEEK_V41_FLASH_MAX_TOKENS);
        expect(row.input,).toEqual([...facts.input]);
        expect(row.reasoning,).toBe(true);
        expect(row.thinkingLevelMap,)
          .toEqual(expandQwenTokenPlanThinkingLevelMap({
            efforts: facts.efforts ?? {},
          },),);
      },
    },),
    it({
      name: 'projects qwen4-max-preview from the core resolved facts',
      fn: async () => {
        /**
         Resolved facts the shared mapping earns for the id.
         */
        const facts = resolvedFactsFor(QWEN4_MAX_PREVIEW,);
        /**
         Projected row for the same id.
         */
        const row = projectQwenTokenPlanIndividualModel({ facts, },);
        expect(row.id,).toBe(QWEN4_MAX_PREVIEW);
        expect(row.name,).toBe(facts.name);
        expect(row.contextWindow,).toBe(DOCUMENTED_CONTEXT_WINDOW);
        expect(row.maxTokens,).toBe(QWEN4_MAX_PREVIEW_MAX_TOKENS);
        expect(row.input,).toEqual([...facts.input]);
        expect(row.reasoning,).toBe(true);
        expect(row.thinkingLevelMap,)
          .toEqual(expandQwenTokenPlanThinkingLevelMap({
            efforts: facts.efforts ?? {},
          },),);
      },
    },),
    it({
      name: 'projects the auto router alias with the schema defaults and no efforts',
      fn: async () => {
        /**
         Resolved facts the shared mapping earns for the router alias.
         */
        const facts = resolvedFactsFor(ROUTER_ALIAS,);
        /**
         Projected row for the same id.
         */
        const row = projectQwenTokenPlanIndividualModel({ facts, },);
        expect(row.id,).toBe(ROUTER_ALIAS);
        expect(row.name,).toBe(ROUTER_ALIAS);
        expect(row.contextWindow,).toBe(PI_SCHEMA_DEFAULT_CONTEXT_WINDOW);
        expect(row.maxTokens,).toBe(PI_SCHEMA_DEFAULT_MAX_TOKENS);
        expect(row.input,).toEqual(['text']);
        expect(row.reasoning,).toBe(true);
        expect(Object.hasOwn(row, 'thinkingLevelMap',),).toBe(false);
      },
    },),
    it({
      name: 'projects every serviceable id the facts table names',
      fn: async () => {
        /**
         Projected catalog the checked-in listing earns.
         */
        const projected = projectQwenTokenPlanIndividualModels({
          facts: resolvedSnapshotFacts(),
        },);
        /**
         Ids the facts table names as chat or router.
         */
        const serviceableIds = qwenTokenPlanIndividualFacts
          .filter(function serviceable(facts,) {
            return (facts.kind === 'chat') || (facts.kind === 'router');
          },)
          .map(function toId(facts,) {
            return facts.id;
          },);
        expect(projected.length,)
          .toBeGreaterThanOrEqual(serviceableIds.length);
        for (const id of serviceableIds)
          expect(projected.some(function serves(one,) {
            return one.id === id;
          },),)
            .toBe(true);
      },
    },),
  ],
},);

//endregion Listing ids the builtin lacks
