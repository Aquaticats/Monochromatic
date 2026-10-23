/**
 Tests for the names a text declares as an ES module under the strict grammar.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { declaredNamesOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: declaredNamesOf.name,
  children: [
    it({
      name: 'READS THE LOCAL NAME OF EACH IMPORT SPECIFIER, the default, the named and the namespace among them',
      fn: async () => {
        expect(declaredNamesOf({ text: 'import a from "b"\n', },),).toEqual(['a',],);
        expect(declaredNamesOf({ text: 'import { c as d, e } from "b"\n', },),).toEqual(['d', 'e',],);
        expect(declaredNamesOf({ text: 'import * as f from "b"\n', },),).toEqual(['f',],);
        expect(declaredNamesOf({ text: 'import g, { h } from "b"\n', },),).toEqual(['g', 'h',],);
      },
    },),

    it({
      name: 'READS THE NAMES AN EXPORT DECLARES, of a variable list, a function, a default and a list of specifiers',
      fn: async () => {
        expect(declaredNamesOf({ text: 'export const i = 1, j = 2\n', },),).toEqual(['i', 'j',],);
        expect(declaredNamesOf({ text: 'export function k() {}\n', },),).toEqual(['k',],);
        expect(declaredNamesOf({ text: 'export default 3\n', },),).toEqual(['default',],);
        expect(declaredNamesOf({ text: 'export const l = 1\nexport { l as m }\n', },),).toEqual(['l', 'm',],);
      },
    },),

    it({
      name: 'READS EVERY MODULE BLOCK OF A TEXT, in order, and nothing of the prose between them',
      fn: async () => {
        expect(declaredNamesOf({
          text: 'import a from "b"\n\n她说\n\nimport c from "b"\n',
        },),).toEqual(['a', 'c',],);
      },
    },),

    it({
      name: 'READS NOTHING FOR A TEXT WITHOUT A MODULE, prose that starts with the word among them, and for a text the '
        + 'strict grammar refuses',
      fn: async () => {
        expect(declaredNamesOf({ text: '她说\n', },),).toEqual([],);
        expect(declaredNamesOf({ text: 'imported a from "b"\n', },),).toEqual([],);
        expect(declaredNamesOf({ text: '  import a from "b"\n', },),).toEqual([],);
        expect(declaredNamesOf({ text: '{ 她说\n', },),).toEqual([],);
      },
    },),
  ],
},);
