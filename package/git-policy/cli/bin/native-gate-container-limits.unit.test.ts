import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultContainerLimits,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import {
  buildMemoryLine,
  gateBuildMemory,
  gateClippyMemory,
  gateContainerBounds,
  gateContainerCpus,
  gateContainerLimitOptions,
  gateLimitsFrom,
  gateSuiteMemory,
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
          givenFlags: [],
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
          givenFlags: [],
        });
        expect(limits.pidsLimit).toBe(generous);
        expect(raised).toBeUndefined();
      },
      timeout: 10_000,
    }),
    it({
      name: 'raise no process bound for a container the repository default already suits, which is two test threads',
      fn: async () => {
        /**
         The limits and the line the gate runs and prints for a two-CPU container.
         */
        const { limits, raised } = gateLimitsFrom({
          given: defaultContainerLimits,
          givenFlags: [],
        });
        expect(limits).toEqual({
          ...defaultContainerLimits,
          memory: gateSuiteMemory,
        });
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
          givenFlags: ['--memory'],
        });
        expect(limits.memory).toBe(memory);
        expect(limits.cpus).toBe(gateContainerCpus);
      },
      timeout: 10_000,
    }),
    it({
      name: 'give the image build its own measured memory bound beside the suite\'s own',
      fn: async () => {
        /**
         The limits the suite's containers and the image build run under when a person names no memory.
         */
        const { limits, buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        expect(limits.memory).toBe(gateSuiteMemory);
        expect(buildLimits.memory).toBe(gateBuildMemory);
        expect(buildLimits.cpus).toBe(limits.cpus);
        expect(buildLimits.pidsLimit).toBe(limits.pidsLimit);
      },
      timeout: 10_000,
    }),
    it({
      name: 'let a person\'s --memory bound the image build as it bounds every other container',
      fn: async () => {
        /**
         Memory a person asked for, which is larger than the build's own bound.
         */
        const memory = '12g';
        /**
         The limits the suite's containers and the image build run under for that option.
         */
        const { limits, buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
            memory,
          },
          givenFlags: ['--memory'],
        });
        expect(limits.memory).toBe(memory);
        expect(buildLimits.memory).toBe(memory);
      },
      timeout: 10_000,
    }),
    it({
      name: 'tell a --memory naming the repository default from a memory nobody named',
      fn: async () => {
        /**
         The repository default, which a person may name again, given here as the option the parser read.
         */
        const { buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: ['--memory'],
        });
        expect(buildLimits.memory).toBe(defaultContainerLimits.memory);
        expect(buildLimits.memory).not.toBe(gateBuildMemory);
      },
      timeout: 10_000,
    }),
    it({
      name: 'name each container\'s memory bound in one line when they differ',
      fn: async () => {
        /**
         The limits the suite's, Clippy's and the image build's containers run under when a person names no memory.
         */
        const { limits, clippyLimits, buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        /**
         The line the gate prints before it builds.
         */
        const line = buildMemoryLine({
          limits,
          clippyLimits,
          buildLimits,
        });
        expect(line).toContain(limits.memory);
        expect(line).toContain(clippyLimits.memory);
        expect(line).toContain(buildLimits.memory);
        expect(line).toContain('--memory');
      },
      timeout: 10_000,
    }),
    it({
      name: 'name the one memory bound every container shares when a person gave it',
      fn: async () => {
        /**
         Memory a person asked for, which every bound takes.
         */
        const memory = '3g';
        /**
         The limits the suite's, Clippy's and the image build's containers run under for that option.
         */
        const { limits, clippyLimits, buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
            memory,
          },
          givenFlags: ['--memory'],
        });
        expect(buildMemoryLine({
          limits,
          clippyLimits,
          buildLimits,
        }),).toBe(`The image build, the suite's container and Clippy's are bounded at ${memory}, the memory this gate was given.`,);
      },
      timeout: 10_000,
    }),
    it({
      name: 'give the suite\'s container a measured memory bound of its own, not the repository default',
      fn: async () => {
        /**
         The limits the suite's, Clippy's and the image build's containers run under when a person names no memory.
         */
        const { limits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        expect(limits.memory).toBe(gateSuiteMemory);
        expect(gateSuiteMemory).not.toBe(defaultContainerLimits.memory);
      },
      timeout: 10_000,
    }),
    it({
      name: 'bound the suite\'s container at its own memory, apart from Clippy\'s and the image build\'s, and name each in one line',
      fn: async () => {
        /**
         The limits the suite's, Clippy's and the image build's containers run under when a person names no memory.
         */
        const { limits, clippyLimits, buildLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        expect(limits.memory,).not.toBe(clippyLimits.memory,);
        expect(limits.memory,).not.toBe(buildLimits.memory,);
        /**
         The line the gate prints before it builds.
         */
        const line = buildMemoryLine({
          limits,
          clippyLimits,
          buildLimits,
        });
        expect(line,).toContain(`the suite's container at ${gateSuiteMemory}`,);
        expect(line,).toContain(`The image build is bounded at ${gateBuildMemory}`,);
        expect(line,).toContain(`Clippy's at ${gateClippyMemory}`,);
        expect(line,).toContain('--memory',);
      },
      timeout: 10_000,
    }),
    it({
      name: 'give the Clippy container a measured memory bound of its own, and keep the suite\'s apart from it',
      fn: async () => {
        /**
         The limits the suite's, Clippy's and the image build's containers run under when a person names no memory.
         */
        const { limits, clippyLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        expect(limits.memory).toBe(gateSuiteMemory);
        expect(clippyLimits.memory).toBe(gateClippyMemory);
        expect(clippyLimits.memory).not.toBe(limits.memory);
        expect(clippyLimits.cpus).toBe(limits.cpus);
        expect(clippyLimits.pidsLimit).toBe(limits.pidsLimit);
      },
      timeout: 10_000,
    }),
    it({
      name: 'let a person\'s --memory bound the Clippy container as it bounds every other container',
      fn: async () => {
        /**
         Memory a person asked for, smaller than the bound Clippy has by itself.
         */
        const memory = '1g';
        /**
         The limits the suite's and Clippy's containers run under for that option.
         */
        const { limits, clippyLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
            memory,
          },
          givenFlags: ['--memory'],
        });
        expect(limits.memory).toBe(memory);
        expect(clippyLimits.memory).toBe(memory);
      },
      timeout: 10_000,
    }),
    it({
      name: 'start each container with its own memory bound and the flags both share',
      fn: async () => {
        /**
         The flags both containers take, which the run writes before a container's own bounds.
         */
        const shared = [
          '--rm',
          '--init',
          '--network=none',
        ];
        /**
         The limits the suite's and Clippy's containers run under when a person names no memory.
         */
        const { limits, clippyLimits } = gateLimitsFrom({
          given: {
            ...defaultContainerLimits,
            cpus: gateContainerCpus,
          },
          givenFlags: [],
        });
        /**
         Both containers' complete `podman run` bounds.
         */
        const bounds = gateContainerBounds({
          shared,
          limits,
          clippyLimits,
        });
        expect(bounds.suite,).toEqual([
          ...shared,
          ...runLimitArguments({
            limits,
            withPidsLimit: true,
          }),
        ],);
        expect(bounds.clippy,).toEqual([
          ...shared,
          ...runLimitArguments({
            limits: clippyLimits,
            withPidsLimit: true,
          }),
        ],);
        expect(bounds.clippy,).not.toEqual(bounds.suite,);
      },
      timeout: 10_000,
    }),
  ],
});
