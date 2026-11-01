/**
 Tests the file walk `reader-measure` makes: regular files at any depth, in
 code point order, and neither a link nor a directory taken for a file.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { filesUnder, } from '../../dist/final/node/index.mjs';

await describe({
  name: filesUnder.name,
  children: [
    it({
      name: 'LISTS THE REGULAR FILES AT ANY DEPTH BY THEIR PATH UNDER THE DIRECTORY, in code point order, and '
        + 'takes a link for neither a file nor a directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-reader-files-', },);
        await mkdir(join(scratch.path, 'b', 'deep',), { recursive: true, },);
        await mkdir(join(scratch.path, 'a',), { recursive: true, },);
        await writeFile(join(scratch.path, 'b', 'deep', 'two.txt',), 'two',);
        await writeFile(join(scratch.path, 'a', 'one.txt',), 'one',);
        await writeFile(join(scratch.path, 'Zed.txt',), 'zed',);
        await symlink(join(scratch.path, 'a', 'one.txt',), join(scratch.path, 'link.txt',),);

        expect(await filesUnder({ dir: scratch.path, },),).toEqual([
          'Zed.txt',
          join('a', 'one.txt',),
          join('b', 'deep', 'two.txt',),
        ],);
      },
    },),
  ],
},);
