/**
 Tests that the nesting bound reads a quoted string inside an open brace as a string: a brace in it opens and closes
 nothing.

 The strict grammar reads `{b "t {"}` as one expression whose JavaScript holds a string with a brace in it. A count of the
 braces alone sees two opens and one close, takes the expression for open at the end of the line, and then counts the
 inline text of the fence that opens on the line after it, which the measure skips when it knows the fence opens at the
 top. A text with a long line of brackets in the fence's info string was refused for a depth it does not have.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { firstNestingExcess, } from '../dist/final/node/index.mjs';

//region Nesting bound expression strings tests

/**
 What reading a body found within the bound.
 */
const WITHIN = { kind: 'within', };

/**
 A fence whose info string holds more open brackets than the bound allows, which the measure skips where the fence opens
 at the top.
 */
const DEEP_INFO_FENCE = `~~~js ${'['.repeat(300,)}\n~~~\n`;

await describe({
  name: `${firstNestingExcess.name} over braces in strings`,
  children: [
    it({
      name: 'READS A BRACE IN A QUOTED STRING INSIDE AN EXPRESSION AS NO OPEN BRACE, so the fence after it opens at the top',
      fn: async () => {
        expect(firstNestingExcess({ body: `{cat "mew {"}\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual(WITHIN,);
        expect(firstNestingExcess({ body: `{cat 'mew {'}\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual(WITHIN,);
      },
    },),
    it({
      name: 'READS AN ESCAPED QUOTE INSIDE THE STRING AS PART OF IT, so the brace after it is still in the string',
      fn: async () => {
        expect(firstNestingExcess({ body: `{cat "a\\" {"}\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual(WITHIN,);
      },
    },),
    it({
      name: 'READS AN OPEN BRACE OUTSIDE A STRING AS OPEN, so the fence after it still counts its info string',
      fn: async () => {
        expect(firstNestingExcess({ body: `{cat "mew" {\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual({
          kind: 'beyond',
          measure: 'brackets',
          bound: 256,
          line: 2,
          column: 263,
        },);
      },
    },),
    it({
      name: 'READS A QUOTE THAT NEVER CLOSES ON ITS LINE AS NO STRING, so the brace after it opens as it did',
      fn: async () => {
        expect(firstNestingExcess({ body: `{cat "mew {\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual({
          kind: 'beyond',
          measure: 'brackets',
          bound: 256,
          line: 2,
          column: 263,
        },);
      },
    },),
  ],
},);

//endregion Nesting bound expression strings tests
