/**
 Tests for telling display math from inline math that runs over lines.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readsAsDisplayMath, } from '../../dist/final/node/index.mjs';

/**
 Whether the math that runs from the first pair of dollar signs of a text to the last is display math.

 @param source - text

 @returns Answer for the stretch from the first `$$` to the end of the last

 @example
 ```ts
 const display = readsFirstToLast({ source: 'a\n    $$\n    {a}\n    $$\n', },);
 ```
 */
function readsFirstToLast({ source, }: { readonly source: string; },): boolean {
  return readsAsDisplayMath({
    source,
    range: {
      start: source.indexOf('$$',),
      end: source.lastIndexOf('$$',) + 2,
    },
  },);
}

await describe({
  name: readsAsDisplayMath.name,
  children: [
    it({
      name: 'SAYS YES FOR MATH THAT OPENS AND CLOSES WITH A LINE OF DOLLAR SIGNS ALONE AFTER A LINE, in a container and with '
        + 'white space after the marks',
      fn: async () => {
        for (
          const source of [
            'a\n    $$\n    {a}\n    $$\n',
            'a\n\t$$\n\t{a}\n\t$$\n',
            '> a\n>     $$\n>     {a}\n>     $$   \n',
            'a\r\n    $$\r\n    {a}\r\n    $$\r\n',
            'a\n    $$\n    {a}\n    $$',
          ]
        )
          expect(readsFirstToLast({ source, },),).toBe(true,);
      },
    },),

    it({
      name: 'SAYS NO FOR A STRETCH THAT DOES NOT OPEN OR DOES NOT CLOSE WITH DOLLAR SIGNS',
      fn: async () => {
        expect(readsAsDisplayMath({
          source: 'a\n    x\n    $$\n',
          range: {
            start: 6,
            end: 15,
          },
        },),).toBe(false,);
        expect(readsAsDisplayMath({
          source: 'a\n    $$\n    {a}\n    x\n',
          range: {
            start: 6,
            end: 23,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS NO FOR MATH THAT STANDS ON ONE LINE OR WHOSE OPENING LINE HOLDS MORE THAN THE DOLLAR SIGNS',
      fn: async () => {
        expect(readsFirstToLast({ source: 'a\n    $$ {a} $$\n', },),).toBe(false,);
        expect(readsFirstToLast({ source: 'a\n    $$ b\n    {a}\n    $$\n', },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS NO FOR MATH THAT DOES NOT START ITS LINE, or starts the text, where the plain reading reads display math',
      fn: async () => {
        expect(readsFirstToLast({ source: 'a $$\n{b}\n$$\n', },),).toBe(false,);
        expect(readsFirstToLast({ source: '$$\n{b}\n$$\n', },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS NO FOR MATH WHOSE CLOSING DOLLAR SIGNS SHARE THEIR LINE WITH TEXT, before them or after them',
      fn: async () => {
        expect(readsFirstToLast({ source: 'a\n    $$\n    {a} $$\n', },),).toBe(false,);
        expect(readsFirstToLast({ source: 'a\n    $$\n    {a}\n    $$ b\n', },),).toBe(false,);
      },
    },),
  ],
},);
