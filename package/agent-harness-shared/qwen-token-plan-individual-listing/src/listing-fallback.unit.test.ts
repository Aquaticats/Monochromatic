import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mapQwenTokenPlanIndividualModels,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
  QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
  type QwenTokenPlanListingEntry,
  type QwenTokenPlanModelFacts,
} from '../dist/final/node/index.mjs';

/**
 Sibling rows one synthetic `widget-max` family carries: the nearest older
 sibling discloses no context window and no efforts, so the walk must reach
 past it per field. Values are invented for the relation under test.
 */
const WIDGET3_ROW: QwenTokenPlanModelFacts = {
  id: 'widget3-max',
  name: 'Widget3 Max',
  kind: 'chat',
  maxTokens: 64_000,
  input: [
    'text',
  ],
  reasoning: true,
  compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
};

/**
 Second sibling in the walk, which fills the fields the nearest one lacks.
 */
const WIDGET2_ROW: QwenTokenPlanModelFacts = {
  id: 'widget2-max',
  name: 'Widget2 Max',
  kind: 'chat',
  contextWindow: 500_000,
  maxTokens: 32_000,
  input: [
    'text',
    'image',
  ],
  reasoning: true,
  efforts: {
    high: 'high',
  },
  compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
};

/**
 Oldest sibling in the walk, which fills nothing while the nearer rows speak.
 */
const WIDGET1_ROW: QwenTokenPlanModelFacts = {
  id: 'widget1-max',
  name: 'Widget1 Max',
  kind: 'chat',
  contextWindow: 250_000,
  maxTokens: 16_000,
  input: [
    'text',
    'image',
  ],
  reasoning: true,
  efforts: {
    low: 'low',
  },
  compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
};

/**
 All three synthetic siblings in nearest-first walk order.
 */
const WIDGET_SIBLINGS: readonly QwenTokenPlanModelFacts[] = [
  WIDGET3_ROW,
  WIDGET2_ROW,
  WIDGET1_ROW,
];

/**
 Single resolved model one synthetic listing maps to, or a thrown error when
 the mapping dropped it.
 */
function mappedModelOf({
  listing,
  facts,
  id,
}: {
  readonly listing: readonly QwenTokenPlanListingEntry[];
  readonly facts: readonly QwenTokenPlanModelFacts[];
  readonly id: string;
}) {
  const model = mapQwenTokenPlanIndividualModels({
    listing,
    facts,
  }).find(function matchesId(candidate) {
    return candidate.id === id;
  });
  if (model === undefined) {
    throw new Error(`mapping dropped expected model ${id}`);
  }
  return model;
}

await describe({
  name: 'sibling fallback walk',
  children: [
    it({
      name: 'fills each field from the first older sibling that has it, nearest first',
      fn: async () => {
        expect(mappedModelOf({
          id: 'widget4-max',
          facts: WIDGET_SIBLINGS,
          listing: [
            {
              id: 'widget4-max',
              created: 4_000,
            },
            {
              id: 'widget3-max',
              created: 3_000,
            },
            {
              id: 'widget2-max',
              created: 2_000,
            },
            {
              id: 'widget1-max',
              created: 1_000,
            },
          ],
        })).toEqual({
          id: 'widget4-max',
          name: 'widget4-max',
          kind: 'chat',
          contextWindow: 500_000,
          maxTokens: 64_000,
          input: [
            'text',
          ],
          reasoning: true,
          efforts: {
            high: 'high',
          },
          compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
        });
      },
    }),
    it({
      name: 'orders the walk by parsed versions when the listing carries no created stamps',
      fn: async () => {
        expect(mappedModelOf({
          id: 'widget4-max',
          facts: WIDGET_SIBLINGS,
          listing: [
            {
              id: 'widget4-max',
            },
          ],
        })).toEqual({
          id: 'widget4-max',
          name: 'widget4-max',
          kind: 'chat',
          contextWindow: 500_000,
          maxTokens: 64_000,
          input: [
            'text',
          ],
          reasoning: true,
          efforts: {
            high: 'high',
          },
          compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
        });
      },
    }),
    it({
      name: 'leaves newer siblings out of the walk',
      fn: async () => {
        expect(mappedModelOf({
          id: 'widget2.5-max',
          facts: WIDGET_SIBLINGS,
          listing: [
            {
              id: 'widget2.5-max',
              created: 2_500,
            },
            {
              id: 'widget3-max',
              created: 3_000,
            },
            {
              id: 'widget2-max',
              created: 2_000,
            },
            {
              id: 'widget1-max',
              created: 1_000,
            },
          ],
        })).toEqual({
          id: 'widget2.5-max',
          name: 'widget2.5-max',
          kind: 'chat',
          contextWindow: 500_000,
          maxTokens: 32_000,
          input: [
            'text',
            'image',
          ],
          reasoning: true,
          efforts: {
            high: 'high',
          },
          compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
        });
      },
    }),
    it({
      name: 'excludes a sibling that only ties the unknown id in age',
      fn: async () => {
        expect(mappedModelOf({
          id: 'WIDGET3-MAX',
          facts: WIDGET_SIBLINGS,
          listing: [
            {
              id: 'WIDGET3-MAX',
              created: 3_000,
            },
            {
              id: 'widget3-max',
              created: 3_000,
            },
            {
              id: 'widget2-max',
              created: 2_000,
            },
          ],
        }).maxTokens).toBe(32_000);
      },
    }),
    it({
      name: 'falls to the schema defaults when every sibling is newer',
      fn: async () => {
        expect(mappedModelOf({
          id: 'widget0-max',
          facts: WIDGET_SIBLINGS,
          listing: [
            {
              id: 'widget0-max',
              created: 500,
            },
            {
              id: 'widget3-max',
              created: 3_000,
            },
            {
              id: 'widget2-max',
              created: 2_000,
            },
            {
              id: 'widget1-max',
              created: 1_000,
            },
          ],
        })).toEqual({
          id: 'widget0-max',
          name: 'widget0-max',
          kind: 'chat',
          contextWindow: QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
          maxTokens: QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
          input: [
            'text',
          ],
          reasoning: true,
          compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
        });
      },
    }),
    it({
      name: 'never draws facts from a media kind sibling',
      fn: async () => {
        expect(mappedModelOf({
          id: 'widget4-max',
          facts: [
            {
              id: 'widget3-max',
              name: 'Widget3 Image Max',
              kind: 'image',
              contextWindow: 900_000,
              maxTokens: 64_000,
              input: [],
              reasoning: false,
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
            },
            WIDGET2_ROW,
          ],
          listing: [
            {
              id: 'widget4-max',
              created: 4_000,
            },
            {
              id: 'widget3-max',
              created: 3_000,
            },
            {
              id: 'widget2-max',
              created: 2_000,
            },
          ],
        })).toEqual({
          id: 'widget4-max',
          name: 'widget4-max',
          kind: 'chat',
          contextWindow: 500_000,
          maxTokens: 32_000,
          input: [
            'text',
            'image',
          ],
          reasoning: true,
          efforts: {
            high: 'high',
          },
          compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
        });
      },
    }),
  ],
});
