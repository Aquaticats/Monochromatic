import { spawnSync } from 'node:child_process';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { acquireCampaignLock } from './campaign-lock.ts';
import { CargoMutantsToolingError } from './error.ts';

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
      name: 'takes over a lock whose process is gone, as a killed run or a reboot leaves it',
      fn: async () => {
        await using directory = await scratch();
        /**
         A process that has already exited, whose number a lock file can name.
         */
        const finished = spawnSync(
          process.execPath,
          [
            '--eval',
            '0'
          ],
        );
        await writeFile(
          join(
            directory.path,
            'campaign.lock',
          ),
          `${String(finished.pid)} some-boot:12345\n`,
        );
        await using taken = await acquireCampaignLock(directory.path);
        expect(taken).toBeDefined();
        expect(await readFile(
          join(
            directory.path,
            'campaign.lock',
          ),
          'utf8',
        )).toContain(String(process.pid));
      },
    }),
    it({
      name: 'takes over a lock that names a running process with another start time, as a reused process number does',
      fn: async () => {
        await using directory = await scratch();
        await writeFile(
          join(
            directory.path,
            'campaign.lock',
          ),
          `${String(process.pid)} another-boot:1\n`,
        );
        await using taken = await acquireCampaignLock(directory.path);
        expect(taken).toBeDefined();
      },
    }),
  ],
});
