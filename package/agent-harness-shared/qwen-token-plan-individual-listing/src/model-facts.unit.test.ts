import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  qwenTokenPlanIndividualFacts,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'curated facts table',
  children: [
    it({
      name: 'pins each row compat.supportsReasoningEffort to the efforts it offers',
      fn: async () => {
        for (const row of qwenTokenPlanIndividualFacts) {
          expect(row.compat.supportsReasoningEffort)
            .toBe(row.efforts !== undefined);
        }
      },
    }),
    it({
      name: 'carries the builtin compat pins on every row',
      fn: async () => {
        for (const row of qwenTokenPlanIndividualFacts) {
          expect(row.compat).toEqual(row.efforts === undefined
            ? QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS
            : QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS);
        }
      },
    }),
    it({
      name: 'gives every chat row the facts a projection copies without guessing',
      fn: async () => {
        for (const row of qwenTokenPlanIndividualFacts) {
          if (row.kind !== 'chat') {
            continue;
          }
          expect(row.name.length).toBeGreaterThan(0);
          expect(row.contextWindow).toBeGreaterThan(0);
          expect(row.maxTokens).toBeGreaterThan(0);
          expect(row.input.length).toBeGreaterThan(0);
        }
      },
    }),
    it({
      name: 'keeps media rows unserviceable: no chat input and no token numbers',
      fn: async () => {
        for (const row of qwenTokenPlanIndividualFacts) {
          if (row.kind === 'chat') {
            continue;
          }
          expect(row.input).toEqual([]);
          expect(row.contextWindow).toBeUndefined();
          expect(row.maxTokens).toBeUndefined();
          expect(row.reasoning).toBe(false);
        }
      },
    }),
    it({
      name: 'carries no row for auto, which serves with pi schema defaults instead',
      fn: async () => {
        expect(qwenTokenPlanIndividualFacts.some(function namesAuto(row) {
          return row.id === 'auto';
        })).toBe(false);
      },
    }),
  ],
});
