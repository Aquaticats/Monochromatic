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

import { prepareAttempt } from './campaign-attempt.ts';
import {
  attemptProtocolFile,
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
 Open the next attempt of the campaign and give the campaign back at once.

 @param root - evidence root

 @returns the campaign's evidence directory and the attempt's report directory

 @example
 ```ts
 await openAndRelease(root.path);
 ```
 */
async function openAndRelease(root: string): Promise<{
  readonly evidence: string;
  readonly reportDirectory: string;
}> {
  await using attempt = await prepareAttempt({
    evidenceRoot: root,
    evidencePrefix: 'native-mutation-',
    baseImage,
    command,
  });
  return {
    evidence: attempt.evidence,
    reportDirectory: attempt.reportDirectory,
  };
}

await describe({
  name: 'the started-marker protocol record of an attempt directory',
  children: [
    it({
      name: 'is written into every attempt directory the runner makes, before anything can start',
      fn: async () => {
        await using root = await evidenceRoot();
        const first = await openAndRelease(root.path);
        const second = await openAndRelease(root.path);
        expect(second.reportDirectory).not.toBe(first.reportDirectory);
        const recorded: readonly unknown[] = await Promise.all([first, second].map(async function recordOf(attempt) {
          return JSON.parse(await readFile(join(attempt.reportDirectory, attemptProtocolFile), 'utf8')) as unknown;
        }));
        for (const record of recorded)
          expect(record).toEqual({ version: startedMarkerProtocolVersion });
      },
    }),
    it({
      name: 'is not given to an attempt an earlier runner made, even in a directory the runner resumes',
      fn: async () => {
        await using root = await evidenceRoot();
        const first = await openAndRelease(root.path);
        // Make the first attempt what an earlier runner left: it has no protocol record.
        await rm(join(first.reportDirectory, attemptProtocolFile), { force: true });
        const second = await openAndRelease(root.path);
        await expect(readFile(join(first.reportDirectory, attemptProtocolFile), 'utf8')).rejects.toThrow();
        const recorded: unknown = JSON.parse(await readFile(join(second.reportDirectory, attemptProtocolFile), 'utf8'));
        expect(recorded).toEqual({ version: startedMarkerProtocolVersion });
      },
    }),
  ],
});
