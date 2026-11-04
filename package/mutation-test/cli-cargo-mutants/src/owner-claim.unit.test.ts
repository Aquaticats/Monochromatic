import { spawnSync } from 'node:child_process';
import {
  readdir,
  readFile,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  plantClaim,
  runClaimHolder,
  scratchDirectory,
  unusedProcessId,
} from './claim-test-support.ts';
import { CargoMutantsToolingError } from './error.ts';
import { noLiveOwner } from './claim-liveness.ts';
import {
  claimPath,
  decodeClaimRecord,
  encodeClaimRecord,
  listGenerations,
  notAClaim,
} from './claim-record.ts';
import {
  type BusyOwner,
  type ClaimOutcome,
  liveOwnerOf,
  type OwnerClaim,
  takeClaim,
} from './owner-claim.ts';
import {
  currentProcessIdentity,
  readBootId,
} from './process-identity.ts';

/**
 Name of the claim series the cases use.
 */
const prefix = 'item.owner';

/**
 Path of the module the holder children import.
 */
const claimModule = join(
  import.meta.dirname,
  'owner-claim.ts',
);

/**
 How old a claim file is made so that no rule on its age could matter: a hundred hours.
 */
const ancientMs = 100 * 3_600_000;

/**
 Birth text of a process that never ran.
 */
const noSuchBirth = 'linux:1@not-a-boot';

/**
 Take the held claim of an outcome.

 @param outcome - what the attempt came to

 @returns the claim

 @throws Error when the claimant was refused

 @example
 ```ts
 held(await takeClaim({ directory: '/d', prefix: 'item.owner' }));
 ```
 */
function held(outcome: ClaimOutcome): OwnerClaim {
  if (outcome.kind !== 'held')
    throw new Error(`Expected a held claim, found the live owner ${String(outcome.owner.pid)}.`);
  return outcome.claim;
}

/**
 Take the live owner of an outcome.

 @param outcome - what the attempt came to

 @returns the owner

 @throws Error when the claimant took the claim

 @example
 ```ts
 busy(await takeClaim({ directory: '/d', prefix: 'item.owner' }));
 ```
 */
function busy(outcome: ClaimOutcome): BusyOwner {
  if (outcome.kind !== 'busy')
    throw new Error(`Expected a live owner, but generation ${String(outcome.claim.generation)} was taken.`);
  return outcome.owner;
}

await describe({
  name: '',
  children: [
    describe({
      name: takeClaim.name,
      children: [
        it({
          name: 'makes the first claim generation one, naming this process by id, birth and namespace',
          fn: async () => {
            await using directory = await scratchDirectory('claim-first-');
            const claim = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            const own = await currentProcessIdentity();
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([1]);
            expect(claim.generation).toBe(1);
            expect(await readFile(
              claim.path,
              'utf8',
            )).toBe(encodeClaimRecord({
              kind: 'owned',
              ...own,
            }));
          },
        }),
        it({
          name: 'refuses a second owner while the first is alive, naming it, creating nothing and not waiting',
          fn: async () => {
            await using directory = await scratchDirectory('claim-live-');
            const first = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            const owner = busy(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            expect(owner).toEqual({
              pid: process.pid,
              path: first.path,
            });
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([1]);
          },
        }),
        it({
          name: 'refuses the owner of another process too, a real child that holds the claim',
          fn: async () => {
            await using directory = await scratchDirectory('claim-child-');
            await using holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix,
            });
            expect(holder.line).toBe('held');
            const owner = busy(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            expect(owner.pid).toBe(holder.pid);
          },
        }),
        it({
          name: 'takes over the claim of an owner that was killed, as the next generation, and removes the older one',
          fn: async () => {
            await using directory = await scratchDirectory('claim-killed-');
            await using holder = await runClaimHolder({
              claimModule,
              directory: directory.path,
              prefix,
            });
            expect(busy(await takeClaim({
              directory: directory.path,
              prefix,
            })).pid).toBe(holder.pid);
            await holder.kill();
            const claim = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            expect(claim.generation).toBe(2);
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([2]);
          },
        }),
        it({
          name: 'takes over the claim whose process id names a live process of another birth, because the id was reused',
          fn: async () => {
            await using directory = await scratchDirectory('claim-reused-');
            const own = await currentProcessIdentity();
            await plantClaim({
              directory: directory.path,
              prefix,
              generation: 1,
              pid: process.pid,
              birth: noSuchBirth,
              ...(own.namespace === undefined ? {} : { namespace: own.namespace }),
            });
            expect(held(await takeClaim({
              directory: directory.path,
              prefix,
            })).generation).toBe(2);
          },
        }),
        it({
          name: 'takes the next generation after a release and keeps the released newest generation as a tombstone, never reusing a number',
          fn: async () => {
            await using directory = await scratchDirectory('claim-released-');
            const first = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            await first.release();
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([1]);
            const second = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            expect(second.generation).toBe(2);
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([2]);
            await second.release();
            await second.release();
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([2]);
            expect(held(await takeClaim({
              directory: directory.path,
              prefix,
            })).generation).toBe(3);
          },
        }),
        it({
          name: 'does not let a claimant that listed before a release and a newer claim reuse the number the newer claim has: the tombstone stops it',
          fn: async () => {
            await using directory = await scratchDirectory('claim-tombstone-');
            const first = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            await first.release();
            let acted = false;
            const outcome = await takeClaim({
              directory: directory.path,
              prefix,
              afterListing: async () => {
                if (acted)
                  return;
                acted = true;
                // Another command takes generation two and gives it back while this one still holds a stale listing.
                const other = held(await takeClaim({
                  directory: directory.path,
                  prefix,
                }));
                await other.release();
              },
            });
            expect(held(outcome).generation).toBe(3);
          },
        }),
        it({
          name: 'lets exactly one of several claimants that start together hold the claim over a dead owner, and tells every other it is busy',
          fn: async () => {
            await using directory = await scratchDirectory('claim-together-');
            await plantClaim({
              directory: directory.path,
              prefix,
              generation: 1,
              pid: await unusedProcessId(),
              birth: noSuchBirth,
            });
            const outcomes = await Promise.all(Array.from(
              { length: 8 },
              async function start() {
                return await takeClaim({
                  directory: directory.path,
                  prefix,
                });
              },
            ));
            expect(outcomes.filter((outcome) => outcome.kind === 'held')).toHaveLength(1);
            expect(outcomes.filter((outcome) => outcome.kind === 'busy')).toHaveLength(7);
          },
        }),
        it({
          name: 'reads only the numbered files of its own series as generations',
          fn: async () => {
            await using directory = await scratchDirectory('claim-names-');
            await Promise.all(['item.owner', 'item.owner.', 'item.owner.x', 'item.owner.1.x', 'item.owner.-1', 'other.owner.9', 'xitem.owner.5'].map(async function plantUnrelated(name) {
              await writeFile(
                join(
                  directory.path,
                  name,
                ),
                'unrelated',
              );
            }));
            expect(held(await takeClaim({
              directory: directory.path,
              prefix,
            })).generation).toBe(1);
          },
        }),
        it({
          name: 'stops with the file named, changing nothing, when the newest generation is not a claim',
          fn: async () => {
            await using directory = await scratchDirectory('claim-garbage-');
            const path = claimPath({
              directory: directory.path,
              prefix,
              generation: 1,
            });
            await writeFile(
              path,
              'somebody else\'s text',
            );
            let caught: unknown;
            try {
              await takeClaim({
                directory: directory.path,
                prefix,
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(caught)).toContain('is not a claim file');
            expect(await readFile(
              path,
              'utf8',
            )).toBe('somebody else\'s text');
          },
        }),
        it({
          name: 'stops with the file named, never hanging, when the newest generation is a named pipe with no writer',
          fn: async () => {
            await using directory = await scratchDirectory('claim-fifo-');
            const path = claimPath({
              directory: directory.path,
              prefix,
              generation: 1,
            });
            const made = spawnSync(
              'mkfifo',
              [path],
            );
            expect(made.status).toBe(0);
            let caught: unknown;
            try {
              await takeClaim({
                directory: directory.path,
                prefix,
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(caught)).toContain('is not a claim file');
          },
          timeout: 15_000,
        }),
        it({
          name: 'retries instead of failing when another claimant removed the generation it published before it could withdraw it',
          fn: async () => {
            await using directory = await scratchDirectory('claim-withdraw-gone-');
            const own = await currentProcessIdentity();
            const newer = 3;
            const published = claimPath({
              directory: directory.path,
              prefix,
              generation: 1,
            });
            const outcome = await takeClaim({
              directory: directory.path,
              prefix,
              afterPublishing: async () => {
                // A later claimant removes the stale lower generation (older than its own) and holds a newer one.
                await rm(published);
                await plantClaim({
                  directory: directory.path,
                  prefix,
                  generation: newer,
                  pid: own.pid,
                  birth: own.birth,
                  ...(own.namespace === undefined ? {} : { namespace: own.namespace }),
                });
              },
            });
            expect(busy(outcome).pid).toBe(own.pid);
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([newer]);
          },
        }),
        it({
          name: 'withdraws the generation it published and reports the live owner when a newer generation appeared after its listing',
          fn: async () => {
            await using directory = await scratchDirectory('claim-stale-listing-');
            const own = await currentProcessIdentity();
            const newer = 3;
            let acted = false;
            const outcome = await takeClaim({
              directory: directory.path,
              prefix,
              afterListing: async () => {
                if (acted)
                  return;
                acted = true;
                await plantClaim({
                  directory: directory.path,
                  prefix,
                  generation: newer,
                  pid: own.pid,
                  birth: own.birth,
                  ...(own.namespace === undefined ? {} : { namespace: own.namespace }),
                });
              },
            });
            expect(busy(outcome).path).toBe(claimPath({
              directory: directory.path,
              prefix,
              generation: newer,
            }));
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([newer]);
            expect((await readdir(directory.path)).toSorted()).toEqual([`${prefix}.${String(newer)}`]);
          },
        }),
      ],
    }),
    describe({
      name: 'an owner in another PID namespace',
      children: [
        it({
          name: 'keeps its claim however old the claim file is, because only evidence about the process can end it',
          fn: async () => {
            await using directory = await scratchDirectory('claim-foreign-old-');
            const path = await plantClaim({
              directory: directory.path,
              prefix,
              generation: 1,
              pid: await unusedProcessId(),
              birth: `linux:1@${await readBootId()}`,
              namespace: 'pid:[not-this-namespace]',
            });
            const longAgo = new Date(Date.now() - ancientMs);
            await utimes(
              path,
              longAgo,
              longAgo,
            );
            expect(busy(await takeClaim({
              directory: directory.path,
              prefix,
            })).path).toBe(path);
          },
        }),
        it({
          name: 'is replaced by the next generation when it began in an earlier boot, though its claim file is young',
          fn: async () => {
            await using directory = await scratchDirectory('claim-foreign-boot-');
            await plantClaim({
              directory: directory.path,
              prefix,
              generation: 1,
              pid: await unusedProcessId(),
              birth: 'linux:1@an-earlier-boot',
              namespace: 'pid:[not-this-namespace]',
            });
            expect(held(await takeClaim({
              directory: directory.path,
              prefix,
            })).generation).toBe(2);
          },
        }),
        it({
          name: 'keeps its claim when its record names no namespace and no process here has its id',
          fn: async () => {
            await using directory = await scratchDirectory('claim-nameless-');
            const path = await plantClaim({
              directory: directory.path,
              prefix,
              generation: 1,
              pid: await unusedProcessId(),
              birth: `linux:1@${await readBootId()}`,
            });
            expect(busy(await takeClaim({
              directory: directory.path,
              prefix,
            })).path).toBe(path);
          },
        }),
      ],
    }),
    describe({
      name: liveOwnerOf.name,
      children: [
        it({
          name: 'names the live owner and creates nothing, and finds none for a released, dead or absent claim',
          fn: async () => {
            await using directory = await scratchDirectory('claim-read-');
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix,
            })).toBe(noLiveOwner);
            const claim = held(await takeClaim({
              directory: directory.path,
              prefix,
            }));
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix,
            })).toEqual({
              pid: process.pid,
              path: claim.path,
            });
            await claim.release();
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix,
            })).toBe(noLiveOwner);
            await plantClaim({
              directory: directory.path,
              prefix,
              generation: 2,
              pid: await unusedProcessId(),
              birth: noSuchBirth,
            });
            expect(await liveOwnerOf({
              directory: directory.path,
              prefix,
            })).toBe(noLiveOwner);
            expect(await listGenerations({
              directory: directory.path,
              prefix,
            })).toEqual([1, 2]);
          },
        }),
        it({
          name: 'finds no owner in a directory that does not exist',
          fn: async () => {
            await using directory = await scratchDirectory('claim-read-missing-');
            expect(await liveOwnerOf({
              directory: join(
                directory.path,
                'missing',
              ),
              prefix,
            })).toBe(noLiveOwner);
          },
        }),
      ],
    }),
    describe({
      name: decodeClaimRecord.name,
      children: [
        it({
          name: 'reads back every kind of record it writes',
          fn: async () => {
            for (const record of [
              { kind: 'released' as const },
              {
                kind: 'owned' as const,
                pid: 12,
                birth: 'linux:42@b',
              },
              {
                kind: 'owned' as const,
                pid: 13,
                birth: 'linux:43@b',
                namespace: 'pid:[1]',
              },
            ])
              expect(
                decodeClaimRecord(encodeClaimRecord(record)),
              ).toEqual(record);
          },
        }),
        ...[
          ['empty text', ''],
          ['another version', 'cargo-mutants owner-claim v2\nreleased\n'],
          ['a process id of zero', 'cargo-mutants owner-claim v1\npid=0\nbirth=x\nnamespace=-\n'],
          ['an empty birth', 'cargo-mutants owner-claim v1\npid=1\nbirth=\nnamespace=-\n'],
          ['a missing namespace line', 'cargo-mutants owner-claim v1\npid=1\nbirth=x\n'],
        ].map(function rejectedText([name, text]) {
          return it({
            name: `reads ${String(name)} as no claim`,
            fn: async () => {
              expect(
                decodeClaimRecord(String(text)),
              ).toBe(notAClaim);
            },
          });
        }),
      ],
    }),
  ],
});
