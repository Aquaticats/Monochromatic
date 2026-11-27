/**
 One check scope's own accounting, read from its control group while the check runs.

 A check's scope is a transient systemd scope, and the check's own children are what its control group holds: `pids.peak`
 and `pids.events` say whether the task bound refused a fork, `memory.peak` and `memory.events` say whether the memory
 bound killed something, and `cpu.stat` says whether the CPU quota held the run back. The scope goes with its work, so
 these figures are read while it runs and the last sample is kept (`bin/native-capped-scope-command.ts` reads them that
 way, as `bin/native-gate-container-memory.ts` reads a gate container's).

 The scope's path is known without asking the manager: `bin/native-capped-scope-command.ts` starts every check with
 `--slice=app.slice`, so its scope is
 `<root>/user.slice/user-<uid>.slice/user@<uid>.service/app.slice/<name>.scope`. Measured on this host on 2026-11-27: a
 `systemd-run --user --scope` that named no slice landed in that same `app.slice`, and `pids.peak`, `pids.events`,
 `memory.peak`, `memory.events`, `memory.max`, `cpu.max` and `cpu.stat` were all there to read, beside the `.local`
 variants of the two event files that count a cgroup's own bound rather than its subtree's.
 */
import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';
import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import { isWholeNumberText, } from './native-gate-arguments.ts';
import {
  cgroupRoot,
  fieldsOf,
} from './native-gate-container-cgroup-memory.ts';

//region Where a check scope's figures are

/**
 The slice every check's scope is started in, which is systemd's own default for a transient user scope and is named on
 the start so this reader's path and the scope agree rather than by luck.
 */
export const checkScopeSlice = 'app.slice';

/**
 Milliseconds between two reads of a check's scope while it runs.

 The figures a check is judged by are counters and peaks, which only rise, so the interval decides one thing: whether a
 reading is still there to be taken. A scope that ends takes its control group with it, and the caller keeps the last
 sample, so the interval is short enough to catch a check that ends quickly and long enough that the reads cost nothing
 beside the check running next to them.
 */
export const checkScopePollMilliseconds = 100;

/**
 A check scope whose counters were never read, which a scope that ended before the first read leaves.

 A sentinel rather than a nullish value, because this repository models absence without nullish unions.
 */
export const SCOPE_NOT_READ: unique symbol = Symbol('check scope whose counters were never read',);

/**
 A figure this host's kernel does not write, which is not a failure: an older kernel has no `pids.peak`, and a scope that
 ends between two reads answers no file at all.
 */
export const FIGURE_NOT_WRITTEN: unique symbol = Symbol('cgroup figure this host kernel does not write',);

/**
 The control group of one check's scope.

 @param name - the scope's name, without its `.scope` suffix

 @param root - the cgroup hierarchy's root, which a test points at a fixture tree

 @param userId - the account whose user manager runs the check

 @returns the absolute path of the check's own scope

 @example
 ```ts
 checkScopeCgroupPath({ name: 'gpc-check-a-20261127T152200-0500', userId: 1000 });
 ```
 */
export function checkScopeCgroupPath({
  name,
  root = cgroupRoot,
  userId = process.getuid?.() ?? 0,
}: {
  readonly name: string;
  readonly root?: string;
  readonly userId?: number;
}): string {
  return join(
    root,
    'user.slice',
    `user-${String(userId)}.slice`,
    `user@${String(userId)}.service`,
    checkScopeSlice,
    `${name}.scope`,
  );
}

//endregion Where a check scope's figures are

//region What the kernel says about one check scope

/**
 One check scope's figures, as the kernel held them at the moment they were read.
 */
export type ScopeAccountingSample = {
  /**
   The scope the figures came from.
   */
  readonly cgroupPath: string;
  /**
   When they were read, in ISO 8601.
   */
  readonly readAt: string;
  /**
   Tasks in the scope's control group now, which the kernel counts as threads and processes together.
   */
  readonly pidsCurrent?: number;
  /**
   The most tasks the scope held since it started, which is the figure the task bound is compared with.
   */
  readonly pidsPeak?: number;
  /**
   The task bound itself: the cgroup's `pids.max`.
   */
  readonly pidsMax?: number;
  /**
   Every counter of this scope's own task bound, by name: `max` counts the forks it refused.
   */
  readonly pidsEvents: Readonly<Record<string, number>>;
  /**
   Memory charged to the scope and its children now.
   */
  readonly memoryCurrentBytes?: number;
  /**
   The highest charge since the scope started, which is the figure the memory bound is compared with.
   */
  readonly memoryPeakBytes?: number;
  /**
   The memory bound itself: the cgroup's `memory.max`.
   */
  readonly memoryMaxBytes?: number;
  /**
   Every counter of this scope's own memory bound, by name: `max` counts the times it was reached and `oom_kill` the
   kills.
   */
  readonly memoryEvents: Readonly<Record<string, number>>;
  /**
   The CPU quota the kernel holds the scope at, in microseconds of a period, absent when the quota is unlimited.
   */
  readonly cpuQuotaMicroseconds?: number;
  /**
   The period that quota is counted over, in microseconds.
   */
  readonly cpuPeriodMicroseconds?: number;
  /**
   Every counter of `cpu.stat`, by name: `usage_usec` is the CPU time the check used, `nr_periods` the periods that
   elapsed, `nr_throttled` the periods the quota held the scope at its bound, and `throttled_usec` the time its tasks
   spent waiting for that bound.
   */
  readonly cpuStat: Readonly<Record<string, number>>;
};

/**
 One check scope's own accounting, or the statement that its control group is not there to read.
 */
export type ScopeAccountingRead = ScopeAccountingSample | typeof SCOPE_NOT_READ;

/**
 Read one file of a cgroup as trimmed text.

 @param path - the file

 @returns its text, or {@link FIGURE_NOT_WRITTEN} when this host's kernel writes no such file

 @throws the error of a file that is there and cannot be read, which is a host this reader cannot account for
 */
async function fileText(path: string,): Promise<string | typeof FIGURE_NOT_WRITTEN> {
  try {
    return (await readFile(
      path,
      'utf8',
    )).trim();
  }
  catch (error) {
    if (isMissing(error,))
      return FIGURE_NOT_WRITTEN;
    throw error;
  }
}

/**
 Read one file of a cgroup as a whole number.

 @param path - the file

 @returns the number it holds, or {@link FIGURE_NOT_WRITTEN} when the file is not there or holds anything else
 */
async function figureOf(path: string,): Promise<number | typeof FIGURE_NOT_WRITTEN> {
  /**
   The file's text.
   */
  const text = await fileText(path,);
  if (((typeof text) === 'symbol') || (!isWholeNumberText(text,)))
    return FIGURE_NOT_WRITTEN;
  return Number(text,);
}

/**
 Read one file of a cgroup as the counters it holds, one `name value` pair per line.

 @param path - the file

 @returns every counter, by name, empty when the file is not there
 */
async function countersOf(path: string,): Promise<Readonly<Record<string, number>>> {
  /**
   The file's text.
   */
  const text = await fileText(path,);
  if ((typeof text) === 'symbol')
    return {};
  /**
   The counters read so far.
   */
  const counters: Record<string, number> = {};
  for (const line of text.split('\n',)) {
    /**
     The two fields a counter line holds.
     */
    const [name, value, ...surplus] = fieldsOf(line,);
    if ((name === undefined) || (value === undefined)
      || (surplus.length > 0)
      || (!isWholeNumberText(value,)))
      continue;
    counters[name] = Number(value,);
  }
  return counters;
}

/**
 Read a scope's `cpu.max` as the quota and period it holds.

 `cpu.max` holds the quota in microseconds of the period, or the word `max` for an unlimited quota, and then the period.

 @param path - the file

 @returns the quota and period, absent when the file is not there or holds no quota this reader can use
 */
async function cpuMaxOf(path: string,): Promise<{
  readonly quota?: number;
  readonly period?: number
}> {
  /**
   The file's text.
   */
  const text = await fileText(path,);
  if ((typeof text) === 'symbol')
    return {};
  /**
   The two fields the file holds.
   */
  const [quota, period, ...surplus] = fieldsOf(text,);
  if ((quota === undefined) || (period === undefined)
    || (surplus.length > 0)
    || (!isWholeNumberText(period,)))
    return {};
  return {
    ...(isWholeNumberText(quota,) ? { quota: Number(quota,) } : {}),
    period: Number(period,),
  };
}

/**
 Read the counters of one bound as the cgroup itself counted it.

 `pids.events` and `memory.events` count their subtree, so a cgroup nested inside the check could read as this scope's
 bound; the `.local` files count only this cgroup's own enforcement, which is the bound these figures name. A kernel that
 writes no `.local` file leaves the subtree counters, which is what it offers.

 @param cgroupPath - the scope's control group

 @param name - the bound's event file, `pids.events` or `memory.events`

 @returns every counter, by name, empty when neither file is there

 @example
 ```ts
 await boundEventsOf({ cgroupPath, name: 'pids.events' });
 ```
 */
async function boundEventsOf({
  cgroupPath,
  name,
}: {
  readonly cgroupPath: string;
  readonly name: string;
},): Promise<Readonly<Record<string, number>>> {
  /**
   What this cgroup's own bound counted, when the kernel writes it.
   */
  const local = await countersOf(join(
    cgroupPath,
    `${name}.local`,
  ),);
  if (Object.keys(local,)
    .length
    > 0)
    return local;
  return countersOf(join(
    cgroupPath,
    name,
  ),);
}

/**
 Tells whether a scope's control group is there to read.

 @param cgroupPath - the scope's control group

 @returns true when the scope exists, false when it ended and took its control group with it

 @throws the error of a path that is there and cannot be read, which is a host this reader cannot account for
 */
async function cgroupIsThere(cgroupPath: string,): Promise<boolean> {
  try {
    await stat(cgroupPath,);
    return true;
  }
  catch (error) {
    if (isMissing(error,))
      return false;
    throw error;
  }
}

/**
 Read one check scope's own accounting.

 @param cgroupPath - the scope's control group, from {@link checkScopeCgroupPath}

 @returns the figures the kernel holds for it now, or {@link SCOPE_NOT_READ} once the scope ended

 @example
 ```ts
 const read = await readScopeAccounting({ cgroupPath: checkScopeCgroupPath({ name }) });
 ```
 */
export async function readScopeAccounting({
  cgroupPath,
}: {
  readonly cgroupPath: string;
},): Promise<ScopeAccountingRead> {
  /**
   Whether the scope's control group is there to read at all.
   */
  const there = await cgroupIsThere(cgroupPath,);
  if (!there)
    return SCOPE_NOT_READ;
  /**
   When this sample was read.
   */
  const readAt = new Date()
    .toISOString();
  /**
   The figures the kernel writes, read together so one sample describes one moment.
   */
  const [
    pidsCurrent,
    pidsPeak,
    pidsMax,
    pidsEvents,
    memoryCurrent,
    memoryPeak,
    memoryMax,
    memoryEvents,
    cpuMax,
    cpuStat,
  ] = await Promise.all([
    figureOf(join(
      cgroupPath,
      'pids.current',
    ),),
    figureOf(join(
      cgroupPath,
      'pids.peak',
    ),),
    figureOf(join(
      cgroupPath,
      'pids.max',
    ),),
    boundEventsOf({
      cgroupPath,
      name: 'pids.events',
    },),
    figureOf(join(
      cgroupPath,
      'memory.current',
    ),),
    figureOf(join(
      cgroupPath,
      'memory.peak',
    ),),
    figureOf(join(
      cgroupPath,
      'memory.max',
    ),),
    boundEventsOf({
      cgroupPath,
      name: 'memory.events',
    },),
    cpuMaxOf(join(
      cgroupPath,
      'cpu.max',
    ),),
    countersOf(join(
      cgroupPath,
      'cpu.stat',
    ),),
  ]);
  /**
   The sample being assembled, whose figures the kernel writes are filled in as each is read.
   */
  const sample: {
    cgroupPath: string;
    readAt: string;
    pidsCurrent?: number;
    pidsPeak?: number;
    pidsMax?: number;
    pidsEvents: Readonly<Record<string, number>>;
    memoryCurrentBytes?: number;
    memoryPeakBytes?: number;
    memoryMaxBytes?: number;
    memoryEvents: Readonly<Record<string, number>>;
    cpuQuotaMicroseconds?: number;
    cpuPeriodMicroseconds?: number;
    cpuStat: Readonly<Record<string, number>>;
  } = {
    cgroupPath,
    readAt,
    pidsEvents,
    memoryEvents,
    cpuStat,
  };
  if ((typeof pidsCurrent) !== 'symbol')
    sample.pidsCurrent = pidsCurrent;
  if ((typeof pidsPeak) !== 'symbol')
    sample.pidsPeak = pidsPeak;
  if ((typeof pidsMax) !== 'symbol')
    sample.pidsMax = pidsMax;
  if ((typeof memoryCurrent) !== 'symbol')
    sample.memoryCurrentBytes = memoryCurrent;
  if ((typeof memoryPeak) !== 'symbol')
    sample.memoryPeakBytes = memoryPeak;
  if ((typeof memoryMax) !== 'symbol')
    sample.memoryMaxBytes = memoryMax;
  if (cpuMax.quota !== undefined)
    sample.cpuQuotaMicroseconds = cpuMax.quota;
  if (cpuMax.period !== undefined)
    sample.cpuPeriodMicroseconds = cpuMax.period;
  return sample;
}

//endregion What the kernel says about one check scope

//region Watching one check scope while it runs

/**
 One running check scope's accounting watch: the last figures it read, and how to end it.

 The last read is kept because a scope's control group goes with it: what a caller asks after the check ended can only
 be what was read while it ran.
 */
export type ScopeAccountingWatch = {
  /**
   The last figures read from the scope, or {@link SCOPE_NOT_READ} when it was never there.
   */
  readonly lastRead: () => ScopeAccountingRead;
  /**
   Stop reading, and answer once the last read is kept. Calling it twice is the same as calling it once.
   */
  readonly stop: () => Promise<void>;
};

/**
 An accounting watch that can be disposed of, which is what `await using` needs of one.
 */
export type DisposableScopeAccountingWatch = ScopeAccountingWatch & AsyncDisposable;

/**
 What one watch has read so far, held in an object so the reading function and its caller share it without a `let` at
 either function's root.
 */
type WatchState = {
  lastRead: ScopeAccountingRead;
  stopped: boolean;
};

/**
 Read one check scope once, keep what it gave, and wait one interval before the next read.

 @param state - what this watch has read so far

 @param cgroupPath - the scope's control group

 @param intervalMilliseconds - how long to wait before the next read
 */
async function readOnce({
  state,
  cgroupPath,
  intervalMilliseconds,
}: {
  readonly state: WatchState;
  readonly cgroupPath: string;
  readonly intervalMilliseconds: number;
},): Promise<void> {
  /**
   What the scope holds now, or the statement that it is gone.
   */
  const read = await readScopeAccounting({ cgroupPath, },);
  if (read !== SCOPE_NOT_READ)
    state.lastRead = read;
  await waitUntil(intervalMilliseconds,);
}

/**
 Watch one check scope's own accounting while it runs.

 Reading starts at once, so a scope that is created and gone before the caller looks again still leaves what it counted
 in the sample this watch kept.

 @param cgroupPath - the scope's control group, from {@link checkScopeCgroupPath}

 @param intervalMilliseconds - how long between two reads, which {@link checkScopePollMilliseconds} sizes

 @returns the watch, whose last read is the last sample the scope gave

 @example
 ```ts
 await using watch = watchScopeAccounting({ cgroupPath: checkScopeCgroupPath({ name }) });
 ```
 */
export function watchScopeAccounting({
  cgroupPath,
  intervalMilliseconds = checkScopePollMilliseconds,
}: {
  readonly cgroupPath: string;
  readonly intervalMilliseconds?: number;
},): DisposableScopeAccountingWatch {
  /**
   What this watch has read, and whether its caller ended it.
   */
  const state: WatchState = {
    lastRead: SCOPE_NOT_READ,
    stopped: false,
  };
  /**
   Read until the caller stops, one interval between two reads.
   */
  const reading = (async function readUntilStopped(): Promise<void> {
    while (!state.stopped) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- a watch reads its scope once per interval
      await readOnce({
        state,
        cgroupPath,
        intervalMilliseconds,
      },);
    }
  })();
  /**
   Stop reading, once, and answer when the last read is kept.
   */
  async function stop(): Promise<void> {
    state.stopped = true;
    await reading;
  }
  return {
    lastRead: function read() {
      return state.lastRead;
    },
    stop,
    [Symbol.asyncDispose]: stop,
  };
}

//endregion Watching one check scope while it runs
