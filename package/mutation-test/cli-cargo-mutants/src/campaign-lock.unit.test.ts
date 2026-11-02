import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  mkdir,
  mkdtempDisposable,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { acquireCampaignLock } from './campaign-lock.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Longest a test waits for the kernel to give back the lock of a process that was killed.
 */
const releaseWaitMilliseconds = 10_000;

/**
 Make a disposable directory under the agent scratch area, never the system temporary directory.

 @returns the directory, removed when disposed

 @example
 ```ts
 await using directory = await scratch();
 ```
 */
async function scratch(): Promise<AsyncDisposable & { readonly path: string }> {
  /**
   Parent of every scratch directory.
   */
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  return mkdtempDisposable(join(
    parent,
    'cargo-mutants-lock-',
  ));
}

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

await describe({
  name: acquireCampaignLock.name,
  children: [
    it({
      name: 'refuses a second run while the first holds the lock, and allows one after the first released it',
      fn: async () => {
        await using directory = await scratch();
        {
          await using held = await acquireCampaignLock(directory.path);
          expect(held).toBeDefined();
          await expect(acquireCampaignLock(directory.path)).rejects.toThrow(CargoMutantsToolingError);
        }
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
      },
    }),
    it({
      name: 'lets exactly one of several runs that start together take the lock, over a leftover lock file of the old runner, and leaves no file of its own',
      fn: async () => {
        await using directory = await scratch();
        // The old runner left a lock file that names a process that is gone; it means nothing now, and no contender has to
        // judge it, which is the step two contenders used to race on.
        await writeFile(join(directory.path, 'campaign.lock'), '999999 some-boot:1\n');
        const settled = await Promise.all(Array.from(
          { length: 8 },
          function start() {
            return attempt(directory.path);
          },
        ));
        const held = settled.flatMap(function lockOf(result) {
          return result.lock;
        });
        expect(held.length).toBe(1);
        expect(settled.filter((result) => result.lock.length === 0).every((result) => result.reason.includes('Another run'))).toBe(true);
        expect((await readdir(directory.path)).toSorted()).toEqual(['campaign.lock']);
        await held[0]?.[Symbol.asyncDispose]();
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
      },
    }),
    it({
      name: 'is given back by the kernel when the process that holds it is killed, so a killed run never leaves a lock to take over',
      fn: async () => {
        await using directory = await scratch();
        const lockModule = join(import.meta.dirname, 'campaign-lock.ts');
        /**
         A process that takes the lock and waits.
         */
        const owner = spawn(
          process.execPath,
          [
            '--input-type=module',
            '--eval',
            `import { acquireCampaignLock } from ${JSON.stringify(lockModule)}; await acquireCampaignLock(${JSON.stringify(directory.path)}); console.log('owner holds'); setTimeout(() => process.exit(0), 30000);`,
          ],
          { stdio: ['ignore', 'pipe', 'inherit'] },
        );
        await once(owner.stdout, 'data');
        expect((await attempt(directory.path)).lock).toEqual([]);
        owner.kill('SIGKILL');
        await once(owner, 'close');
        const deadline = Date.now() + releaseWaitMilliseconds;
        /**
         Take the lock as soon as the kernel has dropped it.

         @returns the lock, as the only element, or none within the wait
         */
        async function takeWhenFree(): Promise<readonly AsyncDisposable[]> {
          const result = await attempt(directory.path);
          if (result.lock.length > 0)
            return result.lock;
          if (Date.now() > deadline)
            return [];
          await sleep(50);
          return await takeWhenFree();
        }
        const taken = await takeWhenFree();
        expect(taken.length).toBe(1);
        await taken[0]?.[Symbol.asyncDispose]();
      },
    }),
    it({
      name: 'does not take a lock on a directory that is missing, and does not call that a second run',
      fn: async () => {
        await using directory = await scratch();
        const outcome = await attempt(join(directory.path, 'missing'));
        expect(outcome.lock).toEqual([]);
        expect(outcome.reason.includes('Another run')).toBe(false);
      },
    }),
  ],
});
