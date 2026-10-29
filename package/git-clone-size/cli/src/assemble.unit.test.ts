/**
 Tests for estimator assembly, pending computation, and snapshot building.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  buildEstimates,
  buildSnapshot,
  computePending,
  type Signals,
} from './assemble.ts';
import type { Source, } from './source.ts';
import type { FusionState, } from './types.ts';

/**
 A minimal fused state for snapshot tests.
 */
const FUSED: FusionState = {
  point: 100,
  lo: 80,
  hi: 200,
  confidence: 'low',
  basis: ['prior',],
};

/**
 Maps estimate names for concise assertions.
 
 @param signals - signal accumulator
 
 @returns estimator names produced for the signals
 */
function namesFor(signals: Signals): readonly string[] {
  return buildEstimates({ signals, defaultBranchOnly: false, }).map((estimate) => estimate.name);
}

await describe({
  name: buildEstimates.name,
  children: [
    it({
      name: 'falls back to the no-tip prior when no signals exist',
      fn: async ({ expect, }) => {
        expect(namesFor({})).toEqual(['prior (no signals yet)',]);
      },
    }),

    it({
      name: 'adds the prior from a known shallow tip',
      fn: async ({ expect, }) => {
        expect(namesFor({ shallowBytes: 1_000, })).toContain('snapshot-multiplier prior');
      },
    }),

    it({
      name: 'adds the deepen estimator when a deepen signal and tip are present',
      fn: async ({ expect, }) => {
        const names = namesFor({
          shallowBytes: 1_000,
          deepen: { marginalLo: 8, marginalPoint: 10, marginalHi: 14, observedCommits: 50, hitCap: false, },
        });
        expect(names.some((name) => name.startsWith('deepen-extrapolation'))).toBe(true);
      },
    }),
  ],
});

await describe({
  name: computePending.name,
  children: [
    it({
      name: 'reports local-exact pending for an unmeasured local repo',
      fn: async ({ expect, }) => {
        const source: Source = { kind: 'local', path: '/repo', };
        expect(computePending({ signals: {}, source, })).toEqual(['local-exact',]);
        expect(computePending({
          signals: { local: { fullBytes: 1, confidence: 'very high', basis: 'x', }, },
          source,
        })).toEqual([]);
      },
    }),

    it({
      name: 'omits host-proxy for an unsupported remote host',
      fn: async ({ expect, }) => {
        const source: Source = { kind: 'remote', url: 'u', host: 'unknown', };
        expect(computePending({ signals: {}, source, })).not.toContain('host-proxy');
      },
    }),
  ],
});

await describe({
  name: buildSnapshot.name,
  children: [
    it({
      name: 'omits shallow/ratio/savings until a shallow measurement exists',
      fn: async ({ expect, }) => {
        const snapshot = buildSnapshot({ fused: FUSED, metric: 'm', scope: 's', pending: ['shallow',], done: false, });
        expect(snapshot.shallow).toBeUndefined();
        expect(snapshot.ratio).toBeUndefined();
        expect(snapshot.savings).toBeUndefined();
        expect(snapshot.full.confidence).toBe('low');
      },
    }),

    it({
      name: 'includes ratio and savings once shallow is known',
      fn: async ({ expect, }) => {
        const snapshot = buildSnapshot({
          fused: FUSED,
          shallowBytes: 20,
          metric: 'm',
          scope: 's',
          pending: [],
          done: true,
        });
        expect(snapshot.shallow?.bytes).toBe(20);
        expect(snapshot.ratio?.point).toBe(0.2);
        expect(snapshot.savings?.point).toBe(80);
      },
    }),
  ],
});
