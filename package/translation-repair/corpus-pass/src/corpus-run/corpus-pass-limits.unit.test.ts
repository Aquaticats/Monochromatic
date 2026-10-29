/**
 Tests for the fixed limits a corpus pass runs under, which an operator's
 launch line and every artifact's comparability read off.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HARD_CAP_MINUTES,
  PASS_MS_PER_MINUTE,
  PLAN_PREVIEW_COUNT,
  SOFT_BUDGET_MS,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'corpus-pass-limits',
  children: [
    it({
      name: 'COUNTS A MINUTE as sixty thousand milliseconds',
      fn: async () => {
        expect(PASS_MS_PER_MINUTE,).toBe(60_000,);
      },
    },),
    it({
      name: 'STOPS STARTING ENTRIES after three days, and ENDS ONE ENTRY after seven hours',
      fn: async () => {
        expect(SOFT_BUDGET_MS,).toBe(259_200_000,);
        expect(HARD_CAP_MINUTES,).toBe(420,);
        expect(HARD_CAP_MINUTES * PASS_MS_PER_MINUTE,).toBe(25_200_000,);
      },
    },),
    it({
      name: 'PREVIEWS five entry ids on the plan line',
      fn: async () => {
        expect(PLAN_PREVIEW_COUNT,).toBe(5,);
      },
    },),
  ],
},);
