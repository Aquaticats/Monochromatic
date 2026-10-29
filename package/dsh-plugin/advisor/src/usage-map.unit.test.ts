/**
 Unit tests for the DSH usage projection.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { mapDshUsageToPiUsage, } from '../dist/final/node/index.mjs';

await describe({
  name: mapDshUsageToPiUsage.name,
  children: [
    it({
      name: 'preserves the provider total and zero costs',
      fn: async () => {
        const usage = mapDshUsageToPiUsage({
          inputTokens: 10,
          outputTokens: 5,
          totalTokens: 17,
          cacheReadTokens: 1,
          cacheWriteTokens: 1,
          reasoningTokens: 3,
        },);
        expect(usage.input,).toBe(10,);
        expect(usage.output,).toBe(5,);
        expect(usage.cacheRead,).toBe(1,);
        expect(usage.cacheWrite,).toBe(1,);
        expect(usage.totalTokens,).toBe(17,);
        expect(usage.reasoning,).toBe(3,);
        expect(usage.cost.total,).toBe(0,);
      },
    },),
    it({
      name: 'falls back to the disjoint-counter sum when no total is reported',
      fn: async () => {
        const usage = mapDshUsageToPiUsage({
          inputTokens: 10,
          outputTokens: 5,
          cacheReadTokens: 2,
          cacheWriteTokens: 3,
        },);
        expect(usage.totalTokens,).toBe(20,);
        expect(usage.cacheRead,).toBe(2,);
        expect(usage.cacheWrite,).toBe(3,);
      },
    },),
    it({
      name: 'omits reasoning when the provider reports none',
      fn: async () => {
        const usage = mapDshUsageToPiUsage({
          inputTokens: 1,
          outputTokens: 1,
        },);
        expect('reasoning' in usage,).toBe(false,);
        expect(usage.totalTokens,).toBe(2,);
      },
    },),
  ],
},);
