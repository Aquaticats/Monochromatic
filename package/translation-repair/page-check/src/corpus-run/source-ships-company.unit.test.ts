/**
 Tests for the runs that parse alone and cannot stand beside another: a fence mark indented four spaces or more, and a
 name an earlier run still declares.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  runsUnfitForCompany,
  sourceRunsOf,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import {
  sourceShippedArtifact,
  type SourceShippedRow,
} from './source-shipped.test-fixture.ts';

/**
 The paragraph the archive covers.
 */
const OPENING_ROW: SourceShippedRow = {
  incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
  sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
};

/**
 A paragraph the archive covers between two runs.
 */
const MIDDLE_ROW: SourceShippedRow = {
  incumbentText: '\nShe slept on the counter.\n',
  sourceText: '\n她睡在柜台上。\n',
};

/**
 The runs of an entry whose runs have the given originals, one slice each, with a paragraph between every two.

 @param sources - originals, one for each run

 @returns Slice indices of the runs that cannot stand beside the others

 @example
 ```ts
 const unfit = unfitAmong({ sources: ['import a from "b"\n', 'import a from "b"\n',], },);
 ```
 */
function unfitAmong({ sources, }: { readonly sources: readonly string[]; },): readonly (readonly number[])[] {
  /**
   Rows of the entry.
   */
  const rows: readonly SourceShippedRow[] = [
    OPENING_ROW,
    ...sources.flatMap(function runRows(sourceText, at,): readonly SourceShippedRow[] {
      return (at === 0)
        ? [{ incumbentText: '', sourceText, },]
        : [MIDDLE_ROW, { incumbentText: '', sourceText, },];
    },),
  ];
  /**
   What each slice would carry.
   */
  const slices = wouldShipTextPerSlice({ artifact: sourceShippedArtifact({ rows, },), },);
  return runsUnfitForCompany({
    runs: sourceRunsOf({ slices, },),
    slices,
  },);
}

await describe({
  name: runsUnfitForCompany.name,
  children: [
    it({
      name: 'TAKES EVERY RUN HOLDING A FENCE MARK INDENTED FOUR SPACES, the one that opens the fence and the one '
        + 'that closes it',
      fn: async () => {
        expect(unfitAmong({ sources: ['    ```\n', '    ```\n    {\n    ```\n',], },),).toEqual([[1,], [3,],],);
      },
    },),

    it({
      name: 'TAKES THE LATER OF TWO RUNS IMPORTING ONE NAME, and keeps the first as the module it is',
      fn: async () => {
        expect(unfitAmong({ sources: ['import a from "b"\n', 'import a from "b"\n',], },),).toEqual([[3,],],);
      },
    },),

    it({
      name: 'TAKES EACH LATER RUN THAT REPEATS A NAME, the third run repeating what the first declared',
      fn: async () => {
        expect(unfitAmong({
          sources: ['import a from "b"\n', 'import c from "b"\n', 'import a from "b"\n',],
        },),).toEqual([[5,],],);
      },
    },),

    it({
      name: 'TAKES NO RUN FOR TWO NAMES THAT DIFFER, nor for runs of prose',
      fn: async () => {
        expect(unfitAmong({ sources: ['import a from "b"\n', 'import c from "b"\n',], },),).toEqual([],);
        expect(unfitAmong({ sources: ['她说\n', '她又说\n',], },),).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS A RUN TAKEN FOR ITS FENCE MARK AS DECLARING NOTHING, since it is written as literal text and no '
        + 'longer a module',
      fn: async () => {
        expect(unfitAmong({
          sources: ['import a from "b"\n\n    ```\n', 'import a from "b"\n',],
        },),).toEqual([[1,],],);
      },
    },),
  ],
},);
