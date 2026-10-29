/**
 Tests for the individual estimators (deepen math, branch correction, priors).
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  PRIOR_ABSENT_TIP_BYTES,
  PRIOR_MULTIPLIER,
} from './constants.ts';
import {
  churnEstimate,
  deepenEstimate,
  priorAbsentEstimate,
  priorEstimate,
} from './estimators.ts';

await describe({
  name: deepenEstimate.name,
  children: [
    it({
      name: 'extrapolates full = C1 + m*(N-1) on the default branch',
      fn: async ({ expect, }) => {
        const estimate = deepenEstimate({
          c1Bytes: 1_000,
          marginal: { lo: 8, point: 10, hi: 14, },
          commitCount: 101,
          commitUncertain: false,
          branches: 1,
          defaultBranchOnly: false,
        });
        expect(estimate.point).toBe(2_000);
        expect(estimate.lo).toBe(1_800);
        expect(estimate.hi).toBe(2_400);
        expect(estimate.confidence).toBe('medium');
      },
    }),

    it({
      name: 'widens the high end and drops to low when the count is a lower bound',
      fn: async ({ expect, }) => {
        const estimate = deepenEstimate({
          c1Bytes: 1_000,
          marginal: { lo: 8, point: 10, hi: 14, },
          commitCount: 101,
          commitUncertain: true,
          branches: 1,
          defaultBranchOnly: false,
        });
        expect(estimate.hi).toBe(1_000 + (14 * ((101 * 2) - 1)));
        expect(estimate.confidence).toBe('low');
      },
    }),

    it({
      name: 'adds the side-branch contribution unless restricted to the default branch',
      fn: async ({ expect, }) => {
        const corrected = deepenEstimate({
          c1Bytes: 1_000,
          marginal: { lo: 8, point: 10, hi: 14, },
          commitCount: 101,
          commitUncertain: false,
          branches: 11,
          defaultBranchOnly: false,
        });
        const restricted = deepenEstimate({
          c1Bytes: 1_000,
          marginal: { lo: 8, point: 10, hi: 14, },
          commitCount: 101,
          commitUncertain: false,
          branches: 11,
          defaultBranchOnly: true,
        });
        expect(corrected.point).toBeGreaterThan(restricted.point);
        expect(corrected.hi).toBeGreaterThan(restricted.hi);
        expect(restricted.point).toBe(2_000);
      },
    }),
  ],
});

await describe({
  name: churnEstimate.name,
  children: [
    it({
      name: 'scales the tip size by the discounted churn factor',
      fn: async ({ expect, }) => {
        const estimate = churnEstimate({ c1Bytes: 1_000, distinctPathObjects: 900, tipFiles: 150, });
        expect(estimate.point).toBe(3_000);
        expect(estimate.lo).toBe(1_200);
        expect(estimate.hi).toBe(6_000);
        expect(estimate.confidence).toBe('low');
      },
    }),
  ],
});

await describe({
  name: priorEstimate.name,
  children: [
    it({
      name: 'multiplies the tip size by the prior multiplier band',
      fn: async ({ expect, }) => {
        const estimate = priorEstimate({ c1Bytes: 1_000, });
        expect(estimate.point).toBe(1_000 * PRIOR_MULTIPLIER.point);
        expect(estimate.lo).toBe(1_000 * PRIOR_MULTIPLIER.lo);
        expect(estimate.hi).toBe(1_000 * PRIOR_MULTIPLIER.hi);
      },
    }),
  ],
});

await describe({
  name: priorAbsentEstimate.name,
  children: [
    it({
      name: 'emits the no-tip absolute prior band',
      fn: async ({ expect, }) => {
        const estimate = priorAbsentEstimate();
        expect(estimate.point).toBe(PRIOR_ABSENT_TIP_BYTES.point);
        expect(estimate.lo).toBe(PRIOR_ABSENT_TIP_BYTES.lo);
        expect(estimate.hi).toBe(PRIOR_ABSENT_TIP_BYTES.hi);
        expect(estimate.confidence).toBe('low');
      },
    }),
  ],
});
