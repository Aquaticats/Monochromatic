/**
 Tests that the nesting bound reads the lines of a body as the parser does: a line feed, a carriage return alone and a
 carriage return with a line feed after it each end a line.

 A body whose lines end in a carriage return alone was read as one line, so a fence that the parser closes on a later
 line stayed open to the measure and hid the brackets and container marks after it, which the parser then read. A text
 the measure admitted was refused by a reader that measures the same text cut into parts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { firstNestingExcess, } from '../dist/final/node/index.mjs';

//region Nesting bound line endings tests

/**
 What reading a body found within the bound.
 */
const WITHIN = { kind: 'within', };

await describe({
  name: `${firstNestingExcess.name} over line endings`,
  children: [
    it({
      name: 'COUNTS the container marks of a line that follows a carriage return alone from that line start',
      fn: async () => {
        expect(firstNestingExcess({ body: `cat\r${'>'.repeat(256,)} mew`, grammar: 'markdown', },),).toEqual(WITHIN,);
        expect(firstNestingExcess({ body: `cat\r${'>'.repeat(257,)} mew`, grammar: 'markdown', },),).toEqual({
          kind: 'beyond',
          measure: 'container markers',
          bound: 256,
          line: 2,
          column: 257,
        },);
      },
    },),
    it({
      name: 'CLOSES a fence on a line that follows a carriage return alone, so the brackets after it are counted',
      fn: async () => {
        /**
         A fence opened and closed by lines that end in a carriage return alone, then a line that ends in both.
         */
        const fenced = '~~~\r{a}\r~~~\r\n';

        expect(firstNestingExcess({ body: `${fenced}${'['.repeat(256,)}\n`, grammar: 'markdown', },),).toEqual(WITHIN,);
        expect(firstNestingExcess({ body: `${fenced}${'['.repeat(257,)}\n`, grammar: 'markdown', },),).toEqual({
          kind: 'beyond',
          measure: 'brackets',
          bound: 256,
          line: 4,
          column: 257,
        },);
      },
    },),
    it({
      name: 'READS a carriage return with a line feed after it as one line ending, not two',
      fn: async () => {
        expect(firstNestingExcess({ body: `cat\r\n${'['.repeat(257,)}`, grammar: 'markdown', },),).toEqual({
          kind: 'beyond',
          measure: 'brackets',
          bound: 256,
          line: 2,
          column: 257,
        },);
      },
    },),
  ],
},);

//endregion Nesting bound line endings tests
