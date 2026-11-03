import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  chmod,
  lstat,
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDirectory } from './claim-test-support.ts';
import { CargoMutantsToolingError } from './error.ts';
import { probeLegacyLock } from './legacy-flock.ts';

/**
 Longest an old-style holder lives when a test forgets to kill it, in seconds.
 */
const holderSeconds = 60;

/**
 An old-style holder: the `flock` command of the earlier release, which takes the lock on a path and runs a command in it.
 */
type OldHolder = AsyncDisposable & {
  /**
   Kill the holder and wait until it has ended.
   */
  readonly kill: () => Promise<void>;
};

/**
 Start an old-style holder as the earlier release did (`flock --nonblock --exclusive <path> <command>`), and wait until it
 holds the lock.

 @param path - the lock file or directory

 @returns the holder; declare it with `await using` so it is killed at the end

 @example
 ```ts
 await using holder = await startOldHolder('/l/slot-1.lock');
 ```
 */
async function startOldHolder(path: string): Promise<OldHolder> {
  /**
   The holder: `flock` runs a shell that reports the lock and then becomes a sleeping process, so one process holds it.
   */
  const child = spawn(
    'flock',
    [
      '--nonblock',
      '--exclusive',
      '--conflict-exit-code',
      '200',
      path,
      'sh',
      '-c',
      `echo held; exec sleep ${String(holderSeconds)}`,
    ],
    {
      // A group of its own: `flock` runs the command as a child and the child holds the lock, so both must be killed.
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'inherit',
      ],
    },
  );
  /**
   The holder's process ID, which is also its group.
   */
  const { pid } = child;
  if ((child.stdout === null) || (pid === undefined))
    throw new Error('The holder has no standard output or process ID.');
  /**
   The group to kill, known now.
   */
  const group: number = pid;
  await once(
    child.stdout,
    'data',
  );
  /**
   Kill the holder and wait for it.
   */
  async function kill(): Promise<void> {
    if ((child.exitCode !== null) || (child.signalCode !== null))
      return;
    process.kill(
      -group,
      'SIGKILL',
    );
    await once(
      child,
      'close',
    );
  }
  return {
    kill,
    async [Symbol.asyncDispose](): Promise<void> {
      await kill();
    },
  };
}

/**
 Write a program that stands in for `flock`.

 @param directory - where to write it

 @param body - the script text, run by Node

 @returns its path

 @example
 ```ts
 await fakeFlock({ directory: '/d', body: 'process.exit(3);' });
 ```
 */
async function fakeFlock({
  directory,
  body,
}: {
  readonly directory: string;
  readonly body: string;
}): Promise<string> {
  /**
   Where the program goes.
   */
  const path = join(
    directory,
    'flock',
  );
  await writeFile(
    path,
    `#!${process.execPath}\n${body}\n`,
  );
  await chmod(
    path,
    0o755,
  );
  return path;
}

await describe({
  name: probeLegacyLock.name,
  children: [
    describe({
      name: 'a lock file',
      children: [
        it({
          name: 'finds nothing to respect or remove where the file is absent',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-absent-');
            expect(await probeLegacyLock({
              path: join(
                directory.path,
                'slot-1.lock',
              ),
              retireWhenFree: true,
            })).toBe('absent');
          },
        }),
        it({
          name: 'treats a file an old command holds as a live owner, and leaves it in place',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-held-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await using holder = await startOldHolder(path);
            expect(holder).toBeDefined();
            expect(await probeLegacyLock({
              path,
              retireWhenFree: true,
            })).toBe('held');
            expect((await lstat(path)).isFile()).toBe(true);
          },
        }),
        it({
          name: 'removes a file nothing holds, and then finds it absent',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-free-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await writeFile(
              path,
              '',
            );
            expect(await probeLegacyLock({
              path,
              retireWhenFree: true,
            })).toBe('retired');
            expect(await probeLegacyLock({
              path,
              retireWhenFree: true,
            })).toBe('absent');
          },
        }),
        it({
          name: 'removes the file once the old command that held it was killed',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-killed-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await using holder = await startOldHolder(path);
            expect(await probeLegacyLock({
              path,
              retireWhenFree: true,
            })).toBe('held');
            await holder.kill();
            expect(await probeLegacyLock({
              path,
              retireWhenFree: true,
            })).toBe('retired');
          },
        }),
        it({
          name: 'keeps a file nothing holds when asked only to probe',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-probe-only-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await writeFile(
              path,
              '',
            );
            expect(await probeLegacyLock({
              path,
              retireWhenFree: false,
            })).toBe('free');
            expect((await lstat(path)).isFile()).toBe(true);
          },
        }),
        it({
          name: 'stops, leaving a link where it is, rather than following it',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-link-');
            const target = join(
              directory.path,
              'elsewhere',
            );
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await writeFile(
              target,
              '',
            );
            await symlink(
              target,
              path,
            );
            let caught: unknown;
            try {
              await probeLegacyLock({
                path,
                retireWhenFree: true,
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CargoMutantsToolingError);
            expect((await lstat(path)).isSymbolicLink()).toBe(true);
            expect((await lstat(target)).isFile()).toBe(true);
          },
        }),
        ...[
          ['a flock that cannot be run', 'missing'],
          ['a flock that ends with an unknown status', 'process.exit(3);'],
          ['a flock that is killed', 'process.kill(process.pid, \'SIGKILL\');'],
        ].map(function failure([name, body]) {
          return it({
            name: `fails closed on ${String(name)}, and removes nothing`,
            fn: async () => {
              await using directory = await scratchDirectory('legacy-fail-');
              const path = join(
                directory.path,
                'slot-1.lock',
              );
              await writeFile(
                path,
                '',
              );
              const program = body === 'missing'
                ? join(
                  directory.path,
                  'no-such-flock',
                )
                : await fakeFlock({
                  directory: directory.path,
                  body: String(body),
                });
              let caught: unknown;
              try {
                await probeLegacyLock({
                  path,
                  retireWhenFree: true,
                  program,
                });
              }
              catch (error) {
                caught = error;
              }
              expect(caught).toBeInstanceOf(CargoMutantsToolingError);
              expect((await lstat(path)).isFile()).toBe(true);
            },
          });
        }),
      ],
    }),
    describe({
      name: 'a directory lock',
      children: [
        it({
          name: 'treats a directory an old command holds as a live owner, and is free once that command is killed, leaving the directory',
          fn: async () => {
            await using scratch = await scratchDirectory('legacy-directory-');
            const path = join(
              scratch.path,
              'evidence',
            );
            await mkdir(path);
            await using holder = await startOldHolder(path);
            expect(await probeLegacyLock({
              path,
              retireWhenFree: false,
            })).toBe('held');
            await holder.kill();
            expect(await probeLegacyLock({
              path,
              retireWhenFree: false,
            })).toBe('free');
            expect((await lstat(path)).isDirectory()).toBe(true);
          },
        }),
      ],
    }),
  ],
});
