/**
 Tests for the join of a code span that starts with a fence mark indented four columns to the line before it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { joinBeforeFenceMark, } from '../../dist/final/node/index.mjs';

await describe({
  name: joinBeforeFenceMark.name,
  children: [
    it({
      name: 'REPLACES THE LINE ENDING AND THE INDENTATION BEFORE THE SPAN WITH A SPACE',
      fn: async () => {
        expect(joinBeforeFenceMark({
          source: 'a\n    ```\n`x\n    ```',
          range: { start: 6, end: 20, },
          definitionEnds: [],
        },),).toEqual([{ start: 1, end: 6, replacement: ' ', },],);
      },
    },),
    it({
      name: 'WRITES THE SPACE BEFORE THE MARK AS A CHARACTER REFERENCE AFTER THE END OF A LINK REFERENCE DEFINITION',
      fn: async () => {
        expect(joinBeforeFenceMark({
          source: '[b]:x\n    ```\na```\n',
          range: { start: 10, end: 17, },
          definitionEnds: [5,],
        },),).toEqual([{ start: 9, end: 10, replacement: '&#32;', },],);
      },
    },),
    it({
      name: 'JOINS NOTHING WHERE THE SPAN DOES NOT START WITH A FENCE MARK INDENTED FOUR COLUMNS',
      fn: async () => {
        expect(joinBeforeFenceMark({
          source: 'a\n`x`',
          range: { start: 2, end: 5, },
          definitionEnds: [],
        },),).toEqual([],);
      },
    },),
  ],
},);
