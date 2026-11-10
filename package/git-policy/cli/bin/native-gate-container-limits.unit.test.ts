import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseContainerLimitOptions, } from './container-limits.ts';
import {
  gateContainerCpus,
  gateContainerLimitOptions,
} from './native-gate-container-limits.ts';

/**
 The limits the gate reads when a person passes nothing after the task name.
 */
const gateLimitsWithNothingGiven = parseContainerLimitOptions({
  options: [...gateContainerLimitOptions],
  acceptPidsLimit: true,
});

await describe({
  name: 'the gate\'s own container limits',
  children: [
    it({
      name: 'size the gate\'s container at the gate\'s CPU count when a person names none',
      fn: async () => {
        expect(gateLimitsWithNothingGiven.limits.cpus).toBe(gateContainerCpus);
        expect(gateLimitsWithNothingGiven.rest).toEqual([]);
      },
      timeout: 10_000,
    }),
    it({
      name: 'keep a person\'s own --cpus over the gate\'s, because the later repeat wins',
      fn: async () => {
        /**
         CPU count a person passed after the task name, smaller than the gate's own.
         */
        const cpus = gateContainerCpus - 1;
        /**
         The limits the gate reads when that option follows its own.
         */
        const { limits: gateLimits } = parseContainerLimitOptions({
          options: [
            ...gateContainerLimitOptions,
            '--cpus',
            String(cpus),
          ],
          acceptPidsLimit: true,
        });
        expect(gateLimits.cpus).toBe(cpus);
      },
      timeout: 10_000,
    }),
    it({
      name: 'leave the memory and process bounds at the repository defaults',
      fn: async () => {
        /**
         The limits the gate does not size itself, which must stay the repository's own.
         */
        const repositoryDefaults = parseContainerLimitOptions({
          options: [],
          acceptPidsLimit: true,
        });
        expect(gateLimitsWithNothingGiven.limits.memory).toBe(repositoryDefaults.limits.memory);
        expect(gateLimitsWithNothingGiven.limits.pidsLimit).toBe(repositoryDefaults.limits.pidsLimit);
      },
      timeout: 10_000,
    }),
  ],
});
