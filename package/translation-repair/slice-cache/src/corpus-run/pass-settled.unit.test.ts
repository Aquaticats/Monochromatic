/**
 Tests for what the scheduler counts as settled.

 The module note records a past silent defect: a directory or a symlink named
 `<id>.json` once marked the entry settled, and the entry was never run again.
 These cases hold that line. The pass's closing line counts this same id set
 rather than a count of its own, so the two cannot drift apart.

 Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { artifactBackedIds, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Writes two regular artifacts, one directory named like one, one symlink
 named like one, and one regular file without the suffix, into a caller-owned
 directory.

 @param dir - case-owned directory to write into

 @returns Nothing; the files are the effect

 @example
 ```ts
 await mixedDirectory({ dir: scratch.path, },);
 ```
 */
async function mixedDirectory({ dir: artifactsDir, }: { readonly dir: string; },): Promise<void> {
  await writeFile(join(artifactsDir, 'whiskers.json',), '{}', 'utf8',);
  await writeFile(join(artifactsDir, 'tabby.json',), '{}', 'utf8',);
  await mkdir(join(artifactsDir, 'mittens.json',),);
  await symlink('tabby.json', join(artifactsDir, 'ghost.json',),);
  await writeFile(join(artifactsDir, 'notes.txt',), 'not an artifact', 'utf8',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: artifactBackedIds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to count a directory named like an artifact, which once marked an entry settled without '
            + 'the entry ever having run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-settled-', },);
            await mixedDirectory({ dir: scratch.path, },);
            const ids = await artifactBackedIds({ artifactsDir: scratch.path, },);

            expect(ids.has('mittens',),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES to count a symlink named like an artifact, for the same reason',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-settled-', },);
            await mixedDirectory({ dir: scratch.path, },);
            const ids = await artifactBackedIds({ artifactsDir: scratch.path, },);

            expect(ids.has('ghost',),).toBe(false,);
          },
        },),

        it({
          name: 'KEEPS every regular artifact under its id, and nothing without the suffix',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-settled-', },);
            await mixedDirectory({ dir: scratch.path, },);
            const ids = await artifactBackedIds({ artifactsDir: scratch.path, },);

            expect([...ids,].toSorted(),).toEqual([
              'tabby',
              'whiskers',
            ],);
          },
        },),
      ],
    },),
  ],
},);
