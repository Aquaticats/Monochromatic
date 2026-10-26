/**
 Tests that the nesting bound leaves no brace open from a link reference definition.

 The destination and the title of a definition are read by the definition itself, which takes a brace for a character of
 either, so `[r]: u{b "t {"` opens no expression for the strict grammar. A count that carried the braces of the line on
 took the expression for open for the rest of the text, and then counted the info string of every fence after it, which
 the measure skips where a fence opens at the top. A page that composed such a definition with a fence whose info string
 holds a long line of brackets was refused for a depth it does not have.

 The braces of the line still count against the bound on the line itself, so a definition cannot hide a long run of them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { firstNestingExcess, } from '../dist/final/node/index.mjs';

//region Nesting bound definitions tests

/**
 What reading a body found within the bound.
 */
const WITHIN = { kind: 'within', };

/**
 A fence whose info string holds more open brackets than the bound allows, which the measure skips where the fence opens at
 the top.
 */
const DEEP_INFO_FENCE = `~~~js ${'['.repeat(300,)}\n~~~\n`;

await describe({
  name: `${firstNestingExcess.name} over definitions`,
  children: [
    it({
      name: 'LEAVES NO BRACE OPEN FROM A DEFINITION, so a fence after it opens at the top however many braces the line held',
      fn: async () => {
        expect(firstNestingExcess({ body: `[r]: u{b "t {"\n\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual(WITHIN,);
        expect(firstNestingExcess({ body: `> [r]: u{{{b\n\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual(WITHIN,);
      },
    },),
    it({
      name: 'STILL COUNTS THE BRACES OF A DEFINITION AGAINST THE BOUND ON ITS OWN LINE',
      fn: async () => {
        expect(firstNestingExcess({ body: `[r]: u${'{'.repeat(257,)}\n`, grammar: 'mdx', },),).toEqual({
          kind: 'beyond',
          measure: 'open braces',
          bound: 256,
          line: 1,
          column: 263,
        },);
      },
    },),
    it({
      name: 'LEAVES THE BRACES OF A LINE THAT IS NO DEFINITION OPEN, so the fence after it still counts its info string',
      fn: async () => {
        expect(firstNestingExcess({ body: `cat: u{b "t {"\n\n${DEEP_INFO_FENCE}`, grammar: 'mdx', },),).toEqual({
          kind: 'beyond',
          measure: 'brackets',
          bound: 256,
          line: 3,
          column: 263,
        },);
      },
    },),
  ],
},);

//endregion Nesting bound definitions tests
