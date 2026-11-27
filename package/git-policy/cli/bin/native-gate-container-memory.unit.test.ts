import { mkdir, mkdtempDisposable, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containerCgroupPathOf,
  containerMemoryKilled,
  containerMemoryKilledLine,
  gateMemoryPollMilliseconds,
  readContainerMemory,
  watchContainerMemory,
} from './native-gate-container-memory.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Milliseconds a test waits for a watcher to read a file it just wrote, which is many times the watcher's own interval
 on a host running other agents' builds.
 */
const watcherWindowMilliseconds = 10_000;

/**
 An ID in the shape podman writes to a container's ID file, which is also the name of the container's cgroup scope.
 */
const containerId = '3f2a9c1e'.repeat(8,);

/**
 Write one cgroup file of a fixture scope, making the directories above it.

 @param scope - the fixture scope's directory

 @param name - the file's name

 @param text - what the file holds

 @returns nothing, once the file is written
 */
async function writeScopeFile({
  scope,
  name,
  text,
}: {
  readonly scope: string;
  readonly name: string;
  readonly text: string;
}): Promise<void> {
  await mkdir(
    dirname(join(scope, name)),
    { recursive: true },
  );
  await writeFile(
    join(scope, name),
    text,
  );
}

/**
 A fixture whose cgroup files hold figures of a container that peaked inside its bound.

 @param root - the fixture's cgroup root

 @param scope - the fixture scope's directory, which the caller writes and reads

 @returns the scope path
 */
async function boundedScope({
  root,
  scope,
}: {
  readonly root: string;
  readonly scope: string;
}): Promise<string> {
  const path = join(
    root,
    scope,
  );
  await writeScopeFile({
    scope: path,
    name: 'memory.current',
    text: '629145600\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.peak',
    text: '1589641216\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.max',
    text: '3221225472\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.swap.current',
    text: '11534336\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.swap.max',
    text: '0\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'pids.current',
    text: '23\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'pids.peak',
    text: '133\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.stat',
    text: 'anon 71093452\nfile 481349632\nshmem 860160\nkernel 25894912\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.events',
    text: 'low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n',
  });
  return path;
}

/**
 Wait, bounded, for a condition a watcher brings about, and fail naming what it waited for.

 @param condition - what must become true

 @param description - what the wait is for, which a failure names

 @returns nothing, once the condition holds
 */
async function until({ condition, description }: {
  readonly condition: () => boolean;
  readonly description: string;
}): Promise<void> {
  const deadline = Date.now() + watcherWindowMilliseconds;
  while (!condition()) {
    if (Date.now() > deadline)
      throw new Error(`waited ${String(watcherWindowMilliseconds)} ms for ${description}`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- one look per pause is the wait itself
    await waitUntil(gateMemoryPollMilliseconds,);
  }
}

/**
 A sample of a container the kernel killed inside its bound, in the shape {@link readContainerMemory} answers.
 */
type KilledSample = NonNullable<ReturnType<typeof readContainerMemory>>;

/**
 A sample whose cgroup counted one out-of-memory kill at its bound.

 @returns the sample
 */
function killedSample(): KilledSample {
  return {
    cgroupPath: '/sys/fs/cgroup/libpod-x.scope',
    readAt: '2026-11-27T13:00:00.000Z',
    currentBytes: 3_221_225_472,
    peakBytes: 3_221_225_472,
    maxBytes: 3_221_225_472,
    swapCurrentBytes: 0,
    swapMaxBytes: 0,
    pidsCurrent: 4,
    pidsPeak: 576,
    stat: { anon: 1_073_741_824, },
    events: {
      max: 1_024,
      oom: 1,
      oom_kill: 1,
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: containerCgroupPathOf.name,
      children: [
        it({
          name: 'names the scope podman makes for a container under the account\'s own slice',
          fn: async () => {
            expect(containerCgroupPathOf({
              containerId,
              root: '/sys/fs/cgroup',
              userId: 1_000,
            }),).toBe(`/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/user.slice/libpod-${containerId}.scope`,);
          },
        }),
      ],
    }),
    describe({
      name: readContainerMemory.name,
      children: [
        it({
          name: 'reads the kernel\'s own figures of one container, its anonymous, file and shared charges among them',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-container-memory-test-',
            ),);
            /**
             The fixture scope's path, which its files are read from.
             */
            const scope = await boundedScope({
              root: tree.path,
              scope: 'bounded.scope',
            });
            /**
             What the reader answered for it.
             */
            const sample = readContainerMemory({ cgroupPath: scope, },);
            expect(sample.currentBytes,).toBe(629_145_600,);
            expect(sample.peakBytes,).toBe(1_589_641_216,);
            expect(sample.maxBytes,).toBe(3_221_225_472,);
            expect(sample.swapCurrentBytes,).toBe(11_534_336,);
            expect(sample.pidsCurrent,).toBe(23,);
            expect(sample.pidsPeak,).toBe(133,);
            expect(sample.stat.anon,).toBe(71_093_452,);
            expect(sample.stat.file,).toBe(481_349_632,);
            expect(sample.stat.shmem,).toBe(860_160,);
            expect(sample.events.oom_kill,).toBe(0,);
            expect(sample.readAt,).toMatch(/^\d{4}-\d{2}-\d{2}T/,);
          },
          timeout: 20_000,
        }),
        it({
          name: 'answers nothing for a cgroup that is not there, which is a container that already ended',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-container-memory-gone-',
            ),);
            expect(readContainerMemory({
              cgroupPath: join(
                tree.path,
                'gone.scope',
              ),
            },),).toBeUndefined();
          },
          timeout: 20_000,
        }),
        it({
          name: 'answers the files a scope holds and nothing for the ones it has not, so a kernel without one is no failure',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-container-memory-partial-',
            ),);
            /**
             A scope holding its events alone, as a kernel that writes no peak figure would leave one.
             */
            const scope = join(
              tree.path,
              'partial.scope',
            );
            await writeScopeFile({
              scope,
              name: 'memory.events',
              text: 'oom_kill 1\n',
            });
            /**
             What the reader answered for it.
             */
            const sample = readContainerMemory({ cgroupPath: scope, },);
            expect(sample.events.oom_kill,).toBe(1,);
            expect(sample.peakBytes,).toBeUndefined();
            expect(sample.stat,).toEqual({},);
          },
          timeout: 20_000,
        }),
      ],
    }),
    describe({
      name: containerMemoryKilled.name,
      children: [
        it({
          name: 'counts a cgroup whose events hold a kill, or an out-of-memory event alone',
          fn: async () => {
            expect(containerMemoryKilled({ events: { oom_kill: 1, }, },),).toBe(true,);
            expect(containerMemoryKilled({ events: { oom: 1, }, },),).toBe(true,);
            expect(containerMemoryKilled({ events: { oom: 0, oom_kill: 0, }, },),).toBe(false,);
            expect(containerMemoryKilled({ events: {}, },),).toBe(false,);
          },
        }),
      ],
    }),
    describe({
      name: containerMemoryKilledLine.name,
      children: [
        it({
          name: 'names the container, the kill, the bound the kernel held it at and the two options that raise it',
          fn: async () => {
            /**
             The line the gate prints when its suite's container was killed inside its bound.
             */
            const line = containerMemoryKilledLine({
              role: 'suite',
              sample: killedSample(),
            });
            expect(line,).toBeDefined();
            expect(line,).toContain('The suite\'s container',);
            expect(line,).toContain('killed for memory',);
            expect(line,).toContain('3.0 GiB',);
            expect(line,).toContain('--memory',);
            expect(line,).toContain('--container-memory',);
            expect(line,).toContain('not a failing test',);
          },
        }),
        it({
          name: 'names Clippy\'s container as its own when Clippy was the one killed',
          fn: async () => {
            expect(containerMemoryKilledLine({
              role: 'clippy',
              sample: killedSample(),
            }),).toContain('Clippy\'s container',);
          },
        }),
        it({
          name: 'says nothing when the container met no kill, so a green log carries no such line',
          fn: async () => {
            /**
             A sample of a container that peaked inside its bound and was not killed.
             */
            const survived = {
              ...killedSample(),
              currentBytes: 629_145_600,
              peakBytes: 1_516_845_056,
              events: {
                max: 0,
                oom: 0,
                oom_kill: 0,
              },
            };
            expect(containerMemoryKilledLine({
              role: 'suite',
              sample: survived,
            }),).toBeUndefined();
          },
        }),
        it({
          name: 'says nothing about a container whose cgroup was never read',
          fn: async () => {
            expect(containerMemoryKilledLine({
              role: 'suite',
              sample: undefined,
            }),).toBeUndefined();
          },
        }),
      ],
    }),
    describe({
      name: watchContainerMemory.name,
      children: [
        it({
          name: 'keeps the last figures a running container\'s cgroup gave, so a kill counted while it ran is still there when it ends',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-container-memory-watch-',
            ),);
            /**
             The scope the watch reads, in the shape the real root holds one.
             */
            const scope = await boundedScope({
              root: tree.path,
              scope: join(
                'user.slice',
                'user-1000.slice',
                'user@1000.service',
                'user.slice',
                `libpod-${containerId}.scope`,
              ),
            });
            await using watch = watchContainerMemory({
              containerId,
              root: tree.path,
              userId: 1_000,
              intervalMilliseconds: gateMemoryPollMilliseconds,
            });
            await until({
              condition: function readOnce() {
                return watch.lastRead() !== undefined;
              },
              description: 'the watch to read the scope\'s figures',
            },);
            expect(watch.lastRead()?.peakBytes,).toBe(1_589_641_216,);
            // The kill arrives after the watch has been reading, which is what a polled counter has to survive.
            await writeScopeFile({
              scope,
              name: 'memory.events',
              text: 'max 1024\noom 1\noom_kill 1\n',
            });
            await until({
              condition: function killed() {
                return containerMemoryKilled({ events: watch.lastRead()?.events ?? {}, },);
              },
              description: 'the watch to read the kill its scope counted',
            },);
            await watch.stop();
            expect(watch.lastRead()?.events.oom_kill,).toBe(1,);
          },
          timeout: 30_000,
        }),
        it({
          name: 'answers nothing for a container whose cgroup never appeared, rather than failing the gate it watches',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-container-memory-absent-',
            ),);
            await using watch = watchContainerMemory({
              containerId,
              root: tree.path,
              userId: 1_000,
              intervalMilliseconds: gateMemoryPollMilliseconds,
            });
            await waitUntil(50,);
            await watch.stop();
            expect(watch.lastRead(),).toBeUndefined();
          },
          timeout: 30_000,
        }),
      ],
    }),
  ],
});
