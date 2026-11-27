import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SCOPE_NOT_READ,
  type ScopeAccountingSample,
} from './native-check-scope-accounting.ts';
import {
  NO_SCOPE_BOUND,
  boundLinesOf,
  memoryKillLine,
  refusedForkLine,
  throttledLine,
} from './native-check-scope-bound-lines.ts';

/**
 Bytes in the 4 GiB memory bound every recorded check ran under, which a stand-in states.
 */
const fourGibibytes = 4_294_967_296;

/**
 The check every line of this file is about, named as the command line names it.
 */
const task = '//package/a:b';

/**
 The counters a control group that met no bound holds, which is an empty set and not a missing reading.
 */
const noCounters: Readonly<Record<string, number>> = {};

/**
 The control group every stand-in reading of this file names.
 */
const cgroupPath = '/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/gpc-check-a-b.scope';

/**
 One scope's reading, with every figure a line does not name left out.

 These are stand-ins: no run of any check produced them, and each test says which bound it asserts on. The numbers a test
 passes are the ones this round measured where a reading exists (issue 6605), and made up where a bound has to be met
 without a run that met it. Each figure is optional because a line reads only the ones its bound writes.

 @param figures - the figures this reading holds

 @returns a reading of one check scope
 */
function sampleOf(figures: {
  readonly pidsCurrent?: number;
  readonly pidsPeak?: number;
  readonly pidsMax?: number;
  readonly pidsEvents?: Readonly<Record<string, number>>;
  readonly memoryCurrentBytes?: number;
  readonly memoryPeakBytes?: number;
  readonly memoryMaxBytes?: number;
  readonly memoryEvents?: Readonly<Record<string, number>>;
  readonly cpuQuotaMicroseconds?: number;
  readonly cpuPeriodMicroseconds?: number;
  readonly cpuStat?: Readonly<Record<string, number>>;
},): ScopeAccountingSample {
  return {
    cgroupPath,
    readAt: '2026-11-27T15:23:29.041Z',
    ...figures,
    pidsEvents: figures.pidsEvents ?? noCounters,
    memoryEvents: figures.memoryEvents ?? noCounters,
    cpuStat: figures.cpuStat ?? noCounters,
  };
}

/**
 A reading of a scope that met no bound.

 @returns the reading, whose every counter holds nothing
 */
function countedNothingSample(): ScopeAccountingSample {
  return {
    cgroupPath,
    readAt: '2026-11-27T15:23:29.041Z',
    pidsEvents: noCounters,
    memoryEvents: noCounters,
    cpuStat: noCounters,
  };
}

await describe({
  name: 'the lines a check prints when its own scope bound it',
  children: [
    it({
      name: 'names a refused fork by the bound that refused it, and says the bound and not a test failed',
      fn: async () => {
        /**
         What a scope that refused forks answers.
         */
        const line = refusedForkLine({
          task,
          sample: sampleOf({
            pidsMax: 400,
            pidsPeak: 434,
            pidsEvents: { max: 6 },
          }),
        });
        expect(line,).not.toBe(NO_SCOPE_BOUND,);
        if (line === NO_SCOPE_BOUND)
          return;
        expect(line,).toContain('6 refused forks',);
        expect(line,).toContain('its 400-task bound',);
        expect(line,).toContain('held 434 tasks at its highest',);
        expect(line,).toContain('That is the bound and not a failing test',);
        expect(line,).toContain('--unit-tasks-max',);
      },
    }),
    it({
      name: 'says one refused fork in the singular, and names no bound or peak the kernel did not write',
      fn: async () => {
        /**
         What a scope that refused one fork answers without a bound or a peak written.
         */
        const line = refusedForkLine({
          task,
          sample: sampleOf({ pidsEvents: { max: 1 }, }),
        });
        expect(line,).not.toBe(NO_SCOPE_BOUND,);
        if (line === NO_SCOPE_BOUND)
          return;
        expect(line,).toContain('1 refused fork at its task bound.',);
        expect(line,).not.toContain('at its highest',);
      },
    }),
    it({
      name: 'names a memory kill by the bound it met and the peak the scope reached',
      fn: async () => {
        /**
         What a scope killed for memory answers.
         */
        const line = memoryKillLine({
          task,
          sample: sampleOf({
            memoryMaxBytes: fourGibibytes,
            memoryPeakBytes: 2_263_310_336,
            memoryEvents: { oom_kill: 2 },
          }),
        });
        expect(line,).not.toBe(NO_SCOPE_BOUND,);
        if (line === NO_SCOPE_BOUND)
          return;
        expect(line,).toContain('2 out-of-memory kills',);
        expect(line,).toContain('the 4.0 GiB memory bound it was given',);
        expect(line,).toContain('a peak of 2,158.5 MiB',);
        expect(line,).toContain('--unit-memory-max',);
      },
    }),
    it({
      name: 'names the CPU quota a check that did not pass waited most of its run for',
      fn: async () => {
        /**
         What a check held at 200% answers, which is one of the two runs this round measured at that quota.
         */
        const line = throttledLine({
          task,
          sample: sampleOf({
            cpuQuotaMicroseconds: 200_000,
            cpuPeriodMicroseconds: 100_000,
            cpuStat: {
              usage_usec: 71_471_962,
              throttled_usec: 651_833_695,
              nr_periods: 378,
              nr_throttled: 332,
            },
          }),
          passed: false,
        });
        expect(line,).not.toBe(NO_SCOPE_BOUND,);
        if (line === NO_SCOPE_BOUND)
          return;
        expect(line,).toContain('its 200% CPU quota',);
        expect(line,).toContain('spent 9.1 times as long waiting for its 200% CPU quota as they spent running',);
        expect(line,).toContain('in 332 of its 378 periods',);
        expect(line,).toContain('--unit-cpu-quota',);
      },
    }),
    it({
      name: 'says nothing about a quota for a check that passed, or one it waited less than twice its run for',
      fn: async () => {
        /**
         A scope held at 800% whose tasks waited 1.3 times as long as they ran, which is the chosen quota's own reading.
         */
        const held = sampleOf({
          cpuQuotaMicroseconds: 800_000,
          cpuPeriodMicroseconds: 100_000,
          cpuStat: {
            usage_usec: 76_928_181,
            throttled_usec: 97_294_323,
            nr_periods: 160,
            nr_throttled: 68,
          },
        });
        // A check that passed was slowed and not failed, and this line exists to tell a bound from a failing test.
        expect(throttledLine({
          task,
          sample: held,
          passed: true,
        }),).toBe(NO_SCOPE_BOUND,);
        expect(throttledLine({
          task,
          sample: held,
          passed: false,
        }),).toBe(NO_SCOPE_BOUND,);
      },
    }),
    it({
      name: 'says nothing when the scope was never read, and nothing for a scope that bound nothing',
      fn: async () => {
        expect(boundLinesOf({
          task,
          accounting: SCOPE_NOT_READ,
          passed: false,
        }),).toEqual([],);
        expect(boundLinesOf({
          task,
          accounting: countedNothingSample(),
          passed: false,
        }),).toEqual([],);
      },
    }),
    it({
      name: 'names every bound a scope met, the fork first, then the kill, then the quota',
      fn: async () => {
        /**
         Every line a scope that met all three bounds earns.
         */
        const lines = boundLinesOf({
          task,
          accounting: sampleOf({
            pidsMax: 400,
            pidsPeak: 434,
            pidsEvents: { max: 6 },
            memoryMaxBytes: fourGibibytes,
            memoryPeakBytes: 2_263_310_336,
            memoryEvents: { oom_kill: 2 },
            cpuQuotaMicroseconds: 200_000,
            cpuPeriodMicroseconds: 100_000,
            cpuStat: {
              usage_usec: 71_471_962,
              throttled_usec: 651_833_695,
              nr_periods: 378,
              nr_throttled: 332,
            },
          }),
          passed: false,
        });
        expect(lines,).toHaveLength(3,);
        expect(lines[0],).toContain('6 refused forks',);
        expect(lines[1],).toContain('2 out-of-memory kills',);
        expect(lines[2],).toContain('9.1 times as long waiting',);
      },
    }),
  ],
});
