/**
 Tests the reading of where the strict grammar reads a table in a text. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { tableRangesOf, } from '../dist/final/node/index.mjs';

/**
 A table of one body row.
 */
const TABLE = '| a | b |\n| - | - |\n| c | d |\n';

await describe({
  name: tableRangesOf.name,
  children: [
    it({
      name: 'READS THE RANGE OF A TABLE, from its header row to the end of its last row',
      fn: async () => {
        expect(tableRangesOf({ body: `她说\n\n${TABLE}`, },),).toEqual([
          {
            start: 4,
            end: (4 + TABLE.length) - 1,
          },
        ],);
      },
    },),

    it({
      name: 'READS EACH TABLE OF A TEXT IN TEXT ORDER, in a block quote and a list item too',
      fn: async () => {
        /**
         Three tables, the second in a block quote and the third in a list item.
         */
        const body = `${TABLE}\n> | e | f |\n> | - | - |\n\n- | g | h |\n  | - | - |\n`;
        expect(tableRangesOf({ body, },).length,).toBe(3,);
        expect(tableRangesOf({ body, },).map(function startOf({ start, },): number {
          return start;
        },),).toEqual([0, body.indexOf('> |',) + 2, body.indexOf('| g',),],);
      },
    },),

    it({
      name: 'READS NO TABLE IN A ROW WITHOUT A DELIMITER ROW, in a text without a pipe and in a text nested past the bound',
      fn: async () => {
        expect(tableRangesOf({ body: '| a | b |\n| c | d |\n', },),).toEqual([],);
        expect(tableRangesOf({ body: '猫\n', },),).toEqual([],);
        expect(tableRangesOf({ body: `${'> '.repeat(300,)}${TABLE}`, },),).toEqual([],);
      },
    },),
  ],
},);
