/**
 Tests that a typography reader which got an all-false mask from a refusal of the page grammar says so (n159, X8): one finding at
 the warning level that names the reader and the slice, so the silence of a rule on that slice is a line a person can find.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  noteRefusedMask,
  textNamed,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from './console-warn-lines.test-fixture.ts';

await describe({
  name: 'typography-mask-refusal (n159)',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES A TEXT BY ITS FIRST FORTY-EIGHT CHARACTERS between quotation marks, when no slice number is at hand',
      fn: async () => {
        expect(textNamed({ text: 'Oops {brace it\'s here.', },),).toBe('the text "Oops {brace it\'s here."',);
        expect(textNamed({ text: 'x'.repeat(100,), },),).toBe(`the text "${'x'.repeat(48,)}"`,);
      },
    },),
    it({
      name: 'WARNS ONCE, naming the reader and the slice',
      fn: async () => {
        /**
         What the call warned.
         */
        const { warned, } = await warnLinesDuring({
          run: async () => {
            noteRefusedMask({
              reader: 'unifyQuoteStyle',
              name: 'slice 7',
            },);
          },
        },);
        expect(warned.length,).toBe(1,);
        expect(warned.filter(function namesBoth(line,): boolean {
          return line.includes('unifyQuoteStyle',) && line.includes('slice 7',) && line.includes('refused',);
        },).length,).toBe(1,);
      },
    },),
  ],
},);
