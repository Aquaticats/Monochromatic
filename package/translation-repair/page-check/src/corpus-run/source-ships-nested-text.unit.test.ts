/**
 Tests for the characters of a nested block and for where an edit of a nested text ends in the whole text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  charactersAt,
  endOfEdit,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the text of a nested block',
  children: [
    describe({
      name: charactersAt.name,
      children: [
        it({
          name: 'JOINS THE CHARACTERS AT THE INDICES, in the order of the indices',
          fn: async () => {
            expect(charactersAt({ text: 'abcd', indices: [3, 0, 2,], },),).toBe('dac',);
            expect(charactersAt({ text: 'abcd', indices: [], },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: endOfEdit.name,
      children: [
        it({
          name: 'ENDS AN EDIT AFTER THE LAST CHARACTER IT REPLACES',
          fn: async () => {
            expect(endOfEdit({
              source: '> ab\n> cd',
              indices: [2, 3, 4, 7, 8,],
              edit: { start: 1, end: 2, replacement: 'x', },
            },),).toBe(4,);
          },
        },),
        it({
          name: 'ENDS AN EDIT THAT TAKES A LINE ENDING AWAY AT THE NEXT CHARACTER OF THE NESTED TEXT, with the marks the lines gave up',
          fn: async () => {
            expect(endOfEdit({
              source: '> ab\n> cd',
              indices: [2, 3, 4, 7, 8,],
              edit: { start: 2, end: 3, replacement: ' ', },
            },),).toBe(7,);
          },
        },),
      ],
    },),
  ],
},);
