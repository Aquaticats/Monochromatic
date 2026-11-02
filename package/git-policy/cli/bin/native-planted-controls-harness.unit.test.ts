import {
  mkdir,
  mkdtempDisposable,
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
  plantedHarnessDigest,
  plantedHarnessFiles,
  plantedHarnessVersion,
} from './native-planted-controls-harness.ts';

/**
 Scratch parent of the test directories; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Write a directory holding files.

 @param files - file name to text

 @returns the disposable directory; declare it with `await using`

 @example
 ```ts
 await using directory = await sourceDirectory({ 'a.ts': 'one' });
 ```
 */
async function sourceDirectory(files: Readonly<Record<string, string>>): Promise<AsyncDisposable & { readonly path: string }> {
  await mkdir(
    scratchParent,
    { recursive: true },
  );
  const directory = await mkdtempDisposable(join(
    scratchParent,
    'planted-harness-',
  ));
  await Promise.all(Object.entries(files)
    .map(function writeEntry([name, text]) {
      return writeFile(
        join(
          directory.path,
          name,
        ),
        text,
      );
    }));
  return directory;
}

await describe({
  name: plantedHarnessDigest.name,
  children: [
    it({
      name: 'is the same for the same sources and changes with the text of either one or with the file list',
      fn: async () => {
        await using same = await sourceDirectory({ 'a.ts': 'run', 'b.ts': 'judge' });
        await using runnerChanged = await sourceDirectory({ 'a.ts': 'run, filter changed', 'b.ts': 'judge' });
        await using judgeChanged = await sourceDirectory({ 'a.ts': 'run', 'b.ts': 'judge, outcome widened' });
        const files = ['a.ts', 'b.ts'];
        const base = await plantedHarnessDigest({ files, directory: same.path });
        expect(await plantedHarnessDigest({ files, directory: same.path })).toBe(base);
        expect(await plantedHarnessDigest({ files, directory: runnerChanged.path })).not.toBe(base);
        expect(await plantedHarnessDigest({ files, directory: judgeChanged.path })).not.toBe(base);
        expect(await plantedHarnessDigest({ files: ['a.ts'], directory: same.path })).not.toBe(base);
      },
    }),
    it({
      name: 'throws for a source it cannot read, so a record is never judged against unseen code',
      fn: async () => {
        await using directory = await sourceDirectory({});
        await expect(plantedHarnessDigest({ files: ['missing.ts'], directory: directory.path })).rejects.toThrow();
      },
    }),
    it({
      name: 'covers the runner, the judge and the process helpers of the planted controls, which exist beside it',
      fn: async () => {
        expect(plantedHarnessFiles).toEqual([
          'native-planted-controls.ts',
          'native-planted-controls-outcome.ts',
          'native-verification-process.ts',
        ]);
        const digest = await plantedHarnessDigest({});
        expect(digest.length).toBe(64);
      },
    }),
    it({
      name: 'pins the harness version to the digest of the sources: editing a source without a new version fails',
      fn: async () => {
        // To change the runner, the judge or the process helpers: bump `plantedHarnessVersion`, then add the digest this
        // test prints for the new sources under the new version. Records made under an older version stop counting.
        const pinned: Readonly<Record<number, string>> = {
          1: '18ac2c8ab8e1026b295e1df9cda6107c5156ec572013c1730b7399f86153d797',
        };
        expect({
          version: plantedHarnessVersion,
          digest: await plantedHarnessDigest({}),
        }).toEqual({
          version: plantedHarnessVersion,
          digest: pinned[plantedHarnessVersion],
        });
      },
    }),
  ],
});
