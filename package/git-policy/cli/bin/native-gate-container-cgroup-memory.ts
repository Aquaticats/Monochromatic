/**
 The kernel's own memory accounting of one gate container, read from that container's own cgroup.

 A container's cgroup exists only while the container does: `podman run --rm` removes it, and with it every file below
 `memory.*`. What a caller needs after the fact is therefore read while the container runs, which
 `bin/native-gate-container-memory.ts` does.

 The path the figures come from is the `libpod-*.scope` podman makes for the container, where the `--memory` bound
 becomes the cgroup's `memory.max`, and where the 2026-11-27 readings of both gate containers were taken
 (`doc/planning/git-policy-cli-cachyos-test-containers.md`, "The Clippy container's own memory bound, measured
 2026-11-27"). `memory.current` of that scope counts the container's children too, and `memory.events` counts the times
 the bound was reached and the kills it caused there, which is exactly what a killed check has to be told apart by.

 The kernel removes a control group's files one at a time, and the directory after them: `cgroup_destroy_locked` clears
 the files with `css_clear_dir` and only then calls `kernfs_remove` on the directory (`kernel/cgroup/cgroup.c`), and a
 file the kernel is removing answers `ENODEV` while it is deactivated (`fs/kernfs/file.c`). A reading taken while a
 container's cgroup goes therefore answers some of its files and not the others, and such a reading never replaces one
 that answered more (`{@link answersEveryPartOf}` in `bin/cgroup-whole-readings.ts`): a `memory.events` that went before
 the read would have the gate report a container killed inside its bound as one that met no kill
 (`bin/native-gate-container-memory.ts` prints that line).
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

//region Where a container's figures are

/**
 Root of this host's cgroup v2 hierarchy, where podman puts a container's scope.
 */
export const cgroupRoot = '/sys/fs/cgroup';

/**
 Bytes in one gibibyte, which is the unit a container's memory bound is stated in.
 */
export const bytesPerGibibyte = 1_073_741_824;

/**
 Bytes in one mebibyte, which is the unit a container's peak memory is stated in.
 */
export const bytesPerMebibyte = 1_048_576;

/**
 A container's cgroup that is not there to read, which a container that already ended leaves behind.

 A sentinel rather than a nullish value, because this repository models absence without nullish unions: every caller
 narrows it with `!==` or `typeof value === 'symbol'` before it reads a figure out of a sample.
 */
export const CGROUP_NOT_THERE: unique symbol = Symbol('container cgroup that is not there to read',);

/**
 A reading of a container's cgroup that is no whole reading of it: the group answered no file of its own, or answered
 less than the reading it would replace, which is a group the kernel is removing.

 Kept apart from {@link CGROUP_NOT_THERE} because a caller that ends something when the cgroup goes cannot end it on a
 reading it merely refused: only a directory the reader cannot stat is gone, and a group whose files are being removed is
 a group the next reading may still answer a part of.
 */
export const READING_NOT_WHOLE: unique symbol = Symbol('a control group answer that lost a part of the reading before it',);

/**
 A figure this host's kernel does not write, which is not a failure: an older kernel has no `memory.peak`, and a scope
 that ends between two reads answers no file at all.
 */
export const FIGURE_NOT_WRITTEN: unique symbol = Symbol('cgroup figure this host kernel does not write',);

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
 One container's own accounting, or the statement that its cgroup is not there to read, or that the reading taken is no
 whole reading of it.
 */
export type ContainerMemoryRead = ContainerMemorySample | typeof CGROUP_NOT_THERE | typeof READING_NOT_WHOLE;

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
 Split one line of a kernel file into its fields.

 The cgroup files separate their fields with one space and the `/proc` files with a tab or a run of spaces, so both count
 as a separator, and the empty fields a run leaves are dropped.

 @param line - the line

 @returns its fields

 @example
 ```ts
 fieldsOf('VmRSS:\t   12345 kB'); // ['VmRSS:', '12345', 'kB']
 ```
 */
export function fieldsOf(line: string,): readonly string[] {
  return line.trim()
    .replaceAll(
    '\t',
    ' ',
    )
    .split(' ',)
    .filter(function filled(field,) {
      return field !== '';
    },);
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
 Tells whether a container's cgroup is there to read.

 @param cgroupPath - the container's scope

 @returns true when the scope exists, false when the container ended and took it with it

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
 Read one container's own memory accounting.

 @param cgroupPath - the container's scope, from {@link containerCgroupPathOf}

 @param replaces - the reading this one would replace, which it must answer every part of

 @returns the figures the kernel holds for it now, or {@link CGROUP_NOT_THERE} once the container ended and
 {@link READING_NOT_WHOLE} once the reading answered no file of the group and once it answered less than the one it would
 replace

 @example
 ```ts
 const read = await readContainerMemory({ cgroupPath: containerCgroupPathOf({ containerId }) });
 ```
 */
export async function readContainerMemory({
  cgroupPath,
  replaces = CGROUP_NOT_THERE,
}: {
  readonly cgroupPath: string;
  readonly replaces?: ContainerMemoryRead;
},): Promise<ContainerMemoryRead> {
  /**
   Whether the container's scope is there to read at all.
   */
  const there = await cgroupIsThere(cgroupPath,);
  if (!there)
    return CGROUP_NOT_THERE;
  /**
   When this sample was read.
   */
  const readAt = new Date()
    .toISOString();
  /**
   Memory charged to the container and its children now.
   */
  const current = await figureOf(join(
    cgroupPath,
    'memory.current',
  ),);
  /**
   The highest charge since the container started.
   */
  const peak = await figureOf(join(
    cgroupPath,
    'memory.peak',
  ),);
  /**
   The bound the kernel holds the container at.
   */
  const max = await figureOf(join(
    cgroupPath,
    'memory.max',
  ),);
  /**
   Swap charged to the container now.
   */
  const swapCurrent = await figureOf(join(
    cgroupPath,
    'memory.swap.current',
  ),);
  /**
   The container's own swap bound.
   */
  const swapMax = await figureOf(join(
    cgroupPath,
    'memory.swap.max',
  ),);
  /**
   Processes in the container's cgroup now.
   */
  const pidsCurrent = await figureOf(join(
    cgroupPath,
    'pids.current',
  ),);
  /**
   The most processes it held since it started.
   */
  const pidsPeak = await figureOf(join(
    cgroupPath,
    'pids.peak',
  ),);
  /**
   What the charge is made of, which says whether a peak is the tests' own memory or the page cache they read.
   */
  const statCounters = await countersOf(join(
    cgroupPath,
    'memory.stat',
  ),);
  /**
   What the bound decided inside the container, which says whether a check was killed for memory.
   */
  const events = await countersOf(join(
    cgroupPath,
    'memory.events',
  ),);
  /**
   The sample being assembled, whose figures the kernel writes are filled in as each is read.
   */
  const sample: {
    cgroupPath: string;
    readAt: string;
    currentBytes?: number;
    peakBytes?: number;
    maxBytes?: number;
    swapCurrentBytes?: number;
    swapMaxBytes?: number;
    pidsCurrent?: number;
    pidsPeak?: number;
    stat: Readonly<Record<string, number>>;
    events: Readonly<Record<string, number>>;
  } = {
    cgroupPath,
    readAt,
    stat: statCounters,
    events,
  };
  if ((typeof current) !== 'symbol')
    sample.currentBytes = current;
  if ((typeof peak) !== 'symbol')
    sample.peakBytes = peak;
  if ((typeof max) !== 'symbol')
    sample.maxBytes = max;
  if ((typeof swapCurrent) !== 'symbol')
    sample.swapCurrentBytes = swapCurrent;
  if ((typeof swapMax) !== 'symbol')
    sample.swapMaxBytes = swapMax;
  if ((typeof pidsCurrent) !== 'symbol')
    sample.pidsCurrent = pidsCurrent;
  if ((typeof pidsPeak) !== 'symbol')
    sample.pidsPeak = pidsPeak;
  /**
   Whether the control group answered any figure at all, which a group that is there always does: a directory the kernel
   is removing answers no file of its own, which must not be read as a reading without figures.
   */
  const wroteAFigure = [
    current,
    peak,
    max,
    swapCurrent,
    swapMax,
    pidsCurrent,
    pidsPeak,
  ].some(function written(figure) {
    return (typeof figure) !== 'symbol';
  });
  /**
   Whether it answered any counter at all.
   */
  const wroteACounter = [
    statCounters,
    events,
  ].some(function counted(counters) {
    return Object.keys(counters,)
      .length
      > 0;
  });
  if ((!wroteAFigure) && (!wroteACounter))
    return READING_NOT_WHOLE;
  if (!answersEveryPartOf({
    read: sample,
    kept: replaces,
  },))
    return READING_NOT_WHOLE;
  return sample;
}

//endregion What the kernel says about one container
