import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultContainerLimits,
  parseContainerLimitOptions,
} from './container-limits.ts';
import {
  gateContainerCpus,
  gateContainerLimitOptions,
  gateLimitsFrom,
  processesForTestThreads,
} from './native-gate-container-limits.ts';

/**
 The limits the gate reads when a person passes nothing after the task name.
 */
const limitsWithNothingGiven = parseContainerLimitOptions({
  options: [...gateContainerLimitOptions],
  acceptPidsLimit: true,
}).limits;

await describe({
  name: 'the gate\'s own container limits',
  children: [
    it({
      name: 'size the gate\'s container at the gate\'s CPU count when a person names none',
      fn: async () => {
        expect(limitsWithNothingGiven.cpus).toBe(gateContainerCpus);
      },
      timeout: 10_000,
    }),
    it({
      name: 'read the gate\'s own CPU count as the first repeat, so a person\'s --cpus after it wins',
      fn: async () => {
        /**
         CPU count a person passed after the task name, smaller than the gate's own.
         */
        const cpus = gateContainerCpus - 1;
        /**
         The limits the gate reads when that option follows its own.
         */
        const { limits: given } = parseContainerLimitOptions({
          options: [
            ...gateContainerLimitOptions,
            '--cpus',
            String(cpus),
          ],
          acceptPidsLimit: true,
        });
        expect(given.cpus).toBe(cpus);
      },
      timeout: 10_000,
    }),
    it({
      name: 'raise the process bound to what the test threads need, and say so',
      fn: async () => {
        /**
         Processes the repository default gives a container, which is below what four test threads need.
         */
        const tooFew = defaultContainerLimits.pidsLimit;
        /**
         The limits and the line the gate runs and prints.
         */
        const { limits, raised } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
        });
        expect(limits.pidsLimit).toBe(processesForTestThreads(gateContainerCpus));
        expect(limits.pidsLimit).toBeGreaterThan(tooFew);
        expect(raised).toContain(String(tooFew));
        expect(raised).toContain(String(limits.pidsLimit));
        expect(raised).toContain('--pids-limit');
      },
      timeout: 10_000,
    }),
    it({
      name: 'keep a process bound a person raised themselves, and say nothing',
      fn: async () => {
        /**
         A process bound a person set above the gate's own floor.
         */
        const generous = processesForTestThreads(gateContainerCpus) * 2;
        /**
         The limits and the line the gate runs and prints.
         */
        const { limits, raised } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
            pidsLimit: generous,
          },
        });
        expect(limits.pidsLimit).toBe(generous);
        expect(raised).toBeUndefined();
      },
      timeout: 10_000,
    }),
    it({
      name: 'leave a container the repository default already suits unchanged, which is two test threads',
      fn: async () => {
        /**
         The limits and the line the gate runs and prints for a two-CPU container, as the mutation campaigns and the
         planted controls still start.
         */
        const { limits, raised } = gateLimitsFrom({ given: defaultContainerLimits });
        expect(limits).toEqual(defaultContainerLimits);
        expect(raised).toBeUndefined();
      },
      timeout: 10_000,
    }),
    it({
      name: 'leave the memory and CPU bounds a person asked for exactly as given',
      fn: async () => {
        /**
         Memory a person asked for.
         */
        const memory = '1g';
        /**
         The limits and the line the gate runs and prints.
         */
        const { limits } = gateLimitsFrom({
          given: {
            memory,
            cpus: gateContainerCpus,
            pidsLimit: defaultContainerLimits.pidsLimit,
          },
        });
        expect(limits.memory).toBe(memory);
        expect(limits.cpus).toBe(gateContainerCpus);
      },
      timeout: 10_000,
    }),
  ],
});
