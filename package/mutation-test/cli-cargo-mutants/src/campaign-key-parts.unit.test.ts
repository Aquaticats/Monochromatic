import {
  mkdtempDisposable,
  mkdir,
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
  ],
});
