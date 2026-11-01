/**
 One read of a cell in a table built to its own bounds: the cell inside them,
 a throw outside them rather than a default.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { tableCell, } from '../dist/final/node/index.mjs';

/**
 Table of three rows, the last shorter than the others, holding a zero and a
 false among its cells so a falsy cell is read as a cell.
 */
const TABLE: readonly (readonly (number | boolean)[])[] = [
  [
    1,
    2,
  ],
  [
    0,
    false,
  ],
  [3,],
];

await describe({
  name: tableCell.name,
  children: [
    it({
      name: 'READS THE CELL AT A ROW AND COLUMN, a zero or false cell as itself',
      fn: async () => {
        expect(tableCell({
          table: TABLE,
          row: 0,
          column: 1,
        },),).toBe(2,);
        expect(tableCell({
          table: TABLE,
          row: 1,
          column: 0,
        },),).toBe(0,);
        expect(tableCell({
          table: TABLE,
          row: 1,
          column: 1,
        },),).toBe(false,);
      },
    },),
    it({
      name: 'THROWS FOR A ROW THE TABLE DOES NOT HOLD',
      fn: async () => {
        expect(function readPastTheRows(): number | boolean {
          return tableCell({
            table: TABLE,
            row: 3,
            column: 0,
          },);
        },).toThrow();
      },
    },),
    it({
      name: 'THROWS FOR A COLUMN THE ROW DOES NOT HOLD',
      fn: async () => {
        expect(function readPastTheShortRow(): number | boolean {
          return tableCell({
            table: TABLE,
            row: 2,
            column: 1,
          },);
        },).toThrow();
      },
    },),
  ],
},);
