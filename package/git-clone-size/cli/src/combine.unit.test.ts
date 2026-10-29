/**
 Tests for the statistical fusion: dominance, tightening, and disagreement.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import { combineEstimates, } from './combine.ts';

await describe({
  name: combineEstimates.name,
  children: [
    it({
      name: 'lets an exact local value dominate and collapses to its band',
      fn: async ({ expect, }) => {
        const fused = combineEstimates({
          estimates: [
            { point: 1_000, lo: 995, hi: 1_005, weight: 1_000, confidence: 'very high', name: 'local', },
            { point: 5_000, lo: 1_200, hi: 30_000, weight: 1, confidence: 'low', name: 'prior', },
          ],
        });
        expect(fused.confidence).toBe('very high');
        expect(fused.point).toBe(1_000);
        expect(fused.lo).toBe(995);
        expect(fused.hi).toBe(1_005);
        expect(fused.basis).toEqual(['local',]);
      },
    }),

    it({
      name: 'preserves a single estimate band exactly',
      fn: async ({ expect, }) => {
        const fused = combineEstimates({
          estimates: [{ point: 100, lo: 80, hi: 120, weight: 20, confidence: 'medium', name: 'only', },],
        });
        expect(fused.lo).toBe(80);
        expect(fused.hi).toBe(120);
        expect(fused.point).toBe(100);
        expect(fused.confidence).toBe('medium');
      },
    }),

    it({
      name: 'tightens the band and bumps confidence when two estimators agree',
      fn: async ({ expect, }) => {
        const fused = combineEstimates({
          estimates: [
            { point: 100, lo: 80, hi: 120, weight: 20, confidence: 'medium', name: 'a', },
            { point: 100, lo: 80, hi: 120, weight: 20, confidence: 'medium', name: 'b', },
          ],
        });
        expect(fused.lo).toBeGreaterThan(80);
        expect(fused.hi).toBeLessThan(120);
        expect(fused.confidence).toBe('high');
      },
    }),

    it({
      name: 'widens to the union and downgrades when estimators conflict',
      fn: async ({ expect, }) => {
        const fused = combineEstimates({
          estimates: [
            { point: 100, lo: 90, hi: 110, weight: 20, confidence: 'medium', name: 'a', },
            { point: 300, lo: 290, hi: 310, weight: 20, confidence: 'medium', name: 'b', },
          ],
        });
        expect(fused.lo).toBe(90);
        expect(fused.hi).toBe(310);
        expect(fused.confidence).toBe('low');
      },
    }),
  ],
});
