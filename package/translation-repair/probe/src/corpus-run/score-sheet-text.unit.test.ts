/**
 Tests for the sheet reader the `score-*` runners share.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { readSheetText, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: readSheetText.name,
  children: [
    it({
      name: 'READS the text of a sheet that is there',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-sheet-text-', },);

        /**
         Sheet the case wrote.
         */
        const path = join(
          scratch.path,
          'sheet.md',
        );
        await writeFile(
          path,
          'The cat sat.\n',
          'utf8',
        );

        expect(await readSheetText({ path, label: 'graded sheet', remedy: 'name it', },),).toBe('The cat sat.\n',);
      },
    },),

    it({
      name: 'REFUSES a sheet that is not there, naming its path, ENOENT and the remedy',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-sheet-text-', },);

        /**
         Path nothing was written to.
         */
        const path = join(
          scratch.path,
          'sheet.md',
        );

        /**
         What reading it raised.
         */
        const refusal = await rejectionOf(async function readsAbsentSheet(): Promise<void> {
          await readSheetText({ path, label: 'graded sheet', remedy: 'name it with --sheet', },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot read the graded sheet at ${path} (ENOENT); name it with --sheet`,
        );
      },
    },),

    it({
      name: 'REFUSES a path that is a directory, naming its path and EISDIR',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-sheet-text-', },);

        /**
         What reading the directory itself raised.
         */
        const refusal = await rejectionOf(async function readsDirectory(): Promise<void> {
          await readSheetText({ path: scratch.path, label: 'repair sheet', remedy: 'name a file', },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot read the repair sheet at ${scratch.path} (EISDIR); name a file`,
        );
      },
    },),
  ],
},);
