/**
 The kernel's own memory accounting of one gate container, read while the container runs, and the line the gate prints
 when one of its containers was killed inside its memory bound.

 A container's cgroup exists only while the container does: `podman run --rm` removes it, and with it every file below
 `memory.*`. So the figures a gate needs after the fact are read while the container runs and kept, which is what
 `watchContainerMemory` does; a reader that asks at the end asks a directory that is gone.

 The path the figures come from is the `libpod-*.scope` podman makes for the container, which is where the `--memory`
 bound becomes the cgroup's `memory.max` and where the 2026-11-27 readings of both gate containers were taken
 (`doc/planning/git-policy-cli-cachyos-test-containers.md`, "The Clippy container's own memory bound, measured
 2026-11-27"). `memory.current` of that scope counts the container's children as well, and `memory.events` counts the
 times the bound was reached and the kills it caused there, which is exactly the question a killed check asks.
 */
import { existsSync, readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';

import type { GateContainerRole, } from './native-gate-container-identity.ts';

//region Where a container's figures are

/**
 Root of this host's cgroup v2 hierarchy, where podman puts a container's scope.
 */
export const cgroupRoot = '/sys/fs/cgroup';

/**
 Milliseconds between two reads of a container's own cgroup while it runs.

 A kill only raises a counter, so the interval decides one thing: how long after a kill the figure is still there to be
 read. A container that ends takes its scope with it, and the gate reads the last sample it kept, so the interval is
 short enough that a kill is seen before the container's own shutdown removes the scope, and long enough that the reads
 cost nothing beside the suite beside them.
 */
export const gateMemoryPollMilliseconds = 100;

/**
 The cgroup of one gate container, as podman makes it under the account's own slice.

 @param containerId - the ID the container's client wrote to its ID file, which names the scope

 @param root - the cgroup hierarchy's root, which a test points at a fixture tree

 @param userId - the account whose user manager started the container, which the scope's path carries

 @returns the absolute path of the container's own scope

 @example
 ```ts
 containerCgroupPathOf({ containerId, userId: 1000 });
 ```
 */
export function containerCgroupPathOf({
  containerId,
  root = cgroupRoot,
  userId = process.getuid?.() ?? 0,
}: {
  readonly containerId: string;
  readonly root?: string;
  readonly userId?: number;
},): string {
  return join(
    root,
    'user.slice',
    `user-${String(userId)}.slice`,
    `user@${String(userId)}.service`,
    'user.slice',
    `libpod-${containerId}.scope`,
  );
}

//endregion Where a container's figures are

//region What the kernel says about one container

/**
 One container's figures, as the kernel held them at the moment they were read.
 */
export type ContainerMemorySample = {
  /**
   The scope the figures came from.
   */
  readonly cgroupPath: string;
  /**
   When they were read, in ISO 8601.
   */
  readonly readAt: string;
  /**
   Memory charged to the container and its children now, which page cache and the fixture root's shared pages count in.
   */
  readonly currentBytes?: number;
  /**
   The highest charge since the container started, which is the figure a bound is compared with.
   */
  readonly peakBytes?: number;
  /**
   The bound itself: the cgroup's `memory.max`, which is the gate's `--memory` for this container.
   */
  readonly maxBytes?: number;
  /**
   Swap charged to the container now, which a host under pressure puts there rather than its bound.
   */
  readonly swapCurrentBytes?: number;
  /**
   The container's swap bound, which the gate does not set.
   */
  readonly swapMaxBytes?: number;
  /**
   Processes in the container's cgroup now.
   */
  readonly pidsCurrent?: number;
  /**
   The most processes it held since it started.
   */
  readonly pidsPeak?: number;
  /**
   Every counter of `memory.stat`, by name: `anon` is the tests' own heap and stacks, `file` the page cache of what
   they read, and `shmem` the fixture root's tmpfs pages.
   */
  readonly stat: Readonly<Record<string, number>>;
  /**
   Every counter of `memory.events`, by name: `max` counts the times the bound was reached and `oom_kill` the kills.
   */
  readonly events: Readonly<Record<string, number>>;
};

/**
 Read one file of a cgroup as trimmed text.

 @param path - the file

 @returns its text, or nothing when the file is not there or cannot be read
 */
function textOf(path: string,): string | undefined {
  try {
    return readFileSync(
      path,
      'utf8',
    ).trim();
  }
  catch {
    // A file a kernel does not write, or a scope that ended between two reads, is no reading at all and not a failure.
    return undefined;
  }
}

/**
 Read one file of a cgroup as a whole number.

 @param path - the file

 @returns the number it holds, or nothing when it holds anything else or is not there
 */
function numberOf(path: string,): number | undefined {
  /**
   The file's text.
   */
  const text = textOf(path,);
  if ((text === undefined) || (text === ''))
    return undefined;
  for (const character of text)
    if ((character < '0') || (character > '9'))
      return undefined;
  return Number(text,);
}

/**
 Read one file of a cgroup as the counters it holds, one `name value` pair per line.

 @param path - the file

 @returns every counter, by name, empty when the file is not there
 */
function countersOf(path: string,): Readonly<Record<string, number>> {
  /**
   The file's text.
   */
  const text = textOf(path,);
  if (text === undefined)
    return {};
  /**
   The counters read so far.
   */
  const counters: Record<string, number> = {};
  for (const line of text.split('\n',)) {
    /**
     The two fields a counter line holds.
     */
    const [name, value, ...surplus] = line.trim()
      .split(/\s+/,);
    if ((name === undefined) || (value === undefined) || (surplus.length > 0))
      continue;
    if (![...value,].every(function isDigit(character,) {
      return (character >= '0') && (character <= '9');
    },))
      continue;
    counters[name] = Number(value,);
  }
  return counters;
}

/**
 Read one container's own memory accounting.

 @param cgroupPath - the container's scope, from {@link containerCgroupPathOf}

 @returns the figures the kernel holds for it, or nothing when that scope is not there, which a container that ended
   leaves behind

 @example
 ```ts
 const sample = readContainerMemory({ cgroupPath: containerCgroupPathOf({ containerId }) });
 ```
 */
export function readContainerMemory({ cgroupPath }: { readonly cgroupPath: string; },): ContainerMemorySample | undefined {
  if (!existsSync(cgroupPath,))
    return undefined;
  /**
   The kernel's own figures, one file each.
   */
  const [current, peak, max, swapCurrent, swapMax, pidsCurrent, pidsPeak] = [
    'memory.current',
    'memory.peak',
    'memory.max',
    'memory.swap.current',
    'memory.swap.max',
    'pids.current',
    'pids.peak',
  ].map(function figureOf(name,) {
    return numberOf(join(
      cgroupPath,
      name,
    ),);
  },);
  return {
    cgroupPath,
    readAt: new Date()
      .toISOString(),
    ...(current === undefined ? {} : { currentBytes: current, }),
    ...(peak === undefined ? {} : { peakBytes: peak, }),
    ...(max === undefined ? {} : { maxBytes: max, }),
    ...(swapCurrent === undefined ? {} : { swapCurrentBytes: swapCurrent, }),
    ...(swapMax === undefined ? {} : { swapMaxBytes: swapMax, }),
    ...(pidsCurrent === undefined ? {} : { pidsCurrent, }),
    ...(pidsPeak === undefined ? {} : { pidsPeak, }),
    stat: countersOf(join(
      cgroupPath,
      'memory.stat',
    ),),
    events: countersOf(join(
      cgroupPath,
      'memory.events',
    ),),
  };
}

/**
 Tells whether a container's own events count a kill for memory.

 `oom_kill` counts kills the cgroup caused, and `oom` counts out-of-memory events there even when the kernel chose no
 process to end; either says the bound decided something.

 @param events - the `memory.events` counters of a container's cgroup

 @returns true when the bound killed or refused something inside it, false when it did not, including for an empty set

 @example
 ```ts
 containerMemoryKilled({ events: { oom_kill: 1 } });
 ```
 */
export function containerMemoryKilled({ events }: { readonly events: Readonly<Record<string, number>>; },): boolean {
  return ((events.oom_kill ?? 0) > 0) || ((events.oom ?? 0) > 0);
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

 @returns the number of GiB to one decimal, with thousands separated, and `no` reading as nothing

 @example
 ```ts
 gibText(3_221_225_472);
 ```
 */
function gibText(bytes: number,): string {
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  },)
    .format(bytes / 1_073_741_824,);
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
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  },)
    .format(bytes / 1_048_576,);
}

/**
 The line the gate prints when one of its containers was killed inside its memory bound, and nothing when none was.

 A test that was ended by the kernel reads like any test that failed, so this line says which container's bound did it,
 how often, and what a person does about it; it is printed beside the check's own refusal and never instead of it.

 @param role - which of the gate's containers the sample is of

 @param sample - the last figures read from that container's cgroup, absent when its scope was never read

 @returns one sentence, ending in a full stop, or nothing when that container met no kill

 @example
 ```ts
 console.log(containerMemoryKilledLine({ role: 'suite', sample }));
 ```
 */
export function containerMemoryKilledLine({
  role,
  sample,
}: {
  readonly role: GateContainerRole;
  readonly sample: ContainerMemorySample | undefined;
},): string | undefined {
  if ((sample === undefined) || !containerMemoryKilled({ events: sample.events, },))
    return undefined;
  /**
   Kills the cgroup counted, which is one or more here.
   */
  const kills = sample.events.oom_kill ?? 0;
  /**
   The bound the kernel held the container at, as the kernel states it.
   */
  const bound = sample.maxBytes === undefined
    ? 'its memory.max bound'
    : `the ${gibText(sample.maxBytes,)} GiB memory.max bound it was given`;
  /**
   The highest charge the container reached, which says how far past the bound a run got.
   */
  const peak = sample.peakBytes === undefined
    ? ''
    : `, with a peak of ${mibText(sample.peakBytes,)} MiB`;
  /**
   How the kills are counted, in the singular when there was one.
   */
  const counted = kills === 1
    ? '1 out-of-memory kill'
    : `${String(kills)} out-of-memory kills`;
  return `${containerNames[role]} was killed for memory: its cgroup counted ${counted} at ${bound}${peak}. That is the bound and not a failing test: raise it with --memory after the task name, or with --container-memory when native:gate starts the run.`;
}

//endregion What the kernel says about one container

//region Watching a container while it runs

/**
 One running container's memory watch: the last figures it read, and how to end it.

 The last read is kept because a container's cgroup goes with it: what a caller asks after the container ended can only
 be what was read while it ran.
 */
export type ContainerMemoryWatch = {
  /**
   The last figures read from the container's cgroup, absent when its scope was never there.
   */
  readonly lastRead: () => ContainerMemorySample | undefined;
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
 Watch one container's own memory accounting while it runs.

 Reading starts at once, so a container that is killed and gone before the gate looks again still leaves the kill in
 the counter this watch kept.

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
  root = cgroupRoot,
  userId = process.getuid?.() ?? 0,
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
    root,
    userId,
  },);
  /**
   The last figures the scope gave, which is what the caller reads once the container is gone.
   */
  let lastRead: ContainerMemorySample | undefined;
  /**
   Whether the caller has ended this watch.
   */
  let stopped = false;
  /**
   Read until the caller stops, one interval between two reads.
   */
  const reading = (async function readUntilStopped(): Promise<void> {
    while (!stopped) {
      /**
       What the scope holds now, absent once the container ended.
       */
      const sample = readContainerMemory({ cgroupPath, },);
      if (sample !== undefined)
        lastRead = sample;
      // oxlint-disable-next-line eslint/no-await-in-loop -- a watch reads once per interval until it is stopped
      await waitUntil(intervalMilliseconds,);
    }
  })();
  /**
   Stop reading, once, and answer when the last read is kept.
   */
  async function stop(): Promise<void> {
    stopped = true;
    await reading;
  }
  return {
    lastRead: function read() {
      return lastRead;
    },
    stop,
    [Symbol.asyncDispose]: stop,
  };
}

//endregion Watching a container while it runs
