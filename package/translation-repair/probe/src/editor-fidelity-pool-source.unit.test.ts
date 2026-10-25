/**
 Tests for the text the pool draw writes as the manifest's drawn parts: ids, offsets and hashes only, in a layout that
 gives the same bytes for the same draw. Specifications are invented.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type FidelityReferenceSpec,
  poolSourceFiles,
} from '../dist/final/node/index.mjs';

/**
 An invented specification with the given id and one damage.

 @param id - spec id, which is also read as its entry id

 @returns The specification

 @example
 ```ts
 const spec = specOf({ id: 'tabby-s1', },);
 ```
 */
function specOf({ id, }: { readonly id: string; },): FidelityReferenceSpec {
  return {
    id,
    entryId: id.split('-s',)[0] ?? id,
    corpusSha: 'a'.repeat(40,),
    source: {
      startOffset: 56,
      endOffset: 1_268,
      hash: 'b'.repeat(64,),
    },
    archive: {
      startOffset: 56,
      endOffset: 12_345,
      hash: 'c'.repeat(64,),
    },
    referenceHash: 'c'.repeat(64,),
    referenceChars: 585,
    edits: [],
    donor: {
      startOffset: 643,
      endOffset: 1_268,
      hash: 'd'.repeat(64,),
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e'.repeat(64,),
        changedChars: 107,
      },
    ],
    reviewedOn: '2026-10-23',
  };
}

/**
 Five specifications.
 */
const SPECS: readonly FidelityReferenceSpec[] = ['a-s0', 'b-s1', 'c-s2', 'd-s0', 'e-s3',].map(function make(id,) {
  return specOf({ id, },);
},);

await describe({
  name: poolSourceFiles.name,
  children: [
    it({
      name: 'WRITES THE SPECIFICATIONS IN PARTS OF A FIXED SIZE and one file that gathers them, in order, so the '
        + 'manifest imports one name however many parts the draw needs',
      fn: async () => {
        const files = poolSourceFiles({
          specs: SPECS,
          seed: 'a seed',
          perPart: 2,
        },);
        expect(files.map(function nameOf(file,): string {
          return file.name;
        },),).toStrictEqual([
          'editor-fidelity-pool-drawn-1.ts',
          'editor-fidelity-pool-drawn-2.ts',
          'editor-fidelity-pool-drawn-3.ts',
          'editor-fidelity-pool-drawn.ts',
        ],);
        const gathered = files.at(-1)?.text ?? '';
        expect(gathered.indexOf('DRAWN_EDITOR_SPECS_1',),).toBeLessThan(gathered.indexOf('DRAWN_EDITOR_SPECS_2',),);
        expect(gathered.indexOf('DRAWN_EDITOR_SPECS_2',),).toBeLessThan(gathered.indexOf('DRAWN_EDITOR_SPECS_3',),);
        expect(gathered.includes('export const DRAWN_EDITOR_SPECS: readonly FidelityReferenceSpec[]',),).toBe(true,);
      },
    },),

    it({
      name: 'WRITES IDS, OFFSETS AND HASHES ONLY, with a numeric separator in numbers of four digits or more as the '
        + 'package\'s formatter writes them, and says in each part which seed drew it',
      fn: async () => {
        const [first,] = poolSourceFiles({
          specs: SPECS,
          seed: 'a seed',
          perPart: 2,
        },);
        const text = first?.text ?? '';
        expect(text.includes('startOffset: 12_345',),).toBe(false,);
        expect(text.includes('endOffset: 12_345',),).toBe(true,);
        expect(text.includes('endOffset: 1_268',),).toBe(true,);
        expect(text.includes('startOffset: 56,',),).toBe(true,);
        expect(text.includes('id: \'a-s0\'',),).toBe(true,);
        expect(text.includes('hash: \'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\'',),).toBe(true,);
        expect(text.includes('seed `a seed`',),).toBe(true,);
      },
    },),

    it({
      name: 'WRITES THE SAME BYTES FOR THE SAME DRAW, so a check can compare the committed files with a fresh draw',
      fn: async () => {
        /**
         The same draw rendered twice.
         */
        const once = poolSourceFiles({
          specs: SPECS,
          seed: 'a seed',
          perPart: 2,
        },);
        const twice = poolSourceFiles({
          specs: SPECS,
          seed: 'a seed',
          perPart: 2,
        },);
        expect(twice,).toStrictEqual(once,);
      },
    },),
  ],
},);
