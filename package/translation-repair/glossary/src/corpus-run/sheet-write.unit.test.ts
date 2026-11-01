/**
 Tests for the refusing sheet-pair writer.

 A grading sheet is a grader's work in progress the moment it lands, so the
 writer lands both files once and refuses to replace either on a rerun.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  StatedRefusalError,
  writeSheetPair,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: writeSheetPair.name,
  children: [
    it({
      name: 'REFUSES to write into a directory already holding the sheet, as a stated refusal naming the path, '
        + 'and leaves both files as they were',
      fn: async () => {
        /**
         Disposable directory the pair lands in.
         */
        await using scratch = await scratchDir({ prefix: 'sheet-write-', },);
        const dir = scratch.path;
        await writeSheetPair({
          dir,
          sheetName: 'damage-sheet.md',
          manifestName: 'damage-manifest.json',
          sheet: '# first sheet\n',
          manifest: '{"items":[]}\n',
        },);

        await expect(writeSheetPair({
          dir,
          sheetName: 'damage-sheet.md',
          manifestName: 'damage-manifest.json',
          sheet: '# second sheet\n',
          manifest: '{"items":[1]}\n',
        },),).rejects.toThrow(StatedRefusalError,);
        expect(await readFile(join(dir, 'damage-sheet.md',), 'utf8',),).toBe('# first sheet\n',);
        expect(await readFile(join(dir, 'damage-manifest.json',), 'utf8',),).toBe('{"items":[]}\n',);
      },
    },),

    it({
      name: 'lands both files with their text and returns the sheet path',
      fn: async () => {
        /**
         Disposable directory the pair lands in.
         */
        await using scratch = await scratchDir({ prefix: 'sheet-write-', },);
        const dir = scratch.path;

        /**
         Where the sheet landed.
         */
        const at = await writeSheetPair({
          dir,
          sheetName: 'probe-verify-sheet.md',
          manifestName: 'probe-verify-manifest.json',
          sheet: '# sheet\n',
          manifest: '{"items":[]}\n',
        },);

        expect(at,).toBe(join(dir, 'probe-verify-sheet.md',),);
        expect(await readFile(at, 'utf8',),).toBe('# sheet\n',);
        expect(await readFile(join(dir, 'probe-verify-manifest.json',), 'utf8',),).toBe('{"items":[]}\n',);
      },
    },),
  ],
},);
