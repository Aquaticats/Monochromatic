/**
 Tests for the list of defects one published page ships with: which families it holds and the order it names them in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type PublishDefect,
  shippedDefects,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';

/**
 Artifact whose consolidation did not run, so it names no slice the archive kept and no tense swap.
 */
const UNCONSOLIDATED = { consolidation: { kind: 'not-run', }, } as Pick<WouldShipSource, 'consolidation'>;

/**
 One defect of a given check, for a family to carry.

 @param check - check the defect names

 @returns Defect

 @example
 ```ts
 const defect = defectOf('headings',);
 ```
 */
function defectOf(check: string,): PublishDefect {
  return { check, message: `entry WhiskerCat: ${check}`, } as PublishDefect;
}

await describe({
  name: shippedDefects.name,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE FAILED CHECKS FIRST, then a page with no body, then the defects of the composed page and of the '
        + 'grammar last',
      fn: async () => {
        expect(shippedDefects({
          entryId: 'WhiskerCat',
          artifact: UNCONSOLIDATED,
          sourceText: '猫醒了。\n',
          pageText: '',
          failedChecks: [defectOf('headings',),],
          composedDefects: [defectOf('page-source-unparseable',),],
          parsingDefects: [defectOf('page-unparseable',),],
        },).map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['headings', 'page-without-body', 'page-source-unparseable', 'page-unparseable',],);
      },
    },),

    it({
      name: 'HOLDS NO DEFECT FOR A PAGE THAT CARRIES A BODY, none of the families having anything to name',
      fn: async () => {
        expect(shippedDefects({
          entryId: 'WhiskerCat',
          artifact: UNCONSOLIDATED,
          sourceText: '猫醒了。\n',
          pageText: 'The cat woke.\n',
          failedChecks: [],
          composedDefects: [],
          parsingDefects: [],
        },),).toEqual([],);
      },
    },),
  ],
},);
