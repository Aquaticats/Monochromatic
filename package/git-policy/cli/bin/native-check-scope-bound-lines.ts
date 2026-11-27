/**
 The lines a check prints when its own scope's bounds, and not a test, are what failed it.

 A check that is refused a fork, killed for memory, or held at its CPU quota reports the same way a failing test does:
 its status, and the lines of its output that say why. Nothing in that account names the bound, so these lines are
 printed beside it and never instead of it (`bin/native-ts-checks.ts` prints them under the check's own result), as the
 gate says the same for its containers (`bin/native-gate-container-memory.ts`).

 The figures are the scope's own (`bin/native-check-scope-accounting.ts`). A refused fork and a memory kill are hard
 events the kernel counts, so they are reported whatever the check's status was. Being held at the CPU quota is softer:
 it is reported only for a check that did not pass, because a check that passed was slowed and not failed, and this line
 exists to tell a bound from a failing test.
 */
import {
  bytesPerGibibyte,
  bytesPerMebibyte,
} from './native-gate-container-cgroup-memory.ts';
import {
  SCOPE_NOT_READ,
  type ScopeAccountingSample,
} from './native-check-scope-accounting.ts';

/**
 The statement that a check's own scope bound nothing, which a caller carries because this repository models absence
 without nullish unions.
 */
export const NO_SCOPE_BOUND: unique symbol = Symbol('check whose own scope bound nothing',);

/**
 How much longer a check's tasks waited for their CPU quota than they ran before the quota is called the bound.

 `cpu.stat` counts the time the scope's tasks spent throttled (`throttled_usec`, which sums the waiting of every task,
 so it can exceed the run's wall time) and the CPU time they used (`usage_usec`). Twice as long waiting as running is
 the line: measured on 2026-11-27 for `//package/git-policy/cli:test:unit` in one control group per run, the ratio was
 1.26 at 800%, 2.36 at 600%, 3.87 at 400%, and 7.5 and 9.1 in two runs at 200%, so the chosen 800% sits well under this
 line and every narrower measured quota is above it.
 */
export const throttledTimePerCpuTimeThatSaysTheQuotaBound: number = 2;

/**
 Format a byte figure as the MiB a peak is stated in.

 @param bytes - the figure

 @returns the number of MiB to one decimal, with thousands separated

 @example
 ```ts
 mibText(2_263_310_336);
 ```
 */
function mibText(bytes: number,): string {
  return new Intl.NumberFormat(
    'en-US',
    {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    },
  )
    .format(bytes / bytesPerMebibyte,);
}

/**
 Format a byte figure as the GiB a memory bound is stated in.

 @param bytes - the figure

 @returns the number of GiB to one decimal, with thousands separated

 @example
 ```ts
 gibText(4_294_967_296);
 ```
 */
function gibText(bytes: number,): string {
  return new Intl.NumberFormat(
    'en-US',
    {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    },
  )
    .format(bytes / bytesPerGibibyte,);
}

/**
 How a count is said in a sentence, in the singular when there was one.

 @param count - how many

 @param singular - the words for one

 @param plural - the words for any number but one

 @returns the count with its words

 @example
 ```ts
 countedText({ count: 1, singular: 'refused fork', plural: 'refused forks' });
 ```
 */
export function countedText({
  count,
  singular,
  plural,
}: {
  readonly count: number;
  readonly singular: string;
  readonly plural: string;
},): string {
  return count === 1 ? `1 ${singular}` : `${String(count)} ${plural}`;
}

/**
 The line a check prints when its scope's task bound refused a fork, and nothing when it refused none.

 @param task - the check, as the command line named it

 @param sample - the last figures read from the check's scope

 @returns one sentence ending in a full stop, or {@link NO_SCOPE_BOUND} when the bound refused no fork

 @example
 ```ts
 const refused = refusedForkLine({ task: '//package/a:b', sample });
 if (refused !== NO_SCOPE_BOUND) console.log(refused);
 ```
 */
export function refusedForkLine({
  task,
  sample,
}: {
  readonly task: string;
  readonly sample: ScopeAccountingSample;
},): string | typeof NO_SCOPE_BOUND {
  /**
   Forks the task bound refused, which is none for a scope that never met its bound.
   */
  const refused = sample.pidsEvents
    .max
    ?? 0;
  if (refused === 0)
    return NO_SCOPE_BOUND;
  /**
   The bound, as the kernel holds it, and how far the scope got.
   */
  const bound = sample.pidsMax === undefined
    ? 'its task bound'
    : `its ${String(sample.pidsMax)}-task bound`;
  /**
   What the scope held at its highest, when the kernel wrote a peak.
   */
  const peak = sample.pidsPeak === undefined
    ? ''
    : `, and held ${String(sample.pidsPeak)} tasks at its highest`;
  return `${task}: its scope counted ${countedText({
    count: refused,
    singular: 'refused fork',
    plural: 'refused forks',
  })} at ${bound}${peak}. That is the bound and not a failing test: raise it with --unit-tasks-max <count> after the native:ts-checks task name.`;
}

/**
 The line a check prints when its scope's memory bound killed something, and nothing when it killed nothing.

 @param task - the check, as the command line named it

 @param sample - the last figures read from the check's scope

 @returns one sentence ending in a full stop, or {@link NO_SCOPE_BOUND} when the bound caused no kill

 @example
 ```ts
 const killed = memoryKillLine({ task: '//package/a:b', sample });
 if (killed !== NO_SCOPE_BOUND) console.log(killed);
 ```
 */
export function memoryKillLine({
  task,
  sample,
}: {
  readonly task: string;
  readonly sample: ScopeAccountingSample;
},): string | typeof NO_SCOPE_BOUND {
  /**
   Kills the cgroup counted, which is none for a scope that met no bound.
   */
  const kills = sample.memoryEvents
    .oom_kill
    ?? 0;
  if (kills === 0)
    return NO_SCOPE_BOUND;
  /**
   The bound the kernel held the scope at, as the kernel states it.
   */
  const bound = sample.memoryMaxBytes === undefined
    ? 'its memory bound'
    : `the ${gibText(sample.memoryMaxBytes,)} GiB memory bound it was given`;
  /**
   What the scope charged at its highest, when the kernel wrote a peak.
   */
  const peak = sample.memoryPeakBytes === undefined
    ? ''
    : `, with a peak of ${mibText(sample.memoryPeakBytes,)} MiB`;
  return `${task}: its scope was killed for memory: its cgroup counted ${countedText({
    count: kills,
    singular: 'out-of-memory kill',
    plural: 'out-of-memory kills',
  })} at ${bound}${peak}. That is the bound and not a failing test: raise it with --unit-memory-max <size> after the native:ts-checks task name.`;
}

/**
 The quota a check's scope carried, as a percentage of one CPU, when the kernel wrote one.

 @param sample - the last figures read from the check's scope

 @returns the quota as a percentage, or the words for a quota the kernel wrote no figure for

 @example
 ```ts
 quotaText({ sample });
 ```
 */
function quotaText({ sample }: { readonly sample: ScopeAccountingSample; },): string {
  if ((sample.cpuQuotaMicroseconds === undefined) || (sample.cpuPeriodMicroseconds === undefined)
    || (sample.cpuPeriodMicroseconds === 0))
    return 'its CPU quota';
  return `its ${String(Math.round(100 * (sample.cpuQuotaMicroseconds / sample.cpuPeriodMicroseconds)))}% CPU quota`;
}

/**
 The line a check prints when its scope held it at its CPU quota for most of its run.

 @param task - the check, as the command line named it

 @param sample - the last figures read from the check's scope

 @param passed - whether the check passed, which leaves a slowed run unexplained and only a failed one accused

 @returns one sentence ending in a full stop, or {@link NO_SCOPE_BOUND} when the quota held less than most of the run

 @example
 ```ts
 const throttled = throttledLine({ task: '//package/a:b', sample, passed: false });
 if (throttled !== NO_SCOPE_BOUND) console.log(throttled);
 ```
 */
export function throttledLine({
  task,
  sample,
  passed,
}: {
  readonly task: string;
  readonly sample: ScopeAccountingSample;
  readonly passed: boolean;
},): string | typeof NO_SCOPE_BOUND {
  /**
   The CPU time the scope's tasks used, and the time they spent waiting for the quota.
   */
  const used = sample.cpuStat
    .usage_usec
    ?? 0;
  /**
   The time the scope's tasks spent throttled, which the quota caused.
   */
  const waited = sample.cpuStat
    .throttled_usec
    ?? 0;
  if (passed || (used === 0)
    || (waited < (used * throttledTimePerCpuTimeThatSaysTheQuotaBound)))
    return NO_SCOPE_BOUND;
  /**
   How many times longer the tasks waited than they ran, which is the reading this line is decided by.
   */
  const times = new Intl.NumberFormat(
    'en-US',
    {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    },
  )
    .format(waited / used,);
  /**
   The periods the quota held the scope, and the periods that elapsed, which say how often it was held.
   */
  const periods = sample.cpuStat
    .nr_periods;
  /**
   How often the quota held the scope, as a clause the kernel's counters make when both were written.
   */
  const heldIn = ((sample.cpuStat
    .nr_throttled
    === undefined) || (periods === undefined))
    ? ''
    : `, in ${String(sample.cpuStat
      .nr_throttled)} of its ${String(periods)} periods`;
  return `${task}: its scope's tasks spent ${times} times as long waiting for ${quotaText({ sample, })} as they spent running${heldIn} (throttled_usec ${String(waited)} against usage_usec ${String(used)}). That is the bound and not a failing test: raise it with --unit-cpu-quota <value> after the native:ts-checks task name.`;
}

/**
 Every line one check prints about its own scope's bounds, in the order the bounds are worth reading.

 @param task - the check, as the command line named it

 @param accounting - the last figures read from the check's scope, or {@link SCOPE_NOT_READ}

 @param passed - whether the check passed

 @returns the lines to print after the check's own result, empty when its scope bound nothing

 @example
 ```ts
 const lines = boundLinesOf({ task: '//package/a:b', accounting, passed: false });
 ```
 */
export function boundLinesOf({
  task,
  accounting,
  passed,
}: {
  readonly task: string;
  readonly accounting: ScopeAccountingSample | typeof SCOPE_NOT_READ;
  readonly passed: boolean;
},): readonly string[] {
  if (accounting === SCOPE_NOT_READ)
    return [];
  return [
    refusedForkLine({
      task,
      sample: accounting,
    }),
    memoryKillLine({
      task,
      sample: accounting,
    }),
    throttledLine({
      task,
      sample: accounting,
      passed,
    }),
  ].filter(function isLine(line): line is string {
    return line !== NO_SCOPE_BOUND;
  });
}
