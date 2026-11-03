import { spawnSync } from 'node:child_process';
import {
  readdir,
  readFile,
  mkdir,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  acquireCampaignLock,
  campaignClaimPrefix,
} from './campaign-lock.ts';
import {
  runClaimHolder,
  scratchDirectory,
  startOldHolder,
} from './claim-test-support.ts';
import { noLiveOwner } from './claim-liveness.ts';
import {
  flockProgram,
  slotConflictExitCode,
} from './constants.ts';
import { decodeClaimRecord } from './claim-record.ts';
import { CargoMutantsToolingError } from './error.ts';
import { liveOwnerOf } from './owner-claim.ts';

/**
 Path of the module the holder children import.
 */
const claimModule = join(
  import.meta.dirname,
  'owner-claim.ts',
);

/**
 The outcome of trying to take the lock.
 */
type Attempt = {
  /**
   The lock, as the only element, or none when it was refused.
   */
  readonly lock: readonly AsyncDisposable[];
  /**
   Why it was refused, empty when it was not.
   */
  readonly reason: string;
};

/**
 Try to take the lock.

 @param directory - evidence directory

 @returns the lock or the reason it was refused

 @example
 ```ts
 await attempt('/e/native-mutation-abc');
 ```
 */
async function attempt(directory: string): Promise<Attempt> {
  try {
    return {
      lock: [await acquireCampaignLock(directory)],
      reason: '',
    };
  }
  catch (error) {
    return {
      lock: [],
      reason: error instanceof CargoMutantsToolingError ? error.message : 'unexpected',
    };
  }
}

/**
 List the claim files of the campaign in a directory.

 @param directory - evidence directory

 @returns their names, sorted

 @example
 ```ts
 await claimFiles('/e/native-mutation-abc');
 ```
 */
async function claimFiles(directory: string): Promise<readonly string[]> {
  return (await readdir(directory))
    .filter(function isClaim(name) {
      return name.startsWith(`${campaignClaimPrefix}.`);
    })
    .toSorted();
}

await describe({
  name: acquireCampaignLock.name,
  children: [
    it({
      name: 'refuses a second run while the first holds the claim, naming the owner and its claim file, and allows one after the first released it',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-');
        {
          await using held = await acquireCampaignLock(directory.path);
          expect(held).toBeDefined();
          const second = await attempt(directory.path);
          expect(second.lock).toEqual([]);
          expect(second.reason).toContain('Another run of this campaign is alive');
          expect(second.reason).toContain(`process ${String(process.pid)}`);
          expect(second.reason).toContain(`${campaignClaimPrefix}.1`);
        }
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
        expect(await claimFiles(directory.path)).toEqual([`${campaignClaimPrefix}.2`]);
      },
    }),
    it({
      name: 'refuses at once and never waits when another process holds the claim',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-child-');
        await using holder = await runClaimHolder({
          claimModule,
          directory: directory.path,
          prefix: campaignClaimPrefix,
        });
        expect(holder.line).toBe('held');
        const started = Date.now();
        const second = await attempt(directory.path);
        expect(second.lock).toEqual([]);
        expect(second.reason).toContain(`process ${String(holder.pid)}`);
        expect(Date.now() - started).toBeLessThan(5_000);
      },
    }),
    it({
      name: 'takes over the claim of a run that was killed, as the next generation, so a killed run never leaves a lock to clean up',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-killed-');
        await using holder = await runClaimHolder({
          claimModule,
          directory: directory.path,
          prefix: campaignClaimPrefix,
        });
        expect((await attempt(directory.path)).lock).toEqual([]);
        await holder.kill();
        const taken = await attempt(directory.path);
        expect(taken.lock).toHaveLength(1);
        expect(await claimFiles(directory.path)).toEqual([`${campaignClaimPrefix}.2`]);
        await taken.lock[0]?.[Symbol.asyncDispose]();
      },
    }),
    it({
      name: 'lets exactly one of several runs that start together take the claim, and tells every other run who holds it',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-together-');
        const settled = await Promise.all(Array.from(
          { length: 8 },
          function start() {
            return attempt(directory.path);
          },
        ));
        const held = settled.flatMap(function lockOf(result) {
          return result.lock;
        });
        expect(held).toHaveLength(1);
        expect(settled.filter((result) => result.lock.length === 0).every((result) => result.reason.includes('Another run'))).toBe(true);
        await held[0]?.[Symbol.asyncDispose]();
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
      },
    }),
    it({
      name: 'leaves the claim file behind as a released tombstone when the run ends, and writes nothing else',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-tombstone-');
        {
          await using held = await acquireCampaignLock(directory.path);
          expect(held).toBeDefined();
        }
        expect(await readdir(directory.path)).toEqual([`${campaignClaimPrefix}.1`]);
        expect(
          decodeClaimRecord(await readFile(
            join(
              directory.path,
              `${campaignClaimPrefix}.1`,
            ),
            'utf8',
          )),
        ).toEqual({ kind: 'released' });
      },
    }),
    it({
      name: 'respects a campaign of the earlier release that still holds the kernel lock on the directory, then proceeds once it ended',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-legacy-');
        const evidence = join(
          directory.path,
          'evidence',
        );
        await mkdir(evidence);
        const holder = await startOldHolder(evidence);
        const refused = await attempt(evidence);
        expect(refused.lock).toEqual([]);
        expect(refused.reason).toContain('earlier release');
        expect(refused.reason).toContain(evidence);
        await holder.kill();
        const taken = await attempt(evidence);
        expect(taken.lock).toHaveLength(1);
        await taken.lock[0]?.[Symbol.asyncDispose]();
      },
    }),
    it({
      name: 'holds the kernel lock on the directory while the lock is held, so a campaign of the earlier release that starts then is refused, and gives it back after',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-legacy-start-');
        const evidence = join(
          directory.path,
          'evidence',
        );
        await mkdir(evidence);
        /**
         What a campaign of the earlier release did to start: the kernel lock on the directory, without waiting.
         */
        function legacyStart(): number {
          /**
           How `flock` ended.
           */
          const { status } = spawnSync(
            flockProgram,
            [
              '--nonblock',
              '--exclusive',
              `--conflict-exit-code=${String(slotConflictExitCode)}`,
              evidence,
              'true',
            ],
          );
          if (status === null)
            throw new Error('flock was ended by a signal.');
          return status;
        }
        const taken = await attempt(evidence);
        expect(taken.lock).toHaveLength(1);
        expect(legacyStart()).toBe(slotConflictExitCode);
        await taken.lock[0]?.[Symbol.asyncDispose]();
        expect(legacyStart()).toBe(0);
      },
    }),
    it({
      name: 'does not leave a claim behind that blocks the next run after refusing for an earlier release',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-legacy-given-back-');
        const holder = await startOldHolder(directory.path);
        expect((await attempt(directory.path)).lock).toEqual([]);
        expect(await liveOwnerOf({
          directory: directory.path,
          prefix: campaignClaimPrefix,
        })).toBe(noLiveOwner);
        await holder.kill();
      },
    }),
    it({
      name: 'does not take a claim on a directory that is missing, and does not call that a second run',
      fn: async () => {
        await using directory = await scratchDirectory('campaign-claim-missing-');
        const outcome = await attempt(join(
          directory.path,
          'missing',
        ));
        expect(outcome.lock).toEqual([]);
        expect(outcome.reason.includes('Another run')).toBe(false);
      },
    }),
  ],
});
