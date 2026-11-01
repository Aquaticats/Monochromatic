/**
 Tests for the reader the coverage census hands its bundle and source reads,
 which a case replaces with a scripted one to choose which read ends first:
 the real one reads a file whole as UTF-8 and rejects with the filesystem's
 own failure. Each case writes into a disposable directory. Names are
 cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readUtf8Text, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

await describe({
  name: readUtf8Text.name,
  children: [
    it({
      name: 'READS A FILE WHOLE AS UTF-8, a character outside the basic plane and every line end kept',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'census-read-text-', },);
        /**
         File the read is asked for.
         */
        const path = join(
          scratch.path,
          'nap.mjs',
        );
        await writeFile(
          path,
          'export const nap = \'\u{1F431}\';\r\nexport const purr = 1;\n',
          'utf8',
        );
        expect(await readUtf8Text({ path, },),).toBe('export const nap = \'\u{1F431}\';\r\nexport const purr = 1;\n',);
      },
    },),
    it({
      name: 'REJECTS WITH THE FILESYSTEM\'S OWN FAILURE, unchanged, where the file is not there',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'census-read-text-', },);
        /**
         File no case wrote.
         */
        const path = join(
          scratch.path,
          'absent.mjs',
        );
        /**
         What the read rejected with.
         */
        const refusal = await rejectionOf({ promise: readUtf8Text({ path, },), },);
        expect(refusal,).toBeInstanceOf(Error,);
        expect(String(refusal,),).toBe(`Error: ENOENT: no such file or directory, open '${path}'`,);
      },
    },),
  ],
},);
