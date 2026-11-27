/**
 The watch a gate keeps of one container's own memory accounting while it runs, and the line it prints when that
 container was killed inside its bound.

 A container killed for memory reads like a failing test: the kernel ends the process the bound picked, the check reports
 a signal, and nothing in that account names the bound. So the gate reads the container's own `memory.events` while the
 container runs, keeps the last figures, and says what it found once the check is over, beside the check's own refusal
 and never instead of it.

 The figures themselves are `bin/native-gate-container-cgroup-memory.ts`'s; this module decides when to read them and
 what to say about a kill.
 */
import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';

import type { GateContainerRole, } from './native-gate-container-identity.ts';
import {
  type ContainerMemoryRead,
  type ContainerMemorySample,
  CGROUP_NOT_THERE,
  bytesPerGibibyte,
  bytesPerMebibyte,
  containerCgroupPathOf,
  readContainerMemory,
} from './native-gate-container-cgroup-memory.ts';

//region What a container's events say

/**
 Milliseconds between two reads of a container's own cgroup while it runs.

 A kill only raises a counter, so the interval decides one thing: whether the figure is still there to be read. A
 container that ends takes its scope with it, and the gate reads the last sample it kept, so the interval is short
 enough that a kill is seen before the container's own shutdown removes the scope, and long enough that the reads cost
 nothing beside the suite running next to them.
 */
export const gateMemoryPollMilliseconds = 100;

/**
 The statement that a container met no kill inside its bound, which a sentinel carries because this repository models
 absence without nullish unions.
 */
export const NO_MEMORY_KILL: unique symbol = Symbol('container that met no memory kill inside its bound',);

/**
 Tells whether a container's own events count a kill for memory.

 `oom_kill` counts the kills the container's bound caused. `oom` counts out-of-memory events there, which an event that
 ended no process also raises, so it alone does not say a kill happened.

 @param events - the `memory.events` counters of a container's cgroup

 @returns true when the bound killed something inside it, false when it did not, including for an empty set

 @example
 ```ts
 containerMemoryKilled({ events: { oom_kill: 1 } });
 ```
 */
export function containerMemoryKilled({ events }: { readonly events: Readonly<Record<string, number>>; },): boolean {
  return (events.oom_kill ?? 0) > 0;
}

/**
 How the gate names each of its containers in a line a person reads.
 */
const containerNames: Readonly<Record<GateContainerRole, string>> = {
  suite: 'The suite\'s container',
  clippy: 'Clippy\'s container',
};

/**
 Format a byte figure as the GiB a bound is stated in.

 @param bytes - the figure

 @returns the number of GiB to one decimal, with thousands separated

 @example
 ```ts
 gibText(3_221_225_472);
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
 Format a byte figure as the MiB a peak is stated in.

 @param bytes - the figure

 @returns the number of MiB to one decimal, with thousands separated

 @example
 ```ts
 mibText(1_516_845_056);
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
 The bound the kernel held a container at, as the kernel itself states it.

 @param sample - the figures of the container

 @returns the bound, as a GiB figure when the kernel wrote one
 */
function boundText({ sample }: { readonly sample: ContainerMemorySample; },): string {
  if (sample.maxBytes === undefined)
    return 'its memory.max bound';
  return `the ${gibText(sample.maxBytes,)} GiB memory.max bound it was given`;
}

/**
 The peak the container reached, which says how far past the bound a run got.

 @param sample - the figures of the container

 @returns the peak as a trailing clause, or nothing when the kernel wrote no peak
 */
function peakText({ sample }: { readonly sample: ContainerMemorySample; },): string {
  if (sample.peakBytes === undefined)
    return '';
  return `, with a peak of ${mibText(sample.peakBytes,)} MiB`;
}

/**
 The line the gate prints when one of its containers was killed inside its memory bound, and nothing when none was.

 A test that the kernel ended reads like any test that failed, so this line says which container's bound did it, how
 often, and what a person does about it.

 @param role - which of the gate's containers the sample is of

 @param sample - the last figures read from that container's cgroup, or {@link CGROUP_NOT_THERE} when its scope was never
   read

 @returns one sentence ending in a full stop, or {@link NO_MEMORY_KILL} when that container met no kill

 @example
 ```ts
 const killed = containerMemoryKilledLine({ role: 'suite', sample });
 if (killed !== NO_MEMORY_KILL) console.log(killed);
 ```
 */
export function containerMemoryKilledLine({
  role,
  sample,
}: {
  readonly role: GateContainerRole;
  readonly sample: ContainerMemoryRead;
},): string | typeof NO_MEMORY_KILL {
  if (((typeof sample) === 'symbol') || (!containerMemoryKilled({ events: sample.events, },)))
    return NO_MEMORY_KILL;
  /**
   Kills the cgroup counted, which is one or more here.
   */
  const kills = sample.events
    .oom_kill
    ?? 0;
  /**
   How the kills are counted, in the singular when there was one.
   */
  const counted = kills === 1
    ? '1 out-of-memory kill'
    : `${String(kills)} out-of-memory kills`;
  return `${containerNames[role]} was killed for memory: its cgroup counted ${counted} at ${boundText({ sample, })}${peakText({ sample, })}. That is the bound and not a failing test: raise it with --memory after the task name, or with --container-memory when native:gate starts the run.`;
}

//endregion What a container's events say

//region Watching a container while it runs

/**
 Read one container's scope once, keep what it gave, and wait one interval before the next read.

 @param state - what this watch has read so far

 @param cgroupPath - the container's scope

 @param intervalMilliseconds - how long to wait before the next read
 */
async function readOnce({
  state,
  cgroupPath,
  intervalMilliseconds,
}: {
  readonly state: MemoryWatchState;
  readonly cgroupPath: string;
  readonly intervalMilliseconds: number;
},): Promise<void> {
  /**
   What the scope holds now, or the statement that it is gone.
   */
  const read = await readContainerMemory({ cgroupPath, },);
  if (read !== CGROUP_NOT_THERE)
    state.lastRead = read;
  await waitUntil(intervalMilliseconds,);
}

/**
 One running container's memory watch: the last figures it read, and how to end it.

 The last read is kept because a container's cgroup goes with it: what a caller asks after the container ended can only
 be what was read while it ran.
 */
export type ContainerMemoryWatch = {
  /**
   The last figures read from the container's cgroup, or {@link CGROUP_NOT_THERE} when its scope was never there.
   */
  readonly lastRead: () => ContainerMemoryRead;
  /**
   Stop reading, and answer once the last read is kept. Calling it twice is the same as calling it once.
   */
  readonly stop: () => Promise<void>;
};

/**
 A memory watch that can be disposed of, which is what `await using` needs of one.
 */
export type DisposableContainerMemoryWatch = ContainerMemoryWatch & AsyncDisposable;

/**
 What one watch has read so far, held in an object so the reading function and its caller share it without a `let` at
 either function's root.

 @param lastRead - the last figures the container's scope gave

 @param stopped - whether the caller has ended this watch
 */
export type MemoryWatchState = {
  lastRead: ContainerMemoryRead;
  stopped: boolean;
};

/**
 Watch one container's own memory accounting while it runs.

 Reading starts at once, so a container that is killed and gone before the gate looks again still leaves the kill in the
 counter this watch kept.

 @param containerId - the ID the container's client wrote to its ID file

 @param root - the cgroup hierarchy's root, which a test points at a fixture tree

 @param userId - the account whose user manager runs the container

 @param intervalMilliseconds - how long between two reads, which {@link gateMemoryPollMilliseconds} sizes

 @returns the watch, whose last read is the last sample the scope gave

 @example
 ```ts
 await using watch = watchContainerMemory({ containerId });
 ```
 */
export function watchContainerMemory({
  containerId,
  root,
  userId,
  intervalMilliseconds = gateMemoryPollMilliseconds,
}: {
  readonly containerId: string;
  readonly root?: string;
  readonly userId?: number;
  readonly intervalMilliseconds?: number;
},): DisposableContainerMemoryWatch {
  /**
   The scope whose figures this watch reads.
   */
  const cgroupPath = containerCgroupPathOf({
    containerId,
    ...(root === undefined ? {} : { root, }),
    ...(userId === undefined ? {} : { userId, }),
  },);
  /**
   What this watch has read, and whether its caller ended it.
   */
  const state: MemoryWatchState = {
    lastRead: CGROUP_NOT_THERE,
    stopped: false,
  };
  /**
   Read until the caller stops, one interval between two reads.
   */
  const reading = (async function readUntilStopped(): Promise<void> {
    while (!state.stopped) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- a watch reads its container's scope once per interval
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

//endregion Watching a container while it runs
