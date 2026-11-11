import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GateCaps,
  checkedGateCaps,
  defaultGateCaps,
  gateCapsProblems,
  maximumUnitIoWeight,
} from './native-gate-caps.ts';
import { NativeGateError } from './native-gate-error.ts';

/**
 The caps every recorded gate ran with, spelled out, so a change to one of them is a change a test sees.

 @returns the caps as the recorded runs had them
 */
function recordedCaps(): GateCaps {
  return {
    unitCpuQuota: '1000%',
    unitMemoryMax: '20G',
    unitTasksMax: 4_000,
    unitIoWeight: 10,
    containerPidsLimit: 512,
  };
}

await describe({
  name: 'the caps of one release gate',
  children: [
    it({
      name: 'are the caps every recorded gate of this repository ran with',
      fn: async () => {
        expect(defaultGateCaps).toEqual(recordedCaps(),);
        expect(defaultGateCaps.unitIoWeight).toBe(maximumUnitIoWeight,);
      },
    }),
    it({
      name: 'are usable as they ship, because a gate started without options never refuses its own defaults',
      fn: async () => {
        expect(gateCapsProblems({ caps: defaultGateCaps, }),).toEqual([],);
        expect(checkedGateCaps({ caps: defaultGateCaps, }),).toEqual(defaultGateCaps,);
      },
    }),
    it({
      name: 'refuse an empty cap value, naming the option that carried it',
      fn: async () => {
        expect(gateCapsProblems({ caps: { ...recordedCaps(), unitMemoryMax: '', }, }).join('\n'),).toContain('--unit-memory-max',);
        expect(gateCapsProblems({ caps: { ...recordedCaps(), unitMemoryMax: '', }, }).join('\n'),).toContain('empty',);
      },
    }),
    it({
      name: 'refuse a cap value systemd cannot read, so a start never fails on a property long after the value was typed',
      fn: async () => {
        const problem = gateCapsProblems({
          caps: {
            ...recordedCaps(),
            unitCpuQuota: '10 threads',
          },
        }).join('\n');
        expect(problem,).toContain('--unit-cpu-quota',);
        expect(problem,).toContain('systemd does not read',);
      },
    }),
    it({
      name: 'refuse a task or process limit that is not a whole number from one',
      fn: async () => {
        expect(gateCapsProblems({ caps: { ...recordedCaps(), unitTasksMax: 0, }, }).join('\n'),).toContain('--unit-tasks-max',);
        expect(gateCapsProblems({ caps: { ...recordedCaps(), containerPidsLimit: 1.5, }, }).join('\n'),).toContain('--container-pids-limit',);
      },
    }),
    it({
      name: 'refuse a disk weight above the ten a per-command control group may carry, quoting the rule',
      fn: async () => {
        const problem = gateCapsProblems({
          caps: {
            ...recordedCaps(),
            unitIoWeight: maximumUnitIoWeight + 1,
          },
        }).join('\n');
        expect(problem,).toContain('--unit-io-weight',);
        expect(problem,).toContain('RXS',);
      },
    }),
    it({
      name: 'refuse a disk weight of zero, which systemd reads as no weight at all',
      fn: async () => {
        expect(gateCapsProblems({ caps: { ...recordedCaps(), unitIoWeight: 0, }, }).join('\n'),).toContain('--unit-io-weight',);
      },
    }),
    it({
      name: 'name every unusable cap in one refusal, so a person fixes them in one round',
      fn: async () => {
        const problem = gateCapsProblems({
          caps: {
            unitCpuQuota: '',
            unitMemoryMax: '20 G',
            unitTasksMax: 0,
            unitIoWeight: 11,
            containerPidsLimit: 0,
          },
        }).join('\n');
        for (const option of [
          '--unit-cpu-quota',
          '--unit-memory-max',
          '--unit-tasks-max',
          '--unit-io-weight',
          '--container-pids-limit',
        ])
          expect(problem,).toContain(option,);
        expect(problem.split('\n'),).toHaveLength(5,);
      },
    }),
    it({
      name: 'stop the launch with a refusal a person can read, and no stack trace',
      fn: async () => {
        expect(() => checkedGateCaps({ caps: { ...recordedCaps(), unitTasksMax: 0, }, }),)
          .toThrow('the gate\'s caps cannot be used',);
        expect(() => checkedGateCaps({ caps: { ...recordedCaps(), unitTasksMax: 0, }, }),)
          .toThrow(NativeGateError,);
      },
    }),
  ],
});
