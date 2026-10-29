/**
 Tests for the check that a sheet pair may still be written, which callers run
 before the work that fills it.

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

import {
  assertSheetPairFree,
  StatedRefusalError,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Runs the check over the cat sheet pair in a directory and hands back what it
 refused with.

 @param dir - directory the pair would land in

 @returns Whatever the check rejected with

 @example
 ```ts
 const refusal = await refusalOfCheckIn({ dir, },);
 ```
 */
async function refusalOfCheckIn({ dir, }: { readonly dir: string; },): Promise<unknown> {
  return await rejectionOf(async function checksTheCatPair(): Promise<unknown> {
    return await assertSheetPairFree({
      dir,
      sheetName: 'cat-sheet.md',
      manifestName: 'cat-manifest.json',
    },);
  },);
}

await describe({
  name: assertSheetPairFree.name,
  children: [
    it({
      name: 'RESOLVES when neither the sheet nor the manifest is in the directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sheet-write-free-', },);

        expect(await assertSheetPairFree({
          dir: scratch.path,
          sheetName: 'cat-sheet.md',
          manifestName: 'cat-manifest.json',
        },),).toBeUndefined();
      },
    },),

    it({
      name: 'REFUSES naming the sheet when the sheet is there, whether or not the manifest is',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sheet-write-free-', },);
        await writeFile(
          join(
            scratch.path,
            'cat-sheet.md',
          ),
          '# sheet\n',
          'utf8',
        );
        await writeFile(
          join(
            scratch.path,
            'cat-manifest.json',
          ),
          '{}',
          'utf8',
        );

        /**
         What the check refused with.
         */
        const refusal = await refusalOfCheckIn({ dir: scratch.path, },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${scratch.path}/cat-sheet.md already exists; grade or move it before rerunning, `
            + 'since a rerun would replace a grader\'s work',
        );
      },
    },),

    it({
      name: 'REFUSES naming the manifest when only the manifest is there',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sheet-write-free-', },);
        await writeFile(
          join(
            scratch.path,
            'cat-manifest.json',
          ),
          '{}',
          'utf8',
        );

        /**
         What the check refused with.
         */
        const refusal = await refusalOfCheckIn({ dir: scratch.path, },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${scratch.path}/cat-manifest.json already exists; grade or move it before rerunning, `
            + 'since a rerun would replace a grader\'s work',
        );
      },
    },),
  ],
},);
