/**
 One check scope's own accounting, read from its control group while the check runs.

 A check's scope is a transient systemd scope, and the check's own children are what its control group holds: `pids.peak`
 and `pids.events` say whether the task bound refused a fork, `memory.peak` and `memory.events` say whether the memory
 bound killed something, and `cpu.stat` says whether the CPU quota held the run back. The scope goes with its work, so
 these figures are read while it runs and the last sample is kept (`bin/native-check-scope-watch-accounting.ts` watches
 them, as `bin/native-gate-container-memory.ts` watches a gate container's).

 The scope's path is known without asking the manager: `bin/native-capped-scope-command.ts` starts every check with
 `--slice=app.slice`, so its scope is
 `<root>/user.slice/user-<uid>.slice/user@<uid>.service/app.slice/<name>.scope`. Measured on this host on 2026-11-27: a
 `systemd-run --user --scope` that named no slice landed in that same `app.slice`, and `pids.peak`, `pids.events`,
 `memory.peak`, `memory.events`, `memory.max`, `cpu.max` and `cpu.stat` were all there to read, beside the `.local`
 variants of the two event files that count a cgroup's own bound rather than its subtree's.

 The kernel removes a control group's files one at a time, and the directory after them: `cgroup_destroy_locked` clears the
 files with `css_clear_dir` and only then calls `kernfs_remove` on the directory (`kernel/cgroup/cgroup.c`). A reading
 taken while a scope ends therefore answers some of the group's files and not the others, and a file the kernel is
 removing answers `ENODEV` while it is deactivated (`fs/kernfs/file.c`). Such a reading never replaces one that answered
 more (`{@link answersEveryPartOf}` in `bin/cgroup-whole-readings.ts`): a figure or a counter this host's kernel wrote a
 moment ago is not one it omits, so what a reading lost is what the group took with it, and a check judged on that
 reading would be told no bound of its own failed it (`bin/native-check-scope-bound-lines.ts`).
 */
import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import {
  answersEveryPartOf,
  isRemoved,
} from './cgroup-whole-readings.ts';
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

 @returns its text, or {@link FIGURE_NOT_WRITTEN} when this host's kernel writes no such file and when the kernel is
 removing it

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
    if (isMissing(error,) || isRemoved(error,))
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
 Tells whether a counter file held any counter, which the kernel writes for a file that is there.

 @param counters - what one counter file gave

 @returns true when the file held at least one `name value` line

 @example
 ```ts
 holdsACounter({ max: 0 });
 ```
 */
function holdsACounter(counters: Readonly<Record<string, number>>,): boolean {
  if (Object.keys(counters,)
    .length
    > 0)
    return true;
  return false;
}

/**
 Read one check scope's own accounting.

 @param cgroupPath - the scope's control group, from {@link checkScopeCgroupPath}

 @param replaces - the reading this one would replace, which it must answer every part of

 @returns the figures the kernel holds for it now, or {@link SCOPE_NOT_READ} once the scope ended and once a reading lost
 a part of the one it would replace

 @example
 ```ts
 const read = await readScopeAccounting({ cgroupPath: checkScopeCgroupPath({ name }) });
 ```
 */
export async function readScopeAccounting({
  cgroupPath,
  replaces = SCOPE_NOT_READ,
}: {
  readonly cgroupPath: string;
  readonly replaces?: ScopeAccountingRead;
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
  /**
   Whether the control group wrote any figure at all: a scope that is still there always writes one, and a directory
   that is being removed writes none, which must not be read as a reading without figures.
   */
  const wroteAFigure = [
    pidsCurrent,
    pidsPeak,
    pidsMax,
    memoryCurrent,
    memoryPeak,
    memoryMax,
    cpuMax.quota,
    cpuMax.period,
  ].some(function written(figure) {
    return (figure !== undefined) && ((typeof figure) !== 'symbol');
  });
  /**
   Whether the control group answered any counter at all.
   */
  const wroteACounter = [
    pidsEvents,
    memoryEvents,
    cpuStat,
  ].some(holdsACounter,);
  if ((!wroteAFigure) && (!wroteACounter))
    return SCOPE_NOT_READ;
  if (!answersEveryPartOf({
    read: sample,
    kept: replaces,
  },))
    return SCOPE_NOT_READ;
  return sample;
}

//endregion What the kernel says about one check scope
