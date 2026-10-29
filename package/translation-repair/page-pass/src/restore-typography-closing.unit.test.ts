/**
 Guards class one hundred eighty-one (TianqiChen66613, 2026-09-27): the page
 is written in Canadian English, whose style (the Canadian Press Stylebook, as
 the archive follows it 224 times to 32) sets a period or comma inside the
 closing quotation mark. The bench wrote `the “high-performance robot”.` on a
 page that also wrote `“Old Man Chen,”`, so the typography restoration moves
 the mark inside.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreTypography, } from '../dist/final/node/index.mjs';

/**
 Left double quotation mark.
 */
const OPEN = '\u{201C}';

/**
 Right double quotation mark.
 */
const CLOSE = '\u{201D}';

/**
 Curly-quoted page the replacements join.
 */
const CURLY_PAGE = `The cat was called ${OPEN}Tabby.${CLOSE}`;

/**
 Restores against the curly-quoted page.

 @param replacement - text the writer produced

 @returns Replacement as the page would carry it

 @example
 ```ts
 const restored = restoreOnCurlyPage({ replacement: 'the “nap champion”.', },);
 ```
 */
function restoreOnCurlyPage({ replacement, }: { readonly replacement: string; },): string {
  return restoreTypography({
    replacement,
    replaced: 'The cat napped.',
    convention: CURLY_PAGE,
  },);
}

await describe({
  name: 'closing punctuation inside the quotation mark (class one hundred eighty-one)',
  children: [
    it({
      name: 'MOVES a period after a closing double quote inside it',
      fn: async () => {
        expect(restoreOnCurlyPage({ replacement: `Everyone remembered the ${OPEN}nap champion${CLOSE}.`, },),)
          .toBe(`Everyone remembered the ${OPEN}nap champion.${CLOSE}`,);
      },
    },),
    it({
      name: 'MOVES a comma after a closing double quote inside it',
      fn: async () => {
        expect(restoreOnCurlyPage({ replacement: `The ${OPEN}nap champion${CLOSE}, a cat, slept.`, },),)
          .toBe(`The ${OPEN}nap champion,${CLOSE} a cat, slept.`,);
      },
    },),
    it({
      name: 'MOVES the period after a straight pair the restoration curled',
      fn: async () => {
        expect(restoreOnCurlyPage({ replacement: 'Everyone remembered the "nap champion".', },),)
          .toBe(`Everyone remembered the ${OPEN}nap champion.${CLOSE}`,);
      },
    },),
    it({
      name: 'LEAVES an ellipsis, a question mark and a quote ending in its own mark alone',
      fn: async () => {
        expect(restoreOnCurlyPage({ replacement: `The ${OPEN}nap champion${CLOSE}... slept.`, },),)
          .toBe(`The ${OPEN}nap champion${CLOSE}... slept.`,);
        expect(restoreOnCurlyPage({ replacement: `Was it the ${OPEN}nap champion${CLOSE}?`, },),)
          .toBe(`Was it the ${OPEN}nap champion${CLOSE}?`,);
        expect(restoreOnCurlyPage({ replacement: `The cat asked ${OPEN}Nap?${CLOSE}.`, },),)
          .toBe(`The cat asked ${OPEN}Nap?${CLOSE}.`,);
      },
    },),
    it({
      name: 'LEAVES a closing quote inside a backtick span alone',
      fn: async () => {
        expect(restoreOnCurlyPage({ replacement: `Run \`echo ${CLOSE}.\` first.`, },),)
          .toBe(`Run \`echo ${CLOSE}.\` first.`,);
      },
    },),
  ],
},);
