/**
 Tests the pictures `reader-measure` tries: every picture file under a
 directory at any depth, in code point order, named by the path under that
 directory.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { loadPictures, } from '../../dist/final/node/index.mjs';

await describe({
  name: loadPictures.name,
  children: [
    it({
      name: 'READS THE PICTURE FILES UNDER A DIRECTORY AT ANY DEPTH, in path order, named by their path, and '
        + 'skips hidden files and other kinds',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-reader-pictures-', },);
        await mkdir(join(scratch.path, 'b', 'photos',), { recursive: true, },);
        await mkdir(join(scratch.path, 'a',), { recursive: true, },);
        await writeFile(join(scratch.path, 'b', 'photos', 'two.WEBP',), 'two',);
        await writeFile(join(scratch.path, 'a', 'one.png',), 'one',);
        await writeFile(join(scratch.path, 'a', '.hidden.png',), 'hidden',);
        await writeFile(join(scratch.path, 'a', 'notes.md',), 'notes',);

        const pictures = await loadPictures({ dir: scratch.path, },);
        expect(pictures.map(function nameOf(picture,): string {
          return picture.assetName;
        },),).toEqual([join('a', 'one.png',), join('b', 'photos', 'two.WEBP',),],);
        expect(Buffer.from(pictures[0]?.bytes ?? new Uint8Array(),).toString(),).toBe('one',);
      },
    },),
  ],
},);
