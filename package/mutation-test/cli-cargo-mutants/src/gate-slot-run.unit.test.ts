import { spawnSync } from 'node:child_process';
import {
  mkdir,
  readdir,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  plantClaim,
  runClaimHolder,
  scratchDirectory,
  startOldHolder,
  unusedProcessId,
} from './claim-test-support.ts';
import {
  noLiveOwner,
} from './claim-liveness.ts';
import { listGenerations } from './claim-record.ts';
import { watchDirectoryChanges } from './directory-watch.ts';
import {
  type GateSlotOptions,
  slotClaimPrefix,
  slotLockPath,
} from './gate-slot.ts';
import {
  runInSlot,
  type SlotsFull,
  type SlotTaken,
  takeAnySlot as takeAnySlotUntracked,
} from './gate-slot-run.ts';
import {
  flockProgram,
  noticeGraceMs,
  slotConflictExitCode,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import { liveOwnerOf } from './owner-claim.ts';
import { readBootId } from './process-identity.ts';

/**
 Try the kernel lock on a path as a waiting gate of the earlier release does on each of its polls (it creates a missing file
 and takes the lock without waiting), and give it straight back.

 @param path - the lock file

 @returns the status of `flock`: zero when the lock was free, the conflict status when someone holds it

 @example
 ```ts
 legacyPollStatus('/l/slot-1.lock');
 ```
 */
function legacyPollStatus(path: string): number {
  /**
   How `flock` ended.
   */
  const { status } = spawnSync(
    flockProgram,
    [
      '--nonblock',
      '--exclusive',
      `--conflict-exit-code=${String(slotConflictExitCode)}`,
      path,
      'true',
    ],
  );
  if (status === null)
    throw new Error('flock was ended by a signal.');
  return status;
}

/**
 The kernel locks of the slots the cases took, closed when the file ends: a lock is an open file, and one left to the garbage
 collector is an error.
 */
const keptLocks: AsyncDisposable[] = [];

/**
 Take a slot as the gate does, and keep its kernel lock for the end of the file.

 @param parameters - the same as {@link takeAnySlotUntracked}

 @returns what it returns

 @example
 ```ts
 await takeAnySlot({ options });
 ```
 */
async function takeAnySlot(parameters: Parameters<typeof takeAnySlotUntracked>[0]): Promise<SlotTaken | SlotsFull> {
  /**
   What the take came to.
   */
  const attempt = await takeAnySlotUntracked(parameters);
  if (attempt.kind === 'taken')
    keptLocks.push(attempt.legacyLock);
  return attempt;
}

/**
 Path of the module the holder children import.
 */
const claimModule = join(
  import.meta.dirname,
  'owner-claim.ts',
);

/**
 Slots the cases use: two are enough to show every behaviour, and keep the processes the cases start few.
 */
const slotCount = 2;

/**
 Longest a case waits for a gate that must wake, in milliseconds.
 */
const wakeBudgetMs = 10_000;

/**
 Milliseconds that count as "did not happen yet" when a case checks that a gate is still waiting.
 */
const stillWaitingMs = 400;

/**
 Settings for a case: two slots in a disposable directory, a fallback re-check so long that only a wake can explain a quick
 start, and the given command.

 @param overrides - the settings a case changes

 @returns the settings

 @example
 ```ts
 gateOptions({ lockDirectory: '/d' });
 ```
 */
function gateOptions(overrides: {
  readonly lockDirectory: string;
  readonly slotCount?: number;
  readonly pollIntervalMs?: number;
  readonly noticeIntervalMs?: number;
}): GateSlotOptions {
  return {
    slotCount,
    pollIntervalMs: 600_000,
    noticeIntervalMs: 600_000,
    command: ['cmd'],
    ...overrides,
  };
}

/**
 Make a gate directory for a case.

 @param name - what the case is, the start of the directory's name

 @returns the directory, removed when disposed

 @example
 ```ts
 await using directory = await gateDirectory('gate-x-');
 ```
 */
async function gateDirectory(name: string): Promise<AsyncDisposable & { readonly path: string }> {
  return await scratchDirectory(name);
}

await describe({
  name: '',
  children: [
    describe({
      name: takeAnySlot.name,
      children: [
        it({
          name: 'takes the lowest free slot at once, and the next one while the first is held',
          fn: async () => {
            await using directory = await gateDirectory('gate-take-');
            const options = gateOptions({ lockDirectory: directory.path });
            const first = await takeAnySlot({ options });
            const second = await takeAnySlot({ options });
            expect([first.kind, second.kind]).toEqual(['taken', 'taken']);
            expect([first, second].map((taken) => (taken.kind === 'taken' ? taken.slot : 0))).toEqual([1, 2]);
          },
        }),
        it({
          name: 'reports every holder, and takes nothing, only when all slots are held by live processes',
          fn: async () => {
            await using directory = await gateDirectory('gate-full-');
            const options = gateOptions({ lockDirectory: directory.path });
            await takeAnySlot({ options });
            await using holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix: slotClaimPrefix(2),
            });
            const full = await takeAnySlot({ options });
            expect(full).toEqual({
              kind: 'full',
              holders: [
                {
                  kind: 'process',
                  slot: 1,
                  pid: process.pid,
                },
                {
                  kind: 'process',
                  slot: 2,
                  pid: holder.pid,
                },
              ],
            });
            expect(await listGenerations({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toEqual([1]);
          },
        }),
        it({
          name: 'frees the slot of a holder that was killed, as the next generation, without a person',
          fn: async () => {
            await using directory = await gateDirectory('gate-dead-');
            const options = gateOptions({ lockDirectory: directory.path });
            const holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            });
            // The second slot is this process's, so the first is the only one that can free up.
            await takeAnySlot({ options });
            const before = await takeAnySlot({ options });
            expect(before.kind).toBe('full');
            await holder.kill();
            const after = await takeAnySlot({ options });
            expect(after.kind).toBe('taken');
            expect(await listGenerations({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toEqual([2]);
          },
        }),
        it({
          name: 'reuses a released slot as the next generation of its series, never the same number',
          fn: async () => {
            await using directory = await gateDirectory('gate-released-');
            const options = gateOptions({
              lockDirectory: directory.path,
              slotCount: 1,
            });
            const first = await takeAnySlot({ options });
            if (first.kind !== 'taken')
              throw new Error('The first take was refused.');
            await first.legacyLock[Symbol.asyncDispose]();
            await first.claim.release();
            const second = await takeAnySlot({ options });
            if (second.kind !== 'taken')
              throw new Error('The second take was refused.');
            expect([first.claim.generation, second.claim.generation]).toEqual([1, 2]);
          },
        }),
        it({
          name: 'keeps a holder in another PID namespace in its slot however old its claim is, and frees the slot once an earlier boot proves it gone',
          fn: async () => {
            await using directory = await gateDirectory('gate-foreign-');
            const options = gateOptions({
              lockDirectory: directory.path,
              slotCount: 1,
            });
            const path = await plantClaim({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
              generation: 1,
              pid: await unusedProcessId(),
              birth: `linux:1@${await readBootId()}`,
              namespace: 'pid:[not-this-namespace]',
            });
            expect((await takeAnySlot({ options })).kind).toBe('full');
            const longAgo = new Date(Date.now() - (100 * 3_600_000));
            await utimes(
              path,
              longAgo,
              longAgo,
            );
            expect((await takeAnySlot({ options })).kind).toBe('full');
            await plantClaim({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
              generation: 1,
              pid: await unusedProcessId(),
              birth: 'linux:1@an-earlier-boot',
              namespace: 'pid:[not-this-namespace]',
            });
            expect((await takeAnySlot({ options })).kind).toBe('taken');
          },
        }),
        it({
          name: 'respects a gate of the earlier release that holds the kernel lock on a slot file, then takes the slot once it ended and keeps the file',
          fn: async () => {
            await using directory = await gateDirectory('gate-legacy-');
            const options = gateOptions({ lockDirectory: directory.path });
            const lockFile = slotLockPath({
              lockDirectory: directory.path,
              slot: 1,
            });
            const holder = await startOldHolder(lockFile);
            const taken = await takeAnySlot({ options });
            expect(taken.kind === 'taken' ? taken.slot : 0).toBe(2);
            const full = await takeAnySlot({ options });
            expect(full).toEqual({
              kind: 'full',
              holders: [
                {
                  kind: 'earlier-release',
                  slot: 1,
                },
                {
                  kind: 'process',
                  slot: 2,
                  pid: process.pid,
                },
              ],
            });
            await holder.kill();
            const freed = await takeAnySlot({ options });
            expect(freed.kind === 'taken' ? freed.slot : 0).toBe(1);
            expect((await readdir(directory.path)).includes('slot-1.lock')).toBe(true);
          },
        }),
        ...[
          [
            'a directory',
            async (path: string): Promise<void> => {
              await mkdir(path);
            },
          ],
          [
            'a named pipe',
            async (path: string): Promise<void> => {
              if (spawnSync(
                'mkfifo',
                [path],
              ).status !== 0)
                throw new Error('mkfifo failed.');
            },
          ],
        ].map(([kind, plant]) => it({
          name: `stops with an error naming the path, and holds no claim, where ${String(kind)} stands in place of a slot file, which no gate of the earlier release makes and so no such gate can hold`,
          fn: async () => {
            await using directory = await gateDirectory('gate-not-a-file-');
            const options = gateOptions({ lockDirectory: directory.path });
            const lockFile = slotLockPath({
              lockDirectory: directory.path,
              slot: 1,
            });
            await (plant as (path: string) => Promise<void>)(lockFile);
            let caught: unknown;
            try {
              await takeAnySlot({ options });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CargoMutantsToolingError);
            expect((caught as Error).message).toContain(lockFile);
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toBe(noLiveOwner);
          },
        })),
      ],
    }),
    describe({
      name: runInSlot.name,
      // One case at a time: the cases measure waits and notices, and flushes of their claim files that pile up on a loaded
      // host (the files are synchronized, as the wrapper's are) would otherwise delay each other's loops.
      concurrency: 1,
      children: [
        it({
          name: 'runs the command in the first free slot at once, holding the claim while it runs and giving it back after, with the command\'s status',
          fn: async () => {
            await using directory = await gateDirectory('gate-run-');
            const seen: unknown[] = [];
            const notices: string[] = [];
            const status = await runInSlot({
              options: gateOptions({ lockDirectory: directory.path }),
              runCommand: async ({ slot }) => {
                seen.push([slot, await liveOwnerOf({
                  directory: directory.path,
                  prefix: slotClaimPrefix(slot),
                })]);
                return 7;
              },
              write: (text) => {
                notices.push(text);
              },
            });
            expect(status).toBe(7);
            expect(seen).toEqual([[1, {
              pid: process.pid,
              path: join(
                directory.path,
                'slot-1.owner.1',
              ),
            }]]);
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toBe(noLiveOwner);
            expect(notices).toEqual([]);
          },
        }),
        it({
          name: 'gives the slot back when the command fails',
          fn: async () => {
            await using directory = await gateDirectory('gate-throws-');
            let caught: unknown;
            try {
              await runInSlot({
                options: gateOptions({ lockDirectory: directory.path }),
                runCommand: async () => {
                  throw new Error('the command could not start');
                },
                write: () => {},
              });
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught)).toContain('the command could not start');
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toBe(noLiveOwner);
          },
        }),
        it({
          name: 'waits only while every slot is held, says what it waits for and who holds it, and wakes on the directory change of a release, not on the next notice or the fallback re-check, whatever time the release takes',
          fn: async () => {
            await using directory = await gateDirectory('gate-wake-');
            const options = gateOptions({
              lockDirectory: directory.path,
              noticeIntervalMs: 600_000,
            });
            const one = await takeAnySlot({ options });
            const two = await takeAnySlot({ options });
            if ((one.kind !== 'taken') || (two.kind !== 'taken'))
              throw new Error('The holders could not be set up.');
            const started: number[] = [];
            const notices: string[] = [];
            /**
             Each watch the gate opened after its notice, with whether it had fired by the time the gate closed it. Such a look can end
             only by a directory change, because the timers that could end it are far off.
             */
            const lateWatches: { fired: boolean; closed: boolean }[] = [];
            const waiting = runInSlot({
              options,
              runCommand: async ({ slot }) => {
                started.push(slot);
                return 0;
              },
              write: (text) => {
                notices.push(text);
              },
              watchDirectory: (watched) => {
                const watch = watchDirectoryChanges(watched);
                if (notices.length === 0)
                  return watch;
                const record = { fired: false, closed: false };
                lateWatches.push(record);
                void (async function noteFiring(): Promise<void> {
                  await watch.changed;
                  record.fired = !record.closed;
                })();
                const close = (): void => {
                  record.closed = true;
                  watch.close();
                };
                return {
                  changed: watch.changed,
                  close,
                  [Symbol.dispose]: close,
                };
              },
            });
            // Past the grace before the first notice, so the next timer is the far-off notice interval, not a short wait.
            await sleep(noticeGraceMs + stillWaitingMs);
            expect(started).toEqual([]);
            expect(notices).toHaveLength(1);
            expect(notices[0]).toContain('waiting for a free slot');
            expect(notices[0]).toContain(String(process.pid));
            expect(notices[0]).toContain('slot 1');
            expect(notices[0]).toContain('slot 2');
            await two.legacyLock[Symbol.asyncDispose]();
            await two.claim.release();
            await waiting;
            expect(started).toEqual([2]);
            // How long the release and the wake took depends on the host (each spawns probes and flushes the directory), so no
            // time is asserted: every look after the notice ended by a change, and only the last, which took the slot, was not
            // woken by one that was still to come.
            expect(lateWatches.length).toBeGreaterThan(0);
            expect(lateWatches.slice(0, -1)
              .every((record) => record.fired)).toBe(true);
          },
        }),
        it({
          name: 'writes a notice at a steady pace while it waits, not one per wake and not a tight poll',
          fn: async () => {
            await using directory = await gateDirectory('gate-notices-');
            const options = gateOptions({
              lockDirectory: directory.path,
              noticeIntervalMs: 100,
              slotCount: 1,
            });
            const held = await takeAnySlot({ options });
            if (held.kind !== 'taken')
              throw new Error('The holder could not be set up.');
            const notices: string[] = [];
            let watches = 0;
            const waiting = runInSlot({
              options,
              runCommand: async () => 0,
              write: (text) => {
                notices.push(text);
              },
              watchDirectory: (watched) => {
                watches += 1;
                return watchDirectoryChanges(watched);
              },
            });
            const waitStarted = Date.now();
            await sleep(1_050);
            await held.legacyLock[Symbol.asyncDispose]();
            await held.claim.release();
            await waiting;
            const waitedMs = Date.now() - waitStarted;
            // The first notice and at least one repeat; how many more depends on how long a loaded host took to run the
            // release, so the upper bound is taken from the measured wait: one per interval, a few more for the boundaries.
            expect(notices.length).toBeGreaterThanOrEqual(2);
            expect(notices.length).toBeLessThanOrEqual(Math.floor(waitedMs / 100) + 3);
            expect(notices[0]).toContain('waiting for a free slot');
            expect(notices.slice(1).every((notice) => notice.includes('still waiting'))).toBe(true);
            // One look per notice, plus a few at the release, which writes a private file, renames it and flushes the directory
            // (each an event): the waits end at the next notice or a change, so the count stays near the notice count.
            expect(watches).toBeLessThanOrEqual(notices.length + 8);
          },
        }),
        it({
          name: 'keeps at most one directory watch open however many times it looks again, so a long wait does not run out of watches',
          fn: async () => {
            await using directory = await gateDirectory('gate-watches-');
            const options = gateOptions({
              lockDirectory: directory.path,
              noticeIntervalMs: 50,
              pollIntervalMs: 50,
              slotCount: 1,
            });
            const held = await takeAnySlot({ options });
            if (held.kind !== 'taken')
              throw new Error('The holder could not be set up.');
            let open = 0;
            let mostOpen = 0;
            let looks = 0;
            const waiting = runInSlot({
              options,
              runCommand: async () => 0,
              write: () => {},
              watchDirectory: (watched) => {
                const watch = watchDirectoryChanges(watched);
                looks += 1;
                open += 1;
                mostOpen = Math.max(
                  mostOpen,
                  open,
                );
                return {
                  changed: watch.changed,
                  close: () => {
                    open -= 1;
                    watch.close();
                  },
                  [Symbol.dispose]: () => {
                    open -= 1;
                    watch.close();
                  },
                };
              },
            });
            await sleep(stillWaitingMs);
            await held.legacyLock[Symbol.asyncDispose]();
            await held.claim.release();
            await waiting;
            expect(looks).toBeGreaterThan(3);
            expect(mostOpen).toBe(1);
            expect(open).toBe(0);
          },
        }),
        it({
          name: 'wakes within the fallback re-check when the holder was killed, which makes no event',
          fn: async () => {
            await using directory = await gateDirectory('gate-killed-wake-');
            const options = gateOptions({
              lockDirectory: directory.path,
              slotCount: 1,
              pollIntervalMs: 200,
            });
            const holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            });
            const started: number[] = [];
            const waiting = runInSlot({
              options,
              runCommand: async ({ slot }) => {
                started.push(slot);
                return 0;
              },
              write: () => {},
            });
            await sleep(stillWaitingMs);
            expect(started).toEqual([]);
            await holder.kill();
            await waiting;
            expect(started).toEqual([1]);
            expect(await listGenerations({
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            })).toEqual([2]);
          },
        }),
        it({
          name: 'never lets more commands run at once than there are slots, and runs every one',
          fn: async () => {
            await using directory = await gateDirectory('gate-cap-');
            const options = gateOptions({
              lockDirectory: directory.path,
              pollIntervalMs: 100,
            });
            let running = 0;
            let peak = 0;
            const finished: number[] = [];
            const statuses = await Promise.all([1, 2, 3, 4, 5].map(async function gate(index) {
              return await runInSlot({
                options,
                runCommand: async () => {
                  running += 1;
                  peak = Math.max(
                    peak,
                    running,
                  );
                  await sleep(150);
                  running -= 1;
                  finished.push(index);
                  return index;
                },
                write: () => {},
              });
            }));
            expect(statuses.toSorted((left, right) => left - right)).toEqual([1, 2, 3, 4, 5]);
            expect(peak).toBeLessThanOrEqual(slotCount);
            expect(finished).toHaveLength(5);
          },
        }),
        it({
          name: 'keeps the kernel lock of its slot while the command runs, so a waiting gate of the earlier release that polls the slot is refused, and gives it back after',
          fn: async () => {
            await using directory = await gateDirectory('gate-legacy-waiter-');
            const lockFile = slotLockPath({
              lockDirectory: directory.path,
              slot: 1,
            });
            const seen: number[] = [];
            await runInSlot({
              options: gateOptions({ lockDirectory: directory.path }),
              runCommand: async () => {
                seen.push(legacyPollStatus(lockFile));
                return 0;
              },
              write: () => {},
            });
            expect(seen).toEqual([slotConflictExitCode]);
            expect(legacyPollStatus(lockFile)).toBe(0);
          },
        }),
        it({
          name: 'names a peer that holds a slot as that process, though the peer holds the kernel lock of the slot too',
          fn: async () => {
            await using directory = await gateDirectory('gate-legacy-peer-');
            const options = gateOptions({
              lockDirectory: directory.path,
              slotCount: 1,
            });
            await using holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix: slotClaimPrefix(1),
            });
            // The peer holds the kernel lock of the slot file too, as a gate of this release does while its command runs.
            await using lockHolder = await startOldHolder(slotLockPath({
              lockDirectory: directory.path,
              slot: 1,
            }));
            expect(await takeAnySlot({ options })).toEqual({
              kind: 'full',
              holders: [
                {
                  kind: 'process',
                  slot: 1,
                  pid: holder.pid,
                },
              ],
            });
          },
        }),
        it({
          name: 'does not wait for a slot file of an earlier release that nothing holds: it keeps the file and runs',
          fn: async () => {
            await using directory = await gateDirectory('gate-legacy-free-');
            await mkdir(directory.path, { recursive: true });
            await Promise.all([1, 2].map(async function leaveLockFile(slot) {
              await writeFile(
                slotLockPath({
                  lockDirectory: directory.path,
                  slot,
                }),
                '',
              );
            }));
            const status = await runInSlot({
              options: gateOptions({ lockDirectory: directory.path }),
              runCommand: async () => 0,
              write: () => {},
            });
            expect(status).toBe(0);
            // The files stay: a gate of the earlier release that polls a slot must find the file it locks, and removing it would
            // let a probe delete a file such a gate has just created and locked.
            expect((await readdir(directory.path)).filter((name) => name.endsWith('.lock'))).toEqual(['slot-1.lock', 'slot-2.lock']);
          },
        }),
      ],
    }),
  ],
});

await Promise.all(keptLocks.map(async function closeLock(lock) {
  await lock[Symbol.asyncDispose]();
}));
