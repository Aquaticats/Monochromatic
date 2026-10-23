/**
 Tests the footnote check of a would-ship page, for the one thing it is told not to read: the original a slice ships
 where nobody wrote a wording.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { assertPageFootnotesIntact, } from '../../dist/final/node/index.mjs';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 An archive slice with a note, and an anchor nobody filled whose original refers to a note it does not define.
 */
const ROWS: readonly SourceShippedRow[] = [
  {
    incumbentText: 'The cat sleeps.[^1]\n\n[^1]: A note.\n',
    sourceText: '猫睡了。\n',
  },
  {
    incumbentText: '',
    sourceText: '她说[^7]\n',
  },
];

await describe({
  name: assertPageFootnotesIntact.name,
  children: [
    it({
      name: 'DOES NOT READ THE ORIGINAL AN UNFILLED ANCHOR SHIPS, which is the original as the artifact records it and '
        + 'no wording of the pipeline: the reference to a note it does not define is not blamed on the page',
      fn: () => {
        expect(() => assertPageFootnotesIntact({
          artifact: sourceShippedArtifact({ rows: ROWS, },),
          slices: sourceShippedSlices({ rows: ROWS, },),
          targetText: sourceShippedArchive({ rows: ROWS, },),
        },),).not.toThrow();
      },
    },),
  ],
},);
