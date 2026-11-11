import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compareQwenTokenPlanModelAgeKeys,
  qwenTokenPlanModelAgeKey,
  qwenTokenPlanModelFamilyKey,
} from '../dist/final/node/index.mjs';

/**
 Family key cases the planning doc's fallback rule names, each expected key
 derived from the id shape it documents.
 */
const FAMILY_KEY_CASES = [
  {
    id: 'qwen4-max-preview',
    familyKey: 'qwen-max',
  },
  {
    id: 'qwen3.8-max',
    familyKey: 'qwen-max',
  },
  {
    id: 'qwen3.7-max',
    familyKey: 'qwen-max',
  },
  {
    id: 'qwen3.8-max-0902',
    familyKey: 'qwen-max',
  },
  {
    id: 'deepseek-v4.1-flash-1017',
    familyKey: 'deepseek-flash',
  },
  {
    id: 'deepseek-v4.1-flash',
    familyKey: 'deepseek-flash',
  },
  {
    id: 'deepseek-v4-flash-0731',
    familyKey: 'deepseek-flash',
  },
  {
    id: 'deepseek-v4-pro',
    familyKey: 'deepseek-pro',
  },
  {
    id: 'deepseek-v4-pro-0813',
    familyKey: 'deepseek-pro',
  },
  {
    id: 'glm-5.2',
    familyKey: 'glm',
  },
  {
    id: 'glm-5.3',
    familyKey: 'glm',
  },
  {
    id: 'auto',
    familyKey: 'auto',
  },
  {
    id: 'QWEN3.8-MAX',
    familyKey: 'qwen-max',
  },
  {
    id: 'v4',
    familyKey: 'v4',
  },
] as const;

await describe({
  name: '',
  children: [
    describe({
      name: qwenTokenPlanModelFamilyKey.name,
      children: FAMILY_KEY_CASES.map(function familyCase(testCase) {
        return it({
          name: `normalizes ${testCase.id} to ${testCase.familyKey}`,
          fn: async () => {
            expect(qwenTokenPlanModelFamilyKey(testCase.id))
              .toBe(testCase.familyKey);
          },
        });
      }),
    }),

    describe({
      name: compareQwenTokenPlanModelAgeKeys.name,
      children: [
        it({
          name: 'orders by listing created when both ids carry one',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({
                id: 'newer-model',
                created: 20,
              }),
              right: qwenTokenPlanModelAgeKey({
                id: 'older-model',
                created: 10,
              }),
            })).toBeGreaterThan(0);
          },
        }),
        it({
          name: 'falls to parsed versions when either id lacks created',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({ id: 'widget4-max', }),
              right: qwenTokenPlanModelAgeKey({ id: 'widget3-max', }),
            })).toBeGreaterThan(0);
          },
        }),
        it({
          name: 'reads a trailing date suffix as the snapshot component',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({
                id: 'deepseek-v4.1-flash-1017',
              }),
              right: qwenTokenPlanModelAgeKey({ id: 'deepseek-v4.1-flash', }),
            })).toBeGreaterThan(0);
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({ id: 'deepseek-v4.1-flash', }),
              right: qwenTokenPlanModelAgeKey({ id: 'deepseek-v4-flash-0731', }),
            })).toBeGreaterThan(0);
          },
        }),
        it({
          name: 'treats a longer version tuple as newer when it extends the shorter',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({ id: 'widget3.5-max', }),
              right: qwenTokenPlanModelAgeKey({ id: 'widget3-max', }),
            })).toBeGreaterThan(0);
          },
        }),
        it({
          name: 'lets version decide when both ids share one created stamp',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({
                id: 'widget3-max',
                created: 30,
              }),
              right: qwenTokenPlanModelAgeKey({
                id: 'widget2-max',
                created: 30,
              }),
            })).toBeGreaterThan(0);
          },
        }),
        it({
          name: 'reports indistinguishable ids as one age',
          fn: async () => {
            expect(compareQwenTokenPlanModelAgeKeys({
              left: qwenTokenPlanModelAgeKey({ id: 'widget3-max', }),
              right: qwenTokenPlanModelAgeKey({ id: 'widget3-max', }),
            })).toBe(0);
          },
        }),
      ],
    }),
  ],
});
