import {
  chmod,
  lstat,
  mkdir,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type OldHolder,
  scratchDirectory,
  startOldHolder,
} from './claim-test-support.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  holdLegacyLock,
  probeLegacyLock,
} from './legacy-flock.ts';

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
          name: 'does not remove a file an older command created in place of the one it locked, and respects that command instead',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-swapped-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await writeFile(
              path,
              '',
            );
            const replacements: OldHolder[] = [];
            const standing = await probeLegacyLock({
              path,
              retireWhenFree: true,
              afterLocking: async () => {
                // Another probe retired the file, and a waiting gate of the earlier release created and locked a new one.
                await rm(path);
                if (replacements.length === 0)
                  replacements.push(await startOldHolder(path));
              },
            });
            expect(standing).toBe('held');
            expect((await lstat(path)).isFile()).toBe(true);
            await Promise.all(replacements.map(async function stop(replacement) {
              await replacement.kill();
            }));
          },
          timeout: 30_000,
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
    describe({
      name: holdLegacyLock.name,
      children: [
        it({
          name: 'creates a missing slot file when asked, keeps the lock on it so a probe finds it held, and leaves the file once the lock is given back',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-hold-file-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            const hold = await holdLegacyLock({
              path,
              create: true,
            });
            if (hold.kind !== 'kept')
              throw new Error(`Expected the lock to be kept, found ${hold.kind}.`);
            expect(await probeLegacyLock({
              path,
              retireWhenFree: false,
            })).toBe('held');
            await hold.lock[Symbol.asyncDispose]();
            expect(await probeLegacyLock({
              path,
              retireWhenFree: false,
            })).toBe('free');
            expect((await lstat(path)).isFile()).toBe(true);
          },
        }),
        it({
          name: 'reports busy for a file an old command holds, and absent for a missing directory it was not asked to create',
          fn: async () => {
            await using directory = await scratchDirectory('legacy-hold-busy-');
            const path = join(
              directory.path,
              'slot-1.lock',
            );
            await using holder = await startOldHolder(path);
            expect(holder).toBeDefined();
            expect(await holdLegacyLock({
              path,
              create: true,
            })).toEqual({ kind: 'busy' });
            expect(await holdLegacyLock({
              path: join(
                directory.path,
                'missing',
              ),
              create: false,
            })).toEqual({ kind: 'absent' });
          },
        }),
        it({
          name: 'keeps the lock on a directory that exists, refusing a second hold until it is given back',
          fn: async () => {
            await using scratch = await scratchDirectory('legacy-hold-directory-');
            const hold = await holdLegacyLock({
              path: scratch.path,
              create: false,
            });
            if (hold.kind !== 'kept')
              throw new Error(`Expected the lock to be kept, found ${hold.kind}.`);
            expect(await holdLegacyLock({
              path: scratch.path,
              create: false,
            })).toEqual({ kind: 'busy' });
            await hold.lock[Symbol.asyncDispose]();
            const again = await holdLegacyLock({
              path: scratch.path,
              create: false,
            });
            expect(again.kind).toBe('kept');
            if (again.kind === 'kept')
              await again.lock[Symbol.asyncDispose]();
          },
        }),
      ],
    }),
  ],
});
