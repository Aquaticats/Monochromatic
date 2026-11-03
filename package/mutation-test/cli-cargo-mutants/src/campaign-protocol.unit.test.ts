import {
  mkdir,
  mkdtempDisposable,
  readFile,
  rm,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { openCampaign } from './campaign-attempt.ts';
import {
  campaignProtocolFile,
  startedMarkerProtocolVersion,
} from './constants.ts';

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
    'campaign-protocol-',
  ));
}

/**
 Open the campaign and give it back at once, returning its evidence directory.

 @param root - evidence root

 @returns the campaign's evidence directory

 @example
 ```ts
 await openAndRelease(root.path);
 ```
 */
async function openAndRelease(root: string): Promise<string> {
  await using campaign = await openCampaign({
    evidenceRoot: root,
    evidencePrefix: 'native-mutation-',
    baseImage,
    command,
  });
  return campaign.evidence;
}

await describe({
  name: 'the started-marker protocol record of a campaign directory',
  children: [
    it({
      name: 'is written into a directory the runner creates, and kept when the campaign is opened again',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = await openAndRelease(root.path);
        const recorded: unknown = JSON.parse(await readFile(join(evidence, campaignProtocolFile), 'utf8'));
        expect(recorded).toEqual({ version: startedMarkerProtocolVersion });
        await openAndRelease(root.path);
        const again: unknown = JSON.parse(await readFile(join(evidence, campaignProtocolFile), 'utf8'));
        expect(again).toEqual({ version: startedMarkerProtocolVersion });
      },
    }),
    it({
      name: 'is not written into a directory that already holds the work of an earlier runner, whose history it cannot vouch for',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = await openAndRelease(root.path);
        // Make the directory what an earlier runner left: it holds its identity file but no protocol record.
        await rm(join(evidence, campaignProtocolFile), { force: true });
        await openAndRelease(root.path);
        await expect(readFile(join(evidence, campaignProtocolFile), 'utf8')).rejects.toThrow();
      },
    }),
  ],
});
