import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expandQwenTokenPlanThinkingLevelMap,
  mapQwenTokenPlanIndividualProviders,
  qwenTokenPlanIndividualFacts,
  qwenTokenPlanIndividualListingSnapshot,
  QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  type QwenTokenPlanListingEntry,
  type QwenTokenPlanModelFacts,
} from '../dist/final/node/index.mjs';
import {
  piAiQwenTokenPlanIndividualCatalog,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/test-fixtures';
import {
  MEASURED_MEDIA_IDS,
  MEASURED_SERVICEABLE_IDS,
} from './measured-listing-fixtures.ts';

//region Test fixtures

/**
 The owner-settled route pins, byte for byte as `doc/planning/qwen-token-plan-individual-provider.md`
 records them for both harnesses.
 */
const SETTLED_ROUTE_PINS = {
  route: 'qwen-token-plan-individual',
  api: 'openai-completions',
  credentialName: 'QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY',
  baseUrl: 'https://token-plan.maas.qwencloudapi.com/compatible-mode/v1',
} as const;

/**
 Reasoning-without-efforts ids the three-state decision names: the projection
 omits `reasoningEfforts` for them so the installed catalog's capability stands.
 */
const REASONING_WITHOUT_EFFORTS_IDS = [
  'qwen3.6-flash',
  'qwen3.7-max',
  'qwen3.7-plus',
] as const;

/**
 Facts row for a non-reasoning chat model, which the carried table has no row
 for today; injected so the non-reasoning state is exercised end to end.
 */
const NON_REASONING_FACTS: QwenTokenPlanModelFacts = {
  id: 'probe-non-reasoning',
  name: 'Probe Non Reasoning',
  kind: 'chat',
  contextWindow: 128_000,
  maxTokens: 16_384,
  input: ['text'],
  reasoning: false,
  compat: {
    thinkingFormat: 'qwen',
    supportsDeveloperRole: false,
    supportsStore: false,
    supportsStrictMode: true,
    supportsReasoningEffort: false,
  },
};

/**
 Served model one id maps to over the checked-in snapshot, or undefined when
 the mapping dropped it as unserviceable.

 @param id - listing id to look up

 @returns served model profile for that id

 @example
 ```ts
 servedModelOf('qwen3.8-max');
 ```
 */
function servedModelOf(id: string) {
  return mapQwenTokenPlanIndividualProviders({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  })[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]
    ?.models
    ?.find(function byId(model) {
      return model.id === id;
    });
}

/**
 Served ids one listing maps to, in mapping order.

 @param listing - listing entries to map

 @returns served model ids

 @example
 ```ts
 servedIdsOf([{ id: 'first' }]);
 ```
 */
function servedIdsOf(listing: readonly QwenTokenPlanListingEntry[]): readonly string[] {
  return mapQwenTokenPlanIndividualProviders({ listing, })[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]
    ?.models
    ?.map(function byId(model) {
      return model.id;
    },) ?? [];
}

/**
 Facts row the carried table holds for one id, or a loud failure when it holds
 none.

 @param id - model id to look up

 @returns carried facts row

 @example
 ```ts
 factsRowOf('qwen3.8-max');
 ```
 */
function factsRowOf(id: string): QwenTokenPlanModelFacts {
  /**
   Facts row the carried table holds for the id, before the premise check.
   */
  const row = qwenTokenPlanIndividualFacts.find(function byId(candidate) {
    return candidate.id === id;
  },);
  if (row === undefined) {
    throw new Error(`expected the carried facts table to hold a row for ${id}`);
  }
  return row;
}

//endregion Test fixtures

await describe({
  name: '',
  children: [
    //region Listing to provider dict

    describe({
      name: mapQwenTokenPlanIndividualProviders.name,
      children: [
        it({
          name: 'serves exactly the serviceable ids the measured listing carried',
          fn: async () => {
            expect(servedIdsOf(qwenTokenPlanIndividualListingSnapshot.entries).toSorted())
              .toEqual([...MEASURED_SERVICEABLE_IDS].toSorted());
          },
        },),
        it({
          name: 'serves none of the media ids the measured listing carried',
          fn: async () => {
            /**
             Served ids that the measured listing classified as media; the
             media skip rule must empty this.
             */
            const servedMediaIds = servedIdsOf(qwenTokenPlanIndividualListingSnapshot.entries)
              .filter(function isMeasuredMedia(id) {
                return (MEASURED_MEDIA_IDS as readonly string[])
                  .includes(id);
              },);
            expect(servedMediaIds).toEqual([]);
          },
        },),
        it({
          name: 'skips an id whose name marks it media even when no facts row covers it',
          fn: async () => {
            expect(servedIdsOf([{ id: 'probe-video', },])).toEqual([]);
          },
        },),
        it({
          name: 'preserves listing order across serviceable models',
          fn: async () => {
            expect(servedIdsOf([
              { id: 'first', },
              { id: 'second', },
              { id: 'third', },
            ],)).toEqual([
              'first',
              'second',
              'third',
            ]);
          },
        },),
        it({
          name: 'pins the credential reference, wire protocol, and endpoint on the route',
          fn: async () => {
            const route = mapQwenTokenPlanIndividualProviders({ listing: [], })[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE];
            expect(route?.apiKeyEnv).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME);
            expect(route?.api).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_API);
            expect(route?.baseURL).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL);
          },
        },),
        it({
          name: 'states the owner-settled route pins byte for byte',
          fn: async () => {
            expect(QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE).toBe(SETTLED_ROUTE_PINS.route);
            expect(QWEN_TOKEN_PLAN_INDIVIDUAL_API).toBe(SETTLED_ROUTE_PINS.api);
            expect(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME)
              .toBe(SETTLED_ROUTE_PINS.credentialName);
            expect(QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL).toBe(SETTLED_ROUTE_PINS.baseUrl);
          },
        },),
      ],
    },),

    //endregion Listing to provider dict

    //region Three reasoning states

    describe({
      name: 'three reasoning states',
      children: [
        it({
          name: 'projects the declared efforts dict for a model that offers selectable efforts',
          fn: async () => {
            /**
             Facts row deciding which efforts `qwen3.8-max` offers; the served
             profile must carry that dict unchanged.
             */
            const factsRow = factsRowOf('qwen3.8-max');
            if (factsRow.efforts === undefined) {
              throw new Error('expected the carried facts table to declare efforts for qwen3.8-max');
            }
            expect(servedModelOf('qwen3.8-max')?.reasoningEfforts).toEqual(factsRow.efforts);
          },
        },),
        ...REASONING_WITHOUT_EFFORTS_IDS.map(function byModelId(id) {
          return it({
            name: `omits reasoningEfforts for ${id}, whose capability the installed catalog holds`,
            fn: async () => {
              const model = servedModelOf(id);
              if (model === undefined) {
                throw new Error(`expected a served model profile for ${id}`);
              }
              expect(Object.hasOwn(
                model,
                'reasoningEfforts'
              ),).toBe(false);
            },
          },);
        },),
        it({
          name: 'omits reasoningEfforts for auto, where the omission reads as "does not reason" in DSH',
          fn: async () => {
            /**
             `auto` is the recorded divergence: no installed catalog entry
             covers it, so the omitted field inherits nothing and the adapter
             sees a non-reasoning model, while pi shows it as reasoning without
             selectable efforts (`doc/planning/qwen-token-plan-individual-provider.md`,
             "three reasoning states").
             */
            const model = servedModelOf('auto');
            if (model === undefined) {
              throw new Error('expected a served model profile for auto');
            }
            expect(Object.hasOwn(
              model,
              'reasoningEfforts'
            ),).toBe(false);
          },
        },),
        it({
          name: 'projects a non-reasoning model as reasoningEfforts false',
          fn: async () => {
            /**
             Served profile the injected non-reasoning row maps to.
             */
            const model = mapQwenTokenPlanIndividualProviders({
              listing: [{ id: NON_REASONING_FACTS.id, },],
              facts: [NON_REASONING_FACTS,],
            },)[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]
              ?.models
              ?.[0];
            expect(model?.reasoningEfforts).toBe(false);
          },
        },),
      ],
    },),

    //endregion Three reasoning states

    //region Builtin row equality

    describe({
      name: 'projected facts against the builtin catalog rows',
      children: Object.values(piAiQwenTokenPlanIndividualCatalog)
        .map(function byRow(row) {
          return it({
            name: `equals the builtin row field for field on ${row.id}`,
            fn: async () => {
              const model = servedModelOf(row.id);
              if (model === undefined) {
                throw new Error(`expected a served model profile for ${row.id}`);
              }
              expect(row.provider).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE);
              expect(row.api).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_API);
              expect(model.id).toBe(row.id);
              expect(model.name).toBe(row.name);
              expect(model.contextWindow).toBe(row.contextWindow);
              expect(model.maxTokens).toBe(row.maxTokens);
              expect(model.input).toEqual(row.input);
              expect(model.compat).toEqual(row.compat);
              if (row.thinkingLevelMap !== undefined) {
                /**
                 Efforts the served profile declares for a builtin row that
                 carries a thinking-level map; expanding it must reproduce that
                 map byte for byte.
                 */
                const efforts = model.reasoningEfforts;
                if ((efforts === undefined) || (efforts === false)) {
                  throw new Error(`expected a declared efforts dict for ${row.id}`);
                }
                expect(expandQwenTokenPlanThinkingLevelMap({ efforts, }))
                  .toEqual(row.thinkingLevelMap);
              } else if (row.reasoning) {
                expect(Object.hasOwn(
                  model,
                  'reasoningEfforts'
                ),).toBe(false);
              } else {
                expect(model.reasoningEfforts).toBe(false);
              }
            },
          },);
        },),
    },),

    //endregion Builtin row equality
  ],
},);
