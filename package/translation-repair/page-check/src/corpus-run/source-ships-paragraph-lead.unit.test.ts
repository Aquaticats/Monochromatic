/**
 Tests for the lead that keeps a block read alone in the paragraph it stood in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isIndentedFourOrMore,
  PARAGRAPH_LEAD,
  spansWithoutLead,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the lead of a block that stood in a paragraph',
  children: [
    describe({
      name: isIndentedFourOrMore.name,
      children: [
        it({
          name: 'IS TRUE FOR FOUR SPACES, MORE SPACES AND A TAB BEFORE THE INDEX ON ITS LINE',
          fn: async () => {
            expect(isIndentedFourOrMore({ source: '[r]: u\n    ```', at: 11, },),).toBe(true,);
            expect(isIndentedFourOrMore({ source: '[r]: u\n      ```', at: 13, },),).toBe(true,);
            expect(isIndentedFourOrMore({ source: '[r]: u\n\t```', at: 8, },),).toBe(true,);
            expect(isIndentedFourOrMore({ source: '  \t ```', at: 4, },),).toBe(true,);
          },
        },),
        it({
          name: 'IS FALSE FOR LESS THAN FOUR COLUMNS, FOR TEXT BEFORE THE INDEX AND FOR THE START OF THE TEXT',
          fn: async () => {
            expect(isIndentedFourOrMore({ source: '[r]: u\n   ```', at: 10, },),).toBe(false,);
            expect(isIndentedFourOrMore({ source: '猫    ```', at: 5, },),).toBe(false,);
            expect(isIndentedFourOrMore({ source: '```', at: 0, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: spansWithoutLead.name,
      children: [
        it({
          name: 'MOVES EVERY SPAN BACK BY THE LEAD AND DROPS WHAT LAY IN IT',
          fn: async () => {
            /**
             Spans found in a block read behind the lead.
             */
            const spans = {
              text: [{ start: 0, end: 1, }, { start: 1, end: 5, }, { start: 6, end: 9, },],
              literal: [{ start: 3, end: 8, },],
              edits: [{ start: 1, end: 2, replacement: 'z', }, { start: 4, end: 5, replacement: 'z', },],
              insideLinks: [{ start: 2, end: 6, },],
              streamStarts: [0, 2, 7,],
            };
            expect(spansWithoutLead({ spans, by: PARAGRAPH_LEAD.length, },),).toEqual({
              text: [{ start: 0, end: 3, }, { start: 4, end: 7, },],
              literal: [{ start: 1, end: 6, },],
              edits: [{ start: 2, end: 3, replacement: 'z', },],
              insideLinks: [{ start: 0, end: 4, },],
              streamStarts: [0, 5,],
            },);
          },
        },),
      ],
    },),
  ],
},);
