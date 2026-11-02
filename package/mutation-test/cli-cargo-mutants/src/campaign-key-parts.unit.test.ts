import {
  mkdir,
  mkdtempDisposable,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  openCampaign,
  prepareAttempt,
} from './campaign-attempt.ts';

/**
 Image ID of the tested image.
 */
const baseImage = 'a'.repeat(64);

/**
 The campaign command.
 */
const command = ['cargo', 'mutants'];

/**
 Create the evidence root a test fills.

 @returns the disposable root; declare it with `await using`

 @example
 ```ts
 await using root = await evidenceRoot();
 ```
 */
async function evidenceRoot(): Promise<AsyncDisposable & { readonly path: string }> {
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  return await mkdtempDisposable(join(
    parent,
    'campaign-key-parts-',
  ));
}

await describe({
  name: 'the product conditions in a campaign directory key',
  children: [
    it({
      name: 'a one-pass campaign finds the same directory for the same conditions and another one when a condition changes',
      fn: async () => {
        await using root = await evidenceRoot();
        /**
         Evidence directory a campaign with the given conditions uses.

         @param keyParts - the product's conditions

         @returns the directory
         */
        async function directoryFor(keyParts: readonly string[]): Promise<string> {
          await using attempt = await prepareAttempt({
            evidenceRoot: root.path,
            evidencePrefix: 'native-mutation-',
            baseImage,
            command,
            keyParts,
          });
          return attempt.evidence;
        }
        const first = await directoryFor(['controls-a', 'harness-a']);
        expect(await directoryFor(['controls-a', 'harness-a'])).toBe(first);
        expect(await directoryFor(['controls-b', 'harness-a'])).not.toBe(first);
        expect(await directoryFor(['controls-a', 'harness-b'])).not.toBe(first);
      },
    }),
    it({
      name: 'a two-pass campaign opens one directory for both passes under the same conditions, and another under other conditions',
      fn: async () => {
        await using root = await evidenceRoot();
        /**
         Evidence directory a two-pass campaign with the given conditions uses.

         @param keyParts - the product's conditions

         @returns the directory
         */
        async function directoryFor(keyParts: readonly string[]): Promise<string> {
          await using campaign = await openCampaign({
            evidenceRoot: root.path,
            evidencePrefix: 'native-mutation-',
            baseImage,
            command,
            keyParts,
            passes: 2,
          });
          return campaign.evidence;
        }
        const first = await directoryFor(['controls-a', 'harness-a']);
        expect(await directoryFor(['controls-a', 'harness-a'])).toBe(first);
        expect(await directoryFor(['controls-a', 'harness-b'])).not.toBe(first);
      },
    }),
    it({
      name: 'the directory\'s campaign.json records the conditions, one-pass and two-pass, so a clash of the shortened key is refused',
      fn: async () => {
        await using root = await evidenceRoot();
        await using attempt = await prepareAttempt({
          evidenceRoot: root.path,
          evidencePrefix: 'native-mutation-',
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-a'],
        });
        const written: unknown = JSON.parse(await readFile(join(attempt.evidence, 'campaign.json'), 'utf8'));
        expect(written).toEqual({ baseImage, command, keyParts: ['controls-a', 'harness-a'] });
        await using campaign = await openCampaign({
          evidenceRoot: root.path,
          evidencePrefix: 'native-mutation-',
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-b'],
          passes: 2,
        });
        const twoPass: unknown = JSON.parse(await readFile(join(campaign.evidence, 'campaign.json'), 'utf8'));
        expect(twoPass).toEqual({ baseImage, command, keyParts: ['controls-a', 'harness-b'], passes: 2 });
      },
    }),
    it({
      name: 'two openers of one directory with different identities race for the lock first: one wins and the file is the winner\'s',
      fn: async () => {
        await using root = await evidenceRoot();
        const options = {
          evidenceRoot: root.path,
          evidencePrefix: 'native-mutation-',
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-a'],
        };
        // The same key, so the same directory, but a one-pass and a two-pass identity, started in the same tick.
        const settled = await Promise.allSettled([
          openCampaign(options),
          openCampaign({ ...options, passes: 2 }),
        ]);
        const winners = settled.flatMap(function won(result) {
          return result.status === 'fulfilled' ? [result.value] : [];
        });
        expect(winners.length).toBe(1);
        const [winner] = winners;
        const written: unknown = JSON.parse(await readFile(join(winner?.evidence ?? '', 'campaign.json'), 'utf8'));
        const winnerIsTwoPass = settled[1]?.status === 'fulfilled';
        expect(written).toEqual({
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-a'],
          ...(winnerIsTwoPass ? { passes: 2 } : {}),
        });
        await winner?.[Symbol.asyncDispose]();
      },
    }),
    it({
      name: 'an opener refused by the identity file leaves the file as it was and gives the lock back',
      fn: async () => {
        await using root = await evidenceRoot();
        const options = {
          evidenceRoot: root.path,
          evidencePrefix: 'native-mutation-',
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-a'],
        };
        const first = await openCampaign(options);
        const identity = await readFile(join(first.evidence, 'campaign.json'), 'utf8');
        await first[Symbol.asyncDispose]();
        // Another campaign's identity sits in the directory this one's key names: a clash of the shortened key.
        await writeFile(join(first.evidence, 'campaign.json'), identity.replace('harness-a', 'harness-z'));
        await expect(openCampaign(options)).rejects.toThrow('belongs to another');
        expect(await readFile(join(first.evidence, 'campaign.json'), 'utf8')).toBe(identity.replace('harness-a', 'harness-z'));
        // The lock was given back: the clash does not lock the directory's real owner out.
        await writeFile(join(first.evidence, 'campaign.json'), identity);
        await using again = await openCampaign(options);
        expect(again.evidence).toBe(first.evidence);
      },
    }),
    it({
      name: 'an opener that finds the directory locked does not create or change the identity file, even when it is absent',
      fn: async () => {
        await using root = await evidenceRoot();
        const options = {
          evidenceRoot: root.path,
          evidencePrefix: 'native-mutation-',
          baseImage,
          command,
          keyParts: ['controls-a', 'harness-a'],
        };
        await using holder = await openCampaign(options);
        // The race, made certain: the holder has the lock and the identity file is not there when the other opener looks.
        await rm(join(holder.evidence, 'campaign.json'));
        await expect(openCampaign({ ...options, passes: 2 })).rejects.toThrow('Another run');
        expect(await readdir(holder.evidence)).not.toContain('campaign.json');
      },
    }),
  ],
});
