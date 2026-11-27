/**
 One check scope's accounting, watched while it runs: the last reading it gave, and the interval between two readings.

 A scope takes its control group with it, so the figures a check is judged by are read while it runs and the last reading
 is kept; `bin/native-check-scope-accounting.ts` reads one moment of it and says what a whole reading is. The kernel
 removes a control group's files one at a time and the directory after them (`kernel/cgroup/cgroup.c`,
 `cgroup_destroy_locked`: `css_clear_dir`, then `kernfs_remove`), so a reading taken while a scope ends answers a part of
 the group. Such a reading is never kept in place of the whole one before it, and what a caller is answered with is the
 last reading that answered every part of its own predecessor.
 */
import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';

import {
  SCOPE_NOT_READ,
  type ScopeAccountingRead,
  readScopeAccounting,
} from './native-check-scope-accounting.ts';

//region Watching one check scope while it runs

/**
 Milliseconds between two reads of a check's scope while it runs.

 The figures a check is judged by are counters and peaks, which only rise, so the interval decides one thing: whether a
 reading is still there to be taken. A scope that ends takes its control group with it, and the caller keeps the last
 sample, so the interval is short enough that a check which waits for something after its first refused fork is still
 read, and fifty milliseconds of file reads cost nothing beside the check running next to them. The limit is a check
 that ends within one interval of the event: measured on 2026-11-27, a unit suite narrowed to `--unit-tasks-max 64`
 died inside that window and printed no bound line, and one narrowed to 120 printed its two refused forks.
 */
export const checkScopePollMilliseconds = 50;

/**
 One running check scope's accounting watch: the last figures it read, and how to end it.

 The last read is kept because a scope's control group goes with it: what a caller asks after the check ended can only
 be what was read while it ran. A reading that answers less than the one before it read the group while it was going, so
 it is not kept: what a caller is told is the last reading that answered every part of the one before it, which is at
 worst the reading of the moment the group began to go.
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
   What the scope holds now, or the statement that it is gone and that it lost a part of what was read before it.
   */
  const read = await readScopeAccounting({
    cgroupPath,
    replaces: state.lastRead,
  },);
  if (read !== SCOPE_NOT_READ)
    state.lastRead = read;
  await waitUntil(intervalMilliseconds,);
}

/**
 Watch one check scope's own accounting while it runs.

 Reading starts at once, so a scope that is created and gone before the caller looks again still leaves what it counted
 in the sample this watch kept.

 @param cgroupPath - the scope's control group, from `checkScopeCgroupPath` in `bin/native-check-scope-accounting.ts`

 @param intervalMilliseconds - how long between two reads, which {@link checkScopePollMilliseconds} sizes

 @returns the watch, whose last read is the last sample the scope gave

 @example
 ```ts
 await using watch = watchScopeAccounting({ cgroupPath });
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
